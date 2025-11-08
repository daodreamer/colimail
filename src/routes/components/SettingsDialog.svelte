<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { toast } from "svelte-sonner";
  import * as Breadcrumb from "$lib/components/ui/breadcrumb";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import BellIcon from "lucide-svelte/icons/bell";
  import GlobeIcon from "lucide-svelte/icons/globe";
  import LockIcon from "lucide-svelte/icons/lock";
  import PaintbrushIcon from "lucide-svelte/icons/paintbrush";
  import SettingsIcon from "lucide-svelte/icons/settings";
  import InfoIcon from "lucide-svelte/icons/info";

  interface SettingsDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
  }

  let { open = $bindable(), onOpenChange }: SettingsDialogProps = $props();

  const data = {
    nav: [
      { name: "Notifications", icon: BellIcon },
      { name: "Appearance", icon: PaintbrushIcon },
      { name: "Language & region", icon: GlobeIcon },
      { name: "Privacy & visibility", icon: LockIcon },
      { name: "Advanced", icon: SettingsIcon },
      { name: "About", icon: InfoIcon },
    ],
  };

  // Navigation state
  let currentPage = $state("Notifications");

  // Settings state
  let syncInterval = $state<number>(300);
  let notificationEnabled = $state<boolean>(true);
  let soundEnabled = $state<boolean>(true);
  let minimizeToTray = $state<boolean>(true);
  let isSaving = $state(false);
  let isCheckingUpdate = $state(false);
  let appVersion = $state("0.6.2");

  // Encryption state
  interface EncryptionStatus {
    enabled: boolean;
    unlocked: boolean;
  }
  let encryptionStatus = $state<EncryptionStatus>({ enabled: false, unlocked: false });
  let masterPassword = $state("");
  let confirmPassword = $state("");
  let unlockPassword = $state("");
  let isEnablingEncryption = $state(false);

  // Load settings when dialog opens
  $effect(() => {
    if (open) {
      loadSettings();
    }
  });

  async function loadSettings() {
    try {
      syncInterval = await invoke<number>("get_sync_interval");
      notificationEnabled = await invoke<boolean>("get_notification_enabled");
      soundEnabled = await invoke<boolean>("get_sound_enabled");
      minimizeToTray = await invoke<boolean>("get_minimize_to_tray");
      encryptionStatus = await invoke<EncryptionStatus>("get_encryption_status");
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  }

  async function enableEncryption() {
    if (masterPassword.length < 8) {
      toast.error("Password must be at least 8 characters long");
      return;
    }
    if (masterPassword !== confirmPassword) {
      toast.error("Passwords do not match");
      return;
    }

    isEnablingEncryption = true;
    try {
      await invoke("enable_encryption", { password: masterPassword });
      encryptionStatus = await invoke<EncryptionStatus>("get_encryption_status");
      toast.success("Encryption enabled successfully!");
      masterPassword = "";
      confirmPassword = "";
    } catch (error) {
      console.error("Failed to enable encryption:", error);
      toast.error(`Failed to enable encryption: ${error}`);
    } finally {
      isEnablingEncryption = false;
    }
  }

  async function unlockEncryption() {
    if (!unlockPassword) {
      toast.error("Please enter your password");
      return;
    }

    try {
      await invoke("unlock_encryption_with_password", { password: unlockPassword });
      encryptionStatus = await invoke<EncryptionStatus>("get_encryption_status");
      toast.success("Encryption unlocked successfully!");
      unlockPassword = "";
    } catch (error) {
      console.error("Failed to unlock encryption:", error);
      toast.error("Invalid password");
    }
  }

  async function lockEncryption() {
    try {
      await invoke("lock_encryption_command");
      encryptionStatus = await invoke<EncryptionStatus>("get_encryption_status");
      toast.success("Encryption locked");
    } catch (error) {
      console.error("Failed to lock encryption:", error);
      toast.error("Failed to lock encryption");
    }
  }

  async function saveNotificationSettings() {
    isSaving = true;
    try {
      await invoke("set_sync_interval", { interval: syncInterval });
      await invoke("set_notification_enabled", { enabled: notificationEnabled });
      await invoke("set_sound_enabled", { enabled: soundEnabled });
      await invoke("set_minimize_to_tray", { enabled: minimizeToTray });
      toast.success("Settings saved successfully!");
    } catch (error) {
      console.error("Failed to save settings:", error);
      toast.error("Failed to save settings");
    } finally {
      isSaving = false;
    }
  }

  function getIntervalDescription(interval: number): string {
    if (interval === -1) return "Never sync (cache only)";
    if (interval === 0) return "Manual sync (refresh button only)";
    if (interval < 60) return `${interval} seconds`;
    if (interval < 3600) return `${Math.floor(interval / 60)} minutes`;
    return `${Math.floor(interval / 3600)} hours`;
  }

  function handleNavClick(pageName: string) {
    currentPage = pageName;
  }

  // Check for updates
  async function checkForUpdates() {
    isCheckingUpdate = true;
    try {
      const update = await check();
      if (update) {
        toast.success(`New version available: ${update.version}`);
        const shouldUpdate = confirm(
          `A new version (${update.version}) is available!\n\nRelease notes:\n${update.body || "No release notes provided."}\n\nWould you like to download and install it now?`
        );
        if (shouldUpdate) {
          toast.info("Downloading update...");
          await update.downloadAndInstall();
          toast.success("Update installed! Restarting application...");
          await relaunch();
        }
      } else {
        toast.success("You are running the latest version!");
      }
    } catch (error) {
      console.error("Failed to check for updates:", error);
      toast.error("Failed to check for updates. Please try again later.");
    } finally {
      isCheckingUpdate = false;
    }
  }
</script>

<Dialog.Root bind:open {onOpenChange}>
  <Dialog.Content
    class="overflow-hidden p-0 md:max-h-[500px] md:max-w-[700px] lg:max-w-[800px]"
    trapFocus={false}
  >
    <Dialog.Title class="sr-only">Settings</Dialog.Title>
    <Dialog.Description class="sr-only">Customize your settings here.</Dialog.Description>
    
    <Sidebar.Provider class="items-start">
      <Sidebar.Root collapsible="none" class="hidden md:flex">
        <Sidebar.Content>
          <Sidebar.Group>
            <Sidebar.GroupContent>
              <Sidebar.Menu>
                {#each data.nav as item (item.name)}
                  <Sidebar.MenuItem>
                    <Sidebar.MenuButton
                      isActive={item.name === currentPage}
                      onclick={() => handleNavClick(item.name)}
                    >
                      {#snippet child({ props })}
                        <button type="button" {...props}>
                          <item.icon />
                          <span>{item.name}</span>
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

      <main class="flex h-[480px] flex-1 flex-col overflow-hidden">
        <header
          class="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-[[data-collapsible=icon]]/sidebar-wrapper:h-12"
        >
          <div class="flex items-center gap-2 px-4">
            <Breadcrumb.Root>
              <Breadcrumb.List>
                <Breadcrumb.Item class="hidden md:block">
                  <Breadcrumb.Link href="##">Settings</Breadcrumb.Link>
                </Breadcrumb.Item>
                <Breadcrumb.Separator class="hidden md:block" />
                <Breadcrumb.Item>
                  <Breadcrumb.Page>{currentPage}</Breadcrumb.Page>
                </Breadcrumb.Item>
              </Breadcrumb.List>
            </Breadcrumb.Root>
          </div>
        </header>

        <div class="flex flex-1 flex-col gap-4 overflow-y-auto p-4 pt-0">
          {#if currentPage === "Notifications"}
            <!-- Sync Settings -->
            <div class="bg-muted/50 rounded-xl p-6 space-y-4 max-w-3xl">
              <div>
                <h4 class="text-sm font-medium mb-3">Sync frequency</h4>
                <div class="space-y-2">
                  <Label for="sync-interval">Automatic sync interval</Label>
                  <select
                    id="sync-interval"
                    bind:value={syncInterval}
                    class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                  >
                    <option value={0}>Manual (refresh button only)</option>
                    <option value={60}>1 minute</option>
                    <option value={180}>3 minutes</option>
                    <option value={300}>5 minutes (recommended)</option>
                    <option value={600}>10 minutes</option>
                    <option value={900}>15 minutes</option>
                    <option value={1800}>30 minutes</option>
                    <option value={-1}>Never (cache only)</option>
                  </select>
                  <p class="text-xs text-muted-foreground">
                    Current: <strong>{getIntervalDescription(syncInterval)}</strong>
                  </p>
                </div>
              </div>

              <Separator />

              <!-- Desktop Notifications -->
              <div class="space-y-3">
                <h4 class="text-sm font-medium">Desktop notifications</h4>
                <div class="flex items-center space-x-3">
                  <input
                    type="checkbox"
                    id="notification-enabled"
                    bind:checked={notificationEnabled}
                    class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
                  />
                  <Label for="notification-enabled" class="text-sm font-normal">
                    Show desktop notifications for new emails
                  </Label>
                </div>
                <p class="text-xs text-muted-foreground ml-7">
                  Display a notification in the corner when new messages arrive
                </p>
              </div>

              <Separator />

              <!-- Sound Settings -->
              <div class="space-y-3">
                <h4 class="text-sm font-medium">Sound alerts</h4>
                <div class="flex items-center space-x-3">
                  <input
                    type="checkbox"
                    id="sound-enabled"
                    bind:checked={soundEnabled}
                    class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
                  />
                  <Label for="sound-enabled" class="text-sm font-normal">
                    Play sound for new emails
                  </Label>
                </div>
                <p class="text-xs text-muted-foreground ml-7">
                  Play an audio alert when new messages arrive
                </p>
              </div>

              <div class="pt-4">
                <Button onclick={saveNotificationSettings} disabled={isSaving}>
                  {isSaving ? "Saving..." : "Save changes"}
                </Button>
              </div>
            </div>

          {:else if currentPage === "Appearance"}
            <div class="bg-muted/50 rounded-xl p-6 space-y-4 max-w-3xl">
              <p class="text-sm text-muted-foreground">
                Theme and appearance settings coming soon...
              </p>
            </div>

          {:else if currentPage === "Language & region"}
            <div class="bg-muted/50 rounded-xl p-6 space-y-4 max-w-3xl">
              <div class="space-y-2">
                <Label>Display language</Label>
                <select
                  disabled
                  class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 opacity-50"
                >
                  <option value="en">English</option>
                </select>
                <p class="text-xs text-muted-foreground">
                  Currently only English is supported
                </p>
              </div>
            </div>

          {:else if currentPage === "Privacy & visibility"}
            <div class="bg-muted/50 rounded-xl p-6 space-y-6 max-w-3xl">
              <!-- Local Data Encryption -->
              <div class="space-y-4">
                <div>
                  <h4 class="text-sm font-medium mb-1">Local data encryption</h4>
                  <p class="text-xs text-muted-foreground">
                    Encrypt email content stored in the local cache database
                  </p>
                </div>

                <!-- Encryption Status Badge -->
                <div class="flex items-center gap-2">
                  {#if encryptionStatus.enabled}
                    {#if encryptionStatus.unlocked}
                      <div class="inline-flex items-center gap-1.5 rounded-full bg-green-100 dark:bg-green-900/30 px-3 py-1 text-xs font-medium text-green-800 dark:text-green-200">
                        <span class="h-1.5 w-1.5 rounded-full bg-green-600 dark:bg-green-400"></span>
                        Unlocked
                      </div>
                    {:else}
                      <div class="inline-flex items-center gap-1.5 rounded-full bg-amber-100 dark:bg-amber-900/30 px-3 py-1 text-xs font-medium text-amber-800 dark:text-amber-200">
                        <span class="h-1.5 w-1.5 rounded-full bg-amber-600 dark:bg-amber-400"></span>
                        Locked
                      </div>
                    {/if}
                  {:else}
                    <div class="inline-flex items-center gap-1.5 rounded-full bg-gray-100 dark:bg-gray-800 px-3 py-1 text-xs font-medium text-gray-700 dark:text-gray-300">
                      <span class="h-1.5 w-1.5 rounded-full bg-gray-400"></span>
                      Disabled
                    </div>
                  {/if}
                </div>

                {#if !encryptionStatus.enabled}
                  <!-- Enable Encryption Form -->
                  <div class="rounded-lg border bg-card p-4 space-y-4">
                    <div class="space-y-2">
                      <Label for="master-password" class="text-sm">
                        Master password
                      </Label>
                      <input
                        id="master-password"
                        type="password"
                        bind:value={masterPassword}
                        placeholder="Enter master password (min. 8 characters)"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                      />
                    </div>
                    <div class="space-y-2">
                      <Label for="confirm-password" class="text-sm">
                        Confirm password
                      </Label>
                      <input
                        id="confirm-password"
                        type="password"
                        bind:value={confirmPassword}
                        placeholder="Re-enter master password"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                      />
                    </div>
                    <div class="rounded-md bg-blue-50 dark:bg-blue-950/30 p-3">
                      <p class="text-xs text-blue-900 dark:text-blue-200">
                        <strong>Important:</strong> Your master password cannot be recovered if lost. Make sure to remember it or store it securely.
                      </p>
                    </div>
                    <Button onclick={enableEncryption} disabled={isEnablingEncryption}>
                      {isEnablingEncryption ? "Enabling..." : "Enable encryption"}
                    </Button>
                  </div>
                {:else if !encryptionStatus.unlocked}
                  <!-- Unlock Encryption Form -->
                  <div class="rounded-lg border bg-card p-4 space-y-4">
                    <div class="space-y-2">
                      <Label for="unlock-password" class="text-sm">
                        Enter master password
                      </Label>
                      <input
                        id="unlock-password"
                        type="password"
                        bind:value={unlockPassword}
                        placeholder="Enter your master password"
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                        onkeydown={(e) => e.key === 'Enter' && unlockEncryption()}
                      />
                    </div>
                    <Button onclick={unlockEncryption}>
                      Unlock encryption
                    </Button>
                  </div>
                {:else}
                  <!-- Encryption Active - Show Lock Button -->
                  <div class="rounded-lg border bg-card p-4 space-y-4">
                    <div class="flex items-center gap-3">
                      <div class="flex-shrink-0">
                        <svg class="h-10 w-10 text-green-600 dark:text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                        </svg>
                      </div>
                      <div class="flex-1">
                        <p class="text-sm font-medium">Encryption is active</p>
                        <p class="text-xs text-muted-foreground">Your email data is being encrypted</p>
                      </div>
                    </div>
                    <Button onclick={lockEncryption} variant="outline">
                      Lock encryption
                    </Button>
                  </div>
                {/if}

                <!-- Info Section -->
                <div class="rounded-lg border bg-muted/50 p-4 space-y-2">
                  <h5 class="text-xs font-semibold">What gets encrypted?</h5>
                  <ul class="text-xs text-muted-foreground space-y-1 ml-4">
                    <li class="list-disc">• Email subjects</li>
                    <li class="list-disc">• Email body content</li>
                    <li class="list-disc">• Email attachments</li>
                  </ul>
                  <p class="text-xs text-muted-foreground mt-3">
                    <strong>Note:</strong> Email metadata (sender, recipient, date) is not encrypted for performance reasons.
                  </p>
                </div>
              </div>
            </div>

          {:else if currentPage === "Advanced"}
            <div class="bg-muted/50 rounded-xl p-6 space-y-6 max-w-3xl">
              <!-- System Tray Settings -->
              <div class="space-y-3">
                <h4 class="text-sm font-medium">System tray</h4>
                <div class="flex items-center space-x-3">
                  <input
                    type="checkbox"
                    id="minimize-to-tray"
                    bind:checked={minimizeToTray}
                    class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
                  />
                  <Label for="minimize-to-tray" class="text-sm font-normal">
                    Close to system tray
                  </Label>
                </div>
                <p class="text-xs text-muted-foreground ml-7">
                  Keep the application running in the system tray when you close the window. Click the tray icon to restore the window.
                </p>
                {#if !minimizeToTray}
                  <div class="rounded-md bg-amber-50 p-2 dark:bg-amber-950/30 ml-7">
                    <p class="text-xs text-amber-900 dark:text-amber-200">
                      ⚠️ Closing the window will exit the application completely
                    </p>
                  </div>
                {/if}
              </div>

              <div class="pt-4">
                <Button onclick={saveNotificationSettings} disabled={isSaving}>
                  {isSaving ? "Saving..." : "Save changes"}
                </Button>
              </div>
            </div>

          {:else if currentPage === "About"}
            <div class="bg-muted/50 rounded-xl p-6 space-y-6 max-w-3xl">
              <!-- Version Info -->
              <div class="space-y-4">
                <div class="flex items-center justify-between">
                  <h4 class="text-sm font-medium">Version</h4>
                  <span class="text-sm text-muted-foreground">{appVersion}</span>
                </div>
                <Separator />
                <div class="flex items-center justify-between">
                  <h4 class="text-sm font-medium">Project</h4>
                  <a
                    href="https://github.com/daodreamer/colimail"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-sm text-primary hover:underline"
                  >
                    GitHub Repository
                  </a>
                </div>
              </div>

              <Separator />

              <!-- Check for Updates -->
              <div class="space-y-3">
                <h4 class="text-sm font-medium">Software Updates</h4>
                <p class="text-xs text-muted-foreground">
                  Check for the latest version to get new features and bug fixes.
                </p>
                <Button
                  onclick={checkForUpdates}
                  disabled={isCheckingUpdate}
                  class="w-full"
                >
                  {isCheckingUpdate ? "Checking for Updates..." : "Check for Updates"}
                </Button>

                <div class="rounded-md bg-blue-50 p-3 dark:bg-blue-950/30">
                  <p class="text-xs text-blue-900 dark:text-blue-200">
                    💡 <strong>Auto Update:</strong> The application automatically checks for updates on startup and will notify you when a new version is available.
                  </p>
                </div>
              </div>
            </div>
          {/if}
        </div>
      </main>
    </Sidebar.Provider>
  </Dialog.Content>
</Dialog.Root>
