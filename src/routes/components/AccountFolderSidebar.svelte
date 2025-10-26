<script lang="ts">
  import { Mail, Folder as FolderIcon, Plus, RefreshCw } from "lucide-svelte";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Badge } from "$lib/components/ui/badge";
  import type { AccountConfig, Folder } from "../lib/types";
  import NavUser from "$lib/components/nav-user.svelte";

  // Props
  let {
    accounts = [] as AccountConfig[],
    selectedAccountId = null as number | null,
    folders = [] as Folder[],
    selectedFolderName = "INBOX",
    isLoadingFolders = false,
    isSyncing = false,
    onAccountSelect,
    onFolderClick,
    onAddAccount,
    onSettings,
    onSyncMail,
  }: {
    accounts?: AccountConfig[];
    selectedAccountId?: number | null;
    folders?: Folder[];
    selectedFolderName?: string;
    isLoadingFolders?: boolean;
    isSyncing?: boolean;
    onAccountSelect: (accountId: number) => void;
    onFolderClick: (folderName: string) => void;
    onAddAccount: () => void;
    onSettings: () => void;
    onSyncMail: () => void;
  } = $props();

  // Get selected account
  const selectedAccount = $derived(
    accounts.find((acc) => acc.id === selectedAccountId)
  );

  // User data for NavUser component
  const userData = $derived({
    name: "User Account",
    email: selectedAccount?.email || "No account",
    avatar: "/avatars/user.jpg",
  });
</script>

<Sidebar.Root collapsible="none" class="!w-[calc(var(--sidebar-width-icon)_+_1px)] border-r">
  <Sidebar.Header>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            {#snippet child({ props })}
              <Sidebar.MenuButton
                {...props}
                size="lg"
                class="md:h-8 md:p-0 data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
              >
                <div
                  class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground"
                >
                  <Mail class="size-4" />
                </div>
                <div class="grid flex-1 text-left text-sm leading-tight">
                  <span class="truncate font-medium">Colimail</span>
                  <span class="truncate text-xs">Email Client</span>
                </div>
              </Sidebar.MenuButton>
            {/snippet}
          </DropdownMenu.Trigger>
          <DropdownMenu.Content class="w-[200px]" align="start">
            {#each accounts as account (account.id)}
              <DropdownMenu.Item onclick={() => onAccountSelect(account.id)}>
                <div class="flex items-center gap-2 w-full">
                  <Mail class="size-4" />
                  <span class="truncate flex-1">{account.email}</span>
                  {#if account.id === selectedAccountId}
                    <Badge variant="default" class="ml-auto">Active</Badge>
                  {/if}
                </div>
              </DropdownMenu.Item>
            {/each}
            <DropdownMenu.Separator />
            <DropdownMenu.Item onclick={onAddAccount}>
              <Plus class="size-4" />
              <span>Add Account</span>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Header>

  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupContent class="px-1.5 md:px-0">
        <Sidebar.Menu>
          {#if isLoadingFolders}
            {#each Array(6) as _, i (i)}
              <Sidebar.MenuItem>
                <Sidebar.MenuSkeleton />
              </Sidebar.MenuItem>
            {/each}
          {:else if folders.length > 0}
            {#each folders as folder (folder.name)}
              <Sidebar.MenuItem>
                <Sidebar.MenuButton
                  tooltipContentProps={{
                    hidden: false,
                  }}
                  onclick={() => onFolderClick(folder.name)}
                  isActive={folder.name === selectedFolderName}
                  class="px-2.5 md:px-2"
                >
                  {#snippet tooltipContent()}
                    {folder.display_name}
                  {/snippet}
                  <FolderIcon class="size-4" />
                  <span>{folder.display_name}</span>
                </Sidebar.MenuButton>
              </Sidebar.MenuItem>
            {/each}
          {:else if selectedAccountId}
            <Sidebar.MenuItem>
              <div class="px-2 py-1.5 text-sm text-muted-foreground">
                No folders found
              </div>
            </Sidebar.MenuItem>
          {:else}
            <Sidebar.MenuItem>
              <div class="px-2 py-1.5 text-sm text-muted-foreground">
                Select an account
              </div>
            </Sidebar.MenuItem>
          {/if}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>

  <Sidebar.Footer>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton
          onclick={!selectedAccountId || isSyncing ? undefined : onSyncMail}
          tooltipContentProps={{
            hidden: false,
          }}
          class="px-2.5 md:px-2 {!selectedAccountId || isSyncing ? 'opacity-50 cursor-not-allowed' : ''}"
        >
          {#snippet tooltipContent()}
            {isSyncing ? "Syncing..." : "Sync Mail"}
          {/snippet}
          <RefreshCw class="size-4 {isSyncing ? 'animate-spin' : ''}" />
          <span>{isSyncing ? "Syncing..." : "Sync Mail"}</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
    <NavUser user={userData} />
  </Sidebar.Footer>
</Sidebar.Root>
