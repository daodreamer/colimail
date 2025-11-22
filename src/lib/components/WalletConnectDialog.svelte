<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { walletStore } from "$lib/stores/wallet.svelte";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";
  import CheckCircleIcon from "@lucide/svelte/icons/check-circle";
  import WalletIcon from "@lucide/svelte/icons/wallet";
  import SmartphoneIcon from "@lucide/svelte/icons/smartphone";

  let {
    open = $bindable(false),
    onOpenChange,
  }: {
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
  } = $props();

  // Track connection attempt state
  let connectionAttempted = $state(false);
  let isCancelling = $state(false); // Track if we're in the process of cancelling

  // Start connection when dialog opens
  $effect(() => {
    if (open && !walletStore.isConnected && !walletStore.isConnecting && !isCancelling && !connectionAttempted) {
      console.log("Dialog opened - initiating connection");
      connectionAttempted = true;
      walletStore.connect().catch((error) => {
        console.error("Failed to initiate wallet connection:", error);
      });
    }
  });

  // Auto-close dialog when connection is successful
  $effect(() => {
    if (walletStore.isConnected && open) {
      // Wait a moment to show success state
      setTimeout(() => {
        open = false;
        if (onOpenChange) onOpenChange(false);
      }, 1500);
    }
  });

  // Handle dialog close
  async function handleOpenChange(newOpen: boolean) {
    console.log("Dialog open state changing:", newOpen);

    // If closing, cancel connection first before updating open state
    if (!newOpen && open) {
      if (walletStore.isConnecting && !walletStore.isConnected) {
        isCancelling = true;
        console.log("Cancelling connection due to dialog close");
        await walletStore.cancelConnection();
        isCancelling = false;
      }
      // Reset local state
      connectionAttempted = false;
    }

    // Update open state after cleanup
    open = newOpen;
    if (onOpenChange) onOpenChange(newOpen);
  }

  // Handle cancel button
  async function handleCancel() {
    console.log("Cancel button clicked");

    // Set cancelling flag to prevent re-connection
    isCancelling = true;

    // Cancel the connection
    await walletStore.cancelConnection();

    // Reset local state
    connectionAttempted = false;

    // Close dialog (this will trigger handleOpenChange, but isCancelling prevents re-connection)
    open = false;
    if (onOpenChange) onOpenChange(false);

    // Reset cancelling flag after a short delay to ensure dialog is closed
    setTimeout(() => {
      isCancelling = false;
      console.log("Cancel complete - state reset");
    }, 100);
  }
</script>

<Dialog.Root {open} onOpenChange={handleOpenChange}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title class="flex items-center gap-2">
        <WalletIcon class="size-5" />
        Connect Wallet
      </Dialog.Title>
      <Dialog.Description>
        Scan the QR code with your mobile wallet app
      </Dialog.Description>
    </Dialog.Header>

    <div class="space-y-4">
      {#if walletStore.isConnected}
        <!-- Success State -->
        <div class="flex flex-col items-center justify-center py-8 space-y-4">
          <div class="rounded-full bg-green-100 dark:bg-green-900/30 p-3">
            <CheckCircleIcon class="size-12 text-green-600 dark:text-green-400" />
          </div>
          <div class="text-center space-y-1">
            <p class="text-lg font-semibold text-green-600 dark:text-green-400">
              Connected Successfully!
            </p>
            <p class="text-sm text-muted-foreground font-mono">
              {walletStore.getDisplayName()}
            </p>
          </div>
        </div>
      {:else if walletStore.isConnecting && walletStore.walletConnectUri}
        <!-- QR Code Display State -->
        <div class="space-y-4">
          <!-- QR Code -->
          <div class="flex justify-center p-4 bg-white dark:bg-muted rounded-lg border-2 border-dashed border-primary/30">
            <img
              src={`https://api.qrserver.com/v1/create-qr-code/?size=240x240&data=${encodeURIComponent(
                walletStore.walletConnectUri,
              )}`}
              alt="WalletConnect QR Code"
              width="240"
              height="240"
              class="rounded"
            />
          </div>

          <!-- Instructions -->
          <div class="space-y-3">
            <div class="flex items-start gap-3 text-sm">
              <div class="rounded-full bg-primary/10 p-1.5 mt-0.5">
                <SmartphoneIcon class="size-4 text-primary" />
              </div>
              <div class="flex-1 space-y-1">
                <p class="font-medium">Step 1: Open your wallet app</p>
                <p class="text-muted-foreground text-xs">
                  MetaMask, Trust Wallet, Rainbow, or any WalletConnect-compatible wallet
                </p>
              </div>
            </div>

            <div class="flex items-start gap-3 text-sm">
              <div class="rounded-full bg-primary/10 p-1.5 mt-0.5">
                <div class="size-4 flex items-center justify-center text-primary font-semibold">
                  2
                </div>
              </div>
              <div class="flex-1 space-y-1">
                <p class="font-medium">Step 2: Scan the QR code</p>
                <p class="text-muted-foreground text-xs">
                  Use the "WalletConnect" or "Scan" feature in your wallet
                </p>
              </div>
            </div>

            <div class="flex items-start gap-3 text-sm">
              <div class="rounded-full bg-primary/10 p-1.5 mt-0.5">
                <div class="size-4 flex items-center justify-center text-primary font-semibold">
                  3
                </div>
              </div>
              <div class="flex-1 space-y-1">
                <p class="font-medium">Step 3: Approve the connection</p>
                <p class="text-muted-foreground text-xs">
                  Confirm the connection request on your mobile device
                </p>
              </div>
            </div>
          </div>

          <!-- Status indicator -->
          <div class="flex items-center justify-center gap-2 text-sm text-muted-foreground">
            <LoaderCircleIcon class="size-4 animate-spin" />
            <span>Waiting for wallet approval...</span>
          </div>

          <!-- Cancel button -->
          <div class="flex justify-center pt-2">
            <Button variant="outline" onclick={handleCancel} class="w-full">
              Cancel
            </Button>
          </div>
        </div>
      {:else if connectionAttempted}
        <!-- Loading State (initial connection) -->
        <div class="flex flex-col items-center justify-center py-12 space-y-4">
          <LoaderCircleIcon class="size-12 animate-spin text-primary" />
          <div class="text-center space-y-1">
            <p class="text-sm font-medium">Initializing connection...</p>
            <p class="text-xs text-muted-foreground">Please wait</p>
          </div>
        </div>
      {:else}
        <!-- Error State or Initial State -->
        <div class="flex flex-col items-center justify-center py-8 space-y-4">
          <div class="rounded-full bg-amber-100 dark:bg-amber-900/30 p-3">
            <WalletIcon class="size-12 text-amber-600 dark:text-amber-400" />
          </div>
          <div class="text-center space-y-2">
            <p class="text-sm font-medium">
              {walletStore.error || "Failed to generate QR code"}
            </p>
            <Button onclick={() => walletStore.connect()} variant="outline" size="sm">
              Try Again
            </Button>
          </div>
        </div>
      {/if}

      {#if walletStore.error && !walletStore.isConnected}
        <div class="rounded-md bg-destructive/10 border border-destructive/20 p-3 text-xs text-destructive">
          {walletStore.error}
        </div>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
