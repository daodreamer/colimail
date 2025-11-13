/**
 * Sync and IDLE Event Handlers
 * Handles email synchronization, IDLE push notifications, and auto-sync timers
 */

import { invoke } from "@tauri-apps/api/core";
import type { AccountConfig, EmailHeader, Folder, IdleEvent } from "../lib/types";
import { state as appState } from "../lib/state.svelte";

/**
 * Start auto-sync timer
 */
export function startAutoSyncTimer(
  syncInterval: number,
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedFolderName: string
): ReturnType<typeof setInterval> | null {
  // Clear existing timer if any
  if (autoSyncTimerRef) {
    clearInterval(autoSyncTimerRef);
    autoSyncTimerRef = null;
  }

  if (syncInterval <= 0) {
    return null;
  }

  const timer = setInterval(async () => {
    if (!selectedAccountId) return;

    const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
    if (!selectedConfig) return;

    try {
      const needsSync = await invoke<boolean>("should_sync", {
        accountId: selectedAccountId,
        folder: selectedFolderName,
        syncInterval,
      });

      if (needsSync && !appState.isSyncing) {
        appState.isSyncing = true;

        const syncedFolders = await invoke<Folder[]>("sync_folders", { config: selectedConfig });
        appState.folders = syncedFolders;

        const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
          config: selectedConfig,
          folder: selectedFolderName,
        });

        appState.emails = syncedEmails;
        appState.lastSyncTime = Math.floor(Date.now() / 1000);
        appState.isSyncing = false;
      }
    } catch (e) {
      console.error("❌ Auto-sync failed:", e);
      appState.isSyncing = false;
    }
  }, 60000);

  autoSyncTimerRef = timer;
  return timer;
}

// Module-level variable to track auto-sync timer
let autoSyncTimerRef: ReturnType<typeof setInterval> | null = null;

/**
 * Handle manual refresh button click - sync all accounts and folders
 */
export async function handleManualRefresh() {
  const accounts = appState.accounts;
  const selectedAccountId = appState.selectedAccountId;
  const selectedFolderName = appState.selectedFolderName;

  if (accounts.length === 0) {
    appState.error = "No accounts configured.";
    return;
  }

  appState.isSyncing = true;
  appState.error = null;

  try {
    // Sync all accounts
    for (const account of accounts) {
      try {
        // Sync folders for this account
        const syncedFolders = await invoke<Folder[]>("sync_folders", { config: account });

        // Update folders if this is the currently selected account
        if (account.id === appState.selectedAccountId) {
          appState.folders = syncedFolders;
        }

        // Sync all folders for this account
        for (const folder of syncedFolders) {
          const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
            config: account,
            folder: folder.name,
          });

          // Update emails if this is the currently selected account and folder
          if (account.id === appState.selectedAccountId && folder.name === appState.selectedFolderName) {
            appState.emails = syncedEmails;
          }
        }
      } catch (e) {
        console.error(`❌ Failed to sync account ${account.email}:`, e);
        // Continue with other accounts even if one fails
      }
    }

    appState.lastSyncTime = Math.floor(Date.now() / 1000);
  } catch (e) {
    appState.error = `Failed to refresh: ${e}`;
  } finally {
    appState.isSyncing = false;
  }
}

/**
 * Handle IDLE push notification events from email server
 */
