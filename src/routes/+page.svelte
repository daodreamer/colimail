<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { save, ask, message } from "@tauri-apps/plugin-dialog";

  // Components
  import AccountsSidebar from "./components/AccountsSidebar.svelte";
  import FoldersSidebar from "./components/FoldersSidebar.svelte";
  import EmailList from "./components/EmailList.svelte";
  import EmailBody from "./components/EmailBody.svelte";
  import ComposeDialog from "./components/ComposeDialog.svelte";
  import ToastNotification from "./components/ToastNotification.svelte";

  // Types and utilities
  import type { AccountConfig, EmailHeader, IdleEvent, Folder } from "./lib/types";
  import { state as appState } from "./lib/state.svelte";
  import { isTrashFolder } from "./lib/utils";

  // Auto-sync timer reference
  let autoSyncTimer: ReturnType<typeof setInterval> | null = null;

  // Custom notification state
  interface ToastNotificationData {
    title: string;
    body: string;
    from: string;
    subject: string;
  }
  let toastNotification = $state<ToastNotificationData | null>(null);

  // Lifecycle: Initialize app
  onMount(() => {
    (async () => {
      try {
        appState.accounts = await invoke<AccountConfig[]>("load_account_configs");
        appState.syncInterval = await invoke<number>("get_sync_interval");

        startAutoSyncTimer();

        // Start IDLE connections for all accounts
        for (const account of appState.accounts) {
          try {
            await invoke("start_idle", {
              accountId: account.id,
              folderName: "INBOX",
              config: account,
            });
          } catch (e) {
            console.error(`❌ Failed to start IDLE for account ${account.email}:`, e);
          }
        }

        // Listen for IDLE push notifications
        const unlisten = await listen<IdleEvent>("idle-event", handleIdleEvent);

        // Listen for custom notification event
        const unlistenNotification = await listen<ToastNotificationData>(
          "show-custom-notification",
          (event) => {
            console.log("📬 Received custom notification event:", event.payload);
            toastNotification = event.payload;
          }
        );

        // Listen for notification sound event
        const unlistenSound = await listen("play-notification-sound", () => {
          playNotificationSound();
        });

        // Update current time every minute
        const timeUpdateTimer = setInterval(() => {
          appState.currentTime = Math.floor(Date.now() / 1000);
        }, 60000);

        return () => {
          unlisten();
          if (autoSyncTimer) clearInterval(autoSyncTimer);
          clearInterval(timeUpdateTimer);
        };
      } catch (e) {
        appState.error = `Failed to load accounts: ${e}`;
      }
    })();
  });

  // Reload sync interval when returning from settings
  $effect(() => {
    const handleVisibilityChange = async () => {
      if (document.visibilityState === "visible") {
        try {
          const newInterval = await invoke<number>("get_sync_interval");
          if (newInterval !== appState.syncInterval) {
            appState.syncInterval = newInterval;
            startAutoSyncTimer();
          }
        } catch (e) {
          console.error("❌ Failed to reload sync interval:", e);
        }
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    window.addEventListener("focus", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      window.removeEventListener("focus", handleVisibilityChange);
    };
  });

  // Auto-sync timer management
  function startAutoSyncTimer() {
    if (autoSyncTimer) {
      clearInterval(autoSyncTimer);
      autoSyncTimer = null;
    }

    if (appState.syncInterval <= 0) {
      return;
    }

    autoSyncTimer = setInterval(async () => {
      if (!appState.selectedAccountId) return;

      const selectedConfig = appState.accounts.find((acc) => acc.id === appState.selectedAccountId);
      if (!selectedConfig) return;

      try {
        const needsSync = await invoke<boolean>("should_sync", {
          accountId: appState.selectedAccountId,
          folder: appState.selectedFolderName,
          syncInterval: appState.syncInterval,
        });

        if (needsSync && !appState.isSyncing) {
          appState.isSyncing = true;

          const syncedFolders = await invoke<Folder[]>("sync_folders", { config: selectedConfig });
          appState.folders = syncedFolders;

          const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
            config: selectedConfig,
            folder: appState.selectedFolderName,
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
  }

  // Event handlers
  async function handleAccountClick(accountId: number) {
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

    const selectedConfig = appState.accounts.find((acc) => acc.id === targetAccountId);
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
        syncInterval: appState.syncInterval,
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

  async function loadEmailsForFolder(folderName: string) {
    const targetAccountId = appState.selectedAccountId;
    const selectedConfig = appState.accounts.find((acc) => acc.id === targetAccountId);

    if (!selectedConfig || !targetAccountId) {
      appState.error = "No account selected.";
      return;
    }

    appState.isLoadingEmails = true;
    appState.resetEmailState();
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
        syncInterval: appState.syncInterval,
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

  async function syncSingleFolder() {
    const targetAccountId = appState.selectedAccountId;
    const targetFolderName = appState.selectedFolderName;
    const selectedConfig = appState.accounts.find((acc) => acc.id === targetAccountId);

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

  async function handleFolderClick(folderName: string) {
    const isSameFolder = appState.selectedFolderName === folderName;

    appState.selectedFolderName = folderName;

    if (isSameFolder) {
      // User clicked the current folder - they want to check for updates
      await syncSingleFolder();
    } else {
      // User clicked a different folder - load from cache
      await loadEmailsForFolder(folderName);
    }
  }

  async function handleEmailClick(uid: number) {
    appState.selectedEmailUid = uid;
    appState.isLoadingBody = true;
    appState.emailBody = null;
    appState.attachments = [];
    appState.error = null;

    const selectedConfig = appState.accounts.find((acc) => acc.id === appState.selectedAccountId);
    if (!selectedConfig) {
      appState.error = "Could not find selected account configuration.";
      appState.isLoadingBody = false;
      return;
    }

    try {
      appState.emailBody = await invoke<string>("fetch_email_body_cached", {
        config: selectedConfig,
        uid,
        folder: appState.selectedFolderName,
      });

      if (appState.selectedAccountId) {
        loadAttachmentsForEmail(appState.selectedAccountId, uid);
      }
    } catch (e) {
      appState.error = `Failed to fetch email body: ${e}`;
    } finally {
      appState.isLoadingBody = false;
    }
  }

  async function loadAttachmentsForEmail(accountId: number, uid: number) {
    appState.isLoadingAttachments = true;
    try {
      appState.attachments = await invoke("load_attachments_info", {
        accountId,
        folderName: appState.selectedFolderName,
        uid,
      });
    } catch (e) {
      console.error("❌ Failed to load attachments:", e);
    } finally {
      appState.isLoadingAttachments = false;
    }
  }

  async function downloadAttachment(attachmentId: number, filename: string) {
    try {
      const filePath = await save({
        defaultPath: filename,
        title: "Save Attachment",
      });

      if (filePath) {
        await invoke("save_attachment_to_file", {
          attachmentId,
          filePath,
        });
      }
    } catch (e) {
      console.error("❌ Failed to save attachment:", e);
      appState.error = `Failed to download attachment: ${e}`;
    }
  }

  async function handleManualRefresh() {
    if (appState.accounts.length === 0) {
      appState.error = "No accounts configured.";
      return;
    }

    appState.isSyncing = true;
    appState.error = null;

    try {
      // Sync all accounts
      for (const account of appState.accounts) {
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

  async function handleDeleteAccount(email: string, event: MouseEvent) {
    event.stopPropagation();

    const confirmed = await ask(`Are you sure you want to delete account ${email}?`, {
      title: "Delete Account",
      kind: "warning",
    });

    if (!confirmed) {
      return;
    }

    try {
      await invoke("delete_account", { email });
      appState.accounts = await invoke<AccountConfig[]>("load_account_configs");

      const deletedAccount = appState.accounts.find((acc) => acc.email === email);
      if (deletedAccount && deletedAccount.id === appState.selectedAccountId) {
        appState.selectedAccountId = null;
        appState.emails = [];
        appState.emailBody = null;
      }
    } catch (e) {
      appState.error = `Failed to delete account: ${e}`;
    }
  }

  async function handleComposeClick() {
    if (!appState.selectedAccountId) {
      appState.error = "Please select an account first.";
      return;
    }
    appState.showComposeDialog = true;
    appState.resetComposeState();
    await updateAttachmentSizeLimit();
  }

  async function handleReplyClick() {
    if (!appState.selectedAccountId || !appState.selectedEmailUid) {
      appState.error = "Please select an email first.";
      return;
    }

    const selectedEmail = appState.emails.find((email) => email.uid === appState.selectedEmailUid);
    if (!selectedEmail) {
      appState.error = "Could not find selected email.";
      return;
    }

    appState.showComposeDialog = true;
    appState.isReplyMode = true;
    appState.isForwardMode = false;
    appState.composeTo = selectedEmail.from;
    appState.composeSubject = selectedEmail.subject.toLowerCase().startsWith("re:")
      ? selectedEmail.subject
      : `Re: ${selectedEmail.subject}`;
    appState.composeBody = "";
    appState.composeAttachments = [];
    appState.error = null;
    await updateAttachmentSizeLimit();
  }

  async function handleForwardClick() {
    if (!appState.selectedAccountId || !appState.selectedEmailUid) {
      appState.error = "Please select an email first.";
      return;
    }

    const selectedEmail = appState.emails.find((email) => email.uid === appState.selectedEmailUid);
    if (!selectedEmail) {
      appState.error = "Could not find selected email.";
      return;
    }

    appState.showComposeDialog = true;
    appState.isReplyMode = false;
    appState.isForwardMode = true;
    appState.composeTo = "";
    appState.composeSubject = selectedEmail.subject.toLowerCase().startsWith("fwd:")
      ? selectedEmail.subject
      : `Fwd: ${selectedEmail.subject}`;
    appState.composeBody = "";
    appState.composeAttachments = [];
    appState.error = null;
    await updateAttachmentSizeLimit();
  }

  function handleCloseCompose() {
    appState.showComposeDialog = false;
    appState.resetComposeState();
  }

  function handleAttachmentSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files) return;

    const newFiles = Array.from(input.files);
    const allFiles = [...appState.composeAttachments, ...newFiles];

    const totalSize = allFiles.reduce((sum, file) => sum + file.size, 0);
    if (totalSize > appState.attachmentSizeLimit) {
      const limitMB = (appState.attachmentSizeLimit / (1024 * 1024)).toFixed(2);
      const totalMB = (totalSize / (1024 * 1024)).toFixed(2);
      appState.error = `Total attachment size (${totalMB} MB) exceeds the limit for your email provider (${limitMB} MB)`;
      input.value = "";
      return;
    }

    appState.composeAttachments = allFiles;
    input.value = "";
    appState.error = null;
  }

  function removeAttachment(index: number) {
    appState.composeAttachments = appState.composeAttachments.filter((_, i) => i !== index);
  }

  async function updateAttachmentSizeLimit() {
    if (!appState.selectedAccountId) return;

    const selectedConfig = appState.accounts.find((acc) => acc.id === appState.selectedAccountId);
    if (!selectedConfig) return;

    try {
      const limit = await invoke<number>("get_attachment_size_limit", {
        email: selectedConfig.email,
      });
      appState.attachmentSizeLimit = limit;
    } catch (e) {
      console.error("❌ Failed to get attachment size limit:", e);
    }
  }

  async function handleSendEmail() {
    if (!appState.selectedAccountId) {
      appState.error = "Please select an account first.";
      return;
    }

    if (!appState.composeTo || !appState.composeSubject) {
      appState.error = "Please fill in recipient and subject fields.";
      return;
    }

    const selectedConfig = appState.accounts.find((acc) => acc.id === appState.selectedAccountId);
    if (!selectedConfig) {
      appState.error = "Could not find selected account configuration.";
      return;
    }

    appState.isSending = true;
    appState.error = null;

    try {
      let attachmentsData: Array<{ filename: string; content_type: string; data: number[] }> | null =
        null;
      if (appState.composeAttachments.length > 0) {
        attachmentsData = [];
        for (const file of appState.composeAttachments) {
          const arrayBuffer = await file.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);
          const dataArray = Array.from(uint8Array);

          attachmentsData.push({
            filename: file.name,
            content_type: file.type || "application/octet-stream",
            data: dataArray,
          });
        }
      }

      let result: string;
      if (appState.isReplyMode) {
        result = await invoke<string>("reply_email", {
          config: selectedConfig,
          to: appState.composeTo,
          originalSubject: appState.composeSubject,
          body: appState.composeBody,
          cc: appState.composeCc || null,
          attachments: attachmentsData,
        });
      } else if (appState.isForwardMode) {
        const selectedEmail = appState.emails.find((email) => email.uid === appState.selectedEmailUid);
        if (!selectedEmail) {
          appState.error = "Could not find selected email.";
          appState.isSending = false;
          return;
        }
        result = await invoke<string>("forward_email", {
          config: selectedConfig,
          params: {
            to: appState.composeTo,
            originalSubject: selectedEmail.subject,
            originalFrom: selectedEmail.from,
            originalTo: selectedEmail.to,
            originalDate: selectedEmail.date,
            originalBody: appState.emailBody || "",
            additionalMessage: appState.composeBody,
            cc: appState.composeCc || null,
            attachments: attachmentsData,
          },
        });
      } else {
        if (!appState.composeBody) {
          appState.error = "Please fill in the message body.";
          appState.isSending = false;
          return;
        }
        result = await invoke<string>("send_email", {
          config: selectedConfig,
          to: appState.composeTo,
          subject: appState.composeSubject,
          body: appState.composeBody,
          cc: appState.composeCc || null,
          attachments: attachmentsData,
        });
      }
      handleCloseCompose();
      await message("Email sent successfully!", { title: "Success", kind: "info" });
    } catch (e) {
      appState.error = `Failed to send email: ${e}`;
    } finally {
      appState.isSending = false;
    }
  }

  async function handleDeleteEmail() {
    if (!appState.selectedAccountId || !appState.selectedEmailUid) {
      appState.error = "Please select an email first.";
      return;
    }

    const selectedEmail = appState.emails.find((email) => email.uid === appState.selectedEmailUid);
    if (!selectedEmail) {
      appState.error = "Could not find selected email.";
      return;
    }

    const selectedConfig = appState.accounts.find((acc) => acc.id === appState.selectedAccountId);
    if (!selectedConfig) {
      appState.error = "Could not find selected account configuration.";
      return;
    }

    const isInTrash = isTrashFolder(appState.selectedFolderName);

    const confirmTitle = isInTrash ? "Permanently Delete Email?" : "Move to Trash?";
    const confirmMessage = isInTrash
      ? `Are you sure you want to PERMANENTLY delete this email?\n\nThis action cannot be undone.\n\nSubject: ${selectedEmail.subject}`
      : `Move this email to trash?\n\nSubject: ${selectedEmail.subject}`;

    // IMPORTANT: Use Tauri's ask() dialog instead of native confirm()
    // This properly blocks execution until user responds
    const userConfirmed = await ask(confirmMessage, {
      title: confirmTitle,
      kind: "warning",
    });

    if (!userConfirmed) {
      return;
    }

    // Only after user confirms, proceed with backend operations
    appState.error = null;

    try {
      const deletedUid = appState.selectedEmailUid;

      if (isInTrash) {
        // Permanently delete from server
        await invoke("delete_email", {
          config: selectedConfig,
          uid: deletedUid,
          folder: appState.selectedFolderName,
        });

        // Immediately remove from UI (optimistic update)
        appState.emails = appState.emails.filter((email) => email.uid !== deletedUid);
        appState.resetEmailState();

        await message("Email permanently deleted!", { title: "Success", kind: "info" });
      } else {
        // Move to trash on server
        await invoke("move_email_to_trash", {
          config: selectedConfig,
          uid: deletedUid,
          folder: appState.selectedFolderName,
        });

        // Immediately remove from current folder UI (optimistic update)
        appState.emails = appState.emails.filter((email) => email.uid !== deletedUid);
        appState.resetEmailState();

        await message("Email moved to trash!", { title: "Success", kind: "info" });
      }

      // Note: The IDLE event handler will sync in the background if needed,
      // but we've already updated the UI for immediate feedback
    } catch (e) {
      appState.error = `Failed to delete email: ${e}`;
      // On error, reload to ensure UI is in sync with server
      await loadEmailsForFolder(appState.selectedFolderName);
    }
  }

  // Play notification sound
  function playNotificationSound() {
    try {
      // Create a simple beep sound using Web Audio API
      const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
      const oscillator = audioContext.createOscillator();
      const gainNode = audioContext.createGain();

      oscillator.connect(gainNode);
      gainNode.connect(audioContext.destination);

      oscillator.frequency.value = 440; // A4 note
      oscillator.type = 'sine';

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

  async function handleIdleEvent(event: { payload: IdleEvent }) {
    const idleEvent = event.payload;
    const eventType = idleEvent.event_type.type;

    if (eventType === "NewMessages") {
      if (
        idleEvent.account_id === appState.selectedAccountId &&
        idleEvent.folder_name === appState.selectedFolderName
      ) {
        const targetAccountId = appState.selectedAccountId;
        const targetFolderName = appState.selectedFolderName;
        const selectedConfig = appState.accounts.find((acc) => acc.id === targetAccountId);

        if (selectedConfig) {
          try {
            appState.isSyncing = true;
            const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
              config: selectedConfig,
              folder: targetFolderName,
            });

            // Check if still viewing same account/folder
            if (appState.selectedAccountId === targetAccountId && appState.selectedFolderName === targetFolderName) {
              appState.emails = syncedEmails;
              appState.lastSyncTime = Math.floor(Date.now() / 1000);
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
        const affectedConfig = appState.accounts.find((acc) => acc.id === idleEvent.account_id);
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
    } else if (eventType === "Expunge" || eventType === "FlagsChanged") {
      if (
        idleEvent.account_id === appState.selectedAccountId &&
        idleEvent.folder_name === appState.selectedFolderName
      ) {
        const targetAccountId = appState.selectedAccountId;
        const targetFolderName = appState.selectedFolderName;
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
            console.error("❌ Failed to refresh after change:", e);
          }
        }
      }
    } else if (eventType === "ConnectionLost") {
      console.warn(`⚠️ IDLE connection lost for account ${idleEvent.account_id}`);
    }
  }
</script>

<div class="main-layout">
  <AccountsSidebar
    accounts={appState.accounts}
    selectedAccountId={appState.selectedAccountId}
    isSyncing={appState.isSyncing}
    lastSyncTime={appState.lastSyncTime}
    currentTime={appState.currentTime}
    onAccountClick={handleAccountClick}
    onCompose={handleComposeClick}
    onRefresh={handleManualRefresh}
    onDeleteAccount={handleDeleteAccount}
  />

  <FoldersSidebar
    bind:folders={appState.folders}
    bind:selectedFolderName={appState.selectedFolderName}
    isLoading={appState.isLoadingFolders}
    selectedAccountId={appState.selectedAccountId}
    onFolderClick={handleFolderClick}
  />

  <EmailList
    emails={appState.emails}
    selectedEmailUid={appState.selectedEmailUid}
    isLoading={appState.isLoadingEmails}
    error={appState.error}
    selectedAccountId={appState.selectedAccountId}
    currentUserEmail={appState.accounts.find((acc) => acc.id === appState.selectedAccountId)?.email || ""}
    onEmailClick={handleEmailClick}
  />

  <EmailBody
    email={appState.selectedEmail}
    body={appState.emailBody}
    attachments={appState.attachments}
    isLoadingBody={appState.isLoadingBody}
    isLoadingAttachments={appState.isLoadingAttachments}
    error={appState.error}
    onReply={handleReplyClick}
    onForward={handleForwardClick}
    onDelete={handleDeleteEmail}
    onDownloadAttachment={downloadAttachment}
  />

  <ComposeDialog
    show={appState.showComposeDialog}
    mode={appState.isReplyMode ? "reply" : appState.isForwardMode ? "forward" : "compose"}
    bind:to={appState.composeTo}
    bind:cc={appState.composeCc}
    bind:subject={appState.composeSubject}
    bind:body={appState.composeBody}
    bind:attachments={appState.composeAttachments}
    attachmentSizeLimit={appState.attachmentSizeLimit}
    totalAttachmentSize={appState.totalAttachmentSize}
    isSending={appState.isSending}
    error={appState.error}
    onSend={handleSendEmail}
    onCancel={handleCloseCompose}
    onAttachmentAdd={handleAttachmentSelect}
    onAttachmentRemove={removeAttachment}
  />

  {#if toastNotification}
    <ToastNotification
      title={toastNotification.title}
      body={toastNotification.body}
      from={toastNotification.from}
      subject={toastNotification.subject}
      onClose={() => {
        toastNotification = null;
      }}
    />
  {/if}
</div>

<style>
  /* Global scrollbar hiding */
  :global(*) {
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  :global(*::-webkit-scrollbar) {
    display: none;
  }

  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    position: fixed;
  }

  :root {
    --border-color: #dcdcdc;
    --sidebar-bg: #e8e8e8;
    --app-bg: #f6f6f6;
    --text-color: #0f0f0f;
    --hover-bg: #dcdcdc;
    --selected-bg: #007bff;
    --selected-text: white;
    --link-bg: #007bff;
    --link-text: white;
    --link-hover-bg: #0056b3;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --border-color: #3a3a3a;
      --sidebar-bg: #252525;
      --app-bg: #2f2f2f;
      --text-color: #f6f6f6;
      --hover-bg: #3a3a3a;
      --selected-bg: #24c8db;
      --selected-text: #1a1a1a;
      --link-bg: #24c8db;
      --link-text: #1a1a1a;
      --link-hover-bg: #1c9aa8;
    }
  }

  .main-layout {
    display: grid;
    grid-template-columns: 240px 200px 320px 1fr;
    height: 100vh;
    width: 100vw;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    background-color: var(--app-bg);
    color: var(--text-color);
    overflow: hidden;
  }
</style>
