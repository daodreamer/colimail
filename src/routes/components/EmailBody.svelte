<script lang="ts">
  import type { EmailHeader, AttachmentInfo } from "../lib/types";
  import { formatFullLocalDateTime } from "../lib/utils";
  import AttachmentList from "./AttachmentList.svelte";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Separator } from "$lib/components/ui/separator";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import * as ButtonGroup from "$lib/components/ui/button-group";

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
    <!-- Skeleton loading state -->
    <div class="flex-shrink-0 border-b bg-muted/40 p-6">
      <Skeleton class="mb-4 h-8 w-3/4" />
      
      <div class="space-y-2">
        <div class="flex gap-2">
          <Skeleton class="h-4 w-16" />
          <Skeleton class="h-4 w-48" />
        </div>
        <div class="flex gap-2">
          <Skeleton class="h-4 w-16" />
          <Skeleton class="h-4 w-64" />
        </div>
        <div class="flex gap-2">
          <Skeleton class="h-4 w-16" />
          <Skeleton class="h-4 w-40" />
        </div>
      </div>

      <div class="mt-6 flex items-center gap-2">
        <Skeleton class="h-8 w-32" />
        <Skeleton class="h-8 w-32" />
        <Skeleton class="h-8 w-32" />
        <Skeleton class="h-8 w-24" />
      </div>
    </div>

    <div class="flex-1 p-6">
      <div class="space-y-3">
        <Skeleton class="h-4 w-full" />
        <Skeleton class="h-4 w-full" />
        <Skeleton class="h-4 w-5/6" />
        <Skeleton class="h-4 w-full" />
        <Skeleton class="h-4 w-4/5" />
        <Skeleton class="h-4 w-full" />
        <Skeleton class="h-4 w-3/4" />
      </div>
    </div>
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

      <div class="mt-6 flex items-center gap-2">
        <ButtonGroup.Root>
          <Button variant="default" size="sm" onclick={onReply}>
            <span class="text-base">↩</span>
            <span class="ml-1.5">Reply</span>
          </Button>
          <Button variant="default" size="sm" onclick={onForward}>
            <span class="text-base">➡</span>
            <span class="ml-1.5">Forward</span>
          </Button>
        </ButtonGroup.Root>

        <ButtonGroup.Root>
          <Button variant="outline" size="sm" onclick={onToggleRead}>
            <span class="text-base">{email.seen ? "✉" : "✅"}</span>
            <span class="ml-1.5">{email.seen ? "Mark Unread" : "Mark Read"}</span>
          </Button>
        </ButtonGroup.Root>

        <Button variant="outline" size="sm" class="text-destructive hover:bg-destructive hover:text-destructive-foreground" onclick={onDelete}>
          <span class="text-base">🗑</span>
          <span class="ml-1.5">Delete</span>
        </Button>
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