export async function handleIdleEvent(
  event: { payload: IdleEvent }
) {
  const idleEvent = event.payload;
  const eventType = idleEvent.event_type.type;

  // Read current state directly from appState to ensure we have the latest values
  const currentAccountId = appState.selectedAccountId;
  const currentFolderName = appState.selectedFolderName;
  const accounts = appState.accounts;

  console.log(`📨 IDLE Event received: ${eventType}`, {
    accountId: idleEvent.account_id,
    folderName: idleEvent.folder_name,
    currentAccountId,
    currentFolderName,
    eventType: idleEvent.event_type
  });

  if (eventType === "NewMessages") {
    const isCurrentView = idleEvent.account_id === currentAccountId && idleEvent.folder_name === currentFolderName;
    console.log(`🔍 NewMessages event - isCurrentView: ${isCurrentView}`);

    if (isCurrentView) {
      const targetAccountId = currentAccountId;
      const targetFolderName = currentFolderName;
      const selectedConfig = accounts.find((acc) => acc.id === targetAccountId);

      if (selectedConfig) {
        try {
          console.log(`🔄 Starting sync for current view after NewMessages event...`);
          appState.isSyncing = true;
          const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
            config: selectedConfig,
            folder: targetFolderName,
          });

          console.log(`✅ Sync completed, received ${syncedEmails.length} emails`);

          // Check if still viewing same account/folder
          if (appState.selectedAccountId === targetAccountId && appState.selectedFolderName === targetFolderName) {
            // Preserve the selected email UID if it exists in the new list
            const currentlySelectedUid = appState.selectedEmailUid;
            const previousEmailCount = appState.emails.length;
            appState.emails = syncedEmails;
            appState.lastSyncTime = Math.floor(Date.now() / 1000);

            console.log(`📬 Email list updated: ${previousEmailCount} → ${syncedEmails.length} emails`);

            // Check if the selected email still exists
            if (currentlySelectedUid) {
              const stillExists = syncedEmails.find((e) => e.uid === currentlySelectedUid);
              if (!stillExists) {
                console.warn(`⚠️ Selected email UID ${currentlySelectedUid} no longer in list after sync!`);
              }
            }
          } else {
            console.log("⚠️ Account/folder changed during IDLE sync, discarding result");
          }
          appState.isSyncing = false;
        } catch (e) {
          console.error("❌ Failed to sync after IDLE event:", e);
          appState.isSyncing = false;
        }
      } else {
        console.error(`❌ No account config found for account ID ${targetAccountId}`);
      }
    } else {
      // Background sync for other folders - don't update UI
      console.log(`🔄 Background sync for non-current folder: ${idleEvent.folder_name} (account ${idleEvent.account_id})`);
      const affectedConfig = accounts.find((acc) => acc.id === idleEvent.account_id);
      if (affectedConfig) {
        try {
          const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
            config: affectedConfig,
            folder: idleEvent.folder_name,
          });
          console.log(`✅ Background sync completed, ${syncedEmails.length} emails synced to cache`);
        } catch (e) {
          console.error(`❌ Background sync failed for account ${idleEvent.account_id}, folder ${idleEvent.folder_name}:`, e);
        }
      } else {
        console.error(`❌ No account config found for background sync (account ID ${idleEvent.account_id})`);
      }
    }
  } else if (eventType === "FlagsChanged") {
    // Sync flags for specific UID only (efficient!)
    const currentAccountId = appState.selectedAccountId;
    const currentFolderName = appState.selectedFolderName;

    if (idleEvent.account_id === currentAccountId && idleEvent.folder_name === currentFolderName) {
      const targetAccountId = currentAccountId;
      const targetFolderName = currentFolderName;
      const selectedConfig = appState.accounts.find((acc) => acc.id === targetAccountId);
      const uid = (idleEvent.event_type as any).uid; // Extract UID from event

      if (selectedConfig && uid) {
        try {
          console.log(`🏴 Syncing flags for UID ${uid} after FlagsChanged event...`);
          await invoke("sync_specific_email_flags", {
            accountId: targetAccountId,
            folderName: targetFolderName,
            uid: uid,
            config: selectedConfig,
          });

          // Reload emails from cache to reflect updated flags
          const updatedEmails = await invoke<EmailHeader[]>("load_emails_from_cache", {
            accountId: targetAccountId,
            folder: targetFolderName,
          });

          // Check if still viewing same account/folder
          if (appState.selectedAccountId === targetAccountId && appState.selectedFolderName === targetFolderName) {
            appState.emails = updatedEmails;
            console.log(`✅ Flags synced successfully for UID ${uid}`);
          } else {
            console.log("Account/folder changed during flag sync, discarding result");
          }
        } catch (e) {
          console.error("❌ Failed to sync flags:", e);
        }
      }
    }
  } else if (eventType === "Expunge") {
    // Full sync for email deletions
    const currentAccountId = appState.selectedAccountId;
    const currentFolderName = appState.selectedFolderName;

    if (idleEvent.account_id === currentAccountId && idleEvent.folder_name === currentFolderName) {
      const targetAccountId = currentAccountId;
      const targetFolderName = currentFolderName;
      const selectedConfig = appState.accounts.find((acc) => acc.id === targetAccountId);

      if (selectedConfig) {
        try {
          const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
            config: selectedConfig,
            folder: targetFolderName,
          });

          // Check if still viewing same account/folder
          if (appState.selectedAccountId === targetAccountId && appState.selectedFolderName === targetFolderName) {
            appState.emails = syncedEmails;
          } else {
            console.log("Account/folder changed during IDLE sync, discarding result");
          }
        } catch (e) {
          console.error("❌ Failed to refresh after deletion:", e);
        }
      }
    }
  } else if (eventType === "ConnectionLost") {
    console.warn(`⚠️ IDLE connection lost for account ${idleEvent.account_id}`);
  }
}

/**
 * Play notification sound
 */
export function playNotificationSound() {
  try {
    // Create a simple beep sound using Web Audio API
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);

    oscillator.frequency.value = 440; // A4 note
    oscillator.type = "sine";

    // Fade out
    gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.3);

    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.3);

    console.log("🔔 Played notification sound");
  } catch (e) {
    console.error("❌ Failed to play notification sound:", e);
  }
}

/**
 * Get the auto-sync timer reference for cleanup
 */
export function getAutoSyncTimer() {
  return autoSyncTimerRef;
}

/**
 * Clear the auto-sync timer
 */
export function clearAutoSyncTimer() {
  if (autoSyncTimerRef) {
    clearInterval(autoSyncTimerRef);
    autoSyncTimerRef = null;
  }
}
