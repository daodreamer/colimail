<script lang="ts">
  import type { EmailHeader, Folder } from "../lib/types";
  import { formatLocalDateTime } from "../lib/utils";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import * as ContextMenu from "$lib/components/ui/context-menu";
  import { Badge } from "$lib/components/ui/badge";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Label } from "$lib/components/ui/label";
  import { Switch } from "$lib/components/ui/switch";
  import Pagination from "./Pagination.svelte";
  import { Mail, Trash2, Star, Eye, EyeOff } from "lucide-svelte";

  // Props
  let {
    emails = [] as EmailHeader[],
    selectedEmailUid = null as number | null,
    isLoading = false,
    error = null as string | null,
    selectedAccountId = null as number | null,
    selectedFolderName = "INBOX",
    folders = [] as Folder[],
    currentUserEmail = "",
    currentPage = 1,
    pageSize = 50,
    onEmailClick,
    onPageChange,
    onStarToggle,
    onMarkAsRead,
    onMarkAsUnread,
    onDeleteEmail,
    openContextMenuType = null,
    openContextMenuId = null,
    onContextMenuChange,
  }: {
    emails?: EmailHeader[];
    selectedEmailUid?: number | null;
    isLoading?: boolean;
    error?: string | null;
    selectedAccountId?: number | null;
    selectedFolderName?: string;
    folders?: Folder[];
    currentUserEmail?: string;
    currentPage?: number;
    pageSize?: number;
    onEmailClick: (uid: number) => void;
    onPageChange: (page: number) => void;
    onStarToggle: (uid: number, flagged: boolean) => void;
    onMarkAsRead?: (uid: number) => void;
    onMarkAsUnread?: (uid: number) => void;
    onDeleteEmail?: (uid: number) => void;
    openContextMenuType?: 'folder' | 'email' | null;
    openContextMenuId?: string | number | null;
    onContextMenuChange?: (type: 'folder' | 'email' | null, id: string | number | null) => void;
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
  let searchQuery = $state("");

  const filteredEmails = $derived(() => {
    let result = showUnreadsOnly ? emails.filter((email) => !email.seen) : emails;
    
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      result = result.filter((email) => 
        email.subject.toLowerCase().includes(query) ||
        email.from.toLowerCase().includes(query) ||
        email.to.toLowerCase().includes(query)
      );
    }
    
    return result;
  });

  // Pagination calculations
  const totalPages = $derived(Math.ceil(filteredEmails().length / pageSize));
  const paginatedEmails = $derived(() => {
    const start = (currentPage - 1) * pageSize;
    const end = start + pageSize;
    return filteredEmails().slice(start, end);
  });

  // If current page exceeds total pages due to filtering, reset to page 1
  $effect(() => {
    if (totalPages > 0 && currentPage > totalPages) {
      onPageChange(1);
    }
  });

  // Get the display name for the current folder
  const currentFolderDisplayName = $derived(
    folders.find((f) => f.name === selectedFolderName)?.display_name || selectedFolderName || "Inbox"
  );

  // Derived state: check if this email's context menu is open
  const isEmailContextMenuOpen = $derived((uid: number) => {
    return openContextMenuType === 'email' && openContextMenuId === uid;
  });
</script>

