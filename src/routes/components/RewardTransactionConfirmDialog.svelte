<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Dialog, DialogContent, DialogHeader, DialogTitle } from "$lib/components/ui/dialog";

  interface Props {
    open: boolean;
    amount: string;
    recipientAddress: string;
    recipientEmail: string;
    emailSubject: string;
    estimatedGas?: string;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    open = $bindable(false),
    amount,
    recipientAddress,
    recipientEmail,
    emailSubject,
    estimatedGas = "~0.0015",
    onConfirm,
    onCancel
  }: Props = $props();

  function formatAddress(addr: string): string {
    return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  }
</script>

<Dialog bind:open>
  <DialogContent class="max-w-md">
    <DialogHeader>
      <DialogTitle class="flex items-center gap-2">
        <span class="text-2xl">💰</span>
        Confirm Reward Transaction
      </DialogTitle>
    </DialogHeader>

    <div class="space-y-4 py-4">
      <!-- Transaction Details -->
      <div class="rounded-lg border bg-muted/40 p-4 space-y-3">
        <div>
          <div class="text-xs text-muted-foreground mb-1">Reward Amount</div>
          <div class="text-2xl font-bold">{amount} wACT</div>
        </div>

        <div class="h-px bg-border"></div>

        <div>
          <div class="text-xs text-muted-foreground mb-1">Recipient Address</div>
          <code class="text-sm font-mono">{formatAddress(recipientAddress)}</code>
        </div>

        <div>
          <div class="text-xs text-muted-foreground mb-1">Estimated Gas Fee</div>
          <div class="text-sm">{estimatedGas} ETH</div>
        </div>
      </div>

      <!-- Email Context -->
      <div class="rounded-lg bg-blue-50 border border-blue-200 p-3 space-y-2">
        <div class="text-xs font-medium text-blue-900">📧 Email Information</div>
        <div class="space-y-1">
          <div class="text-xs text-blue-800">
            <span class="font-medium">Recipient:</span> {recipientEmail}
          </div>
          <div class="text-xs text-blue-800">
            <span class="font-medium">Subject:</span> {emailSubject}
          </div>
        </div>
      </div>

      <!-- Warning -->
      <div class="rounded-lg bg-amber-50 border border-amber-200 p-3">
        <p class="text-xs text-amber-800">
          ⚠️ Please carefully verify the recipient address and amount. Blockchain transactions are irreversible.
        </p>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex gap-3 justify-end">
      <Button variant="outline" onclick={onCancel}>
        Cancel
      </Button>
      <Button onclick={onConfirm}>
        Confirm Transaction
      </Button>
    </div>
  </DialogContent>
</Dialog>
