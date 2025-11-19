<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "$lib/components/ui/dialog";

  interface Props {
    open: boolean;
    session: WalletSession | null;
    onConfirm: () => void;
    onReject: () => void;
  }

  let { open = $bindable(false), session, onConfirm, onReject }: Props = $props();

  let dontAskAgain = $state(false);

  function formatTimestamp(timestamp: number): string {
    const diff = Math.floor(Date.now() / 1000) - timestamp;
    if (diff < 60) return `${diff} seconds ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)} minutes ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)} hours ago`;
    return `${Math.floor(diff / 86400)} days ago`;
  }

  function formatAddress(addr: string): string {
    return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  }

  function handleConfirm() {
    if (dontAskAgain) {
      // TODO: Implement "don't ask again for 7 days" logic
      console.log("User chose to not ask again for 7 days");
    }
    onConfirm();
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-md">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <span class="text-2xl">🔐</span>
        Wallet Connection Detected
      </DialogTitle>
      <DialogDescription>
        A previously connected wallet session is still active
      </DialogDescription>
    </DialogHeader>

    {#if session}
      <div class="space-y-4 py-4">
        <!-- Session Info -->
        <div class="rounded-lg border bg-muted/40 p-4 space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-sm text-muted-foreground">Wallet Address</span>
            <code class="text-sm font-mono">{formatAddress(session.address)}</code>
          </div>

          <div class="flex items-center justify-between">
            <span class="text-sm text-muted-foreground">Connection Method</span>
            <span class="text-sm font-medium">
              📱 WalletConnect
            </span>
          </div>

          <div class="flex items-center justify-between">
            <span class="text-sm text-muted-foreground">Last Activity</span>
            <span class="text-sm">{formatTimestamp(session.last_active_timestamp)}</span>
          </div>
        </div>

        <!-- Warning -->
        <div class="rounded-lg bg-amber-50 border border-amber-200 p-3">
          <p class="text-xs text-amber-800">
            ⚠️ If this is not your device or you're unsure about this connection, click "Disconnect"
          </p>
        </div>

        <!-- Don't ask again option -->
        <div class="flex items-center gap-2">
          <input
            type="checkbox"
            id="dont-ask"
            bind:checked={dontAskAgain}
            class="h-4 w-4 rounded border-gray-300"
          />
          <label for="dont-ask" class="text-sm text-muted-foreground cursor-pointer">
            Don't ask again for 7 days
          </label>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-3 justify-end">
        <Button variant="outline" onclick={onReject}>
          Disconnect
        </Button>
        <Button onclick={handleConfirm}>
          Continue Using
        </Button>
      </div>
    {/if}
  </DialogContent>
</Dialog>
