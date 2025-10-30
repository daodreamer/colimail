/**
 * Account and Folder Management Handlers
 * Handles account selection, folder operations, and email loading
 */

import { invoke } from "@tauri-apps/api/core";
import type { AccountConfig, EmailHeader, Folder } from "../lib/types";
import { state as appState } from "../lib/state.svelte";

/**
 * Handle account selection - load folders and emails
 */
export async function handleAccountClick(
  accountId: number,
  accounts: AccountConfig[],
  syncInterval: number,
  loadEmailsForFolder: (folderName: string) => Promise<void>
) {
  if (appState.selectedAccountId === accountId) {
    return;
  }

  // Store the accountId locally to avoid race conditions
  const targetAccountId = accountId;

  appState.selectedAccountId = targetAccountId;
  appState.selectedFolderName = "INBOX";
  appState.resetEmailState();
  appState.emails = [];
  appState.error = null;

  const selectedConfig = accounts.find((acc) => acc.id === targetAccountId);
  if (!selectedConfig) {
    appState.error = "Could not find selected account configuration.";
    return;
  }

  appState.isLoadingFolders = true;

  try {
    const cachedFolders = await invoke<Folder[]>("load_folders", { accountId: targetAccountId });

    // Check if user switched accounts during the async operation
    if (appState.selectedAccountId !== targetAccountId) {
      console.log("Account switched during folder load, aborting");
      return;
    }

    appState.folders = cachedFolders;
    appState.isLoadingFolders = false;

    await loadEmailsForFolder("INBOX");

    // Check again after loading emails
    if (appState.selectedAccountId !== targetAccountId) {
      console.log("Account switched during email load, aborting");
      return;
    }

    const needsFolderSync = await invoke<boolean>("should_sync", {
      accountId: targetAccountId,
      folder: "__folders__",
      syncInterval,
    });

    if (needsFolderSync && appState.selectedAccountId === targetAccountId) {
      const syncedFolders = await invoke<Folder[]>("sync_folders", { config: selectedConfig });

      // Final check before updating state
      if (appState.selectedAccountId === targetAccountId) {
        appState.folders = syncedFolders;
      }
    }
  } catch (e) {
    appState.error = `Failed to load folders: ${e}`;
    appState.isLoadingFolders = false;
  }
}

/**
 * Load emails for a specific folder
 */
export async function loadEmailsForFolder(
  folderName: string,
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  syncInterval: number
) {
  const targetAccountId = selectedAccountId;
  const selectedConfig = accounts.find((acc) => acc.id === targetAccountId);

  if (!selectedConfig || !targetAccountId) {
    appState.error = "No account selected.";
    return;
  }

  appState.isLoadingEmails = true;
  // DON'T reset email state if we're just refreshing the same folder
  // Only reset when switching folders
  if (appState.selectedFolderName !== folderName) {
    appState.resetEmailState();
  }
  appState.error = null;

  try {
    const cachedEmails = await invoke<EmailHeader[]>("load_emails_from_cache", {
      accountId: targetAccountId,
      folder: folderName,
    });

    // Check if account or folder changed during async operation
    if (appState.selectedAccountId !== targetAccountId || appState.selectedFolderName !== folderName) {
      console.log("Account/folder changed during cache load, aborting");
      return;
    }

    appState.emails = cachedEmails;
    appState.isLoadingEmails = false;

    appState.lastSyncTime = await invoke<number>("get_last_sync_time", {
      accountId: targetAccountId,
      folder: folderName,
    });

    const needsSync = await invoke<boolean>("should_sync", {
      accountId: targetAccountId,
      folder: folderName,
      syncInterval,
    });

    if (needsSync && appState.selectedAccountId === targetAccountId && appState.selectedFolderName === folderName) {
      appState.isSyncing = true;
      const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
        config: selectedConfig,
        folder: folderName,
      });

      // Final check before updating state
      if (appState.selectedAccountId === targetAccountId && appState.selectedFolderName === folderName) {
        appState.emails = syncedEmails;
        appState.lastSyncTime = Math.floor(Date.now() / 1000);
      }
      appState.isSyncing = false;
    }
  } catch (e) {
    appState.error = `Failed to load emails: ${e}`;
    appState.isLoadingEmails = false;
    appState.isSyncing = false;
  }
}

/**
 * Sync a single folder without affecting global sync timer
 */
