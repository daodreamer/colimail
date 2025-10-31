<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "svelte-sonner";
  import type { AccountConfig } from "../lib/types";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import * as Alert from "$lib/components/ui/alert";
  import * as Breadcrumb from "$lib/components/ui/breadcrumb";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import { Separator } from "$lib/components/ui/separator";
  import MailIcon from "lucide-svelte/icons/mail";
  import AlertCircleIcon from "lucide-svelte/icons/alert-circle";

  // Props
  let {
    open = $bindable(false),
    accounts = [] as AccountConfig[],
    onAccountDeleted,
    onAccountUpdated,
    onAddAccount,
  }: {
    open?: boolean;
    accounts?: AccountConfig[];
    onAccountDeleted: (email: string) => void;
    onAccountUpdated: () => void;
    onAddAccount?: () => void;
  } = $props();

  let selectedAccount = $state<AccountConfig | null>(null);
  let isEditing = $state(false);
  let editingConfig = $state<AccountConfig | null>(null);
  let isDeleting = $state(false);
  let isSaving = $state(false);
  let showDeleteDialog = $state(false);
  let isDetectingDisplayName = $state(false);

  // Auto-select first account when dialog opens
  $effect(() => {
    if (open && accounts.length > 0 && !selectedAccount) {
      selectedAccount = accounts[0];
    }
  });

  function selectAccount(account: AccountConfig) {
    selectedAccount = account;
    isEditing = false;
    editingConfig = null;
  }

  function startEditing() {
    if (!selectedAccount) return;

    isEditing = true;
    editingConfig = { ...selectedAccount };
  }

  function cancelEditing() {
    isEditing = false;
    editingConfig = null;
  }

  async function saveConfig() {
    if (!editingConfig) return;

    isSaving = true;
    const savedEmail = editingConfig.email; // Save email before clearing editingConfig
    const savedAccountId = editingConfig.id; // Save account ID
    try {
      await invoke("save_account_config", {
        config: editingConfig,
      });

      toast.success("Account configuration updated successfully!");

      isEditing = false;
      editingConfig = null;

      // Notify parent to reload accounts - this will update appState.accounts
      await onAccountUpdated();

      // After parent updates, find the account in the updated accounts prop
      // Use ID instead of email for more reliable matching
      selectedAccount = accounts.find((acc) => acc.id === savedAccountId) ||
                       accounts.find((acc) => acc.email === savedEmail) ||
                       null;
    } catch (error) {
      console.error("Failed to update configuration:", error);
      toast.error(`Failed to update account configuration: ${error}`);
    } finally {
      isSaving = false;
    }
  }

  async function deleteAccount() {
    if (!selectedAccount) return;

    showDeleteDialog = false;
    isDeleting = true;
    const deletedEmail = selectedAccount.email;

    try {
      await invoke("delete_account", { email: selectedAccount.email });

      // Show success toast
      toast.success("Account deleted successfully", {
        description: `The account ${deletedEmail} and all associated data have been permanently removed.`
      });

      onAccountDeleted(selectedAccount.email);

      // Clear selection
      selectedAccount = null;
      isEditing = false;
      editingConfig = null;

      // Close dialog if no accounts left
      if (accounts.length === 1) {
        open = false;
      }
    } catch (error) {
      console.error("Failed to delete account:", error);
      toast.error(`Failed to delete account: ${error}`);
    } finally {
      isDeleting = false;
    }
  }

  function getAuthTypeBadge(authType?: "basic" | "oauth2") {
    if (authType === "oauth2") {
      return { label: "OAuth2", variant: "default" as const };
    }
    return { label: "Basic", variant: "secondary" as const };
  }

  async function detectAndApplyDisplayName() {
    if (!editingConfig) return;

    isDetectingDisplayName = true;
    try {
      const detectedName = await invoke<string | null>("detect_display_name_from_sent", {
        config: editingConfig
      });

      if (detectedName) {
        editingConfig.display_name = detectedName;
        toast.success(`Display name detected: "${detectedName}"`);
      } else {
        toast.info("No display name found in recent sent emails");
      }
    } catch (error) {
      console.error("Failed to detect display name:", error);
      toast.error("Failed to detect display name");
    } finally {
      isDetectingDisplayName = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content
    class="overflow-hidden p-0 md:max-h-[600px] md:max-w-[700px] lg:max-w-[900px]"
    trapFocus={false}
  >
    <Dialog.Title class="sr-only">Manage Accounts</Dialog.Title>
    <Dialog.Description class="sr-only">
      View, edit, or delete your email accounts
    </Dialog.Description>

    {#if accounts.length === 0}
      <div class="py-12 px-6 text-center">
        <p class="text-muted-foreground mb-4">No accounts configured.</p>
        <Button onclick={() => {
          open = false;
          if (onAddAccount) {
            onAddAccount();
          }
        }}>Add your first account</Button>
      </div>
    {:else}
      <Sidebar.Provider class="items-start">
        <Sidebar.Root collapsible="none" class="hidden md:flex">
          <Sidebar.Content>
            <Sidebar.Group>
              <Sidebar.GroupContent>
                <Sidebar.Menu>
                  {#each accounts as account (account.id)}
                    <Sidebar.MenuItem>
                      <Sidebar.MenuButton
                        isActive={selectedAccount?.id === account.id}
                        onclick={() => selectAccount(account)}
                      >
                        {#snippet child({ props })}
                          <button type="button" {...props} class="w-full">
                            <MailIcon class="size-4" />
                            <div class="flex flex-col items-start flex-1 min-w-0">
                              <span class="truncate text-sm w-full text-left">
                                {account.email}
                              </span>
                              <Badge
                                {...getAuthTypeBadge(account.auth_type)}
                                class="text-xs mt-1"
                              >
                                {getAuthTypeBadge(account.auth_type).label}
                              </Badge>
                            </div>
                          </button>
                        {/snippet}
                      </Sidebar.MenuButton>
                    </Sidebar.MenuItem>
                  {/each}
                </Sidebar.Menu>
              </Sidebar.GroupContent>
            </Sidebar.Group>
          </Sidebar.Content>
        </Sidebar.Root>

        <main class="flex h-[600px] flex-1 flex-col overflow-hidden">
          <header
            class="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-[[data-collapsible=icon]]/sidebar-wrapper:h-12"
          >
            <div class="flex items-center gap-2 px-4">
              <Breadcrumb.Root>
                <Breadcrumb.List>
                  <Breadcrumb.Item class="hidden md:block">
                    <Breadcrumb.Link href="##">Manage Accounts</Breadcrumb.Link>
                  </Breadcrumb.Item>
                  <Breadcrumb.Separator class="hidden md:block" />
                  <Breadcrumb.Item>
                    <Breadcrumb.Page>
                      {selectedAccount?.email || "Select an account"}
                    </Breadcrumb.Page>
                  </Breadcrumb.Item>
                </Breadcrumb.List>
              </Breadcrumb.Root>
            </div>
          </header>

          <div class="flex flex-1 flex-col gap-4 overflow-y-auto p-4 pt-0">
            {#if selectedAccount}
              <div class="bg-muted/50 rounded-xl p-6 space-y-4 max-w-3xl">
                {#if !isEditing}
                  <!-- View Mode -->
                  <div class="space-y-4">
                    <div class="flex items-start justify-between">
                      <div class="space-y-1">
                        <h3 class="text-lg font-semibold">{selectedAccount.email}</h3>
                        <p class="text-sm text-muted-foreground">
                          Authentication: {selectedAccount.auth_type === "oauth2" ? "OAuth2" : "Basic"}
                        </p>
                      </div>
                      <div class="flex gap-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onclick={startEditing}
                        >
                          Edit
                        </Button>
                        <AlertDialog.Root bind:open={showDeleteDialog}>
                          <AlertDialog.Trigger>
                            <Button
                              variant="destructive"
                              size="sm"
                              disabled={isDeleting}
                            >
                              {isDeleting ? "Deleting..." : "Delete"}
                            </Button>
                          </AlertDialog.Trigger>
                          <AlertDialog.Content>
                            <AlertDialog.Header>
                              <AlertDialog.Title>Are you absolutely sure?</AlertDialog.Title>
                              <AlertDialog.Description>
                                <Alert.Root variant="destructive" class="mt-2">
                                  <AlertCircleIcon />
                                  <Alert.Title>Unable to undo this action</Alert.Title>
                                  <Alert.Description>
                                    <p>This will permanently delete the account <strong>{selectedAccount.email}</strong> and remove all associated data:</p>
                                    <ul class="list-disc list-inside mt-2 space-y-1 text-sm">
                                      <li>Check account configuration will be removed</li>
                                      <li>Ensure all emails will be deleted</li>
                                      <li>Verify all folders will be removed</li>
                                      <li>All attachments will be permanently deleted</li>
                                    </ul>
                                  </Alert.Description>
                                </Alert.Root>
                              </AlertDialog.Description>
                            </AlertDialog.Header>
                            <AlertDialog.Footer>
                              <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
                              <AlertDialog.Action onclick={deleteAccount}>
                                Delete Account
                              </AlertDialog.Action>
                            </AlertDialog.Footer>
                          </AlertDialog.Content>
                        </AlertDialog.Root>
                      </div>
                    </div>

                    <Separator />

                    <div class="space-y-4">
                      <div class="space-y-1">
                        <Label class="text-sm font-medium text-muted-foreground">Email Address</Label>
                        <p class="text-sm">{selectedAccount.email}</p>
                      </div>

                      {#if selectedAccount.display_name}
                        <div class="space-y-1">
                          <Label class="text-sm font-medium text-muted-foreground">Display Name</Label>
                          <p class="text-sm">{selectedAccount.display_name}</p>
                        </div>
                      {/if}

                      <div class="space-y-1">
                        <Label class="text-sm font-medium text-muted-foreground">IMAP Server</Label>
                        <p class="text-sm font-mono">
                          {selectedAccount.imap_server}:{selectedAccount.imap_port}
                        </p>
                      </div>

                      <div class="space-y-1">
                        <Label class="text-sm font-medium text-muted-foreground">SMTP Server</Label>
                        <p class="text-sm font-mono">
                          {selectedAccount.smtp_server}:{selectedAccount.smtp_port}
                        </p>
                      </div>

                      {#if selectedAccount.auth_type === "oauth2"}
                        <div class="rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-800 dark:bg-blue-950/30">
                          <p class="text-sm font-medium text-blue-900 dark:text-blue-200 mb-1">
                            OAuth2 Account
                          </p>
                          <p class="text-xs text-blue-800 dark:text-blue-300">
                            This account uses OAuth2 authentication. You can edit the display name, but server settings cannot be modified.
                            To change server settings, please remove and re-add the account.
                          </p>
                        </div>
                      {/if}
                    </div>
                  </div>
                {:else if editingConfig}
                  <!-- Edit Mode -->
                  <div class="space-y-4">
                    <div class="flex items-center justify-between">
                      <h3 class="text-lg font-semibold">Edit Account Configuration</h3>
                    </div>

                    <Separator />

                    <form
                      onsubmit={(e) => {
                        e.preventDefault();
                        saveConfig();
                      }}
                      class="space-y-4"
                    >
                      <div class="space-y-2">
                        <Label for="edit-email">Email Address</Label>
                        <Input
                          id="edit-email"
                          type="email"
                          bind:value={editingConfig.email}
                          disabled
                          class="bg-muted"
                        />
                        <p class="text-xs text-muted-foreground">
                          Email address cannot be changed
                        </p>
                      </div>

                      <div class="space-y-2">
                        <Label for="edit-display-name">Display Name (Optional)</Label>
                        <div class="flex gap-2">
                          <Input
                            id="edit-display-name"
                            type="text"
                            bind:value={editingConfig.display_name}
                            placeholder="Your Name (e.g., John Doe)"
                            class="flex-1"
                          />
                          <Button
                            type="button"
                            variant="outline"
                            onclick={detectAndApplyDisplayName}
                            disabled={isDetectingDisplayName}
                          >
                            {isDetectingDisplayName ? "Detecting..." : "Detect"}
                          </Button>
                        </div>
                        <p class="text-xs text-muted-foreground">
                          This name will be shown to recipients when you send emails. Click "Detect" to auto-fill from your sent emails.
                        </p>
                      </div>

                      {#if editingConfig.auth_type !== "oauth2"}
                        <div class="space-y-2">
                          <Label for="edit-password">Password</Label>
                          <Input
                            id="edit-password"
                            type="password"
                            bind:value={editingConfig.password}
                            placeholder="Enter new password to change"
                          />
                          <p class="text-xs text-muted-foreground">
                            Leave blank to keep current password
                          </p>
                        </div>

                        <Separator />

                        <div class="grid grid-cols-2 gap-4">
                          <div class="space-y-2">
                            <Label for="edit-imap-server">IMAP Server</Label>
                            <Input
                              id="edit-imap-server"
                              bind:value={editingConfig.imap_server}
                              required
                              placeholder="imap.example.com"
                            />
                          </div>

                          <div class="space-y-2">
                            <Label for="edit-imap-port">IMAP Port</Label>
                            <Input
                              id="edit-imap-port"
                              type="number"
                              bind:value={editingConfig.imap_port}
                              required
                              placeholder="993"
                            />
                          </div>
                        </div>

                        <div class="grid grid-cols-2 gap-4">
                          <div class="space-y-2">
                            <Label for="edit-smtp-server">SMTP Server</Label>
                            <Input
                              id="edit-smtp-server"
                              bind:value={editingConfig.smtp_server}
                              required
                              placeholder="smtp.example.com"
                            />
                          </div>

                          <div class="space-y-2">
                            <Label for="edit-smtp-port">SMTP Port</Label>
                            <Input
                              id="edit-smtp-port"
                              type="number"
                              bind:value={editingConfig.smtp_port}
                              required
                              placeholder="465"
                            />
                          </div>
                        </div>
                      {:else}
                        <div class="rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-800 dark:bg-blue-950/30">
                          <p class="text-sm font-medium text-blue-900 dark:text-blue-200 mb-1">
                            OAuth2 Account
                          </p>
                          <p class="text-xs text-blue-800 dark:text-blue-300">
                            This account uses OAuth2 authentication. You can only edit the display name.
                            Server settings cannot be modified. To change other settings, please remove and re-add the account.
                          </p>
                        </div>
                      {/if}

                      <Separator />

                      <div class="flex gap-2 pt-2">
                        <Button type="submit" disabled={isSaving}>
                          {isSaving ? "Saving..." : "Save Changes"}
                        </Button>
                        <Button type="button" variant="outline" onclick={cancelEditing}>
                          Cancel
                        </Button>
                      </div>
                    </form>
                  </div>
                {/if}
              </div>
            {:else}
              <div class="flex items-center justify-center h-full">
                <p class="text-muted-foreground">Select an account to view details</p>
              </div>
            {/if}
          </div>
        </main>
      </Sidebar.Provider>
    {/if}
  </Dialog.Content>
</Dialog.Root>
