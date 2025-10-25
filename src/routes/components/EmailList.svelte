<script lang="ts">
  import type { EmailHeader } from "../lib/types";
  import { formatLocalDateTime } from "../lib/utils";
  import { Card } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Badge } from "$lib/components/ui/badge";

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
    <p class="flex flex-1 items-center justify-center text-sm text-muted-foreground">Loading emails...</p>
  {:else if error && emails.length === 0}
    <p class="flex flex-1 items-center justify-center text-sm text-destructive">{error}</p>
  {:else if emails.length > 0}
    <ScrollArea class="flex-1 p-2">
      <div class="space-y-2">
        {#each emails as email (email.uid)}
          <Card
            class="cursor-pointer transition-colors hover:bg-accent {email.uid === selectedEmailUid ? 'border-l-4 border-l-primary bg-accent' : ''}"
            onclick={() => onEmailClick(email.uid)}
          >
            <div class="p-3">
              <div class="flex items-start gap-2">
                <div class="flex flex-wrap items-center gap-1">
                  {#if !email.seen}
                    <Badge variant="default" class="h-2 w-2 rounded-full bg-blue-500 p-0" title="Unread email" />
                  {/if}
                  {#if email.has_attachments}
                    <span class="text-sm opacity-70" title="This email has attachments">📎</span>
                  {/if}
                  {#if isCcRecipient(email)}
                    <Badge variant="secondary" class="text-xs" title="You received this as CC">CC</Badge>
                  {/if}
                </div>
                <div class="flex-1 min-w-0">
                  <div class="text-sm {!email.seen ? 'font-bold' : 'font-medium'}">{email.from}</div>
                  <div class="text-sm {!email.seen ? 'font-bold' : ''} truncate">{email.subject}</div>
                </div>
                <time class="shrink-0 text-xs text-muted-foreground">{formatLocalDateTime(email.timestamp)}</time>
              </div>
            </div>
          </Card>
        {/each}
      </div>
    </ScrollArea>
  {:else if selectedAccountId}
    <p class="flex flex-1 items-center justify-center text-sm text-muted-foreground">No emails found in this inbox.</p>
  {/if}
</div>