<Sidebar.Root collapsible="none" class="hidden flex-1 md:flex">
  <Sidebar.Header class="gap-1 border-b p-1">
    <div class="flex w-full items-center justify-between">
      <div class="text-base font-medium text-foreground">
        {currentFolderDisplayName}
      </div>
      <Label class="flex items-center gap-2 text-sm">
        <span>Unreads</span>
        <Switch bind:checked={showUnreadsOnly} class="shadow-none" />
      </Label>
    </div>
    <Sidebar.Input bind:value={searchQuery} placeholder="Type to search..." />
    
    <!-- Pagination component -->
    <Pagination 
      currentPage={currentPage}
      totalPages={totalPages}
      pageSize={pageSize}
      totalItems={filteredEmails().length}
      onPageChange={onPageChange}
    />
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
        {:else if paginatedEmails().length > 0}
          {#each paginatedEmails() as email (email.uid)}
            <ContextMenu.Root
              open={isEmailContextMenuOpen(email.uid)}
              onOpenChange={(isOpen) => {
                if (onContextMenuChange) {
                  onContextMenuChange(isOpen ? 'email' : null, isOpen ? email.uid : null);
                }
              }}
            >
              <ContextMenu.Trigger class="w-full">
                <button
                  onclick={() => onEmailClick(email.uid)}
                  class="flex w-full flex-col items-start gap-2 whitespace-nowrap border-b p-4 text-sm leading-tight last:border-b-0 text-left transition-colors {email.uid === selectedEmailUid ? 'bg-sidebar-accent text-sidebar-accent-foreground' : 'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
                >
                  <div class="flex w-[252px] items-center gap-2">
                    <span class="{!email.seen ? 'font-semibold' : ''} flex min-w-0 flex-1 items-center gap-1.5">
                      {#if !email.seen}
                        <span class="h-2 w-2 rounded-full bg-primary shrink-0" title="Unread email"></span>
                      {/if}
                      {#if isCcRecipient(email)}
                        <Badge variant="secondary" class="shrink-0 text-xs px-1 py-0 h-4" title="You received this as CC">CC</Badge>
                      {/if}
                      {#if email.has_attachments}
                        <span class="shrink-0 text-md opacity-100" title="This email has attachments">📎</span>
                      {/if}
                      <span
                        onclick={(e) => {
                          e.stopPropagation();
                          onStarToggle(email.uid, !email.flagged);
                        }}
                        role="button"
                        tabindex="0"
                        onkeydown={(e) => {
                          if (e.key === 'Enter' || e.key === ' ') {
                            e.stopPropagation();
                            e.preventDefault();
                            onStarToggle(email.uid, !email.flagged);
                          }
                        }}
                        class="shrink-0 text-md opacity-70 hover:opacity-100 transition-opacity cursor-pointer"
                        title={email.flagged ? "Remove star" : "Add star"}
                      >
                        {email.flagged ? "⭐" : "☆"}
                      </span>
                      <span class="truncate text-xs">{email.from}</span>
                    </span>
                    <span class="ml-auto shrink-0 text-xs">{formatLocalDateTime(email.timestamp)}</span>
                  </div>
                  <span class="line-clamp-1 w-[252px] whitespace-normal font-medium {!email.seen ? 'font-semibold' : ''}">{email.subject}</span>
                </button>
              </ContextMenu.Trigger>
              <ContextMenu.Content class="w-56">
                <ContextMenu.Item
                  onclick={() => {
                    onEmailClick(email.uid);
                    if (onContextMenuChange) {
                      onContextMenuChange(null, null);
                    }
                  }}
                >
                  <Mail class="mr-2 size-4" />
                  Open Email
                </ContextMenu.Item>
                <ContextMenu.Separator />
                {#if email.seen && onMarkAsUnread}
                  <ContextMenu.Item
                    onclick={() => {
                      onMarkAsUnread(email.uid);
                      if (onContextMenuChange) {
                        onContextMenuChange(null, null);
                      }
                    }}
                  >
                    <EyeOff class="mr-2 size-4" />
                    Mark as Unread
                  </ContextMenu.Item>
                {:else if !email.seen && onMarkAsRead}
                  <ContextMenu.Item
                    onclick={() => {
                      onMarkAsRead(email.uid);
                      if (onContextMenuChange) {
                        onContextMenuChange(null, null);
                      }
                    }}
                  >
                    <Eye class="mr-2 size-4" />
                    Mark as Read
                  </ContextMenu.Item>
                {/if}
                <ContextMenu.Item
                  onclick={() => {
                    onStarToggle(email.uid, !email.flagged);
                    if (onContextMenuChange) {
                      onContextMenuChange(null, null);
                    }
                  }}
                >
                  <Star class="mr-2 size-4" />
                  {email.flagged ? "Remove Star" : "Add Star"}
                </ContextMenu.Item>
                {#if onDeleteEmail}
                  <ContextMenu.Separator />
                  <ContextMenu.Item
                    onclick={() => {
                      onDeleteEmail(email.uid);
                      if (onContextMenuChange) {
                        onContextMenuChange(null, null);
                      }
                    }}
                    class="text-destructive focus:text-destructive"
                  >
                    <Trash2 class="mr-2 size-4" />
                    Delete Email
                  </ContextMenu.Item>
                {/if}
              </ContextMenu.Content>
            </ContextMenu.Root>
          {/each}
        {:else if searchQuery.trim()}
          <div class="p-4 text-center text-sm text-muted-foreground">No emails match your search</div>
        {:else if showUnreadsOnly}
          <div class="p-4 text-center text-sm text-muted-foreground">No unread emails</div>
        {:else if selectedAccountId}
          <div class="p-4 text-center text-sm text-muted-foreground">No emails found in this folder.</div>
        {/if}
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
</Sidebar.Root>
