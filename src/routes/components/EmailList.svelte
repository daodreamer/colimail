<script lang="ts">
  import type { EmailHeader } from "../lib/types";
  import { formatLocalDateTime } from "../lib/utils";
  import { Card } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Badge } from "$lib/components/ui/badge";
  import { Skeleton } from "$lib/components/ui/skeleton";

  // Props
  let {
    emails = [] as EmailHeader[],
    selectedEmailUid = null as number | null,
    isLoading = false,
    error = null as string | null,
    selectedAccountId = null as number | null,
    currentUserEmail = "",
    onEmailClick,
  }: {
    emails?: EmailHeader[];
    selectedEmailUid?: number | null;
    isLoading?: boolean;
    error?: string | null;
    selectedAccountId?: number | null;
    currentUserEmail?: string;
    onEmailClick: (uid: number) => void;
  } = $props();

  // Check if the current user is a CC recipient (not in To field)
  function isCcRecipient(email: EmailHeader): boolean {
    if (!email.cc || !currentUserEmail) return false;

    // Check if current user email is in CC list
    const isInCc = email.cc.toLowerCase().includes(currentUserEmail.toLowerCase());

    // Check if current user email is NOT in To field
    const isInTo = email.to.toLowerCase().includes(currentUserEmail.toLowerCase());

    // User is CC recipient if they're in CC but not in To
    return isInCc && !isInTo;
  }
</script>

<div class="flex h-screen flex-col border-r">
  {#if isLoading}
    <ScrollArea class="flex-1 px-1.5 py-1">
      <div class="space-y-1">
        {#each Array(8) as _, i}
          <Card class="px-2 py-0.5">
            <div class="mb-0.5 flex items-center justify-between gap-2">
              <div class="flex min-w-0 flex-1 items-center gap-2">
                <Skeleton class="h-2 w-2 rounded-full" />
                <Skeleton class="h-4 w-32" />
              </div>
              <Skeleton class="h-3 w-16" />
            </div>
            <Skeleton class="h-4 w-full" />
          </Card>
        {/each}
      </div>
    </ScrollArea>
  {:else if error && emails.length === 0}
    <p class="flex flex-1 items-center justify-center text-sm text-destructive">{error}</p>
  {:else if emails.length > 0}
    <ScrollArea class="flex-1 px-1.5 py-1">
      <div class="space-y-1">
        {#each emails as email (email.uid)}
          <Card
            class="cursor-pointer transition-all hover:shadow-sm {email.uid === selectedEmailUid ? 'border-primary bg-accent shadow-sm' : 'hover:border-muted-foreground/20'}"
            onclick={() => onEmailClick(email.uid)}
          >
            <div class="px-2 py-0.5">
              <div class="mb-0.5 flex items-center justify-between gap-2">
                <div class="flex min-w-0 flex-1 items-center gap-2">
                  {#if !email.seen}
                    <Badge variant="default" class="h-2 w-2 rounded-full bg-primary p-0" title="Unread email" />
                  {/if}
                  <span class="truncate text-sm font-medium {!email.seen ? 'font-semibold' : ''}">{email.from}</span>
                  {#if isCcRecipient(email)}
                    <Badge variant="secondary" class="shrink-0 text-[10px] px-1 py-0 h-4" title="You received this as CC">CC</Badge>
                  {/if}
                  {#if email.has_attachments}
                    <span class="shrink-0 text-xs opacity-60" title="This email has attachments">📎</span>
                  {/if}
                </div>
                <time class="shrink-0 text-xs text-muted-foreground">{formatLocalDateTime(email.timestamp)}</time>
              </div>
              <div class="truncate text-sm {!email.seen ? 'font-medium' : 'text-muted-foreground'}">{email.subject}</div>
            </div>
          </Card>
        {/each}
      </div>
    </ScrollArea>
  {:else if selectedAccountId}
    <p class="flex flex-1 items-center justify-center text-sm text-muted-foreground">No emails found in this inbox.</p>
  {/if}
</div>
