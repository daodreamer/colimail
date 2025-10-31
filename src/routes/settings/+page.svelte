<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Label } from "$lib/components/ui/label";
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { Separator } from "$lib/components/ui/separator";
  import { toast } from "svelte-sonner";

  let syncInterval = $state<number>(300); // Default 5 minutes
  let isSavingSyncSettings = $state(false);
  let notificationEnabled = $state<boolean>(true);
  let soundEnabled = $state<boolean>(true);
  let isSavingNotificationSettings = $state(false);
  let minimizeToTray = $state<boolean>(true);
  let isSavingWindowSettings = $state(false);

  // Load settings on mount
  onMount(async () => {
    try {
      syncInterval = await invoke<number>("get_sync_interval");
    } catch (error) {
      console.error("Failed to load sync interval:", error);
    }

    try {
      notificationEnabled = await invoke<boolean>("get_notification_enabled");
    } catch (error) {
      console.error("Failed to load notification setting:", error);
    }

    try {
      soundEnabled = await invoke<boolean>("get_sound_enabled");
    } catch (error) {
      console.error("Failed to load sound setting:", error);
    }

    try {
      minimizeToTray = await invoke<boolean>("get_minimize_to_tray");
    } catch (error) {
      console.error("Failed to load minimize to tray setting:", error);
    }
  });

  // Save sync interval setting
  async function saveSyncSettings() {
    isSavingSyncSettings = true;
    try {
      await invoke("set_sync_interval", { interval: syncInterval });
      toast.success("Sync settings saved successfully!");
    } catch (error) {
      console.error("Failed to save sync settings:", error);
      toast.error("Failed to save sync settings");
    } finally {
      isSavingSyncSettings = false;
    }
  }

  // Save notification settings
  async function saveNotificationSettings() {
    isSavingNotificationSettings = true;
    try {
      await invoke("set_notification_enabled", { enabled: notificationEnabled });
      await invoke("set_sound_enabled", { enabled: soundEnabled });
      toast.success("Notification settings saved successfully!");
    } catch (error) {
      console.error("Failed to save notification settings:", error);
      toast.error("Failed to save notification settings");
    } finally {
      isSavingNotificationSettings = false;
    }
  }

  // Save window behavior settings
  async function saveWindowSettings() {
    isSavingWindowSettings = true;
    try {
      await invoke("set_minimize_to_tray", { enabled: minimizeToTray });
      toast.success("Window settings saved successfully!");
    } catch (error) {
      console.error("Failed to save window settings:", error);
      toast.error("Failed to save window settings");
    } finally {
      isSavingWindowSettings = false;
    }
  }

  // Get interval description
  function getIntervalDescription(interval: number): string {
    if (interval === -1) return "Never sync (cache only)";
    if (interval === 0) return "Manual sync (refresh button only)";
    if (interval < 60) return `${interval} seconds`;
    if (interval < 3600) return `${Math.floor(interval / 60)} minutes`;
    return `${Math.floor(interval / 3600)} hours`;
  }
</script>

<div class="mx-auto max-w-3xl p-8 space-y-6">
  <div class="flex items-center gap-4">
    <Button variant="secondary" href="/">← Back</Button>
    <h1 class="text-3xl font-bold">Settings</h1>
  </div>

  <Card>
    <CardHeader>
      <CardTitle>Sync Settings</CardTitle>
      <CardDescription>
        Configure email sync frequency to balance real-time updates and resource usage.
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="space-y-2">
        <Label for="sync-interval">Sync Interval</Label>
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
        <p class="text-sm text-muted-foreground">
          Current setting: <strong class="text-primary">{getIntervalDescription(syncInterval)}</strong>
        </p>
        <p class="text-sm text-muted-foreground">
          {#if syncInterval === 0}
            Emails and folders will only sync when you click the "Refresh" button.
          {:else if syncInterval === -1}
            Emails and folders will not auto-sync, only cached content will be displayed.
          {:else}
            When switching folders or accounts, auto-sync occurs if more than {getIntervalDescription(syncInterval)} have passed since last sync.
          {/if}
        </p>
      </div>

      <Button
        class="w-full"
        onclick={saveSyncSettings}
        disabled={isSavingSyncSettings}
      >
        {isSavingSyncSettings ? "Saving..." : "Save Settings"}
      </Button>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Notification Settings</CardTitle>
      <CardDescription>
        Configure how you're notified when new emails arrive.
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="flex items-center space-x-3">
        <input
          type="checkbox"
          id="notification-enabled"
          bind:checked={notificationEnabled}
          class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
        />
        <Label for="notification-enabled" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
          Enable desktop notifications
        </Label>
      </div>
      <p class="text-sm text-muted-foreground ml-7">
        Show a desktop notification in the bottom-right corner with sender and subject information.
      </p>

      <Separator />

      <div class="flex items-center space-x-3">
        <input
          type="checkbox"
          id="sound-enabled"
          bind:checked={soundEnabled}
          class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
        />
        <Label for="sound-enabled" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
          Enable sound alerts
        </Label>
      </div>
      <p class="text-sm text-muted-foreground ml-7">
        Play a sound when new emails arrive.
      </p>

      <Button
        class="w-full"
        onclick={saveNotificationSettings}
        disabled={isSavingNotificationSettings}
      >
        {isSavingNotificationSettings ? "Saving..." : "Save Settings"}
      </Button>

      <div class="rounded-md bg-blue-50 p-3 dark:bg-blue-950/30">
        <p class="text-sm text-blue-900 dark:text-blue-200">
          💡 <strong>Tip:</strong> When desktop notifications are enabled, a floating notification will appear in the bottom-right corner when new emails arrive, and it will automatically disappear after 3 seconds.
        </p>
      </div>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Window Behavior</CardTitle>
      <CardDescription>
        Configure how the application behaves when you close the window.
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="flex items-center space-x-3">
        <input
          type="checkbox"
          id="minimize-to-tray"
          bind:checked={minimizeToTray}
          class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
        />
        <Label for="minimize-to-tray" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
          Minimize to system tray when closing window
        </Label>
      </div>
      <p class="text-sm text-muted-foreground ml-7">
        When enabled, clicking the close button (X) will minimize the app to the system tray instead of exiting.
        You can restore the window by clicking the tray icon or quit from the tray menu.
      </p>

      {#if !minimizeToTray}
        <div class="rounded-md bg-amber-50 p-3 dark:bg-amber-950/30 ml-7">
          <p class="text-sm text-amber-900 dark:text-amber-200">
            ⚠️ <strong>Warning:</strong> When this option is disabled, closing the window will completely exit the application.
          </p>
        </div>
      {/if}

      <Button
        class="w-full"
        onclick={saveWindowSettings}
        disabled={isSavingWindowSettings}
      >
        {isSavingWindowSettings ? "Saving..." : "Save Settings"}
      </Button>
    </CardContent>
  </Card>
</div>
