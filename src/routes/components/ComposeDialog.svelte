<script lang="ts">
  import { formatFileSize } from "../lib/utils";
  import RichTextEditor from "./RichTextEditor.svelte";
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import * as ButtonGroup from "$lib/components/ui/button-group";

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
    onSend: () => void;
    onCancel: () => void;
    onAttachmentAdd: (event: Event) => void;
    onAttachmentRemove: (index: number) => void;
  } = $props();

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

<Dialog open={show} onOpenChange={(open) => { if (!open) onCancel(); }}>
  <DialogContent
    class="max-w-2xl max-h-[90vh] flex flex-col"
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

    <div class="flex-1 space-y-4 overflow-y-auto px-1">
      {#if error}
        <div class="rounded-md border border-destructive bg-destructive/10 p-3 text-sm text-destructive">
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
            {#each attachments as file, index}
              <div class="flex items-center gap-2 rounded-md border bg-muted/40 p-2">
                <span class="flex-1 truncate text-sm">{file.name}</span>
                <Badge variant="secondary" class="text-xs">{formatFileSize(file.size)}</Badge>
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
              Total: {formatFileSize(totalAttachmentSize)} / {formatFileSize(attachmentSizeLimit)}
            </div>
          </div>
        {/if}
      </div>
    </div>

    <DialogFooter class="gap-2">
      <ButtonGroup.Root>
        <Button variant="outline" onclick={onCancel} disabled={isSending}>Cancel</Button>
        <Button variant="default" onclick={onSend} disabled={isSending}>
          {isSending ? "Sending..." : "Send"}
        </Button>
      </ButtonGroup.Root>
    </DialogFooter>
  </DialogContent>
</Dialog>
