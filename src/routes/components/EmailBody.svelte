<script lang="ts">
  import type {
    EmailHeader,
    AttachmentInfo,
    CMVHVerificationResult,
  } from "../lib/types";
  import { formatFullLocalDateTime } from "../lib/utils";
  import AttachmentList from "./AttachmentList.svelte";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Separator } from "$lib/components/ui/separator";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import * as ButtonGroup from "$lib/components/ui/button-group";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import { loadConfig } from "$lib/cmvh";
  import { RewardService } from "$lib/cmvh/reward-service";
  import { walletStore } from "$lib/stores/wallet.svelte";
  import type { RewardInfo, EmailContent } from "$lib/cmvh/types";

  // Props
  let {
    email = null as EmailHeader | null,
    body = null as string | null,
    attachments = [] as AttachmentInfo[],
    isLoadingBody = false,
    isLoadingAttachments = false,
    error = null as string | null,
    cmvhVerification = null as CMVHVerificationResult | null,
    onReply,
    onForward,
    onDelete,
    onDownloadAttachment,
    onToggleRead,
    onVerifyOnChain,
  }: {
    email?: EmailHeader | null;
    body?: string | null;
    attachments?: AttachmentInfo[];
    isLoadingBody?: boolean;
    isLoadingAttachments?: boolean;
    error?: string | null;
    cmvhVerification?: CMVHVerificationResult | null;
    onReply: () => void;
    onForward: () => void;
    onDelete: () => void;
    onDownloadAttachment: (attachmentId: number, filename: string) => void;
    onToggleRead: () => void;
    onVerifyOnChain?: () => void;
  } = $props();

  /**
   * Sanitize and prepare email HTML for safe display
   * Injects a script to intercept link clicks and send them to parent window
   */
  function sanitizeEmailHtml(html: string): string {
    // Inject link interception script at the beginning of the HTML
    const scriptTag = '<' + 'script>';
    const closeScriptTag = '</' + 'script>';

    const linkInterceptScript =
      scriptTag + `
        (function() {
          // Wait for DOM to be ready
          if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', setupLinkHandlers);
          } else {
            setupLinkHandlers();
          }

          function setupLinkHandlers() {
            document.addEventListener('click', function(e) {
              var target = e.target;
              var link = target.closest('a');

              if (link && link.href) {
                e.preventDefault();
                e.stopPropagation();

                // Send message to parent window
                window.parent.postMessage({
                  type: 'OPEN_LINK',
                  url: link.href
                }, '*');
              }
            }, true);
          }
        })();
      ` + closeScriptTag;

    // Insert the script after <head> tag or at the beginning if no <head>
    if (html.includes('<head>')) {
      return html.replace('<head>', '<head>' + linkInterceptScript);
    } else if (html.includes('<html>')) {
      return html.replace('<html>', '<html><head>' + linkInterceptScript + '</head>');
    } else {
      return linkInterceptScript + html;
    }
  }

  // Handle iframe link clicks
  /**
   * Setup link handlers for iframe (fallback method)
   * The primary method is the injected script in sanitizeEmailHtml
   */
  function setupLinkHandlers(iframe: HTMLIFrameElement) {
    // This is now a no-op since we inject the script directly into the HTML
    // Keeping this function for backwards compatibility
    console.log('Link handlers are injected via script in HTML');
  }

  /**
   * Listen for messages from iframe (for link clicks)
   */
  function handleIframeMessage(event: MessageEvent) {
    if (event.data && event.data.type === 'OPEN_LINK') {
      const url = event.data.url;
      if (url && (url.startsWith('http://') || url.startsWith('https://'))) {
        openUrl(url).catch((err: unknown) => {
          console.error("Failed to open link:", err);
        });
      }
    }
  }

  // Setup message listener on component mount
  $effect(() => {
    window.addEventListener('message', handleIframeMessage);
    return () => {
      window.removeEventListener('message', handleIframeMessage);
    };
  });
  // Reward State
  let rewardInfo = $state<RewardInfo | null>(null);
  let isCheckingReward = $state(false);
  let isClaimingReward = $state(false);

  // Check for rewards when email changes
  $effect(() => {
    if (email) {
      checkReward(email);
    } else {
      rewardInfo = null;
    }
  });

  async function checkReward(currentEmail: EmailHeader) {
    isCheckingReward = true;
    rewardInfo = null;
    try {
      // Construct EmailContent
      const emailContent: EmailContent = {
        subject: currentEmail.subject,
        from: currentEmail.from,
        to: currentEmail.to,
        body: body || "",
      };

      // Check blockchain for reward
      const cmvhConfig = loadConfig();
      const rewardService = new RewardService(cmvhConfig);

      let userAddress = walletStore.address;
      if (!userAddress && cmvhConfig.derivedAddress) {
        userAddress = cmvhConfig.derivedAddress as `0x${string}`;
      }

      if (userAddress) {
        const rewardId = await rewardService.findRewardForEmail(
          userAddress,
          emailContent,
        );

        if (rewardId) {
          const info = await rewardService.getRewardInfo(rewardId);
          rewardInfo = {
            rewardId: rewardId,
            sender: info.sender,
            recipient: info.recipient,
            amount: info.amount.toString(),
            timestamp: info.timestamp.toString(),
            expiryTime: info.expiryTime.toString(),
            claimed: info.claimed,
            emailHash: info.emailHash,
          };
          console.log("💰 Found reward:", rewardInfo);
        } else {
          console.log("No reward found for this email.");
        }
      }
    } catch (e) {
      console.error("Failed to check for reward:", e);
    } finally {
      isCheckingReward = false;
    }
  }

  async function handleClaimReward() {
    if (!rewardInfo || !email) return;

    if (!walletStore.isConnected) {
      toast.error("Please connect your wallet to claim this reward.");
      walletStore.connect();
      return;
    }

    // Check for CMVH signature
    if (!cmvhVerification?.headers?.signature) {
      toast.error("Cannot claim reward: Missing CMVH signature on email.");
      return;
    }

    isClaimingReward = true;
    try {
      toast.loading("Claiming reward...");
      const cmvhConfig = loadConfig();
      const rewardService = new RewardService(cmvhConfig);

      const emailContent: EmailContent = {
        subject: email.subject,
        from: email.from,
        to: email.to,
        body: body || "",
      };

      // The service claimReward signature: (rewardId, emailContent, signature)
      await rewardService.claimReward(
        rewardInfo.rewardId,
        emailContent,
        cmvhVerification.headers.signature,
      );

      toast.success(
        "Reward claimed successfully! Funds will appear in your wallet shortly.",
      );

      // Optimistically update UI
      rewardInfo.claimed = true;
    } catch (e) {
      console.error("Failed to claim reward:", e);
      toast.error("Failed to claim reward: " + e);
    } finally {
      isClaimingReward = false;
      toast.dismiss();
    }
  }
