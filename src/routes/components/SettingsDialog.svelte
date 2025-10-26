<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
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
    ],
  };

  // Navigation state
  let currentPage = $state("Notifications");

  // Settings state
  let syncInterval = $state<number>(300);
  let notificationEnabled = $state<boolean>(true);
  let soundEnabled = $state<boolean>(true);
  let isSaving = $state(false);

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
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  }

  async function saveNotificationSettings() {
    isSaving = true;
    try {
      await invoke("set_sync_interval", { interval: syncInterval });
      await invoke("set_notification_enabled", { enabled: notificationEnabled });
      await invoke("set_sound_enabled", { enabled: soundEnabled });
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
            <div class="bg-muted/50 rounded-xl p-6 space-y-4 max-w-3xl">
              <p class="text-sm text-muted-foreground">
                Privacy and visibility settings coming soon...
              </p>
            </div>

          {:else if currentPage === "Advanced"}
            <div class="bg-muted/50 rounded-xl p-6 space-y-4 max-w-3xl">
              <p class="text-sm text-muted-foreground">
                Advanced configuration options coming soon...
              </p>
            </div>
          {/if}
        </div>
      </main>
    </Sidebar.Provider>
  </Dialog.Content>
</Dialog.Root>
