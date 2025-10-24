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

  // Types and utilities
  import type { AccountConfig, EmailHeader, IdleEvent, Folder } from "./lib/types";
  import { state } from "./lib/state.svelte";
  import { isTrashFolder } from "./lib/utils";

  // Auto-sync timer reference
  let autoSyncTimer: ReturnType<typeof setInterval> | null = null;

  // Lifecycle: Initialize app
  onMount(() => {
    (async () => {
      try {
        state.accounts = await invoke<AccountConfig[]>("load_account_configs");
        state.syncInterval = await invoke<number>("get_sync_interval");

        startAutoSyncTimer();

        // Start IDLE connections for all accounts
        console.log("🔔 Starting IDLE connections for all accounts...");
        for (const account of state.accounts) {
          try {
            await invoke("start_idle", {
              accountId: account.id,
              folderName: "INBOX",
              config: account,
            });
            console.log(`✅ IDLE enabled for account ${account.id} (${account.email})`);
          } catch (e) {
            console.warn(`Failed to start IDLE for account ${account.id}:`, e);
          }
        }

        // Listen for IDLE push notifications
        const unlisten = await listen<IdleEvent>("idle-event", handleIdleEvent);

        // Update current time every minute
        const timeUpdateTimer = setInterval(() => {
          state.currentTime = Math.floor(Date.now() / 1000);
        }, 60000);

        return () => {
          unlisten();
          if (autoSyncTimer) clearInterval(autoSyncTimer);
          clearInterval(timeUpdateTimer);
        };
      } catch (e) {
        state.error = `Failed to load accounts: ${e}`;
      }
    })();
  });

  // Reload sync interval when returning from settings
  $effect(() => {
    const handleVisibilityChange = async () => {
      if (document.visibilityState === "visible") {
        try {
          const newInterval = await invoke<number>("get_sync_interval");
          if (newInterval !== state.syncInterval) {
            console.log(`Sync interval changed from ${state.syncInterval} to ${newInterval}`);
            state.syncInterval = newInterval;
            startAutoSyncTimer();
          }
        } catch (e) {
          console.error("Failed to reload sync interval:", e);
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

    if (state.syncInterval <= 0) {
      console.log("Auto-sync disabled (interval:", state.syncInterval, ")");
      return;
    }

    console.log(`Starting auto-sync timer with interval: ${state.syncInterval} seconds`);

    autoSyncTimer = setInterval(async () => {
      if (!state.selectedAccountId) return;

      const selectedConfig = state.accounts.find((acc) => acc.id === state.selectedAccountId);
      if (!selectedConfig) return;

      try {
        const needsSync = await invoke<boolean>("should_sync", {
          accountId: state.selectedAccountId,
          folder: state.selectedFolderName,
          syncInterval: state.syncInterval,
        });

        if (needsSync && !state.isSyncing) {
          console.log(`Auto-sync triggered for ${state.selectedFolderName}`);
          state.isSyncing = true;

          const syncedFolders = await invoke<Folder[]>("sync_folders", { config: selectedConfig });
          state.folders = syncedFolders;

          const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
            config: selectedConfig,
            folder: state.selectedFolderName,
          });

          state.emails = syncedEmails;
          state.lastSyncTime = Math.floor(Date.now() / 1000);
          state.isSyncing = false;

          console.log(`Auto-sync completed for ${state.selectedFolderName}`);
        }
      } catch (e) {
        console.error("Auto-sync failed:", e);
        state.isSyncing = false;
      }
    }, 60000);
  }

  // Event handlers
  async function handleAccountClick(accountId: number) {
    if (state.selectedAccountId === accountId) {
      console.log("Same account already selected, ignoring click");
      return;
    }

    state.selectedAccountId = accountId;
    state.selectedFolderName = "INBOX";
    state.resetEmailState();
    state.emails = [];
    state.error = null;

    const selectedConfig = state.accounts.find((acc) => acc.id === accountId);
    if (!selectedConfig) {
      state.error = "Could not find selected account configuration.";
      return;
    }

    state.isLoadingFolders = true;

    try {
      const cachedFolders = await invoke<Folder[]>("load_folders", { accountId });
      state.folders = cachedFolders;
      state.isLoadingFolders = false;

      console.log(`Loaded ${cachedFolders.length} folders from cache`);

      await loadEmailsForFolder("INBOX");

      const needsFolderSync = await invoke<boolean>("should_sync", {
        accountId: accountId,
        folder: "__folders__",
        syncInterval: state.syncInterval,
      });

      if (needsFolderSync) {
        console.log("Syncing folders in background...");
        const syncedFolders = await invoke<Folder[]>("sync_folders", { config: selectedConfig });
        state.folders = syncedFolders;
        console.log(`Synced ${syncedFolders.length} folders from server`);
      } else {
        console.log("Using cached folders, sync not needed yet");
      }
    } catch (e) {
      state.error = `Failed to load folders: ${e}`;
      state.isLoadingFolders = false;
    }
  }

  async function loadEmailsForFolder(folderName: string) {
    const selectedConfig = state.accounts.find((acc) => acc.id === state.selectedAccountId);
    if (!selectedConfig || !state.selectedAccountId) {
      state.error = "No account selected.";
      return;
    }

    state.isLoadingEmails = true;
    state.resetEmailState();
    state.error = null;

    try {
      const cachedEmails = await invoke<EmailHeader[]>("load_emails_from_cache", {
        accountId: state.selectedAccountId,
        folder: folderName,
      });

      state.emails = cachedEmails;
      state.isLoadingEmails = false;

      console.log(`Loaded ${cachedEmails.length} emails from cache`);

      state.lastSyncTime = await invoke<number>("get_last_sync_time", {
        accountId: state.selectedAccountId,
        folder: folderName,
      });

      const needsSync = await invoke<boolean>("should_sync", {
        accountId: state.selectedAccountId,
        folder: folderName,
        syncInterval: state.syncInterval,
      });

      if (needsSync) {
        console.log("Sync needed, syncing emails in background...");
        state.isSyncing = true;
        const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
          config: selectedConfig,
          folder: folderName,
        });

        state.emails = syncedEmails;
        state.lastSyncTime = Math.floor(Date.now() / 1000);
        state.isSyncing = false;
        console.log(`Synced ${syncedEmails.length} emails from server`);
      } else {
        console.log("Using cache, sync not needed yet");
      }
    } catch (e) {
      state.error = `Failed to load emails: ${e}`;
      state.isLoadingEmails = false;
      state.isSyncing = false;
    }
  }

  async function handleFolderClick(folderName: string) {
    state.selectedFolderName = folderName;
    await loadEmailsForFolder(folderName);
  }

  async function handleEmailClick(uid: number) {
    state.selectedEmailUid = uid;
    state.isLoadingBody = true;
    state.emailBody = null;
    state.attachments = [];
    state.error = null;

    const selectedConfig = state.accounts.find((acc) => acc.id === state.selectedAccountId);
    if (!selectedConfig) {
      state.error = "Could not find selected account configuration.";
      state.isLoadingBody = false;
      return;
    }

    try {
      state.emailBody = await invoke<string>("fetch_email_body_cached", {
        config: selectedConfig,
        uid,
        folder: state.selectedFolderName,
      });

      if (state.selectedAccountId) {
        loadAttachmentsForEmail(state.selectedAccountId, uid);
      }
    } catch (e) {
      state.error = `Failed to fetch email body: ${e}`;
    } finally {
      state.isLoadingBody = false;
    }
  }

  async function loadAttachmentsForEmail(accountId: number, uid: number) {
    state.isLoadingAttachments = true;
    try {
      state.attachments = await invoke("load_attachments_info", {
        accountId,
        folderName: state.selectedFolderName,
        uid,
      });
    } catch (e) {
      console.error("Failed to load attachments:", e);
    } finally {
      state.isLoadingAttachments = false;
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
        console.log("Attachment saved successfully:", filePath);
      }
    } catch (e) {
      console.error("Failed to save attachment:", e);
      state.error = `Failed to download attachment: ${e}`;
    }
  }

  async function handleManualRefresh() {
    if (!state.selectedAccountId) {
      state.error = "Please select an account first.";
      return;
    }

    const selectedConfig = state.accounts.find((acc) => acc.id === state.selectedAccountId);
    if (!selectedConfig) {
      state.error = "Could not find selected account configuration.";
      return;
    }

    state.isSyncing = true;
    state.error = null;

    try {
      console.log("Manual refresh: syncing folders...");
      state.folders = await invoke<Folder[]>("sync_folders", { config: selectedConfig });

      console.log("Manual refresh: syncing emails...");
      const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
        config: selectedConfig,
        folder: state.selectedFolderName,
      });

      state.emails = syncedEmails;
      state.lastSyncTime = Math.floor(Date.now() / 1000);

      console.log("Manual refresh completed");
    } catch (e) {
      state.error = `Failed to refresh: ${e}`;
    } finally {
      state.isSyncing = false;
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
      state.accounts = await invoke<AccountConfig[]>("load_account_configs");

      const deletedAccount = state.accounts.find((acc) => acc.email === email);
      if (deletedAccount && deletedAccount.id === state.selectedAccountId) {
        state.selectedAccountId = null;
        state.emails = [];
        state.emailBody = null;
      }
    } catch (e) {
      state.error = `Failed to delete account: ${e}`;
    }
  }

  async function handleComposeClick() {
    if (!state.selectedAccountId) {
      state.error = "Please select an account first.";
      return;
    }
    state.showComposeDialog = true;
    state.resetComposeState();
    await updateAttachmentSizeLimit();
  }

  async function handleReplyClick() {
    if (!state.selectedAccountId || !state.selectedEmailUid) {
      state.error = "Please select an email first.";
      return;
    }

    const selectedEmail = state.emails.find((email) => email.uid === state.selectedEmailUid);
    if (!selectedEmail) {
      state.error = "Could not find selected email.";
      return;
    }

    state.showComposeDialog = true;
    state.isReplyMode = true;
    state.isForwardMode = false;
    state.composeTo = selectedEmail.from;
    state.composeSubject = selectedEmail.subject.toLowerCase().startsWith("re:")
      ? selectedEmail.subject
      : `Re: ${selectedEmail.subject}`;
    state.composeBody = "";
    state.composeAttachments = [];
    state.error = null;
    await updateAttachmentSizeLimit();
  }

  async function handleForwardClick() {
    if (!state.selectedAccountId || !state.selectedEmailUid) {
      state.error = "Please select an email first.";
      return;
    }

    const selectedEmail = state.emails.find((email) => email.uid === state.selectedEmailUid);
    if (!selectedEmail) {
      state.error = "Could not find selected email.";
      return;
    }

    state.showComposeDialog = true;
    state.isReplyMode = false;
    state.isForwardMode = true;
    state.composeTo = "";
    state.composeSubject = selectedEmail.subject.toLowerCase().startsWith("fwd:")
      ? selectedEmail.subject
      : `Fwd: ${selectedEmail.subject}`;
    state.composeBody = "";
    state.composeAttachments = [];
    state.error = null;
    await updateAttachmentSizeLimit();
  }

  function handleCloseCompose() {
    state.showComposeDialog = false;
    state.resetComposeState();
  }

  function handleAttachmentSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files) return;

    const newFiles = Array.from(input.files);
    const allFiles = [...state.composeAttachments, ...newFiles];

    const totalSize = allFiles.reduce((sum, file) => sum + file.size, 0);
    if (totalSize > state.attachmentSizeLimit) {
      const limitMB = (state.attachmentSizeLimit / (1024 * 1024)).toFixed(2);
      const totalMB = (totalSize / (1024 * 1024)).toFixed(2);
      state.error = `Total attachment size (${totalMB} MB) exceeds the limit for your email provider (${limitMB} MB)`;
      input.value = "";
      return;
    }

    state.composeAttachments = allFiles;
    input.value = "";
    state.error = null;
  }

  function removeAttachment(index: number) {
    state.composeAttachments = state.composeAttachments.filter((_, i) => i !== index);
  }

  async function updateAttachmentSizeLimit() {
    if (!state.selectedAccountId) return;

    const selectedConfig = state.accounts.find((acc) => acc.id === state.selectedAccountId);
    if (!selectedConfig) return;

    try {
      const limit = await invoke<number>("get_attachment_size_limit", {
        email: selectedConfig.email,
      });
      state.attachmentSizeLimit = limit;
    } catch (e) {
      console.error("Failed to get attachment size limit:", e);
    }
  }

  async function handleSendEmail() {
    if (!state.selectedAccountId) {
      state.error = "Please select an account first.";
      return;
    }

    if (!state.composeTo || !state.composeSubject) {
      state.error = "Please fill in recipient and subject fields.";
      return;
    }

    const selectedConfig = state.accounts.find((acc) => acc.id === state.selectedAccountId);
    if (!selectedConfig) {
      state.error = "Could not find selected account configuration.";
      return;
    }

    state.isSending = true;
    state.error = null;

    try {
      let attachmentsData: Array<{ filename: string; content_type: string; data: number[] }> | null =
        null;
      if (state.composeAttachments.length > 0) {
        attachmentsData = [];
        for (const file of state.composeAttachments) {
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
      if (state.isReplyMode) {
        result = await invoke<string>("reply_email", {
          config: selectedConfig,
          to: state.composeTo,
          originalSubject: state.composeSubject,
          body: state.composeBody,
          attachments: attachmentsData,
        });
      } else if (state.isForwardMode) {
        const selectedEmail = state.emails.find((email) => email.uid === state.selectedEmailUid);
        if (!selectedEmail) {
          state.error = "Could not find selected email.";
          state.isSending = false;
          return;
        }
        result = await invoke<string>("forward_email", {
          config: selectedConfig,
          to: state.composeTo,
          originalSubject: selectedEmail.subject,
          originalFrom: selectedEmail.from,
          originalTo: selectedEmail.to,
          originalDate: selectedEmail.date,
          originalBody: state.emailBody || "",
          additionalMessage: state.composeBody,
          attachments: attachmentsData,
        });
      } else {
        if (!state.composeBody) {
          state.error = "Please fill in the message body.";
          state.isSending = false;
          return;
        }
        result = await invoke<string>("send_email", {
          config: selectedConfig,
          to: state.composeTo,
          subject: state.composeSubject,
          body: state.composeBody,
          attachments: attachmentsData,
        });
      }
      console.log("Send result:", result);
      handleCloseCompose();
      await message("Email sent successfully!", { title: "Success", kind: "info" });
    } catch (e) {
      state.error = `Failed to send email: ${e}`;
    } finally {
      state.isSending = false;
    }
  }

  async function handleDeleteEmail() {
    if (!state.selectedAccountId || !state.selectedEmailUid) {
      state.error = "Please select an email first.";
      return;
    }

    const selectedEmail = state.emails.find((email) => email.uid === state.selectedEmailUid);
    if (!selectedEmail) {
      state.error = "Could not find selected email.";
      return;
    }

    const selectedConfig = state.accounts.find((acc) => acc.id === state.selectedAccountId);
    if (!selectedConfig) {
      state.error = "Could not find selected account configuration.";
      return;
    }

    const isInTrash = isTrashFolder(state.selectedFolderName);

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
    state.error = null;

    try {
      if (isInTrash) {
        await invoke("delete_email", {
          config: selectedConfig,
          uid: state.selectedEmailUid,
          folder: state.selectedFolderName,
        });
        await message("Email permanently deleted!", { title: "Success", kind: "info" });
      } else {
        await invoke("move_email_to_trash", {
          config: selectedConfig,
          uid: state.selectedEmailUid,
          folder: state.selectedFolderName,
        });
        await message("Email moved to trash!", { title: "Success", kind: "info" });
      }

      state.resetEmailState();
      await loadEmailsForFolder(state.selectedFolderName);
    } catch (e) {
      state.error = `Failed to delete email: ${e}`;
    }
  }

  async function handleIdleEvent(event: { payload: IdleEvent }) {
    const idleEvent = event.payload;
    console.log("📬 Received IDLE event:", idleEvent);

    const eventType = idleEvent.event_type.type;
    console.log(`📬 Event: ${eventType} for account ${idleEvent.account_id} folder ${idleEvent.folder_name}`);

    if (eventType === "NewMessages") {
      console.log(`✨ ${idleEvent.event_type.count} new message(s) detected`);

      if (
        idleEvent.account_id === state.selectedAccountId &&
        idleEvent.folder_name === state.selectedFolderName
      ) {
        console.log("🔄 Syncing currently displayed folder...");

        const selectedConfig = state.accounts.find((acc) => acc.id === state.selectedAccountId);
        if (selectedConfig) {
          try {
            state.isSyncing = true;
            const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
              config: selectedConfig,
              folder: state.selectedFolderName,
            });
            state.emails = syncedEmails;
            state.lastSyncTime = Math.floor(Date.now() / 1000);
            state.isSyncing = false;

            console.log("✅ Auto-sync completed via IDLE push");
          } catch (e) {
            console.error("Failed to sync after IDLE event:", e);
            state.isSyncing = false;
          }
        }
      } else {
        console.log("📥 Background sync for non-displayed folder");
        const affectedConfig = state.accounts.find((acc) => acc.id === idleEvent.account_id);
        if (affectedConfig) {
          try {
            await invoke<EmailHeader[]>("sync_emails", {
              config: affectedConfig,
              folder: idleEvent.folder_name,
            });
            console.log(`✅ Background sync completed for account ${idleEvent.account_id}`);
          } catch (e) {
            console.error("Background sync failed:", e);
          }
        }
      }
    } else if (eventType === "Expunge" || eventType === "FlagsChanged") {
      console.log("🗑️ Message(s) changed, refreshing...");

      if (
        idleEvent.account_id === state.selectedAccountId &&
        idleEvent.folder_name === state.selectedFolderName
      ) {
        const selectedConfig = state.accounts.find((acc) => acc.id === state.selectedAccountId);
        if (selectedConfig) {
          try {
            const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
              config: selectedConfig,
              folder: state.selectedFolderName,
            });
            state.emails = syncedEmails;
            console.log("✅ Refreshed after change");
          } catch (e) {
            console.error("Failed to refresh after change:", e);
          }
        }
      }
    } else if (eventType === "ConnectionLost") {
      console.warn(`⚠️ IDLE connection lost for account ${idleEvent.account_id}, will reconnect automatically`);
    }
  }
