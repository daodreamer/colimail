<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { revealItemInDir, openUrl } from "@tauri-apps/plugin-opener";
  import { toast } from "svelte-sonner";
  import * as Breadcrumb from "$lib/components/ui/breadcrumb";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import * as Alert from "$lib/components/ui/alert";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import BellIcon from "lucide-svelte/icons/bell";
  import GlobeIcon from "lucide-svelte/icons/globe";
  import LockIcon from "lucide-svelte/icons/lock";
  import PaintbrushIcon from "lucide-svelte/icons/paintbrush";
  import SettingsIcon from "lucide-svelte/icons/settings";
  import InfoIcon from "lucide-svelte/icons/info";
  import ShieldIcon from "lucide-svelte/icons/shield";
  import RotateCcwIcon from "lucide-svelte/icons/rotate-ccw";
  import SaveIcon from "lucide-svelte/icons/save";
  import XIcon from "lucide-svelte/icons/x";
  import CircleDollarSignIcon from "lucide-svelte/icons/circle-dollar-sign";
  import { state as appState } from "../lib/state.svelte";
  import {
    loadConfig,
    saveConfig,
    resetConfig,
    type CMVHConfig,
    NETWORK_CONFIG,
  } from "$lib/cmvh";
  import { RewardService } from "$lib/cmvh/reward-service";
  import { walletStore } from "$lib/stores/wallet.svelte";
  import type { RewardInfo, UserStats } from "$lib/cmvh/types";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { ScrollArea } from "$lib/components/ui/scroll-area";

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
      { name: "CMVH Verification", icon: ShieldIcon },
      { name: "CMVH Rewards", icon: CircleDollarSignIcon },
      { name: "Advanced", icon: SettingsIcon },
      { name: "About", icon: InfoIcon },
    ],
  };

  // Navigation state
  let currentPage = $state("Notifications");

  // Settings state
  let syncInterval = $state<number>(300);
  let walletSessionTimeout = $state<number>(86400); // Default: 24 hours
  let notificationEnabled = $state<boolean>(true);
  let soundEnabled = $state<boolean>(true);
  let minimizeToTray = $state<boolean>(true);
  let isSaving = $state(false);
  let isCheckingUpdate = $state(false);
  let isExportingLogs = $state(false);
  let appVersion = $state("0.6.3");

  // CMVH settings state
  let cmvhConfig = $state<CMVHConfig>(loadConfig());
  let showResetCMVHDialog = $state(false);

  // CMVH Rewards state
  let rewardsSent = $state<RewardInfo[]>([]);
  let rewardsReceived = $state<RewardInfo[]>([]);
  let userStats = $state<UserStats | null>(null);
  let isLoadingRewards = $state(false);
  let isCancellingReward = $state<string | null>(null);

  // WalletConnect QR code display state (local to Settings Dialog)
  let showWalletConnectQR = $state(false);

  // Encryption state
  interface EncryptionStatus {
    enabled: boolean;
    unlocked: boolean;
  }
  let encryptionStatus = $state<EncryptionStatus>({
    enabled: false,
    unlocked: false,
  });
  let masterPassword = $state("");
  let confirmPassword = $state("");
  let unlockPassword = $state("");
  let isEnablingEncryption = $state(false);

  // Change password state
  let showChangePassword = $state(false);
  let oldPassword = $state("");
  let newPassword = $state("");
  let confirmNewPassword = $state("");
  let isChangingPassword = $state(false);
  let showConfirmPasswordChange = $state(false);

  // Load settings when dialog opens
  $effect(() => {
    if (open) {
      loadSettings();
    }
  });

  async function loadSettings() {
    try {
      syncInterval = await invoke<number>("get_sync_interval");
      walletSessionTimeout = await invoke<number>("get_wallet_session_timeout");
      notificationEnabled = await invoke<boolean>("get_notification_enabled");
      soundEnabled = await invoke<boolean>("get_sound_enabled");
      minimizeToTray = await invoke<boolean>("get_minimize_to_tray");
      encryptionStatus = await invoke<EncryptionStatus>(
        "get_encryption_status",
      );

      // Reload CMVH config from storage
      const { loadConfigAsync } = await import("$lib/cmvh");
      cmvhConfig = await loadConfigAsync();
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
      encryptionStatus = await invoke<EncryptionStatus>(
        "get_encryption_status",
      );
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
      await invoke("unlock_encryption_with_password", {
        password: unlockPassword,
      });
      encryptionStatus = await invoke<EncryptionStatus>(
        "get_encryption_status",
      );
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
      encryptionStatus = await invoke<EncryptionStatus>(
        "get_encryption_status",
      );
      toast.success("Encryption locked");
    } catch (error) {
      console.error("Failed to lock encryption:", error);
      toast.error("Failed to lock encryption");
    }
  }

  function validateAndShowConfirmation() {
    // Validation
    if (!oldPassword) {
      toast.error("Please enter your current password");
      return;
    }
    if (newPassword.length < 8) {
      toast.error("New password must be at least 8 characters long");
      return;
    }
    if (newPassword !== confirmNewPassword) {
      toast.error("New passwords do not match");
      return;
    }
    if (oldPassword === newPassword) {
      toast.error("New password must be different from current password");
      return;
    }

    // Show confirmation dialog
    showConfirmPasswordChange = true;
  }

  async function confirmChangeMasterPassword() {
    showConfirmPasswordChange = false;
    isChangingPassword = true;
    try {
      await invoke("change_master_password", {
        oldPassword: oldPassword,
        newPassword: newPassword,
      });
      toast.success("Master password changed successfully!");

      // Clear email cache from UI state
      appState.emails = [];
      appState.selectedEmailUid = null;
      appState.emailBody = null;
      appState.attachments = [];

      // Clear form and hide
      oldPassword = "";
      newPassword = "";
      confirmNewPassword = "";
      showChangePassword = false;
    } catch (error) {
      console.error("Failed to change password:", error);
      const errorMsg = String(error);

      // Show specific error message for wrong password
      if (errorMsg.includes("Invalid old password")) {
        toast.error("Current password is incorrect");
      } else {
        toast.error(errorMsg);
      }
    } finally {
      isChangingPassword = false;
    }
  }

  function cancelChangePassword() {
    oldPassword = "";
    newPassword = "";
    confirmNewPassword = "";
    showChangePassword = false;
  }

  async function saveNotificationSettings() {
    isSaving = true;
    try {
      await invoke("set_sync_interval", { interval: syncInterval });
      await invoke("set_wallet_session_timeout", { timeoutSeconds: walletSessionTimeout });
      await invoke("set_notification_enabled", {
        enabled: notificationEnabled,
      });
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

  function getWalletTimeoutDescription(seconds: number): string {
    if (seconds === 0) return "Never expire";
    if (seconds === 3600) return "1 hour";
    if (seconds === 86400) return "1 day (24 hours)";
    if (seconds === 604800) return "1 week (7 days)";
    if (seconds === 2592000) return "1 month (30 days)";
    return `${seconds} seconds`;
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
          `A new version (${update.version}) is available!\n\nRelease notes:\n${update.body || "No release notes provided."}\n\nWould you like to download and install it now?`,
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

  // Export logs as ZIP
  async function exportLogs() {
    isExportingLogs = true;
    try {
      const zipPath = await invoke<string>("export_logs_as_zip");

      // Automatically reveal the file in explorer
      try {
        await revealItemInDir(zipPath);
        toast.success(`Logs exported successfully!`);
      } catch (openError) {
        console.error("Failed to reveal file:", openError);
        // Show success with file path if reveal fails
        toast.success(`Logs exported successfully!\nSaved to: ${zipPath}`, {
          duration: 8000,
        });
      }
    } catch (error) {
      console.error("Failed to export logs:", error);
      toast.error(`Failed to export logs: ${error}`);
    } finally {
      isExportingLogs = false;
    }
  }

  // CMVH Settings Functions
  async function saveCMVHSettings() {
    // Validate signing configuration if enabled
    if (cmvhConfig.enableSigning && !walletStore.isConnected) {
      toast.error("Please connect your wallet before enabling email signing");
      return;
    }

    try {
      await saveConfig(cmvhConfig);
      toast.success("CMVH settings saved successfully");
    } catch (error) {
      console.error("Failed to save CMVH settings:", error);
      toast.error(`Failed to save settings: ${error}`);
    }
  }

  async function handleResetCMVH() {
    try {
      cmvhConfig = await resetConfig();
      showResetCMVHDialog = false;
      toast.success("CMVH settings reset to defaults");
    } catch (error) {
      console.error("Failed to reset CMVH settings:", error);
      toast.error(`Failed to reset settings: ${error}`);
    }
  }

  async function dismissOnboarding() {
    try {
      cmvhConfig.hasSeenOnboarding = true;
      await saveConfig(cmvhConfig);
    } catch (error) {
      console.error("Failed to save onboarding status:", error);
    }
  }
  async function fetchRewards() {
    // Only fetch rewards if wallet is connected
    if (!walletStore.isConnected || !walletStore.address) {
      return;
    }

    const addressToCheck = walletStore.address;
    if (!addressToCheck) return;

    isLoadingRewards = true;
    try {
      const rewardService = new RewardService(cmvhConfig);

      // Fetch user stats
      userStats = await rewardService.getUserStats(addressToCheck);

      // Fetch all rewards involved with user (sent and received)
      // The contract `getUserRewards` returns rewards where user is recipient.
      // To get sent rewards, we might need to filter events or if the contract supports it.
      // `getUserRewards` in `CMVHRewardPool` returns `rewardIds` for a user.
      // Wait, `getUserRewards` returns `uint256[]`.
      // And `getRewardInfo` gets info for a ID.
      // The contract doesn't explicitly separate "sent" vs "received" in `getUserRewards`?
      // Let's check `CMVHRewardPool.sol`.
      // `userRewards[user]` stores IDs where user is recipient?
      // `mapping(address => uint256[]) public userRewards;`
      // `_createReward` pushes to `userRewards[recipient]`.
      // It does NOT push to sender.
      // So `getUserRewards` only returns RECEIVED rewards.
      // To get SENT rewards, we would need to index events or store locally.
      // For now, we will only show RECEIVED rewards from chain.
      // We can show "Sent" if we had a local store, but we don't yet.
      // So I will only show "Received Rewards" for now, and maybe "Sent" if I can find a way.
      // Actually, `RewardService` could query events?
      // `getPastEvents("RewardCreated", { filter: { creator: user } })`?
      // `viem` supports `getContractEvents`.
      // I'll stick to Received for now to be safe and simple.

      const rewardIds = await rewardService.getUserRewards(
        addressToCheck,
        true,
      );

      // Fetch details for each reward
      const rewardPromises = rewardIds.map((id) =>
        rewardService.getRewardInfo(id),
      );
      rewardsReceived = await Promise.all(rewardPromises);
    } catch (error) {
      console.error("Failed to fetch rewards:", error);
      toast.error("Failed to fetch rewards");
    } finally {
      isLoadingRewards = false;
    }
  }

  async function handleCancelReward(rewardId: string) {
    if (!walletStore.isConnected) {
      toast.error("Please connect wallet to cancel reward");
      return;
    }

    if (
      !confirm(
        "Are you sure you want to cancel this reward? The funds will be returned to you.",
      )
    )
      return;

    isCancellingReward = rewardId;
    try {
      const rewardService = new RewardService(cmvhConfig);
      const tx = await rewardService.cancelReward(rewardId);
      toast.success("Reward cancelled successfully");
      await fetchRewards();
    } catch (error) {
      console.error("Failed to cancel reward:", error);
      toast.error(`Failed to cancel reward: ${error}`);
    } finally {
      isCancellingReward = null;
    }
  }

  // Fetch rewards when tab is selected
  $effect(() => {
    if (currentPage === "CMVH Rewards" && open) {
      fetchRewards();
    }
  });

  // Format address helper
  function formatAddress(address: string): string {
    if (!address || address.length < 10) return address;
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  }

  // Handle wallet connect click in Settings
  async function handleSettingsWalletConnect() {
    showWalletConnectQR = true;
    await walletStore.connect();
  }

  // Handle cancel connection in Settings
  async function handleSettingsCancelConnection() {
    await walletStore.cancelConnection();
    showWalletConnectQR = false;
  }

  // Monitor wallet connection status to hide QR when connected
  $effect(() => {
    if (walletStore.isConnected) {
      showWalletConnectQR = false;
    }
  });

  // Auto-cancel connection when Settings Dialog closes
  $effect(() => {
    // Cleanup function - runs when dialog closes
    if (!open && showWalletConnectQR && walletStore.isConnecting && !walletStore.isConnected) {
      console.log("Settings Dialog closed - auto-cancelling wallet connection");
      handleSettingsCancelConnection();
    }
  });
</script>

<Dialog.Root bind:open {onOpenChange}>
  <Dialog.Content
    class="overflow-hidden p-0 md:max-h-[500px] md:max-w-[700px] lg:max-w-[800px]"
    trapFocus={false}
  >
    <Dialog.Title class="sr-only">Settings</Dialog.Title>
    <Dialog.Description class="sr-only"
      >Customize your settings here.</Dialog.Description
    >

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
                    Current: <strong
                      >{getIntervalDescription(syncInterval)}</strong
                    >
                  </p>
                </div>
              </div>

              <Separator />

              <!-- Wallet Security -->
              <div class="space-y-3">
                <h4 class="text-sm font-medium mb-3">Wallet Security</h4>
                <div class="space-y-2">
                  <Label for="wallet-timeout">Wallet connection timeout</Label>
                  <select
                    id="wallet-timeout"
                    bind:value={walletSessionTimeout}
                    class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                  >
                    <option value={3600}>1 hour</option>
                    <option value={86400}>1 day (24 hours) - Recommended</option>
                    <option value={604800}>1 week (7 days)</option>
                    <option value={2592000}>1 month (30 days)</option>
                    <option value={0}>Never expire (not recommended)</option>
                  </select>
                  <p class="text-xs text-muted-foreground">
                    Current: <strong>{getWalletTimeoutDescription(walletSessionTimeout)}</strong>
                  </p>
                  <p class="text-xs text-amber-600">
                    🔐 Wallet session will expire after this period. You'll need to reconnect for security.
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
                  <h4 class="text-sm font-medium mb-1">
                    Local data encryption
                  </h4>
                  <p class="text-xs text-muted-foreground">
                    Encrypt email content stored in the local cache database
                  </p>
                </div>

                <!-- Encryption Status Badge -->
                <div class="flex items-center gap-2">
                  {#if encryptionStatus.enabled}
                    {#if encryptionStatus.unlocked}
                      <div
                        class="inline-flex items-center gap-1.5 rounded-full bg-green-100 dark:bg-green-900/30 px-3 py-1 text-xs font-medium text-green-800 dark:text-green-200"
                      >
                        <span
                          class="h-1.5 w-1.5 rounded-full bg-green-600 dark:bg-green-400"
                        ></span>
                        Unlocked
                      </div>
                    {:else}
                      <div
                        class="inline-flex items-center gap-1.5 rounded-full bg-amber-100 dark:bg-amber-900/30 px-3 py-1 text-xs font-medium text-amber-800 dark:text-amber-200"
                      >
                        <span
                          class="h-1.5 w-1.5 rounded-full bg-amber-600 dark:bg-amber-400"
                        ></span>
                        Locked
                      </div>
                    {/if}
                  {:else}
                    <div
                      class="inline-flex items-center gap-1.5 rounded-full bg-gray-100 dark:bg-gray-800 px-3 py-1 text-xs font-medium text-gray-700 dark:text-gray-300"
                    >
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
                        <strong>Important:</strong> Your master password cannot be
                        recovered if lost. Make sure to remember it or store it securely.
                      </p>
                    </div>
                    <Button
                      onclick={enableEncryption}
                      disabled={isEnablingEncryption}
                    >
                      {isEnablingEncryption
                        ? "Enabling..."
                        : "Enable encryption"}
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
                        onkeydown={(e) =>
                          e.key === "Enter" && unlockEncryption()}
                      />
                    </div>
                    <Button onclick={unlockEncryption}>
                      Unlock encryption
                    </Button>
                  </div>
                {:else}
                  <!-- Encryption Active - Show Lock Button and Change Password -->
                  <div class="rounded-lg border bg-card p-4 space-y-4">
                    <div class="flex items-center gap-3">
                      <div class="flex-shrink-0">
                        <svg
                          class="h-10 w-10 text-green-600 dark:text-green-400"
                          fill="none"
                          viewBox="0 0 24 24"
                          stroke="currentColor"
                        >
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                          />
                        </svg>
                      </div>
                      <div class="flex-1">
                        <p class="text-sm font-medium">Encryption is active</p>
                        <p class="text-xs text-muted-foreground">
                          Your email data is being encrypted
                        </p>
                      </div>
                    </div>

                    <div class="flex gap-2">
                      <Button onclick={lockEncryption} variant="outline">
                        Lock encryption
                      </Button>
                      <Button
                        onclick={() =>
                          (showChangePassword = !showChangePassword)}
                        variant="outline"
                      >
                        {showChangePassword ? "Cancel" : "Change password"}
                      </Button>
                    </div>

                    {#if showChangePassword}
                      <Separator />
                      <div class="space-y-4 pt-2">
                        <h5 class="text-sm font-medium">
                          Change master password
                        </h5>

                        <div class="space-y-2">
                          <Label for="old-password" class="text-sm">
                            Current password
                          </Label>
                          <input
                            id="old-password"
                            type="password"
                            bind:value={oldPassword}
                            placeholder="Enter current password"
                            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                          />
                        </div>

                        <div class="space-y-2">
                          <Label for="new-password" class="text-sm">
                            New password
                          </Label>
                          <input
                            id="new-password"
                            type="password"
                            bind:value={newPassword}
                            placeholder="Enter new password (min. 8 characters)"
                            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                          />
                        </div>

                        <div class="space-y-2">
                          <Label for="confirm-new-password" class="text-sm">
                            Confirm new password
                          </Label>
                          <input
                            id="confirm-new-password"
                            type="password"
                            bind:value={confirmNewPassword}
                            placeholder="Re-enter new password"
                            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                          />
                        </div>

                        <div
                          class="rounded-md bg-amber-50 dark:bg-amber-950/30 p-3 space-y-2"
                        >
                          <p class="text-xs text-amber-900 dark:text-amber-200">
                            <strong>Important:</strong>
                          </p>
                          <ul
                            class="text-xs text-amber-900 dark:text-amber-200 list-disc list-inside space-y-1"
                          >
                            <li>
                              There is no password recovery option. Make sure to
                              remember your new password.
                            </li>
                            <li>
                              <strong
                                >All cached email data will be cleared</strong
                              > after changing the password.
                            </li>
                            <li>
                              You will need to sync your emails again after the
                              password change.
                            </li>
                          </ul>
                        </div>

                        <div class="flex gap-2">
                          <Button
                            onclick={validateAndShowConfirmation}
                            disabled={isChangingPassword ||
                              !oldPassword ||
                              !newPassword ||
                              !confirmNewPassword}
                          >
                            {isChangingPassword
                              ? "Changing..."
                              : "Change password"}
                          </Button>
                          <Button
                            onclick={cancelChangePassword}
                            variant="outline"
                          >
                            Cancel
                          </Button>
                        </div>
                      </div>
                    {/if}
                  </div>
                {/if}

                <!-- Info Section -->
                <div class="rounded-lg border bg-muted/50 p-4 space-y-2">
                  <h5 class="text-xs font-semibold">What gets encrypted?</h5>
                  <ul
                    class="text-xs text-muted-foreground space-y-1 ml-4 list-disc"
                  >
                    <li>Email subjects</li>
                    <li>Email body content</li>
                    <li>Email attachments</li>
                  </ul>
                  <p class="text-xs text-muted-foreground mt-3">
                    <strong>Note:</strong> Email metadata (sender, recipient, date)
                    is not encrypted for performance reasons.
                  </p>
                </div>
              </div>
            </div>
          {:else if currentPage === "CMVH Verification"}
            <div class="bg-muted/50 rounded-xl p-6 space-y-6 max-w-3xl">
              <!-- Onboarding Guide -->
              {#if !cmvhConfig.hasSeenOnboarding}
                <div class="relative">
                  <Alert.Root
                    class="border-blue-200 bg-blue-50 dark:border-blue-900 dark:bg-blue-950/30"
                  >
                    <InfoIcon class="text-blue-600 dark:text-blue-400" />
                    <Alert.Title
                      class="text-base font-semibold text-blue-900 dark:text-blue-100"
                    >
                      What is CMVH?
                    </Alert.Title>
                    <Alert.Description
                      class="text-sm text-blue-800 dark:text-blue-200 space-y-3"
                    >
                      <p>
                        CMVH (ColiMail Verification Header) uses blockchain
                        cryptography to enhance email security and
                        authenticity:
                      </p>
                      <ul class="list-disc pl-5 space-y-1">
                        <li>
                          Prove your identity without revealing passwords
                        </li>
                        <li>Prevent email spoofing and tampering</li>
                        <li>Enable verifiable on-chain reputation</li>
                        <li>Maintain privacy while ensuring authenticity</li>
                      </ul>
                      <div class="flex items-center gap-3 pt-2">
                        <Button
                          variant="link"
                          class="h-auto p-0 text-blue-700 dark:text-blue-300 hover:text-blue-900 dark:hover:text-blue-100"
                          onclick={() =>
                            openUrl("https://docs.colimail.net/cmvh")}
                        >
                          Learn more →
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          class="text-blue-700 dark:text-blue-300 hover:text-blue-900 dark:hover:text-blue-100"
                          onclick={dismissOnboarding}
                        >
                          Got it, don't show again
                        </Button>
                      </div>
                    </Alert.Description>
                  </Alert.Root>
                  <Button
                    variant="ghost"
                    size="icon"
                    class="absolute top-3 right-3 h-6 w-6 text-blue-600 dark:text-blue-400 hover:text-blue-900 dark:hover:text-blue-100"
                    onclick={dismissOnboarding}
                  >
                    <XIcon class="h-4 w-4" />
                  </Button>
                </div>
              {/if}

              <!-- Email Signing Settings -->
              <div class="space-y-4">
                <div>
                  <h4 class="text-sm font-medium mb-1">
                    Email Signature Creation
                  </h4>
                  <p class="text-xs text-muted-foreground">
                    Sign outgoing emails with your private key to prove
                    authenticity
                  </p>
                </div>

                <div class="space-y-0.5">
                  <Label for="cmvh-signing-enabled" class="text-sm"
                    >Enable Email Signing</Label
                  >
                  <p class="text-xs text-muted-foreground">
                    Sign outgoing emails with your wallet via WalletConnect
                  </p>
                </div>
                <input
                  type="checkbox"
                  id="cmvh-signing-enabled"
                  bind:checked={cmvhConfig.enableSigning}
                  class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
                />

                {#if cmvhConfig.enableSigning}
                  <Separator />

                  <!-- WalletConnect Integration -->
                  <div class="rounded-lg border bg-card p-4 space-y-4">
                    {#if walletStore.isConnected}
                      <!-- Connected State -->
                      <div class="space-y-4">
                        <div class="flex items-center justify-between">
                          <div class="flex items-center gap-2">
                            <div class="h-2 w-2 rounded-full bg-green-500 animate-pulse"></div>
                            <span class="text-sm font-medium text-green-700 dark:text-green-400">
                              Wallet Connected
                            </span>
                          </div>
                          <Button
                            variant="outline"
                            size="sm"
                            onclick={() => walletStore.disconnect()}
                          >
                            Disconnect
                          </Button>
                        </div>

                        <div class="space-y-3 rounded-md bg-muted/50 p-3">
                          <!-- ENS Name or Address -->
                          <div class="space-y-1">
                            <Label class="text-xs text-muted-foreground">Identity</Label>
                            <div class="flex items-center gap-2">
                              {#if walletStore.isResolvingENS}
                                <div class="flex items-center gap-2">
                                  <div class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
                                  <span class="text-sm">Resolving ENS...</span>
                                </div>
                              {:else if walletStore.ensName}
                                <div class="flex flex-col gap-1">
                                  <span class="text-sm font-medium font-mono">
                                    {walletStore.ensName}
                                  </span>
                                  <span class="text-xs text-muted-foreground font-mono">
                                    {walletStore.formatAddress()}
                                  </span>
                                </div>
                              {:else}
                                <span class="text-sm font-mono">
                                  {walletStore.formatAddress(walletStore.address, 'full')}
                                </span>
                              {/if}
                            </div>
                          </div>

                          <!-- Chain Info -->
                          {#if walletStore.chainId}
                            <div class="space-y-1">
                              <Label class="text-xs text-muted-foreground">Network</Label>
                              <span class="text-sm">
                                {walletStore.chainId === 421614 ? 'Arbitrum Sepolia' :
                                 walletStore.chainId === 42161 ? 'Arbitrum One' :
                                 `Chain ${walletStore.chainId}`}
                              </span>
                            </div>
                          {/if}
                        </div>

                        <Alert.Root class="border-green-200 bg-green-50 dark:border-green-900 dark:bg-green-950/30">
                          <InfoIcon class="text-green-600 dark:text-green-400" />
                          <Alert.Description class="text-sm text-green-800 dark:text-green-200">
                            Your wallet is securely connected via WalletConnect.
                            Your private key never leaves your device.
                            All signatures are confirmed on your mobile wallet.
                          </Alert.Description>
                        </Alert.Root>
                      </div>
                    {:else}
                      <!-- Disconnected State -->
                      <div class="space-y-4">
                        <Alert.Root>
                          <InfoIcon />
                          <Alert.Title>Wallet Not Connected</Alert.Title>
                          <Alert.Description>
                            <p>
                              Connect your wallet via WalletConnect to enable CMVH email signing.
                              Your private key stays secure on your mobile device.
                            </p>
                            <ul class="list-disc pl-5 space-y-1 mt-2">
                              <li>No private key storage in the app</li>
                              <li>Sign emails directly from your mobile wallet</li>
                              <li>Works with MetaMask, Trust Wallet, Rainbow, and more</li>
                              <li>Hardware wallet support (Ledger, Trezor)</li>
                            </ul>
                          </Alert.Description>
                        </Alert.Root>

                        <div class="flex justify-center">
                          <Button
                            onclick={handleSettingsWalletConnect}
                            disabled={walletStore.isConnecting}
                            class="w-full sm:w-auto"
                          >
                            {#if walletStore.isConnecting}
                              <div class="flex items-center gap-2">
                                <div class="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent"></div>
                                <span>Connecting...</span>
                              </div>
                            {:else}
                              🔌 Connect Wallet via QR Code
                            {/if}
                          </Button>
                        </div>

                        {#if showWalletConnectQR && walletStore.walletConnectUri}
                          <div class="rounded-lg border-2 border-dashed border-primary/50 bg-white dark:bg-muted p-4 space-y-3">
                            <p class="text-xs font-medium mb-3 text-center">
                              Scan with mobile wallet
                            </p>
                            <div class="flex justify-center">
                              <img
                                src={`https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(
                                  walletStore.walletConnectUri,
                                )}`}
                                alt="WalletConnect QR Code"
                                width="200"
                                height="200"
                                class="rounded"
                              />
                            </div>
                            <p class="text-xs text-muted-foreground text-center">
                              Open MetaMask, Trust Wallet, Rainbow, or any WalletConnect-compatible wallet
                            </p>
                            <div class="flex justify-center pt-2">
                              <Button
                                variant="outline"
                                size="sm"
                                onclick={handleSettingsCancelConnection}
                                class="w-full sm:w-auto"
                              >
                                Cancel Connection
                              </Button>
                            </div>
                          </div>
                        {/if}

                        {#if walletStore.error}
                          <Alert.Root variant="destructive">
                            <Alert.Description>
                              {walletStore.error}
                            </Alert.Description>
                          </Alert.Root>
                        {/if}
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>

              <Separator />

              <!-- General Settings -->
              <div class="space-y-4">
                <div>
                  <h4 class="text-sm font-medium mb-1">
                    Email Signature Verification
                  </h4>
                  <p class="text-xs text-muted-foreground">
                    Verify email signatures using CMVH (ColiMail Verification
                    Header) blockchain standard
                  </p>
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label for="cmvh-enabled" class="text-sm"
                      >Enable CMVH Verification</Label
                    >
                    <p class="text-xs text-muted-foreground">
                      Automatically verify email signatures when available
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    id="cmvh-enabled"
                    bind:checked={cmvhConfig.enabled}
                    class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary"
                  />
                </div>

                <Separator />

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label for="cmvh-auto-verify" class="text-sm"
                      >Auto-verify on Email Open</Label
                    >
                    <p class="text-xs text-muted-foreground">
                      Automatically verify signatures when opening emails
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    id="cmvh-auto-verify"
                    bind:checked={cmvhConfig.autoVerify}
                    disabled={!cmvhConfig.enabled}
                    class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary disabled:opacity-50"
                  />
                </div>

                <Separator />

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label for="cmvh-onchain" class="text-sm"
                      >Enable On-Chain Verification</Label
                    >
                    <p class="text-xs text-muted-foreground">
                      Verify signatures using smart contracts (slower but more
                      secure)
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    id="cmvh-onchain"
                    bind:checked={cmvhConfig.verifyOnChain}
                    disabled={!cmvhConfig.enabled}
                    class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-2 focus:ring-primary disabled:opacity-50"
                  />
                </div>
              </div>

              <Separator />

              <!-- Blockchain Settings -->
              <div class="space-y-4">
                <div>
                  <h4 class="text-sm font-medium mb-1">Blockchain Settings</h4>
                  <p class="text-xs text-muted-foreground">
                    Configure blockchain network and RPC endpoint for on-chain
                    verification
                  </p>
                </div>

                <div class="space-y-2">
                  <Label for="cmvh-network" class="text-sm">Network</Label>
                  <select
                    id="cmvh-network"
                    bind:value={cmvhConfig.network}
                    disabled={!cmvhConfig.enabled}
                    class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50"
                  >
                    <option value="arbitrum-sepolia"
                      >Arbitrum Sepolia (Testnet)</option
                    >
                    <option value="arbitrum">Arbitrum One (Mainnet)</option>
                  </select>
                  <p class="text-xs text-muted-foreground">
                    Chain ID: {NETWORK_CONFIG[cmvhConfig.network].chainId} | Explorer:
                    {NETWORK_CONFIG[cmvhConfig.network].explorerUrl}
                  </p>
                </div>

                <div class="space-y-2">
                  <Label for="cmvh-rpc" class="text-sm">RPC Endpoint</Label>
                  <input
                    id="cmvh-rpc"
                    type="url"
                    bind:value={cmvhConfig.rpcUrl}
                    placeholder={NETWORK_CONFIG[cmvhConfig.network].rpcUrl}
                    disabled={!cmvhConfig.enabled}
                    class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-50"
                  />
                  <p class="text-xs text-muted-foreground">
                    Leave empty to use default public RPC endpoint
                  </p>
                </div>

                <div class="space-y-2">
                  <Label for="cmvh-contract" class="text-sm"
                    >Contract Address</Label
                  >
                  <input
                    id="cmvh-contract"
                    type="text"
                    value={cmvhConfig.contractAddress ||
                      NETWORK_CONFIG[cmvhConfig.network].contractAddress}
                    readonly
                    disabled={!cmvhConfig.enabled}
                    class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono ring-offset-background focus-visible:outline-none disabled:opacity-50"
                  />
                  <p class="text-xs text-muted-foreground">
                    CMVHVerifier contract address (read-only)
                  </p>
                </div>
              </div>

              <!-- Info Section -->
              <div
                class="rounded-lg border bg-blue-50 dark:bg-blue-950/30 p-4 space-y-2"
              >
                <h5
                  class="text-xs font-semibold text-blue-900 dark:text-blue-200"
                >
                  What is CMVH?
                </h5>
                <p class="text-xs text-blue-900 dark:text-blue-200">
                  CMVH (ColiMail Verification Header) is a blockchain-based
                  email authentication system that allows you to verify the
                  sender's identity and ensure email content hasn't been
                  tampered with.
                </p>
              </div>

              <!-- Save/Reset Buttons -->
              <div class="flex justify-between gap-2 pt-4">
                <Button
                  variant="outline"
                  onclick={() => (showResetCMVHDialog = true)}
                >
                  <RotateCcwIcon class="h-4 w-4 mr-2" />
                  Reset to Defaults
                </Button>
                <Button onclick={saveCMVHSettings}>
                  <SaveIcon class="h-4 w-4 mr-2" />
                  Save Settings
                </Button>
              </div>
            </div>
          {:else if currentPage === "CMVH Rewards"}
            <div class="bg-muted/50 rounded-xl p-6 space-y-6 max-w-3xl">
              <!-- User Stats -->
              <div class="grid grid-cols-2 gap-4">
                <Card>
                  <CardHeader class="pb-2">
                    <CardTitle class="text-sm font-medium"
                      >Total Received</CardTitle
                    >
                  </CardHeader>
                  <CardContent>
                    <div class="text-2xl font-bold">
                      {userStats?.totalReceived || "0"}
                    </div>
                    <p class="text-xs text-muted-foreground">
                      {userStats?.totalAmountReceived
                        ? (
                            Number(userStats.totalAmountReceived) / 1e18
                          ).toFixed(4)
                        : "0"} ETH
                    </p>
                  </CardContent>
                </Card>
                <Card>
                  <CardHeader class="pb-2">
                    <CardTitle class="text-sm font-medium"
                      >Active Rewards</CardTitle
                    >
                  </CardHeader>
                  <CardContent>
                    <div class="text-2xl font-bold">
                      {userStats?.activeRewards || "0"}
                    </div>
                    <p class="text-xs text-muted-foreground">
                      Waiting to be claimed
                    </p>
                  </CardContent>
                </Card>
              </div>

              <Separator />

              <!-- Rewards List -->
              <div class="space-y-4">
                <div class="flex items-center justify-between">
                  <h4 class="text-sm font-medium">Received Rewards</h4>
                  <Button
                    variant="outline"
                    size="sm"
                    onclick={fetchRewards}
                    disabled={isLoadingRewards}
                  >
                    {#if isLoadingRewards}
                      <svg
                        class="animate-spin -ml-1 mr-2 h-4 w-4"
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                      >
                        <circle
                          class="opacity-25"
                          cx="12"
                          cy="12"
                          r="10"
                          stroke="currentColor"
                          stroke-width="4"
                        ></circle>
                        <path
                          class="opacity-75"
                          fill="currentColor"
                          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                        ></path>
                      </svg>
                      Refreshing...
                    {:else}
                      <RotateCcwIcon class="h-4 w-4 mr-2" />
                      Refresh
                    {/if}
                  </Button>
                </div>

                {#if isLoadingRewards && rewardsReceived.length === 0}
                  <div class="space-y-3">
                    <Skeleton class="h-20 w-full" />
                    <Skeleton class="h-20 w-full" />
                  </div>
                {:else if rewardsReceived.length === 0}
                  <div
                    class="flex flex-col items-center justify-center py-8 text-center text-muted-foreground"
                  >
                    <CircleDollarSignIcon class="h-12 w-12 mb-4 opacity-20" />
                    <p>No rewards found</p>
                  </div>
                {:else}
                  <ScrollArea class="h-[300px] pr-4">
                    <div class="space-y-3">
                      {#each rewardsReceived as reward}
                        <div
                          class="rounded-lg border bg-card p-4 transition-colors hover:bg-accent/50"
                        >
                          <div class="flex items-start justify-between">
                            <div class="space-y-1">
                              <div class="flex items-center gap-2">
                                <span class="font-semibold">
                                  {(Number(reward.amount) / 1e18).toFixed(4)} ETH
                                </span>
                                {#if reward.claimed}
                                  <span
                                    class="rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900/30 dark:text-green-400"
                                  >
                                    Claimed
                                  </span>
                                {:else}
                                  <span
                                    class="rounded-full bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-800 dark:bg-blue-900/30 dark:text-blue-400"
                                  >
                                    Available
                                  </span>
                                {/if}
                              </div>
                              <p class="text-xs text-muted-foreground">
                                From: {formatAddress(reward.sender)}
                              </p>
                              <p class="text-xs text-muted-foreground">
                                Date: {new Date(
                                  Number(reward.timestamp) * 1000,
                                ).toLocaleString()}
                              </p>
                            </div>
                            {#if !reward.claimed && reward.sender.toLowerCase() === (walletStore.address?.toLowerCase() || "")}
                              <Button
                                variant="destructive"
                                size="sm"
                                disabled={isCancellingReward ===
                                  reward.rewardId}
                                onclick={() =>
                                  handleCancelReward(reward.rewardId)}
                              >
                                {isCancellingReward === reward.rewardId
                                  ? "Cancelling..."
                                  : "Cancel"}
                              </Button>
                            {/if}
                          </div>
                        </div>
                      {/each}
                    </div>
                  </ScrollArea>
                {/if}
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
                  Keep the application running in the system tray when you close
                  the window. Click the tray icon to restore the window.
                </p>
                {#if !minimizeToTray}
                  <div
                    class="rounded-md bg-amber-50 p-2 dark:bg-amber-950/30 ml-7"
                  >
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
                  <span class="text-sm text-muted-foreground">{appVersion}</span
                  >
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
                  Check for the latest version to get new features and bug
                  fixes.
                </p>
                <Button
                  onclick={checkForUpdates}
                  disabled={isCheckingUpdate}
                  class="w-full"
                >
                  {isCheckingUpdate
                    ? "Checking for Updates..."
                    : "Check for Updates"}
                </Button>

                <div class="rounded-md bg-blue-50 p-3 dark:bg-blue-950/30">
                  <p class="text-xs text-blue-900 dark:text-blue-200">
                    💡 <strong>Auto Update:</strong> The application automatically
                    checks for updates on startup and will notify you when a new
                    version is available.
                  </p>
                </div>
              </div>

              <Separator />

              <!-- Export Logs -->
              <div class="space-y-3">
                <h4 class="text-sm font-medium">Debug Logs</h4>
                <p class="text-xs text-muted-foreground">
                  Export application logs as a ZIP file to help the development
                  team troubleshoot issues.
                </p>
                <Button
                  onclick={exportLogs}
                  disabled={isExportingLogs}
                  variant="outline"
                  class="w-full"
                >
                  {isExportingLogs ? "Exporting Logs..." : "Export Logs as ZIP"}
                </Button>

                <div class="rounded-md bg-amber-50 p-3 dark:bg-amber-950/30">
                  <p class="text-xs text-amber-900 dark:text-amber-200">
                    📋 <strong>Bug Reports:</strong> When reporting issues on GitHub,
                    please attach the exported log file to help us diagnose the problem
                    faster.
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

<!-- Confirm Password Change Dialog -->
<AlertDialog.Root bind:open={showConfirmPasswordChange}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Confirm Password Change</AlertDialog.Title>
      <AlertDialog.Description>
        Are you sure you want to change your master password? This action will:
        <ul class="list-disc list-inside mt-2 space-y-1">
          <li>Delete all cached email data</li>
          <li>Require you to sync your emails again</li>
          <li>Cannot be undone</li>
        </ul>
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action onclick={confirmChangeMasterPassword}>
        Yes, change password
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

<!-- Confirm CMVH Reset Dialog -->
<AlertDialog.Root bind:open={showResetCMVHDialog}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Reset CMVH Settings?</AlertDialog.Title>
      <AlertDialog.Description>
        This will reset all CMVH verification settings to their default values.
        This action cannot be undone.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={() => (showResetCMVHDialog = false)}
        >Cancel</AlertDialog.Cancel
      >
      <AlertDialog.Action onclick={handleResetCMVH}>Reset</AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
