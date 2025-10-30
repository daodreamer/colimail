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
export async function handleManualRefresh(accounts: AccountConfig[], selectedAccountId: number | null, selectedFolderName: string) {
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
        if (account.id === selectedAccountId) {
          appState.folders = syncedFolders;
        }

        // Sync all folders for this account
        for (const folder of syncedFolders) {
          const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
            config: account,
            folder: folder.name,
          });

          // Update emails if this is the currently selected account and folder
          if (account.id === selectedAccountId && folder.name === selectedFolderName) {
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
  event: { payload: IdleEvent },
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedFolderName: string
) {
  const idleEvent = event.payload;
  const eventType = idleEvent.event_type.type;

  if (eventType === "NewMessages") {
    if (idleEvent.account_id === selectedAccountId && idleEvent.folder_name === selectedFolderName) {
      const targetAccountId = selectedAccountId;
      const targetFolderName = selectedFolderName;
      const selectedConfig = accounts.find((acc) => acc.id === targetAccountId);

      if (selectedConfig) {
        try {
          appState.isSyncing = true;
          const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
            config: selectedConfig,
            folder: targetFolderName,
          });

          // Check if still viewing same account/folder
          if (appState.selectedAccountId === targetAccountId && appState.selectedFolderName === targetFolderName) {
            // Preserve the selected email UID if it exists in the new list
            const currentlySelectedUid = appState.selectedEmailUid;
            appState.emails = syncedEmails;
            appState.lastSyncTime = Math.floor(Date.now() / 1000);

            // Check if the selected email still exists
            if (currentlySelectedUid) {
              const stillExists = syncedEmails.find((e) => e.uid === currentlySelectedUid);
              if (!stillExists) {
                console.warn(`⚠️ Selected email UID ${currentlySelectedUid} no longer in list after sync!`);
              }
            }
          } else {
            console.log("Account/folder changed during IDLE sync, discarding result");
          }
          appState.isSyncing = false;
        } catch (e) {
          console.error("❌ Failed to sync after IDLE event:", e);
          appState.isSyncing = false;
        }
      }
    } else {
      // Background sync for other folders - don't update UI
      const affectedConfig = accounts.find((acc) => acc.id === idleEvent.account_id);
      if (affectedConfig) {
        try {
          await invoke<EmailHeader[]>("sync_emails", {
            config: affectedConfig,
            folder: idleEvent.folder_name,
          });
        } catch (e) {
          console.error(`❌ Background sync failed for account ${idleEvent.account_id}:`, e);
        }
      }
    }
  } else if (eventType === "FlagsChanged") {
    // Sync flags for specific UID only (efficient!)
    if (idleEvent.account_id === selectedAccountId && idleEvent.folder_name === selectedFolderName) {
      const targetAccountId = selectedAccountId;
      const targetFolderName = selectedFolderName;
      const selectedConfig = accounts.find((acc) => acc.id === targetAccountId);
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
    if (idleEvent.account_id === selectedAccountId && idleEvent.folder_name === selectedFolderName) {
      const targetAccountId = selectedAccountId;
      const targetFolderName = selectedFolderName;
      const selectedConfig = accounts.find((acc) => acc.id === targetAccountId);

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
