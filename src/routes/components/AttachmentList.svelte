<script lang="ts">
  import type { AttachmentInfo } from "../lib/types";
  import { formatFileSize } from "../lib/utils";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";

  // Props
  let {
    attachments = [] as AttachmentInfo[],
    isLoading = false,
    onDownload,
  }: {
    attachments?: AttachmentInfo[];
    isLoading?: boolean;
    onDownload: (attachmentId: number, filename: string) => void;
  } = $props();
</script>

{#if isLoading}
  <div class="flex-shrink-0 border-b bg-muted/40 p-4">
    <h3 class="mb-2 text-sm font-semibold">Attachments</h3>
    <p class="text-xs text-muted-foreground">Loading attachments...</p>
  </div>
{:else if attachments.length > 0}
  <div class="flex-shrink-0 border-b bg-muted/40 p-4">
    <h3 class="mb-3 text-sm font-semibold">
      📎 Attachments
      <Badge variant="secondary" class="ml-2">{attachments.length}</Badge>
    </h3>
    <div class="space-y-2">
      {#each attachments as attachment (attachment.id)}
        <Button
          variant="outline"
          class="h-auto w-full justify-start gap-3 p-3"
          onclick={() => onDownload(attachment.id, attachment.filename)}
        >
          <span class="text-xl">📎</span>
          <div class="flex flex-1 flex-col items-start gap-1 overflow-hidden">
            <span class="w-full truncate text-sm font-medium">{attachment.filename}</span>
            <span class="text-xs text-muted-foreground">{formatFileSize(attachment.size)}</span>
          </div>
          <span class="shrink-0 text-xl text-primary">⬇</span>
        </Button>
      {/each}
    </div>
  </div>
{/if}
