<script lang="ts">
  import type { EmailHeader, AttachmentInfo, CMVHVerificationResult } from "../lib/types";
  import { formatFullLocalDateTime } from "../lib/utils";
  import AttachmentList from "./AttachmentList.svelte";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Separator } from "$lib/components/ui/separator";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import * as ButtonGroup from "$lib/components/ui/button-group";

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
        <div class="flex gap-2">
          <Skeleton class="h-4 w-16" />
          <Skeleton class="h-4 w-40" />
        </div>
      </div>

      <div class="mt-6 flex items-center gap-2">
        <Skeleton class="h-8 w-32" />
        <Skeleton class="h-8 w-32" />
        <Skeleton class="h-8 w-32" />
        <Skeleton class="h-8 w-24" />
      </div>
    </div>

    <div class="flex-1 p-6">
      <div class="space-y-3">
        <Skeleton class="h-4 w-full" />
        <Skeleton class="h-4 w-full" />
        <Skeleton class="h-4 w-5/6" />
        <Skeleton class="h-4 w-full" />
        <Skeleton class="h-4 w-4/5" />
        <Skeleton class="h-4 w-full" />
        <Skeleton class="h-4 w-3/4" />
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
        {#if email.cc && email.cc.trim()}
          <div class="flex gap-2">
            <span class="w-16 font-semibold text-muted-foreground">CC:</span>
            <span class="break-words">{email.cc}</span>
          </div>
        {/if}
        <div class="flex gap-2">
          <span class="w-16 font-semibold text-muted-foreground">Date:</span>
          <span>{formatFullLocalDateTime(email.timestamp)}</span>
        </div>
      </div>

      <!-- CMVH Verification Badge -->
      {#if cmvhVerification?.hasCMVH}
        <div class="mt-4 rounded-lg border bg-card p-3">
          <div class="flex items-center gap-2">
            {#if cmvhVerification.isOnChainVerified}
              <!-- On-Chain Verified (Blue) -->
              <div class="flex items-center gap-2 text-blue-600 dark:text-blue-400">
                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
                <span class="text-sm font-semibold">CMVH On-Chain Verified</span>
              </div>
            {:else if cmvhVerification.isValid}
              <!-- Locally Verified (Green) -->
              <div class="flex items-center gap-2 text-green-600 dark:text-green-400">
                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
                <span class="text-sm font-semibold">CMVH Verified (Local)</span>
              </div>
            {:else}
              <!-- Invalid Signature -->
              <div class="flex items-center gap-2 text-amber-600 dark:text-amber-400">
                <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <span class="text-sm font-semibold">CMVH Signature Invalid</span>
              </div>
            {/if}
          </div>
          {#if cmvhVerification.headers}
            <div class="mt-2 text-xs text-muted-foreground space-y-1">
              <div class="flex gap-2">
                <span class="font-semibold">Signer:</span>
                <span class="font-mono break-all">{cmvhVerification.headers.address}</span>
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
                  <span>{new Date(cmvhVerification.onChainVerifiedAt).toLocaleString()}</span>
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
                    <svg class="animate-spin -ml-1 mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
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
            <span class="ml-1.5">{email.seen ? "Mark Unread" : "Mark Read"}</span>
          </Button>
        </ButtonGroup.Root>

        <Button variant="outline" size="sm" class="text-destructive hover:bg-destructive hover:text-destructive-foreground" onclick={onDelete}>
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
          srcdoc={body}
          title="Email content"
          class="w-full border-0"
          style="min-height: 500px; height: 100%;"
          sandbox="allow-same-origin allow-popups allow-popups-to-escape-sandbox"
          onload={(e) => {
            // Auto-resize iframe to fit content
            const iframe = e.target as HTMLIFrameElement;
            try {
              const doc = iframe.contentDocument || iframe.contentWindow?.document;
              if (doc) {
                const height = doc.documentElement.scrollHeight;
                iframe.style.height = `${height}px`;
              }
            } catch (err) {
              console.warn('Cannot access iframe content for auto-resize:', err);
            }
          }}
        ></iframe>
      </div>
    </ScrollArea>
  {:else if error}
    <div class="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
      <p class="text-lg font-semibold text-destructive">⚠️ Error loading email</p>
      <p class="text-sm text-muted-foreground">{error}</p>
      <p class="text-xs italic text-muted-foreground">
        The email may have been deleted or moved. Please try refreshing the folder.
      </p>
    </div>
  {:else if email && !body}
    <div class="flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center">
      <p class="text-sm text-muted-foreground">Email selected but content not loaded yet...</p>
      <p class="text-xs text-muted-foreground">If this persists, try selecting another email.</p>
    </div>
  {:else}
    <div class="flex flex-1 items-center justify-center p-8">
      <p class="text-sm text-muted-foreground">Select an email to read its content.</p>
    </div>
  {/if}
</main>
