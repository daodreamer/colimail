<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { save, ask, message } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";
  import { Toaster } from "$lib/components/ui/sonner";
  import * as Sidebar from "$lib/components/ui/sidebar";

  // Components
  import AccountFolderSidebar from "./components/AccountFolderSidebar.svelte";
  import EmailListSidebar from "./components/EmailListSidebar.svelte";
  import EmailBody from "./components/EmailBody.svelte";
  import ComposeDialog from "./components/ComposeDialog.svelte";

  // Types and utilities
  import type { AccountConfig, EmailHeader, IdleEvent, Folder } from "./lib/types";
  import { state as appState } from "./lib/state.svelte";
  import { isTrashFolder } from "./lib/utils";

  // Auto-sync timer reference
  let autoSyncTimer: ReturnType<typeof setInterval> | null = null;

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
        const unlistenNotification = await listen<{ title: string; body: string; from: string; subject: string }>(
          "show-custom-notification",
          (event) => {
            console.log("📬 Received custom notification event:", event.payload);
            toast.success(event.payload.title, {
              description: `From: ${event.payload.from}\nSubject: ${event.payload.subject}`,
            });
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

      console.log(`📧 Loaded email body for UID ${uid}, length: ${appState.emailBody?.length || 0} bytes`);
      console.log(`📧 Body preview (first 200 chars):`, appState.emailBody?.substring(0, 200));

      if (appState.selectedAccountId) {
        loadAttachmentsForEmail(appState.selectedAccountId, uid);
      }

      // Auto-mark as read when opening email
      const selectedEmail = appState.emails.find((email) => email.uid === uid);
      if (selectedEmail && !selectedEmail.seen) {
        try {
          await invoke("mark_email_as_read", {
            config: selectedConfig,
            uid,
            folder: appState.selectedFolderName,
          });

          // Update local state
          selectedEmail.seen = true;
          appState.emails = [...appState.emails];
        } catch (e) {
          console.error("Failed to mark email as read:", e);
        }
      }
    } catch (e) {
      console.error(`❌ Failed to fetch email body:`, e);
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

  async function handleToggleReadStatus() {
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

    try {
      if (selectedEmail.seen) {
        // Mark as unread
        await invoke("mark_email_as_unread", {
          config: selectedConfig,
          uid: appState.selectedEmailUid,
          folder: appState.selectedFolderName,
        });
        selectedEmail.seen = false;
      } else {
        // Mark as read
        await invoke("mark_email_as_read", {
          config: selectedConfig,
          uid: appState.selectedEmailUid,
          folder: appState.selectedFolderName,
        });
        selectedEmail.seen = true;
      }

      // Update local state to trigger re-render
      appState.emails = [...appState.emails];
    } catch (e) {
      appState.error = `Failed to toggle read status: ${e}`;
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

    // Only ask for confirmation when permanently deleting from trash folder
    if (isInTrash) {
      const confirmMessage = `Are you sure you want to PERMANENTLY delete this email?\n\nThis action cannot be undone.\n\nSubject: ${selectedEmail.subject}`;

      // IMPORTANT: Use Tauri's ask() dialog instead of native confirm()
      // This properly blocks execution until user responds
      const userConfirmed = await ask(confirmMessage, {
        title: "Permanently Delete Email?",
        kind: "warning",
      });

      if (!userConfirmed) {
        return;
      }
    }

    // Proceed with backend operations
    appState.error = null;

    try {
      const deletedUid = appState.selectedEmailUid;

      // Immediately remove from UI (optimistic update) for instant feedback
      appState.emails = appState.emails.filter((email) => email.uid !== deletedUid);
      appState.resetEmailState();

      if (isInTrash) {
        // Permanently delete from server
        await invoke("delete_email", {
          config: selectedConfig,
          uid: deletedUid,
          folder: appState.selectedFolderName,
        });

        // No success message needed - the UI update provides instant feedback
      } else {
        // Move to trash on server (no confirmation needed)
        await invoke("move_email_to_trash", {
          config: selectedConfig,
          uid: deletedUid,
          folder: appState.selectedFolderName,
        });

        // No success message needed - instant UI feedback is better for smooth UX
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
              // Preserve the selected email UID if it exists in the new list
              const currentlySelectedUid = appState.selectedEmailUid;
              appState.emails = syncedEmails;
              appState.lastSyncTime = Math.floor(Date.now() / 1000);

              // Check if the selected email still exists
              if (currentlySelectedUid) {
                const stillExists = syncedEmails.find(e => e.uid === currentlySelectedUid);
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

<Sidebar.Provider style="--sidebar-width: 350px;">
  <Sidebar.Root
    collapsible="icon"
    class="overflow-hidden [&>[data-sidebar=sidebar]]:flex-row"
  >
    <AccountFolderSidebar
      accounts={appState.accounts}
      selectedAccountId={appState.selectedAccountId}
      folders={appState.folders}
      selectedFolderName={appState.selectedFolderName}
      isLoadingFolders={appState.isLoadingFolders}
      isSyncing={appState.isSyncing}
      onAccountSelect={handleAccountClick}
      onFolderClick={handleFolderClick}
      onAddAccount={() => window.location.href = '/account'}
      onSettings={() => window.location.href = '/settings'}
      onSyncMail={handleManualRefresh}
    />

    <EmailListSidebar
      emails={appState.emails}
      selectedEmailUid={appState.selectedEmailUid}
      isLoading={appState.isLoadingEmails}
      error={appState.error}
      selectedAccountId={appState.selectedAccountId}
      selectedFolderName={appState.selectedFolderName}
      currentUserEmail={appState.accounts.find((acc) => acc.id === appState.selectedAccountId)?.email || ""}
      onEmailClick={handleEmailClick}
      onComposeClick={handleComposeClick}
    />
  </Sidebar.Root>

  <Sidebar.Inset class="flex flex-col">
    <header class="sticky top-0 flex shrink-0 items-center gap-2 border-b bg-background p-4 z-10">
      <Sidebar.Trigger class="-ml-1" />
    </header>
    <div class="flex-1 overflow-hidden">
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
        onToggleRead={handleToggleReadStatus}
      />
    </div>
  </Sidebar.Inset>

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

  <Toaster />
</Sidebar.Provider>

