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

  // Resizable dialog state
  let dialogElement = $state<HTMLDivElement | null>(null);
  let isResizing = $state(false);
  let resizeDirection = $state<'se' | 'e' | 's' | null>(null);
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

  function startResize(event: MouseEvent, direction: 'se' | 'e' | 's') {
    event.preventDefault();
    event.stopPropagation();

    isResizing = true;
    resizeDirection = direction;
    startX = event.clientX;
    startY = event.clientY;
    startWidth = dialogWidth;
    startHeight = dialogHeight;

    document.addEventListener('mousemove', handleResize);
    document.addEventListener('mouseup', stopResize);
    document.body.style.cursor = direction === 'se' ? 'nwse-resize' : direction === 'e' ? 'ew-resize' : 'ns-resize';
    document.body.style.userSelect = 'none';
  }

  function handleResize(event: MouseEvent) {
    if (!isResizing || !resizeDirection) return;

    const { maxWidth, maxHeight, minWidth, minHeight } = getViewportConstraints();
    const deltaX = event.clientX - startX;
    const deltaY = event.clientY - startY;

    if (resizeDirection === 'se' || resizeDirection === 'e') {
      const newWidth = Math.max(minWidth, Math.min(maxWidth, startWidth + deltaX));
      dialogWidth = newWidth;
    }

    if (resizeDirection === 'se' || resizeDirection === 's') {
      const newHeight = Math.max(minHeight, Math.min(maxHeight, startHeight + deltaY));
      dialogHeight = newHeight;
    }
  }

  function stopResize() {
    isResizing = false;
    resizeDirection = null;
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
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

<Dialog open={show} onOpenChange={(open) => { if (!open) onCancel(); }}>
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
      onmousedown={(e) => startResize(e, 'e')}
    ></div>
    <div
      class="resize-handle resize-handle-s"
      role="button"
      tabindex="-1"
      aria-label="Resize vertically"
      onmousedown={(e) => startResize(e, 's')}
    ></div>
    <div
      class="resize-handle resize-handle-se"
      role="button"
      tabindex="-1"
      aria-label="Resize diagonally"
      onmousedown={(e) => startResize(e, 'se')}
    ></div>

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
            {#each attachments as file, index (file.name + file.size + index)}
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
    content: '';
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
