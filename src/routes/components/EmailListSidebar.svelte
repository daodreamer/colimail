<script lang="ts">
  import type { EmailHeader } from "../lib/types";
  import { formatLocalDateTime } from "../lib/utils";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import { Badge } from "$lib/components/ui/badge";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";

  // Props
  let {
    emails = [] as EmailHeader[],
    selectedEmailUid = null as number | null,
    isLoading = false,
    error = null as string | null,
    selectedAccountId = null as number | null,
    selectedFolderName = "INBOX",
    currentUserEmail = "",
    onEmailClick,
    onComposeClick,
  }: {
    emails?: EmailHeader[];
    selectedEmailUid?: number | null;
    isLoading?: boolean;
    error?: string | null;
    selectedAccountId?: number | null;
    selectedFolderName?: string;
    currentUserEmail?: string;
    onEmailClick: (uid: number) => void;
    onComposeClick: () => void;
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

  let showUnreadsOnly = $state(false);

  const filteredEmails = $derived(
    showUnreadsOnly ? emails.filter((email) => !email.seen) : emails
  );
</script>

<Sidebar.Root collapsible="none" class="hidden flex-1 md:flex">
  <Sidebar.Header class="gap-3.5 border-b p-4">
    <div class="flex w-full items-center justify-between">
      <div class="text-base font-medium text-foreground">
        {selectedFolderName || "Inbox"}
      </div>
      <Label class="flex items-center gap-2 text-sm">
        <span>Unreads</span>
        <Switch bind:checked={showUnreadsOnly} class="shadow-none" />
      </Label>
    </div>
    <button
      onclick={onComposeClick}
      disabled={!selectedAccountId}
      class="w-full rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      ✉️ Compose
    </button>
  </Sidebar.Header>

  <Sidebar.Content>
    <Sidebar.Group class="px-0">
      <Sidebar.GroupContent>
        {#if isLoading}
          {#each Array(8) as _, i (i)}
            <div class="flex flex-col items-start gap-2 border-b p-4 leading-tight last:border-b-0">
              <div class="flex w-full items-center gap-2">
                <Skeleton class="h-4 w-32" />
                <Skeleton class="ml-auto h-3 w-16" />
              </div>
              <Skeleton class="h-4 w-full" />
            </div>
          {/each}
        {:else if error && emails.length === 0}
          <div class="p-4 text-center text-sm text-destructive">{error}</div>
        {:else if filteredEmails.length > 0}
          {#each filteredEmails as email (email.uid)}
            <button
              onclick={() => onEmailClick(email.uid)}
              class="flex w-full flex-col items-start gap-2 border-b p-4 text-sm leading-tight last:border-b-0 text-left transition-colors {email.uid === selectedEmailUid ? 'bg-sidebar-accent text-sidebar-accent-foreground' : 'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
            >
              <div class="flex w-full items-center gap-2">
                <span class="{!email.seen ? 'font-semibold' : ''} flex items-center gap-1.5">
                  {#if !email.seen}
                    <span class="h-2 w-2 rounded-full bg-primary shrink-0" title="Unread email"></span>
                  {/if}
                  {email.from}
                  {#if isCcRecipient(email)}
                    <Badge variant="secondary" class="shrink-0 text-[10px] px-1 py-0 h-4" title="You received this as CC">CC</Badge>
                  {/if}
                  {#if email.has_attachments}
                    <span class="shrink-0 text-xs opacity-60" title="This email has attachments">📎</span>
                  {/if}
                </span>
                <span class="ml-auto text-xs">{formatLocalDateTime(email.timestamp)}</span>
              </div>
              <span class="font-medium {!email.seen ? 'font-semibold' : ''}">{email.subject}</span>
            </button>
          {/each}
        {:else if showUnreadsOnly}
          <div class="p-4 text-center text-sm text-muted-foreground">No unread emails</div>
        {:else if selectedAccountId}
          <div class="p-4 text-center text-sm text-muted-foreground">No emails found in this folder.</div>
        {/if}
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
</Sidebar.Root>
