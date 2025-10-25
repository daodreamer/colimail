<script lang="ts">
  import type { EmailHeader, AttachmentInfo } from "../lib/types";
  import { formatFullLocalDateTime } from "../lib/utils";
  import AttachmentList from "./AttachmentList.svelte";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Separator } from "$lib/components/ui/separator";

  // Props
  let {
    email = null as EmailHeader | null,
    body = null as string | null,
    attachments = [] as AttachmentInfo[],
    isLoadingBody = false,
    isLoadingAttachments = false,
    error = null as string | null,
    onReply,
    onForward,
    onDelete,
    onDownloadAttachment,
    onToggleRead,
  }: {
    email?: EmailHeader | null;
    body?: string | null;
    attachments?: AttachmentInfo[];
    isLoadingBody?: boolean;
    isLoadingAttachments?: boolean;
    error?: string | null;
    onReply: () => void;
    onForward: () => void;
    onDelete: () => void;
    onDownloadAttachment: (attachmentId: number, filename: string) => void;
    onToggleRead: () => void;
  } = $props();
</script>

<main class="flex h-screen flex-col">
  {#if isLoadingBody}
    <p class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Loading email content...</p>
  {:else if email && body}
    <div class="flex-shrink-0 border-b bg-muted/40 p-6">
      <h2 class="mb-4 text-2xl font-semibold">{email.subject}</h2>
      
      <div class="space-y-2 text-sm">
        <div class="flex gap-2">
          <span class="w-16 font-semibold text-muted-foreground">From:</span>
          <span class="break-words">{email.from}</span>
        </div>
        <div class="flex gap-2">
          <span class="w-16 font-semibold text-muted-foreground">To:</span>
          <span class="break-words">{email.to}</span>
        </div>
        {#if email.cc && email.cc.trim()}
          <div class="flex gap-2">
            <span class="w-16 font-semibold text-muted-foreground">CC:</span>
            <span class="break-words">{email.cc}</span>
          </div>
        {/if}
        <div class="flex gap-2">
          <span class="w-16 font-semibold text-muted-foreground">Date:</span>
          <span>{formatFullLocalDateTime(email.timestamp)}</span>
        </div>
      </div>

      <div class="mt-4 flex flex-wrap gap-2">
        <Button variant="default" size="sm" onclick={onReply}>↩ Reply</Button>
        <Button variant="default" size="sm" class="bg-green-600 hover:bg-green-700" onclick={onForward}>
          ➡ Forward
        </Button>
        <Button variant="secondary" size="sm" onclick={onToggleRead}>
          {email.seen ? "✉ Mark Unread" : "✅ Mark Read"}
        </Button>
        <Button variant="destructive" size="sm" onclick={onDelete}>🗑 Delete</Button>
      </div>
    </div>

    <AttachmentList
      {attachments}
      isLoading={isLoadingAttachments}
      onDownload={onDownloadAttachment}
    />

    <ScrollArea class="flex-1 p-6">
      <div class="prose prose-sm max-w-none dark:prose-invert">
        {@html body}
      </div>
    </ScrollArea>
  {:else if error}
    <div class="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
      <p class="text-lg font-semibold text-destructive">⚠️ Error loading email</p>
      <p class="text-sm text-muted-foreground">{error}</p>
      <p class="text-xs italic text-muted-foreground">
        The email may have been deleted or moved. Please try refreshing the folder.
      </p>
    </div>
  {:else if email && !body}
    <div class="flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center">
      <p class="text-sm text-muted-foreground">Email selected but content not loaded yet...</p>
      <p class="text-xs text-muted-foreground">If this persists, try selecting another email.</p>
    </div>
  {:else}
    <div class="flex flex-1 items-center justify-center p-8">
      <p class="text-sm text-muted-foreground">Select an email to read its content.</p>
    </div>
  {/if}
</main>