</script>

<main class="flex flex-1 flex-col overflow-hidden">
  {#if isLoadingBody}
    <!-- Skeleton loading state -->
    <div class="flex-shrink-0 border-b bg-muted/40 p-6">
      <Skeleton class="mb-4 h-8 w-3/4" />
      <div class="space-y-2">
        <div class="flex gap-2">
          <Skeleton class="h-4 w-16" />
          <Skeleton class="h-4 w-48" />
        </div>
        <div class="flex gap-2">
          <Skeleton class="h-4 w-16" />
          <Skeleton class="h-4 w-64" />
        </div>
      </div>
    </div>
  {:else if email && body}
    <div class="flex-shrink-0 border-b bg-muted/40 p-6">
      <h2 class="mb-4 text-2xl font-semibold">{email.subject}</h2>

      <div class="space-y-2 text-sm">
        <div class="flex gap-2">
          <span class="w-16 font-semibold text-muted-foreground">From:</span>
          <span class="break-words">{email.from}</span>
        </div>
        <div class="flex gap-2">
          <span class="w-16 font-semibold text-muted-foreground">To:</span>
          <span class="break-words">{email.to}</span>
        </div>
        <div class="flex gap-2">
          <span class="w-16 font-semibold text-muted-foreground">Date:</span>
          <span>{formatFullLocalDateTime(email.timestamp)}</span>
        </div>
      </div>

      <!-- CMVH Verification Badge -->
      <!-- Reward Banner -->
      {#if isCheckingReward}
        <div class="mt-4 rounded-lg border bg-muted/40 p-3">
          <div class="flex items-center gap-2 text-sm text-muted-foreground">
            <svg
              class="animate-spin h-4 w-4"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
              ></circle>
              <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            Checking for rewards...
          </div>
        </div>
      {:else if rewardInfo}
        <div
          class="mt-4 rounded-lg border-2 {rewardInfo.claimed
            ? 'border-green-200 bg-green-50 dark:border-green-900 dark:bg-green-950/30'
            : 'border-yellow-200 bg-yellow-50 dark:border-yellow-900 dark:bg-yellow-950/30'} p-4"
        >
          <div class="flex items-start justify-between">
            <div class="space-y-2">
              <div class="flex items-center gap-2">
                <span class="text-2xl">💰</span>
                <h3
                  class="text-lg font-bold {rewardInfo.claimed
                    ? 'text-green-900 dark:text-green-200'
                    : 'text-yellow-900 dark:text-yellow-200'}"
                >
                  {rewardInfo.claimed ? "Reward Claimed" : "Reward Available!"}
                </h3>
              </div>
              <div
                class="space-y-1 {rewardInfo.claimed
                  ? 'text-green-800 dark:text-green-300'
                  : 'text-yellow-800 dark:text-yellow-300'}"
              >
                <p class="text-sm">
                  <span class="font-semibold">Amount:</span>
                  {(Number(rewardInfo.amount) / 1e18).toFixed(4)} wACT
                </p>
                <p class="text-xs">
                  <span class="font-semibold">From:</span>
                  <span class="font-mono"
                    >{rewardInfo.sender.slice(0, 6)}...{rewardInfo.sender.slice(
                      -4,
                    )}</span
                  >
                </p>
                <p class="text-xs">
                  <span class="font-semibold">Expires:</span>
                  {new Date(Number(rewardInfo.expiryTime) * 1000).toLocaleString()}
                </p>
              </div>
            </div>

            {#if !rewardInfo.claimed}
              <Button
                variant="default"
                size="sm"
                onclick={handleClaimReward}
                disabled={isClaimingReward}
                class="bg-yellow-600 hover:bg-yellow-700 text-white"
              >
                {#if isClaimingReward}
                  <svg
                    class="animate-spin -ml-1 mr-2 h-4 w-4"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      class="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      stroke-width="4"
                    ></circle>
                    <path
                      class="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                  Claiming...
                {:else}
                  Claim Reward
                {/if}
              </Button>
            {:else}
              <div
                class="rounded-full bg-green-100 dark:bg-green-900/30 px-3 py-1 text-xs font-medium text-green-800 dark:text-green-200"
              >
                ✓ Claimed
              </div>
            {/if}
          </div>

          {#if !rewardInfo.claimed && !walletStore.isConnected}
            <div
              class="mt-3 rounded-md bg-amber-100 dark:bg-amber-900/30 p-2 text-xs text-amber-900 dark:text-amber-200"
            >
              <p>
                <strong>Note:</strong> You need to connect your wallet to claim this
                reward.
              </p>
            </div>
          {/if}
        </div>
      {/if}

      {#if cmvhVerification?.hasCMVH}
        <div class="mt-4 rounded-lg border bg-card p-3">
          <div class="flex items-center gap-2">
            {#if cmvhVerification.isOnChainVerified}
              <!-- On-Chain Verified (Blue) -->
              <div
                class="flex items-center gap-2 text-blue-600 dark:text-blue-400"
              >
                <svg
                  class="h-5 w-5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path stroke-linecap="round"> </path></svg
                >
              </div>
            {:else if cmvhVerification.isValid}
              <!-- Locally Verified (Green) -->
              <div
                class="flex items-center gap-2 text-green-600 dark:text-green-400"
              >
                <svg
                  class="h-5 w-5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                  />
                </svg>
                <span class="text-sm font-semibold">CMVH Verified (Local)</span>
              </div>
            {:else}
              <!-- Invalid Signature -->
              <div
                class="flex items-center gap-2 text-amber-600 dark:text-amber-400"
              >
                <svg
                  class="h-5 w-5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                  />
                </svg>
                <span class="text-sm font-semibold">CMVH Signature Invalid</span
                >
              </div>
            {/if}
          </div>
          {#if cmvhVerification.headers}
            <div class="mt-2 text-xs text-muted-foreground space-y-1">
              <div class="flex gap-2">
                <span class="font-semibold">Signer:</span>
                <span class="font-mono break-all"
                  >{cmvhVerification.headers.address}</span
                >
              </div>
              <div class="flex gap-2">
                <span class="font-semibold">Chain:</span>
                <span>{cmvhVerification.headers.chain}</span>
              </div>
              <div class="flex gap-2">
                <span class="font-semibold">Timestamp:</span>
                <span>{cmvhVerification.headers.timestamp}</span>
              </div>
              {#if cmvhVerification.isOnChainVerified && cmvhVerification.onChainVerifiedAt}
                <div class="flex gap-2">
                  <span class="font-semibold">On-Chain Verified:</span>
                  <span
                    >{new Date(
                      cmvhVerification.onChainVerifiedAt,
                    ).toLocaleString()}</span
                  >
                </div>
              {/if}
            </div>

            <!-- On-Chain Verification Button -->
            {#if cmvhVerification.isValid && !cmvhVerification.isOnChainVerified && onVerifyOnChain}
              <div class="mt-3">
                <Button
                  variant="outline"
                  size="sm"
                  onclick={onVerifyOnChain}
                  disabled={cmvhVerification.isVerifyingOnChain}
                >
                  {#if cmvhVerification.isVerifyingOnChain}
                    <svg
                      class="animate-spin -ml-1 mr-2 h-4 w-4"
                      xmlns="http://www.w3.org/2000/svg"
                      fill="none"
                      viewBox="0 0 24 24"
                    >
                      <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                      ></circle>
                      <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      ></path>
                    </svg>
                    Verifying On-Chain...
                  {:else}
                    🔗 Verify On-Chain
                  {/if}
                </Button>
              </div>
            {/if}
          {/if}
        </div>
      {/if}

      <div class="mt-6 flex items-center gap-2">
        <ButtonGroup.Root>
          <Button variant="default" size="sm" onclick={onReply}>
            <span class="text-base">↩</span>
            <span class="ml-1.5">Reply</span>
          </Button>
          <Button variant="default" size="sm" onclick={onForward}>
            <span class="text-base">➡</span>
            <span class="ml-1.5">Forward</span>
          </Button>
        </ButtonGroup.Root>

        <ButtonGroup.Root>
          <Button variant="outline" size="sm" onclick={onToggleRead}>
            <span class="text-base">{email.seen ? "✉" : "✅"}</span>
            <span class="ml-1.5"
              >{email.seen ? "Mark Unread" : "Mark Read"}</span
            >
          </Button>
        </ButtonGroup.Root>

        <Button
          variant="outline"
          size="sm"
          class="text-destructive hover:bg-destructive hover:text-destructive-foreground"
          onclick={onDelete}
        >
          <span class="text-base">🗑</span>
          <span class="ml-1.5">Delete</span>
        </Button>
      </div>
    </div>

    <AttachmentList
      {attachments}
      isLoading={isLoadingAttachments}
      onDownload={onDownloadAttachment}
    />

    <ScrollArea class="flex-1 p-6">
      <div class="w-full overflow-auto">
        <iframe
          srcdoc={sanitizeEmailHtml(body)}
          title="Email content"
          class="w-full border-0"
          style="min-height: 500px; height: 100%;"
          sandbox="allow-same-origin allow-scripts allow-popups allow-popups-to-escape-sandbox"
          onload={(e) => {
            // Auto-resize iframe to fit content
            const iframe = e.target as HTMLIFrameElement;
            try {
              const doc =
                iframe.contentDocument || iframe.contentWindow?.document;
              if (doc) {
                const height = doc.documentElement.scrollHeight;
                iframe.style.height = `${height}px`;
              }
            } catch (err) {
              console.warn(
                "Cannot access iframe content for auto-resize:",
                err,
              );
            }

            // Setup link click handlers to open in default browser
            setupLinkHandlers(iframe);
          }}
        ></iframe>
      </div>
    </ScrollArea>
  {:else if error}
    <div
      class="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center"
    >
      <p class="text-lg font-semibold text-destructive">
        ⚠️ Error loading email
      </p>
      <p class="text-sm text-muted-foreground">{error}</p>
      <p class="text-xs italic text-muted-foreground">
        The email may have been deleted or moved. Please try refreshing the
        folder.
      </p>
    </div>
  {:else if email && !body}
    <div
      class="flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center"
    >
      <p class="text-sm text-muted-foreground">
        Email selected but content not loaded yet...
      </p>
      <p class="text-xs text-muted-foreground">
        If this persists, try selecting another email.
      </p>
    </div>
  {:else}
    <div class="flex flex-1 items-center justify-center p-8">
      <p class="text-sm text-muted-foreground">
        Select an email to read its content.
      </p>
    </div>
  {/if}
</main>
