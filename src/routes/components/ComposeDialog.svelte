<script lang="ts">
  import { formatFileSize } from "../lib/utils";
  import RichTextEditor from "./RichTextEditor.svelte";
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
  } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import * as ButtonGroup from "$lib/components/ui/button-group";
  import { loadConfig } from "$lib/cmvh";
  import { walletStore } from "$lib/stores/wallet.svelte";
  import { state as appState } from "../lib/state.svelte";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";

  // Props
  let {
    show = false,
    mode = "compose" as "compose" | "reply" | "forward",
    to = $bindable(""),
    cc = $bindable(""),
    subject = $bindable(""),
    body = $bindable(""),
    attachments = $bindable<File[]>([]),
    attachmentSizeLimit = 10 * 1024 * 1024,
    totalAttachmentSize = 0,
    isSending = false,
    isDraft = false,
    error = null as string | null,
    enableCMVHSigning = $bindable(false),
    onSend,
    onCancel,
    onAttachmentAdd,
    onAttachmentRemove,
  }: {
    show?: boolean;
    mode?: "compose" | "reply" | "forward";
    to?: string;
    cc?: string;
    subject?: string;
    body?: string;
    attachments?: File[];
    attachmentSizeLimit?: number;
    totalAttachmentSize?: number;
    isSending?: boolean;
    isDraft?: boolean;
    error?: string | null;
    enableCMVHSigning?: boolean;
    onSend: () => void;
    onCancel: () => void;
    onAttachmentAdd: (event: Event) => void;
    onAttachmentRemove: (index: number) => void;
  } = $props();

  // Load CMVH config to check if signing is available
  const cmvhConfig = $derived(loadConfig());

  // WalletConnect QR code display state (local to Compose Dialog)
  let showComposeWalletConnectQR = $state(false);

  // Monitor wallet connection status to hide QR when connected
  $effect(() => {
    if (walletStore.isConnected) {
      showComposeWalletConnectQR = false;
    }
  });

  // Handle wallet connect click in Compose
  async function handleComposeWalletConnect() {
    showComposeWalletConnectQR = true;
    await walletStore.connect();
  }

  // Handle cancel connection in Compose
  async function handleComposeCancelConnection() {
    await walletStore.cancelConnection();
    showComposeWalletConnectQR = false;
  }

  // Auto-cancel connection when Compose Dialog closes
  $effect(() => {
    // Cleanup function - runs when dialog closes
    if (!show && showComposeWalletConnectQR && walletStore.isConnecting && !walletStore.isConnected) {
      console.log("Compose Dialog closed - auto-cancelling wallet connection");
      handleComposeCancelConnection();
    }
  });

  // Resizable dialog state
  let dialogElement = $state<HTMLDivElement | null>(null);
  let isResizing = $state(false);
  let resizeDirection = $state<"se" | "e" | "s" | null>(null);
  let dialogWidth = $state(672); // 42rem = 672px (tailwind max-w-2xl)
  let dialogHeight = $state(600); // reasonable default height
  let startX = $state(0);
  let startY = $state(0);
  let startWidth = $state(0);
  let startHeight = $state(0);

  // Reset dialog size when dialog opens
  $effect(() => {
    if (show) {
      // Reset to default size when dialog opens
      const { maxWidth, maxHeight } = getViewportConstraints();
      dialogWidth = Math.min(672, maxWidth); // 42rem or max available
      dialogHeight = Math.min(600, maxHeight); // 600px or max available
    }
  });

  // Get viewport constraints
  function getViewportConstraints() {
    const margin = 32; // 2rem margin on each side
    const maxWidth = window.innerWidth - margin * 2;
    const maxHeight = window.innerHeight - margin * 2;
    const minWidth = 400;
    const minHeight = 400;
    return { maxWidth, maxHeight, minWidth, minHeight };
  }

  function startResize(event: MouseEvent, direction: "se" | "e" | "s") {
    event.preventDefault();
    event.stopPropagation();

    isResizing = true;
    resizeDirection = direction;
    startX = event.clientX;
    startY = event.clientY;
    startWidth = dialogWidth;
    startHeight = dialogHeight;

    document.addEventListener("mousemove", handleResize);
    document.addEventListener("mouseup", stopResize);
    document.body.style.cursor =
      direction === "se"
        ? "nwse-resize"
        : direction === "e"
          ? "ew-resize"
          : "ns-resize";
    document.body.style.userSelect = "none";
  }

  function handleResize(event: MouseEvent) {
    if (!isResizing || !resizeDirection) return;

    const { maxWidth, maxHeight, minWidth, minHeight } =
      getViewportConstraints();
    const deltaX = event.clientX - startX;
    const deltaY = event.clientY - startY;

    if (resizeDirection === "se" || resizeDirection === "e") {
      const newWidth = Math.max(
        minWidth,
        Math.min(maxWidth, startWidth + deltaX),
      );
      dialogWidth = newWidth;
    }

    if (resizeDirection === "se" || resizeDirection === "s") {
      const newHeight = Math.max(
        minHeight,
        Math.min(maxHeight, startHeight + deltaY),
      );
      dialogHeight = newHeight;
    }
  }

  function stopResize() {
    isResizing = false;
    resizeDirection = null;
    document.removeEventListener("mousemove", handleResize);
    document.removeEventListener("mouseup", stopResize);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  }

  function getModalTitle(): string {
    const baseTitle = (() => {
      switch (mode) {
        case "reply":
          return "Reply to Email";
        case "forward":
          return "Forward Email";
        default:
          return "Compose Email";
      }
    })();

    return isDraft ? `${baseTitle} (Draft)` : baseTitle;
  }
