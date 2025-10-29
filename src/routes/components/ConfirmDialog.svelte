<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";

  let {
    open = $bindable(false),
    title = "Confirm Action",
    description = "Are you sure you want to proceed?",
    confirmText = "Continue",
    cancelText = "Cancel",
    variant = "default" as "default" | "destructive",
    onConfirm,
    onCancel,
  }: {
    open?: boolean;
    title?: string;
    description?: string;
    confirmText?: string;
    cancelText?: string;
    variant?: "default" | "destructive";
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();
</script>

<AlertDialog.Root bind:open>
  <AlertDialog.Portal>
    <AlertDialog.Overlay />
    <AlertDialog.Content>
      <AlertDialog.Header>
        <AlertDialog.Title>{title}</AlertDialog.Title>
        <AlertDialog.Description>
          {description}
        </AlertDialog.Description>
      </AlertDialog.Header>
      <AlertDialog.Footer>
        <Button variant="outline" onclick={onCancel}>
          {cancelText}
        </Button>
        <Button variant={variant} onclick={onConfirm}>
          {confirmText}
        </Button>
      </AlertDialog.Footer>
    </AlertDialog.Content>
  </AlertDialog.Portal>
</AlertDialog.Root>
