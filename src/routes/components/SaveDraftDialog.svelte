<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
    DialogDescription,
  } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import * as ButtonGroup from "$lib/components/ui/button-group";

  let {
    show = false,
    onSave,
    onDiscard,
    onCancel,
  }: {
    show?: boolean;
    onSave: () => void;
    onDiscard: () => void;
    onCancel: () => void;
  } = $props();
</script>

<Dialog open={show} onOpenChange={(open) => { if (!open) onCancel(); }}>
  <DialogContent class="max-w-md">
    <DialogHeader>
      <DialogTitle>Save Draft?</DialogTitle>
      <DialogDescription>
        Do you want to save this email as a draft before closing?
      </DialogDescription>
    </DialogHeader>

    <DialogFooter class="gap-2 sm:gap-2">
      <ButtonGroup.Root class="w-full">
        <Button variant="destructive" onclick={onDiscard} class="flex-1">
          Discard
        </Button>
        <Button variant="outline" onclick={onCancel} class="flex-1">
          Cancel
        </Button>
        <Button variant="default" onclick={onSave} class="flex-1">
          Save Draft
        </Button>
      </ButtonGroup.Root>
    </DialogFooter>
  </DialogContent>
</Dialog>
