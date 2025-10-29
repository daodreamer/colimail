<script lang="ts">
  import type { DraftListItem } from "../lib/types";
  import { formatLocalDateTime } from "../lib/utils";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import { Badge } from "$lib/components/ui/badge";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Label } from "$lib/components/ui/label";
  import Trash2Icon from "lucide-svelte/icons/trash-2";
  import Pagination from "./Pagination.svelte";

  let {
    drafts = [],
    selectedDraftId = null,
    isLoading = false,
    onDraftClick,
    onDraftDelete,
  }: {
    drafts?: DraftListItem[];
    selectedDraftId?: number | null;
    isLoading?: boolean;
    onDraftClick: (draftId: number) => void;
    onDraftDelete: (draftId: number) => void;
  } = $props();

  function getDraftTypeLabel(type: string): string {
    switch (type) {
      case "reply":
        return "Reply";
      case "forward":
        return "Forward";
      default:
        return "Draft";
    }
  }

  function getDraftTypeColor(type: string): "default" | "secondary" | "outline" {
    switch (type) {
      case "reply":
        return "secondary";
      case "forward":
        return "outline";
      default:
        return "default";
    }
  }

  let searchQuery = $state("");
  let currentPage = $state(1);
  const pageSize = 50;

  const filteredDrafts = $derived(() => {
    if (!searchQuery.trim()) return drafts;

    const query = searchQuery.toLowerCase();
    return drafts.filter((draft) =>
      draft.subject.toLowerCase().includes(query) ||
      draft.to_addr.toLowerCase().includes(query) ||
      (draft.cc_addr && draft.cc_addr.toLowerCase().includes(query))
    );
  });

  // Pagination calculations
  const totalPages = $derived(Math.ceil(filteredDrafts().length / pageSize));
  const paginatedDrafts = $derived(() => {
    const start = (currentPage - 1) * pageSize;
    const end = start + pageSize;
    return filteredDrafts().slice(start, end);
  });

  // If current page exceeds total pages due to filtering, reset to page 1
  $effect(() => {
    if (totalPages > 0 && currentPage > totalPages) {
      currentPage = 1;
    }
  });

  function handlePageChange(page: number) {
    currentPage = page;
  }
</script>

<Sidebar.Root collapsible="none" class="hidden flex-1 md:flex">
  <Sidebar.Header class="gap-1 border-b p-1">
    <div class="flex w-full items-center justify-between">
      <div class="text-base font-medium text-foreground">
        Drafts
      </div>
    </div>
    <Sidebar.Input bind:value={searchQuery} placeholder="Type to search..." />

    <!-- Pagination component -->
    <Pagination
      currentPage={currentPage}
      totalPages={totalPages}
      pageSize={pageSize}
      totalItems={filteredDrafts().length}
      onPageChange={handlePageChange}
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
        {:else if paginatedDrafts().length > 0}
          {#each paginatedDrafts() as draft (draft.id)}
            <button
              onclick={() => onDraftClick(draft.id)}
              class="flex w-full flex-col items-start gap-2 whitespace-nowrap border-b p-4 text-sm leading-tight last:border-b-0 text-left transition-colors group {draft.id === selectedDraftId ? 'bg-sidebar-accent text-sidebar-accent-foreground' : 'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'}"
            >
              <div class="flex w-[252px] items-center gap-2">
                <span class="flex min-w-0 flex-1 items-center gap-1.5">
                  <Badge variant={getDraftTypeColor(draft.draft_type)} class="shrink-0 text-xs px-1 py-0 h-4">
                    {getDraftTypeLabel(draft.draft_type)}
                  </Badge>
                  <span
                    onclick={(e) => {
                      e.stopPropagation();
                      onDraftDelete(draft.id);
                    }}
                    role="button"
                    tabindex="0"
                    onkeydown={(e) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        e.stopPropagation();
                        e.preventDefault();
                        onDraftDelete(draft.id);
                      }
                    }}
                    class="shrink-0 opacity-0 group-hover:opacity-70 hover:opacity-100 transition-opacity cursor-pointer"
                    title="Delete draft"
                  >
                    <Trash2Icon class="h-3.5 w-3.5" />
                  </span>
                  <span class="truncate text-xs">To: {draft.to_addr || "(no recipient)"}</span>
                </span>
                <span class="ml-auto shrink-0 text-xs">{formatLocalDateTime(draft.updated_at)}</span>
              </div>
              <span class="line-clamp-1 w-[252px] whitespace-normal font-medium">
                {draft.subject || "(no subject)"}
              </span>
            </button>
          {/each}
        {:else if searchQuery.trim()}
          <div class="p-4 text-center text-sm text-muted-foreground">No drafts match your search</div>
        {:else}
          <div class="p-4 text-center text-sm text-muted-foreground">
            <p class="mb-1 text-sm font-medium">No drafts</p>
            <p class="text-xs">Your saved drafts will appear here</p>
          </div>
        {/if}
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>
</Sidebar.Root>
