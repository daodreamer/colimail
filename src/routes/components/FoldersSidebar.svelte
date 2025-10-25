<script lang="ts">
  import type { Folder } from "../lib/types";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";

  // Props
  let {
    folders = $bindable<Folder[]>([]),
    selectedFolderName = $bindable<string>("INBOX"),
    isLoading = false,
    selectedAccountId = null as number | null,
    onFolderClick,
  }: {
    folders?: Folder[];
    selectedFolderName?: string;
    isLoading?: boolean;
    selectedAccountId?: number | null;
    onFolderClick: (folderName: string) => void;
  } = $props();
</script>

<aside class="flex h-screen flex-col border-r bg-muted/40">
  <div class="border-b p-4">
    <h2 class="text-lg font-semibold">Folders</h2>
  </div>

  {#if isLoading}
    <p class="flex flex-1 items-center justify-center text-sm text-muted-foreground">
      Loading folders...
    </p>
  {:else if folders.length > 0}
    <ScrollArea class="flex-1 px-3 py-2">
      <div class="space-y-1">
        {#each folders as folder (folder.name)}
          <Button
            variant={folder.name === selectedFolderName ? "default" : "ghost"}
            class="w-full justify-start gap-2 overflow-hidden"
            onclick={() => onFolderClick(folder.name)}
            title={folder.name}
          >
            <span class="shrink-0">📁</span>
            <span class="truncate text-sm">{folder.display_name}</span>
          </Button>
        {/each}
      </div>
    </ScrollArea>
  {:else if selectedAccountId}
    <p class="flex flex-1 items-center justify-center text-sm text-muted-foreground">
      No folders found.
    </p>
  {:else}
    <p class="flex flex-1 items-center justify-center text-center text-sm text-muted-foreground px-4">
      Select an account to view folders.
    </p>
  {/if}
</aside>
