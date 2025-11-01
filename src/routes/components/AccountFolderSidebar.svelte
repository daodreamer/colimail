<script lang="ts">
  import { Mail, Plus, RefreshCw, PenSquare, MailPlus, FolderPlus, HardDrive } from "lucide-svelte";
  import CircleUserRound from "lucide-svelte/icons/circle-user-round";
  import InboxIcon from "lucide-svelte/icons/inbox";
  import FileIcon from "lucide-svelte/icons/file";
  import FilePenIcon from "lucide-svelte/icons/file-pen";
  import SendIcon from "lucide-svelte/icons/send";
  import ArchiveXIcon from "lucide-svelte/icons/archive-x";
  import Trash2Icon from "lucide-svelte/icons/trash-2";
  import FolderIcon from "lucide-svelte/icons/folder";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import * as ContextMenu from "$lib/components/ui/context-menu";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import type { AccountConfig, Folder } from "../lib/types";
  import NavUser from "$lib/components/nav-user.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";

  // Props
  let {
    accounts = [] as AccountConfig[],
    selectedAccountId = null as number | null,
    folders = [] as Folder[],
    selectedFolderName = "INBOX",
    isLoadingFolders = false,
    isSyncing = false,
    showDraftsFolder = false,
    onAccountSelect,
    onFolderClick,
    onAddAccount,
    onManageAccounts,
    onSettings,
    onSyncMail,
    onComposeClick,
    onShowDrafts,
    onFolderCreated,
    onFolderDeleted,
    openContextMenuType = null,
    openContextMenuId = null,
    onContextMenuChange,
  }: {
    accounts?: AccountConfig[];
    selectedAccountId?: number | null;
    folders?: Folder[];
    selectedFolderName?: string;
    isLoadingFolders?: boolean;
    isSyncing?: boolean;
    showDraftsFolder?: boolean;
    onAccountSelect: (accountId: number) => void;
    onFolderClick: (folderName: string) => void;
    onAddAccount: () => void;
    onManageAccounts: () => void;
    onSettings: () => void;
    onSyncMail: () => void;
    onComposeClick: () => void;
    onShowDrafts: () => void;
    onFolderCreated?: () => void;
    onFolderDeleted?: () => void;
    openContextMenuType?: 'folder' | 'email' | null;
    openContextMenuId?: string | number | null;
    onContextMenuChange?: (type: 'folder' | 'email' | null, id: string | number | null) => void;
  } = $props();

  // Import auth store
  import { authStore } from "$lib/stores/auth.svelte";
  import { signOut } from "$lib/supabase";
  import { goto } from "$app/navigation";

  // Get selected account
  const selectedAccount = $derived(
    accounts.find((acc) => acc.id === selectedAccountId)
  );

  // User data for NavUser component - use auth user if authenticated, otherwise show guest
  const userData = $derived({
    name: authStore.user?.name || authStore.user?.email?.split('@')[0] || "Guest",
    email: authStore.user?.email || "Not logged in",
    avatar: authStore.user?.avatarUrl || "", // Empty string when no custom avatar
  });

  // Handle logout
  async function handleLogout() {
    try {
      console.log('[Logout] Signing out user...');
      await signOut();
      console.log('[Logout] User signed out successfully');
      // Stay on main UI - user can use app features without being logged in
      // No redirect needed, NavUser will update to show guest menu
      toast.success("Logged out successfully");
    } catch (error) {
      console.error("[Logout] Failed to logout:", error);
      toast.error("Failed to logout");
    }
  }

  // Handle upgrade
  function handleUpgrade() {
    // TODO: Open upgrade dialog
    console.log("Upgrade to Pro clicked");
  }

  // Handle account settings
  function handleAccount() {
    // TODO: Open account settings dialog
    console.log("Account settings clicked");
  }

  // Handle billing
  function handleBilling() {
    // TODO: Open billing dialog
    console.log("Billing clicked");
  }

  // Handle notifications settings
  function handleNotifications() {
    // TODO: Open notifications settings dialog
    console.log("Notifications clicked");
  }

  // Get icon for folder based on its name or display name
  function getFolderIcon(folder: Folder) {
    const name = folder.name.toLowerCase();
    const displayName = folder.display_name.toLowerCase();

    // Check common folder patterns
    if (name === 'inbox' || displayName === 'inbox') {
      return InboxIcon;
    }
    if (name.includes('draft') || displayName.includes('draft')) {
      return FileIcon;
    }
    if (name.includes('sent') || displayName.includes('sent')) {
      return SendIcon;
    }
    if (name.includes('junk') || name.includes('spam') || displayName.includes('junk') || displayName.includes('spam')) {
      return ArchiveXIcon;
    }
    if (name.includes('trash') || name.includes('deleted') || displayName.includes('trash') || displayName.includes('deleted')) {
      return Trash2Icon;
    }

    // Default folder icon
    return FolderIcon;
  }

  // States for folder management
  let showCreateDialog = $state(false);
  let newFolderName = $state("");
  let isCreatingFolder = $state(false);
  let folderToDelete = $state<Folder | null>(null);
  let isDeletingFolder = $state(false);
  let supportsRemoteFolders = $state(true); // Will be checked on first folder creation

  // Derived state: check if this folder's context menu is open
  const isFolderContextMenuOpen = $derived((folderName: string) => {
    return openContextMenuType === 'folder' && openContextMenuId === folderName;
  });

  // Create folder
  async function handleCreateFolder() {
    if (!newFolderName.trim() || !selectedAccount) {
      toast.error("Please enter a folder name");
      return;
    }

    isCreatingFolder = true;
    try {
      // Check if server supports remote folder creation (only check once)
      let shouldCreateRemote = false;
      if (supportsRemoteFolders) {
        try {
          const supports = await invoke<boolean>("check_folder_capabilities", {
            config: selectedAccount,
          });
          supportsRemoteFolders = supports;
          shouldCreateRemote = supports;
        } catch (error) {
          console.warn("Failed to check folder capabilities, creating local folder:", error);
          supportsRemoteFolders = false;
        }
      }

      if (shouldCreateRemote) {
        // Create remote IMAP folder
        await invoke<Folder>("create_remote_folder", {
          config: selectedAccount,
          folderName: newFolderName.trim(),
        });
        toast.success(`Created remote folder "${newFolderName}"`);
      } else {
        // Create local-only folder
        await invoke<Folder>("create_local_folder", {
          accountId: selectedAccountId!,
          folderName: newFolderName.trim(),
        });
        toast.success(`Created local folder "${newFolderName}"`);
      }

      newFolderName = "";
      showCreateDialog = false;

      // Notify parent to refresh folder list
      if (onFolderCreated) {
        onFolderCreated();
      }
    } catch (error) {
      console.error("Failed to create folder:", error);
      toast.error(`Failed to create folder: ${error}`);
    } finally {
      isCreatingFolder = false;
    }
  }

  // Delete folder
  async function handleDeleteFolder(folder: Folder) {
    if (!selectedAccount) return;

    isDeletingFolder = true;
    try {
      if (folder.is_local) {
        // Delete local folder
        await invoke("delete_local_folder", {
          accountId: selectedAccountId!,
          folderName: folder.name,
        });
        toast.success(`Deleted local folder "${folder.display_name}"`);
      } else {
        // Delete remote IMAP folder
        await invoke("delete_remote_folder", {
          config: selectedAccount,
          folderName: folder.name,
        });
        toast.success(`Deleted remote folder "${folder.display_name}"`);
      }

      folderToDelete = null;

      // Notify parent to refresh folder list
      if (onFolderDeleted) {
        onFolderDeleted();
      }
    } catch (error) {
      console.error("Failed to delete folder:", error);
      toast.error(`Failed to delete folder: ${error}`);
    } finally {
      isDeletingFolder = false;
    }
  }

  // Check if folder can be deleted (prevent deleting system folders)
  function canDeleteFolder(folder: Folder): boolean {
    const systemFolders = ['inbox', 'sent', 'drafts', 'trash', 'junk', 'spam'];
    const name = folder.name.toLowerCase();
    const displayName = folder.display_name.toLowerCase();

    return !systemFolders.some(sf => name.includes(sf) || displayName.includes(sf));
  }
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
                  <span class="truncate text-xs">{accounts.length === 0 ? "No accounts" : "Email Client"}</span>
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
            <DropdownMenu.Item onclick={onManageAccounts}>
              <CircleUserRound class="size-4" />
              <span>Manage Account</span>
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
          {#if accounts.length === 0}
            <!-- Show add account button when no accounts exist -->
            <Sidebar.MenuItem>
              <Sidebar.MenuButton
                tooltipContentProps={{
                  hidden: false,
                }}
                onclick={onAddAccount}
                class="px-2.5 md:px-2"
              >
                {#snippet tooltipContent()}
                  Add Account
                {/snippet}
                <MailPlus class="size-4" />
                <span>Add Account</span>
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {:else}
            <!-- Drafts (Local Storage) -->
            <Sidebar.MenuItem>
              <Sidebar.MenuButton
                tooltipContentProps={{
                  hidden: false,
                }}
                onclick={onShowDrafts}
                isActive={showDraftsFolder}
                class="px-2.5 md:px-2"
              >
                {#snippet tooltipContent()}
                  Drafts
                {/snippet}
                <FilePenIcon class="size-4" />
                <span>Drafts</span>
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/if}
          {#if isLoadingFolders}
            {#each Array(6) as _, i (i)}
              <Sidebar.MenuItem>
                <Sidebar.MenuSkeleton />
              </Sidebar.MenuItem>
            {/each}
          {:else if folders.length > 0}
            {#each folders as folder (folder.name)}
              {@const IconComponent = getFolderIcon(folder)}
              <Sidebar.MenuItem>
                <ContextMenu.Root
                  open={isFolderContextMenuOpen(folder.name)}
                  onOpenChange={(isOpen) => {
                    if (onContextMenuChange) {
                      onContextMenuChange(isOpen ? 'folder' : null, isOpen ? folder.name : null);
                    }
                  }}
                >
                  <ContextMenu.Trigger class="w-full">
                    <Sidebar.MenuButton
                      tooltipContentProps={{
                        hidden: false,
                      }}
                      onclick={() => onFolderClick(folder.name)}
                      isActive={folder.name === selectedFolderName}
                      class="px-2.5 md:px-2"
                    >
                      {#snippet tooltipContent()}
                        {folder.display_name}{folder.is_local ? " (Local)" : ""}
                      {/snippet}
                      <IconComponent class="size-4" />
                      <span class="flex items-center gap-1.5">
                        {folder.display_name}
                        {#if folder.is_local}
                          <HardDrive class="size-3 text-muted-foreground" />
                        {/if}
                      </span>
                    </Sidebar.MenuButton>
                  </ContextMenu.Trigger>
                  {#if canDeleteFolder(folder)}
                    <ContextMenu.Content class="w-48">
                      <ContextMenu.Item
                        onclick={() => {
                          folderToDelete = folder;
                          if (onContextMenuChange) {
                            onContextMenuChange(null, null); // Close menu after clicking
                          }
                        }}
                        class="text-destructive focus:text-destructive"
                      >
                        <Trash2Icon class="mr-2 size-4" />
                        Delete Folder
                      </ContextMenu.Item>
                    </ContextMenu.Content>
                  {/if}
                </ContextMenu.Root>
              </Sidebar.MenuItem>
            {/each}
            <!-- Add New Folder Button -->
            {#if selectedAccountId}
              <Sidebar.MenuItem>
                <Sidebar.MenuButton
                  tooltipContentProps={{
                    hidden: false,
                  }}
                  onclick={() => { showCreateDialog = true; }}
                  class="px-2.5 md:px-2 text-muted-foreground hover:text-foreground"
                >
                  {#snippet tooltipContent()}
                    New Folder
                  {/snippet}
                  <FolderPlus class="size-4" />
                  <span>New Folder</span>
                </Sidebar.MenuButton>
              </Sidebar.MenuItem>
            {/if}
          {:else if selectedAccountId}
            <Sidebar.MenuItem>
              <div class="px-2 py-1.5 text-sm text-muted-foreground">
                No folders found
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
          onclick={!selectedAccountId ? undefined : onComposeClick}
          tooltipContentProps={{
            hidden: false,
          }}
          class="px-2.5 md:px-2 bg-primary text-primary-foreground hover:bg-primary/90 {!selectedAccountId ? 'opacity-50 cursor-not-allowed' : ''}"
        >
          {#snippet tooltipContent()}
            Compose Email
          {/snippet}
          <PenSquare class="size-4" />
          <span>Compose</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
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
    <NavUser
      user={userData}
      isAuthenticated={authStore.isAuthenticated}
      onSettings={onSettings}
      onUpgrade={handleUpgrade}
      onAccount={handleAccount}
      onBilling={handleBilling}
      onNotifications={handleNotifications}
      onLogout={handleLogout}
    />
  </Sidebar.Footer>
</Sidebar.Root>

<!-- Create Folder Dialog -->
<Dialog.Root bind:open={showCreateDialog}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Create New Folder</Dialog.Title>
      <Dialog.Description>
        Create a new folder to organize your emails. {supportsRemoteFolders ? "This will be synced with your email server." : "This folder will be local only."}
      </Dialog.Description>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="folder-name">Folder Name</Label>
        <Input
          id="folder-name"
          bind:value={newFolderName}
          placeholder="Enter folder name"
          onkeydown={(e) => {
            if (e.key === 'Enter' && !isCreatingFolder) {
              handleCreateFolder();
            }
          }}
        />
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={() => { showCreateDialog = false; }}>
        Cancel
      </Button>
      <Button onclick={handleCreateFolder} disabled={isCreatingFolder || !newFolderName.trim()}>
        {isCreatingFolder ? "Creating..." : "Create"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<!-- Delete Folder Confirmation Dialog -->
<AlertDialog.Root open={folderToDelete !== null}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Delete Folder?</AlertDialog.Title>
      <AlertDialog.Description>
        Are you sure you want to delete "{folderToDelete?.display_name}"?
        {#if folderToDelete?.is_local}
          This local folder and all its emails will be permanently deleted.
        {:else}
          This will delete the folder from your email server.
        {/if}
        This action cannot be undone.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={() => { folderToDelete = null; }}>
        Cancel
      </AlertDialog.Cancel>
      <AlertDialog.Action
        onclick={() => { if (folderToDelete) handleDeleteFolder(folderToDelete); }}
        disabled={isDeletingFolder}
        class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
      >
        {isDeletingFolder ? "Deleting..." : "Delete"}
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