</script>

<Dialog
  open={show}
  onOpenChange={(open) => {
    if (!open) onCancel();
  }}
>
  <DialogContent
    bind:ref={dialogElement}
    class="flex flex-col resize-dialog !max-w-none !max-h-none !translate-x-0 !translate-y-0"
    style="width: {dialogWidth}px; height: {dialogHeight}px; left: 50%; top: 50%; transform: translate(-50%, -50%);"
    onInteractOutside={(e: Event) => {
      // Prevent dialog from closing when clicking outside
      e.preventDefault();
      // Trigger the cancel handler (which shows save draft dialog)
      onCancel();
    }}
  >
    <DialogHeader>
      <DialogTitle>{getModalTitle()}</DialogTitle>
    </DialogHeader>

    <!-- Resize handles -->
    <div
      class="resize-handle resize-handle-e"
      role="button"
      tabindex="-1"
      aria-label="Resize horizontally"
      onmousedown={(e) => startResize(e, "e")}
    ></div>
    <div
      class="resize-handle resize-handle-s"
      role="button"
      tabindex="-1"
      aria-label="Resize vertically"
      onmousedown={(e) => startResize(e, "s")}
    ></div>
    <div
      class="resize-handle resize-handle-se"
      role="button"
      tabindex="-1"
      aria-label="Resize diagonally"
      onmousedown={(e) => startResize(e, "se")}
    ></div>

    <div class="flex-1 space-y-4 overflow-y-auto px-1">
      {#if error}
        <div
          class="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive"
        >
          {error}
        </div>
      {/if}

      <div class="space-y-2">
        <Label for="compose-to">To:</Label>
        <Input
          type="email"
          id="compose-to"
          bind:value={to}
          placeholder="recipient@example.com"
          disabled={isSending}
        />
      </div>

      <div class="space-y-2">
        <Label for="compose-cc">CC:</Label>
        <Input
          type="text"
          id="compose-cc"
          bind:value={cc}
          placeholder="cc@example.com (separate multiple with commas)"
          disabled={isSending}
        />
      </div>

      <div class="space-y-2">
        <Label for="compose-subject">Subject:</Label>
        <Input
          type="text"
          id="compose-subject"
          bind:value={subject}
          placeholder="Email subject"
          disabled={isSending}
        />
      </div>

      <div class="space-y-2">
        <Label for="compose-body">Body:</Label>
        <RichTextEditor
          bind:value={body}
          disabled={isSending}
          placeholder="Write your message here..."
        />
      </div>

      <div class="space-y-2">
        <Label for="compose-attachments">
          Attachments:
          <span class="text-xs text-muted-foreground ml-1">
            (Max: {formatFileSize(attachmentSizeLimit)})
          </span>
        </Label>
        <Input
          type="file"
          id="compose-attachments"
          multiple
          onchange={onAttachmentAdd}
          disabled={isSending}
        />

        {#if attachments.length > 0}
          <div class="space-y-2 pt-2">
            {#each attachments as file, index (file.name + file.size + index)}
              <div
                class="flex items-center gap-2 rounded-md border bg-muted/40 p-2"
              >
                <span class="flex-1 truncate text-sm">{file.name}</span>
                <Badge variant="secondary" class="text-xs"
                  >{formatFileSize(file.size)}</Badge
                >
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-6 w-6 shrink-0 text-destructive hover:bg-destructive hover:text-destructive-foreground"
                  onclick={() => onAttachmentRemove(index)}
                  disabled={isSending}
                  title="Remove attachment"
                >
                  ×
                </Button>
              </div>
            {/each}
            <div class="rounded-md bg-muted p-2 text-right text-xs font-medium">
              Total: {formatFileSize(totalAttachmentSize)} / {formatFileSize(
                attachmentSizeLimit,
              )}
            </div>
          </div>
        {/if}
      </div>

      <!-- CMVH Signing Toggle -->
      {#if cmvhConfig.enableSigning && walletStore.isConnected && walletStore.address}
        <div class="space-y-2 pt-2">
          <div
            class="flex items-center space-x-3 rounded-md border bg-muted/40 p-3"
          >
            <input
              type="checkbox"
              id="enable-cmvh-signing"
              bind:checked={enableCMVHSigning}
              disabled={isSending}
              class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
            />
            <Label for="enable-cmvh-signing" class="text-sm font-normal flex-1">
              Sign this email with CMVH (blockchain verification)
            </Label>
            {#if enableCMVHSigning}
              <Badge variant="secondary" class="text-xs">
                🔐 Signing enabled
              </Badge>
            {/if}
          </div>
          {#if enableCMVHSigning}
            <p class="text-xs text-muted-foreground pl-3">
              Signing address: <span class="font-mono"
                >{walletStore.getDisplayName()}</span
              >
            </p>
          {/if}
        </div>
      {/if}

      <!-- CMVH Reward Toggle -->
      <div class="space-y-2 pt-2">
        <div
          class="flex items-center space-x-3 rounded-md border bg-muted/40 p-3"
        >
          <input
            type="checkbox"
            id="attach-reward"
            bind:checked={appState.attachReward}
            disabled={isSending}
            class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
          />
          <div class="flex-1">
            <Label for="attach-reward" class="text-sm font-normal">
              Attach Crypto Reward (wACT)
            </Label>
            <p class="text-xs text-muted-foreground">
              Incentivize the recipient to read/reply
            </p>
          </div>
          {#if appState.attachReward}
            <Badge
              variant="secondary"
              class="text-xs bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-200"
            >
              💰 Reward Attached
            </Badge>
          {/if}
        </div>

        {#if appState.attachReward}
          <div class="pl-4 pr-4 pb-2 space-y-3 border-l-2 border-muted ml-3">
            {#if !walletStore.isConnected}
              <div class="space-y-3">
                <div
                  class="rounded-md bg-amber-50 dark:bg-amber-950/30 p-2 text-xs text-amber-900 dark:text-amber-200"
                >
                  Connect your mobile wallet via WalletConnect to attach rewards
                </div>

                <Button
                  size="sm"
                  variant="outline"
                  onclick={handleComposeWalletConnect}
                  disabled={walletStore.isConnecting}
                  class="w-full"
                >
                  📱 Connect WalletConnect
                </Button>

                {#if showComposeWalletConnectQR && walletStore.walletConnectUri}
                  <div
                    class="rounded-lg border-2 border-dashed border-primary/50 bg-white dark:bg-muted p-3 space-y-2"
                  >
                    <p class="text-xs font-medium mb-2 text-center">
                      Scan with mobile wallet
                    </p>
                    <div class="flex justify-center">
                      <img
                        src={`https://api.qrserver.com/v1/create-qr-code/?size=180x180&data=${encodeURIComponent(
                          walletStore.walletConnectUri,
                        )}`}
                        alt="WalletConnect QR Code"
                        width="180"
                        height="180"
                        class="rounded"
                      />
                    </div>
                    <p class="text-xs text-muted-foreground text-center">
                      Open MetaMask, Trust Wallet, or any compatible wallet
                    </p>
                    <div class="flex justify-center pt-1">
                      <Button
                        variant="outline"
                        size="sm"
                        onclick={handleComposeCancelConnection}
                        class="w-full"
                      >
                        Cancel Connection
                      </Button>
                    </div>
                  </div>
                {/if}

                {#if walletStore.error}
                  <p class="text-xs text-destructive">
                    {walletStore.error}
                  </p>
                {/if}
              </div>
            {:else}
              <div class="space-y-3">
                <!-- Wallet Status Display -->
                <div
                  class="rounded-md border bg-muted/40 p-3 flex items-center justify-between"
                >
                  <div class="flex-1">
                    <p class="text-xs font-medium text-muted-foreground mb-1">
                      Connected Wallet
                    </p>
                    {#if walletStore.isResolvingENS}
                      <div class="flex items-center gap-2">
                        <LoaderCircleIcon class="size-3 animate-spin text-muted-foreground" />
                        <span class="text-sm text-muted-foreground">Resolving ENS...</span>
                      </div>
                    {:else}
                      <p class="text-sm font-medium font-mono">
                        {walletStore.getDisplayName()}
                      </p>
                      {#if walletStore.ensName && walletStore.address}
                        <p class="text-xs text-muted-foreground font-mono mt-0.5">
                          {walletStore.formatAddress(walletStore.address, 'short')}
                        </p>
                      {/if}
                    {/if}
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onclick={async () => {
                      await walletStore.disconnect();
                      showComposeWalletConnectQR = false;
                    }}
                    disabled={isSending}
                    class="text-xs"
                  >
                    Change Wallet
                  </Button>
                </div>

                <div class="space-y-2">
                  <Label for="reward-recipient">Recipient Wallet Address</Label>
                  <Input
                    type="text"
                    id="reward-recipient"
                    bind:value={appState.rewardRecipient}
                    placeholder="0x... (Ethereum address)"
                    disabled={isSending}
                    class="font-mono text-sm"
                  />
                  <p class="text-xs text-muted-foreground">
                    The Ethereum address that will receive the reward
                  </p>
                </div>

                <div class="space-y-2">
                  <Label for="reward-amount">Amount (wACT)</Label>
                  <div class="flex items-center gap-2">
                    <Input
                      type="number"
                      id="reward-amount"
                      bind:value={appState.rewardAmount}
                      min="0.1"
                      step="0.1"
                      placeholder="10"
                      disabled={isSending}
                      class="w-32"
                    />
                    <span class="text-sm font-medium">wACT</span>
                  </div>
                </div>

                <div
                  class="rounded-md bg-blue-50 dark:bg-blue-950/30 p-3 text-xs text-blue-900 dark:text-blue-200"
                >
                  <p class="font-semibold mb-1">How it works:</p>
                  <ul class="list-disc list-inside space-y-0.5">
                    <li>Email is sent first, then reward is created on-chain</li>
                    <li>Recipient can claim reward after reading the email</li>
                    <li>Reward expires after 30 days if not claimed</li>
                  </ul>
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>

    <DialogFooter class="gap-2">
      <ButtonGroup.Root>
        <Button variant="outline" onclick={onCancel} disabled={isSending}
          >Cancel</Button
        >
        <Button variant="default" onclick={onSend} disabled={isSending}>
          {isSending ? "Sending..." : "Send"}
        </Button>
      </ButtonGroup.Root>
    </DialogFooter>
  </DialogContent>
</Dialog>

<style>
  :global(.resize-dialog) {
    position: relative;
    transition: none !important;
  }

  .resize-handle {
    position: absolute;
    background-color: transparent;
    z-index: 10;
  }

  .resize-handle:hover {
    background-color: rgba(59, 130, 246, 0.1);
  }

  .resize-handle-e {
    top: 0;
    right: 0;
    width: 8px;
    height: 100%;
    cursor: ew-resize;
  }

  .resize-handle-s {
    left: 0;
    bottom: 0;
    width: 100%;
    height: 8px;
    cursor: ns-resize;
  }

  .resize-handle-se {
    right: 0;
    bottom: 0;
    width: 16px;
    height: 16px;
    cursor: nwse-resize;
  }

  .resize-handle-se::after {
    content: "";
    position: absolute;
    right: 2px;
    bottom: 2px;
    width: 12px;
    height: 12px;
    background: linear-gradient(135deg, transparent 50%, currentColor 50%);
    opacity: 0.3;
  }

  .resize-handle-se:hover::after {
    opacity: 0.6;
  }
</style>
