<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { walletStore } from "$lib/stores/wallet.svelte";
  import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";
  import CheckCircleIcon from "@lucide/svelte/icons/check-circle";
  import WalletIcon from "@lucide/svelte/icons/wallet";
  import SmartphoneIcon from "@lucide/svelte/icons/smartphone";
  import { scale, fly, fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

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
  let hasShownError = $state(false); // Track if we've shown an error

  // Start connection when dialog opens
  $effect(() => {
    if (open && !walletStore.isConnected && !walletStore.isConnecting && !isCancelling && !connectionAttempted) {
      console.log("Dialog opened - initiating connection");
      connectionAttempted = true;
      hasShownError = false;
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

  // Track error state and auto-close dialog after error is cleared
  $effect(() => {
    if (walletStore.error && open) {
      // Mark that we've shown an error
      hasShownError = true;
    } else if (hasShownError && !walletStore.error && !walletStore.isConnecting && !walletStore.isConnected && open) {
      // Error was cleared - close the dialog
      console.log("Error cleared - closing dialog");
      setTimeout(() => {
        open = false;
        if (onOpenChange) onOpenChange(false);
        // Reset states
        connectionAttempted = false;
        hasShownError = false;
      }, 500); // Small delay to show the cleared state
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
      hasShownError = false;
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
    hasShownError = false;

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
        <div
          class="flex flex-col items-center justify-center py-8 space-y-4"
          in:scale={{ duration: 400, easing: cubicOut, start: 0.8 }}
          out:fade={{ duration: 200 }}
        >
          <div
            class="rounded-full bg-green-100 dark:bg-green-900/30 p-3"
            in:scale={{ duration: 600, delay: 200, easing: cubicOut, start: 0 }}
          >
            <CheckCircleIcon class="size-12 text-green-600 dark:text-green-400" />
          </div>
          <div
            class="text-center space-y-1"
            in:fly={{ y: 10, duration: 400, delay: 300, easing: cubicOut }}
          >
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
        <div
          class="space-y-4"
          in:fly={{ y: 20, duration: 400, easing: cubicOut }}
          out:fade={{ duration: 200 }}
        >
          <!-- QR Code -->
          <div
            class="flex justify-center p-4 bg-white dark:bg-muted rounded-lg border-2 border-dashed border-primary/30"
            in:scale={{ duration: 400, delay: 100, easing: cubicOut, start: 0.9 }}
          >
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
      {:else if connectionAttempted && !walletStore.error}
        <!-- Loading State (initial connection) -->
        <div
          class="flex flex-col items-center justify-center py-12 space-y-4"
          in:fade={{ duration: 300 }}
          out:fade={{ duration: 200 }}
        >
          <LoaderCircleIcon class="size-12 animate-spin text-primary" />
          <div class="text-center space-y-1">
            <p class="text-sm font-medium">Initializing connection...</p>
            <p class="text-xs text-muted-foreground">Please wait</p>
          </div>
        </div>
      {:else}
        <!-- Error State or Initial State -->
        <div
          class="flex flex-col items-center justify-center py-8 space-y-4"
          in:scale={{ duration: 300, easing: cubicOut, start: 0.95 }}
          out:fade={{ duration: 200 }}
        >
          {#if walletStore.error}
            <!-- User cancelled or error occurred -->
            <div class="rounded-full bg-red-100 dark:bg-red-900/30 p-3">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-12 text-red-600 dark:text-red-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </div>
            <div class="text-center space-y-3">
              <div>
                <p class="text-base font-semibold text-red-600 dark:text-red-400">
                  {walletStore.error}
                </p>
                <p class="text-xs text-muted-foreground mt-1">
                  You can try connecting again when ready
                </p>
              </div>
              <Button
                onclick={() => {
                  connectionAttempted = true;
                  walletStore.connect();
                }}
                variant="default"
                size="sm"
                class="mt-2"
              >
                Try Again
              </Button>
            </div>
          {:else}
            <!-- Initial state or unknown error -->
            <div class="rounded-full bg-amber-100 dark:bg-amber-900/30 p-3">
              <WalletIcon class="size-12 text-amber-600 dark:text-amber-400" />
            </div>
            <div class="text-center space-y-2">
              <p class="text-sm font-medium">Ready to connect</p>
              <Button
                onclick={() => {
                  connectionAttempted = true;
                  walletStore.connect();
                }}
                variant="outline"
                size="sm"
              >
                Connect Wallet
              </Button>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
