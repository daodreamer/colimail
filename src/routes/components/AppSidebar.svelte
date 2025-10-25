<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Separator } from "$lib/components/ui/separator";
  import type { AccountConfig, Folder } from "../lib/types";
  import { ChevronDown, ChevronUp, Mail, Folder as FolderIcon, Plus, Settings, LogOut } from "lucide-svelte";

  // Props
  let {
    accounts = [] as AccountConfig[],
    selectedAccountId = null as number | null,
    folders = [] as Folder[],
    selectedFolderName = "INBOX",
    isLoadingFolders = false,
    onAccountSelect,
    onFolderClick,
    onAddAccount,
    onSettings,
  }: {
    accounts?: AccountConfig[];
    selectedAccountId?: number | null;
    folders?: Folder[];
    selectedFolderName?: string;
    isLoadingFolders?: boolean;
    onAccountSelect: (accountId: number) => void;
    onFolderClick: (folderName: string) => void;
    onAddAccount: () => void;
    onSettings: () => void;
  } = $props();

  // Get selected account
  const selectedAccount = $derived(
    accounts.find((acc) => acc.id === selectedAccountId)
  );
</script>

<Sidebar.Root>
  <Sidebar.Header>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            {#snippet child({ props }: { props: Record<string, any> })}
              <Sidebar.MenuButton {...props} class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground">
                <div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                  <Mail class="size-4" />
                </div>
                <div class="grid flex-1 text-left text-sm leading-tight">
                  <span class="truncate font-semibold">
                    {selectedAccount?.email || "Select Account"}
                  </span>
                  <span class="truncate text-xs">
                    {selectedAccount ? "Active" : "No account selected"}
                  </span>
                </div>
                <ChevronDown class="ml-auto" />
              </Sidebar.MenuButton>
            {/snippet}
          </DropdownMenu.Trigger>
          <DropdownMenu.Content class="w-[--bits-dropdown-menu-anchor-width]" align="start">
            {#each accounts as account (account.id)}
              <DropdownMenu.Item onclick={() => onAccountSelect(account.id)}>
                <div class="flex items-center gap-2">
                  <Mail class="size-4" />
                  <span class="truncate">{account.email}</span>
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
      <Sidebar.GroupLabel>Folders</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
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
                  isActive={folder.name === selectedFolderName}
                  onclick={() => onFolderClick(folder.name)}
                >
                  {#snippet child({ props }: { props: Record<string, any> })}
                    <button {...props}>
                      <FolderIcon class="size-4" />
                      <span>{folder.display_name}</span>
                    </button>
                  {/snippet}
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
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            {#snippet child({ props }: { props: Record<string, any> })}
              <Sidebar.MenuButton
                {...props}
                class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
              >
                <div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                  <span class="text-sm font-semibold">U</span>
                </div>
                <div class="grid flex-1 text-left text-sm leading-tight">
                  <span class="truncate font-semibold">User Account</span>
                  <span class="truncate text-xs">管理账户</span>
                </div>
                <ChevronUp class="ml-auto" />
              </Sidebar.MenuButton>
            {/snippet}
          </DropdownMenu.Trigger>
          <DropdownMenu.Content
            side="top"
            class="w-[--bits-dropdown-menu-anchor-width]"
          >
            <DropdownMenu.Item>
              <span>订阅方案</span>
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={onSettings}>
              <Settings class="size-4" />
              <span>设置</span>
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item>
              <LogOut class="size-4" />
              <span>登出</span>
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.Footer>
</Sidebar.Root>