async function syncSingleFolder(
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedFolderName: string
) {
  const targetAccountId = selectedAccountId;
  const targetFolderName = selectedFolderName;
  const selectedConfig = accounts.find((acc) => acc.id === targetAccountId);

  if (!selectedConfig || !targetAccountId) {
    appState.error = "No account selected.";
    return;
  }

  appState.isSyncing = true;
  appState.error = null;

  try {
    // Sync only this folder without affecting global sync timer
    const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
      config: selectedConfig,
      folder: targetFolderName,
    });

    // Check if account/folder changed during sync
    if (appState.selectedAccountId === targetAccountId && appState.selectedFolderName === targetFolderName) {
      appState.emails = syncedEmails;
      // Note: We intentionally do NOT update appState.lastSyncTime here
      // because this is a user-triggered folder refresh, not a global sync
    } else {
      console.log("Account/folder changed during sync, discarding result");
    }
  } catch (e) {
    appState.error = `Failed to sync folder: ${e}`;
  } finally {
    appState.isSyncing = false;
  }
}

/**
 * Handle folder click - load or refresh folder
 */
export async function handleFolderClick(
  folderName: string,
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedFolderName: string,
  syncInterval: number
) {
  const isSameFolder = selectedFolderName === folderName;

  appState.selectedFolderName = folderName;
  appState.showDraftsFolder = false; // Hide drafts view when switching to email folder

  if (isSameFolder) {
    // User clicked the current folder - they want to check for updates
    await syncSingleFolder(accounts, selectedAccountId, folderName);
  } else {
    // User clicked a different folder - load from cache
    await loadEmailsForFolder(folderName, accounts, selectedAccountId, syncInterval);
  }
}

/**
 * Handle folder created - refresh folder list
 */
export async function handleFolderCreated(selectedAccountId: number | null) {
  if (selectedAccountId) {
    try {
      appState.isLoadingFolders = true;
      appState.folders = await invoke<Folder[]>("load_folders", {
        accountId: selectedAccountId,
      });
    } catch (error) {
      console.error("Failed to reload folders:", error);
    } finally {
      appState.isLoadingFolders = false;
    }
  }
}

/**
 * Handle folder deleted - refresh folder list and switch to INBOX if needed
 */
export async function handleFolderDeleted(
  selectedAccountId: number | null,
  selectedFolderName: string,
  accounts: AccountConfig[],
  syncInterval: number
) {
  if (selectedAccountId) {
    try {
      appState.isLoadingFolders = true;
      appState.folders = await invoke<Folder[]>("load_folders", {
        accountId: selectedAccountId,
      });

      // If deleted folder was selected, switch to INBOX
      if (!appState.folders.find((f) => f.name === selectedFolderName)) {
        appState.selectedFolderName = "INBOX";
        await loadEmailsForFolder("INBOX", accounts, selectedAccountId, syncInterval);
      }
    } catch (error) {
      console.error("Failed to reload folders:", error);
    } finally {
      appState.isLoadingFolders = false;
    }
  }
}

/**
 * Handle account added callback
 */
export async function handleAccountAdded(
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  syncInterval: number,
  loadEmailsForFolder: (folderName: string) => Promise<void>,
  handleAccountClick: (accountId: number) => Promise<void>
) {
  try {
    // Reload accounts
    appState.accounts = await invoke<AccountConfig[]>("load_account_configs");

    // Auto-select the newly added account if it's the first one
    if (appState.accounts.length === 1 && !selectedAccountId) {
      await handleAccountClick(appState.accounts[0].id);
    }
  } catch (error) {
    console.error("Failed to reload accounts:", error);
  }
}

/**
 * Handle account deleted callback
 */
export async function handleAccountDeleted(
  email: string,
  selectedAccountId: number | null,
  syncInterval: number,
  loadEmailsForFolder: (folderName: string) => Promise<void>,
  handleAccountClick: (accountId: number) => Promise<void>
) {
  try {
    // Reload accounts
    appState.accounts = await invoke<AccountConfig[]>("load_account_configs");

    // If the deleted account was selected, clear selection or select another
    const deletedAccount = appState.accounts.find((acc) => acc.email === email);
    if (!deletedAccount && selectedAccountId) {
      // Check if deleted account was the selected one
      const stillExists = appState.accounts.find((acc) => acc.id === selectedAccountId);
      if (!stillExists) {
        appState.selectedAccountId = null;
        appState.selectedFolderName = "INBOX";
        appState.folders = [];
        appState.emails = [];
        appState.selectedEmailUid = null;
        appState.emailBody = null;

        // Select first available account if any
        if (appState.accounts.length > 0) {
          await handleAccountClick(appState.accounts[0].id);
        }
      }
    }
  } catch (error) {
    console.error("Failed to reload accounts:", error);
  }
}

/**
 * Handle account updated callback
 */
export async function handleAccountUpdated() {
  try {
    // Reload accounts
    appState.accounts = await invoke<AccountConfig[]>("load_account_configs");
  } catch (error) {
    console.error("Failed to reload accounts:", error);
  }
}