</script>

<div class="main-layout">
  <AccountsSidebar
    accounts={state.accounts}
    selectedAccountId={state.selectedAccountId}
    isSyncing={state.isSyncing}
    lastSyncTime={state.lastSyncTime}
    currentTime={state.currentTime}
    onAccountClick={handleAccountClick}
    onCompose={handleComposeClick}
    onRefresh={handleManualRefresh}
    onDeleteAccount={handleDeleteAccount}
  />

  <FoldersSidebar
    bind:folders={state.folders}
    bind:selectedFolderName={state.selectedFolderName}
    isLoading={state.isLoadingFolders}
    selectedAccountId={state.selectedAccountId}
    onFolderClick={handleFolderClick}
  />

  <EmailList
    emails={state.emails}
    selectedEmailUid={state.selectedEmailUid}
    isLoading={state.isLoadingEmails}
    error={state.error}
    selectedAccountId={state.selectedAccountId}
    onEmailClick={handleEmailClick}
  />

  <EmailBody
    email={state.selectedEmail}
    body={state.emailBody}
    attachments={state.attachments}
    isLoadingBody={state.isLoadingBody}
    isLoadingAttachments={state.isLoadingAttachments}
    error={state.error}
    onReply={handleReplyClick}
    onForward={handleForwardClick}
    onDelete={handleDeleteEmail}
    onDownloadAttachment={downloadAttachment}
  />

  <ComposeDialog
    show={state.showComposeDialog}
    mode={state.isReplyMode ? "reply" : state.isForwardMode ? "forward" : "compose"}
    bind:to={state.composeTo}
    bind:subject={state.composeSubject}
    bind:body={state.composeBody}
    bind:attachments={state.composeAttachments}
    attachmentSizeLimit={state.attachmentSizeLimit}
    totalAttachmentSize={state.totalAttachmentSize}
    isSending={state.isSending}
    error={state.error}
    onSend={handleSendEmail}
    onCancel={handleCloseCompose}
    onAttachmentAdd={handleAttachmentSelect}
    onAttachmentRemove={removeAttachment}
  />
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
