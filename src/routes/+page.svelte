<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { save } from "@tauri-apps/plugin-dialog";

  // --- 类型定义 ---
  interface AccountConfig {
    id: number;
    email: string;
    password: string;
    imap_server: string;
    imap_port: number;
    smtp_server: string;
    smtp_port: number;
  }

  interface EmailHeader {
    uid: number;
    subject: string;
    from: string;
    to: string;
    date: string;
    timestamp: number; // Unix timestamp in seconds
    has_attachments?: boolean; // Whether email has attachments
  }

  interface AttachmentInfo {
    id: number;
    filename: string;
    content_type: string;
    size: number;
  }

  interface Attachment {
    id?: number;
    filename: string;
    content_type: string;
    size: number;
    data?: number[]; // Uint8Array as number array for JSON
  }

  interface Folder {
    id: number | null;
    account_id: number;
    name: string;              // Original IMAP folder name (for operations)
    display_name: string;      // User-friendly display name
    delimiter: string | null;
    flags: string | null;
  }

  interface IdleEvent {
    account_id: number;
    folder_name: string;
    event_type: {
      type: "NewMessages" | "Expunge" | "FlagsChanged" | "ConnectionLost";
      count?: number;
      uid?: number;
    };
  }

  // --- 状态管理 ---
  let accounts = $state<AccountConfig[]>([]);
  let emails = $state<EmailHeader[]>([]);
  let folders = $state<Folder[]>([]);
  let emailBody = $state<string | null>(null);
  let error = $state<string | null>(null);
  let selectedAccountId = $state<number | null>(null);
  let selectedFolderName = $state<string>("INBOX");
  let selectedEmailUid = $state<number | null>(null);
  let isLoadingEmails = $state<boolean>(false);
  let isLoadingBody = $state<boolean>(false);
  let isLoadingFolders = $state<boolean>(false);
  let isSyncing = $state<boolean>(false);
  let lastSyncTime = $state<number>(0);
  let syncInterval = $state<number>(300); // Default 5 minutes
  let currentTime = $state<number>(Math.floor(Date.now() / 1000)); // Current time for reactive updates

  // Compose email state
  let showComposeDialog = $state<boolean>(false);
  let composeTo = $state<string>("");
  let composeSubject = $state<string>("");
  let composeBody = $state<string>("");
  let isSending = $state<boolean>(false);
  let isReplyMode = $state<boolean>(false);
  let isForwardMode = $state<boolean>(false);

  // Attachment state
  let attachments = $state<AttachmentInfo[]>([]);
  let isLoadingAttachments = $state<boolean>(false);
  let composeAttachments = $state<File[]>([]); // Attachments to send
  let attachmentSizeLimit = $state<number>(10 * 1024 * 1024); // Default 10MB
  let totalAttachmentSize = $derived<number>(
    composeAttachments.reduce((sum, file) => sum + file.size, 0)
  );

  // --- 生命周期 ---
  onMount(() => {
    (async () => {
      try {
        accounts = await invoke<AccountConfig[]>("load_account_configs");
        // Load sync interval setting
        syncInterval = await invoke<number>("get_sync_interval");

        // Start automatic sync timer
        startAutoSyncTimer();

        // Start IDLE connections for ALL accounts' INBOX
        console.log("🔔 Starting IDLE connections for all accounts...");
        for (const account of accounts) {
          try {
            await invoke("start_idle", {
              accountId: account.id,
              folderName: "INBOX",
              config: account
            });
            console.log(`✅ IDLE enabled for account ${account.id} (${account.email})`);
          } catch (e) {
            console.warn(`Failed to start IDLE for account ${account.id}:`, e);
          }
        }

        // Listen for IDLE push notifications from ANY account
        const unlisten = await listen<IdleEvent>("idle-event", async (event) => {
          const idleEvent = event.payload;
          console.log("📬 Received IDLE event:", idleEvent);

          if (idleEvent.event_type.type === "NewMessages") {
            console.log(`✨ New message(s) in account ${idleEvent.account_id} folder ${idleEvent.folder_name}`);

            // If it's for the currently viewing account/folder, update UI immediately
            if (
              idleEvent.account_id === selectedAccountId &&
              idleEvent.folder_name === selectedFolderName
            ) {
              console.log("🔄 Syncing currently displayed folder...");

              const selectedConfig = accounts.find(acc => acc.id === selectedAccountId);
              if (selectedConfig) {
                try {
                  isSyncing = true;
                  const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
                    config: selectedConfig,
                    folder: selectedFolderName
                  });
                  emails = syncedEmails;
                  lastSyncTime = Math.floor(Date.now() / 1000);
                  isSyncing = false;

                  console.log("✅ Auto-sync completed via IDLE push");
                } catch (e) {
                  console.error("Failed to sync after IDLE event:", e);
                  isSyncing = false;
                }
              }
            } else {
              // For other accounts/folders, just sync in background (update cache)
              console.log("📥 Background sync for non-displayed folder");
              const affectedConfig = accounts.find(acc => acc.id === idleEvent.account_id);
              if (affectedConfig) {
                try {
                  await invoke<EmailHeader[]>("sync_emails", {
                    config: affectedConfig,
                    folder: idleEvent.folder_name
                  });
                  console.log(`✅ Background sync completed for account ${idleEvent.account_id}`);
                } catch (e) {
                  console.error("Background sync failed:", e);
                }
              }
            }
          } else if (idleEvent.event_type.type === "ConnectionLost") {
            console.warn(`⚠️ IDLE connection lost for account ${idleEvent.account_id}, will reconnect automatically`);
          }
        });

        // Store unlisten function for cleanup
        return () => {
          unlisten();
          if (autoSyncTimer) {
            clearInterval(autoSyncTimer);
            autoSyncTimer = null;
          }
          clearInterval(timeUpdateTimer);
        };
      } catch (e) {
        error = `Failed to load accounts: ${e}`;
      }
    })();

    // Update current time every minute to refresh "last sync" display
    const timeUpdateTimer = setInterval(() => {
      currentTime = Math.floor(Date.now() / 1000);
    }, 60000); // Update every 60 seconds
  });

  // Reload sync interval when returning from settings page
  $effect(() => {
    // Check if we need to reload sync interval
    // This runs when the component becomes visible again
    const handleVisibilityChange = async () => {
      if (document.visibilityState === 'visible') {
        try {
          const newInterval = await invoke<number>("get_sync_interval");
          if (newInterval !== syncInterval) {
            console.log(`Sync interval changed from ${syncInterval} to ${newInterval}`);
            syncInterval = newInterval;
            startAutoSyncTimer();
          }
        } catch (e) {
          console.error("Failed to reload sync interval:", e);
        }
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    window.addEventListener('focus', handleVisibilityChange);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      window.removeEventListener('focus', handleVisibilityChange);
    };
  });

  // Automatic sync timer
  let autoSyncTimer: ReturnType<typeof setInterval> | null = null;

  function startAutoSyncTimer() {
    // Clear any existing timer
    if (autoSyncTimer) {
      clearInterval(autoSyncTimer);
      autoSyncTimer = null;
    }

    // Don't start timer if sync is disabled
    if (syncInterval <= 0) {
      console.log("Auto-sync disabled (interval:", syncInterval, ")");
      return;
    }

    console.log(`Starting auto-sync timer with interval: ${syncInterval} seconds`);

    // Check and sync every minute (we'll check if sync is needed based on interval)
    autoSyncTimer = setInterval(async () => {
      // Only sync if we have a selected account
      if (!selectedAccountId) {
        return;
      }

      const selectedConfig = accounts.find(acc => acc.id === selectedAccountId);
      if (!selectedConfig) {
        return;
      }

      try {
        // Check if sync is needed based on interval
        const needsSync = await invoke<boolean>("should_sync", {
          accountId: selectedAccountId,
          folder: selectedFolderName,
          syncInterval
        });

        if (needsSync && !isSyncing) {
          console.log(`Auto-sync triggered for ${selectedFolderName}`);
          isSyncing = true;

          // Sync folders
          const syncedFolders = await invoke<Folder[]>("sync_folders", { config: selectedConfig });
          folders = syncedFolders;

          // Sync emails for current folder
          const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
            config: selectedConfig,
            folder: selectedFolderName
          });

          emails = syncedEmails;
          lastSyncTime = Math.floor(Date.now() / 1000);
          isSyncing = false;

          console.log(`Auto-sync completed for ${selectedFolderName}`);
        }
      } catch (e) {
        console.error("Auto-sync failed:", e);
        isSyncing = false;
      }
    }, 60000); // Check every minute
  }

  // --- 事件处理 ---
  async function handleAccountClick(accountId: number) {
    // If clicking the same account that's already selected, do nothing
    if (selectedAccountId === accountId) {
      console.log("Same account already selected, ignoring click");
      return;
    }

    selectedAccountId = accountId;
    selectedFolderName = "INBOX";
    selectedEmailUid = null;
    emailBody = null;
    emails = [];
    error = null;

    const selectedConfig = accounts.find(acc => acc.id === accountId);
    if (!selectedConfig) {
        error = "Could not find selected account configuration.";
        return;
    }

    isLoadingFolders = true;

    try {
      // First, load folders from cache for instant display
      const cachedFolders = await invoke<Folder[]>("load_folders", { accountId });
      folders = cachedFolders;
      isLoadingFolders = false;

      console.log(`Loaded ${cachedFolders.length} folders from cache`);

      // Then load emails from INBOX using cache-first strategy
      await loadEmailsForFolder("INBOX");

      // Check if we should sync folders (based on sync interval)
      const needsFolderSync = await invoke<boolean>("should_sync", {
        accountId: accountId,
        folder: "__folders__",  // Special folder name for folders sync
        syncInterval
      });

      if (needsFolderSync) {
        // Sync folders in the background to get updates
        console.log("Syncing folders in background...");
        const syncedFolders = await invoke<Folder[]>("sync_folders", { config: selectedConfig });
        folders = syncedFolders;
        console.log(`Synced ${syncedFolders.length} folders from server`);
      } else {
        console.log("Using cached folders, sync not needed yet");
      }

      // Note: IDLE connections are managed globally at app startup,
      // no need to start/stop when switching accounts

    } catch (e) {
      error = `Failed to load folders: ${e}`;
      isLoadingFolders = false;
    }
  }

  async function loadEmailsForFolder(folderName: string) {
    const selectedConfig = accounts.find(acc => acc.id === selectedAccountId);
    if (!selectedConfig) {
        error = "Could not find selected account configuration.";
        return;
    }

    if (!selectedAccountId) {
        error = "No account selected.";
        return;
    }

    isLoadingEmails = true;
    selectedEmailUid = null;
    emailBody = null;
    error = null;

    try {
      // First, load from cache for instant display
      const cachedEmails = await invoke<EmailHeader[]>("load_emails_from_cache", {
        accountId: selectedAccountId,
        folder: folderName
      });

      // Display cached emails immediately
      emails = cachedEmails;
      isLoadingEmails = false;

      console.log(`Loaded ${cachedEmails.length} emails from cache`);

      // Get last sync time
      lastSyncTime = await invoke<number>("get_last_sync_time", {
        accountId: selectedAccountId,
        folder: folderName
      });

      // Check if sync is needed based on interval
      const needsSync = await invoke<boolean>("should_sync", {
        accountId: selectedAccountId,
        folder: folderName,
        syncInterval
      });

      if (needsSync) {
        // Sync in the background to get updates
        console.log("Sync needed, syncing emails in background...");
        isSyncing = true;
        const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
          config: selectedConfig,
          folder: folderName
        });

        // Update with fresh data from server
        emails = syncedEmails;
        lastSyncTime = Math.floor(Date.now() / 1000);
        isSyncing = false;
        console.log(`Synced ${syncedEmails.length} emails from server`);
      } else {
        console.log("Using cache, sync not needed yet");
      }

    } catch (e) {
      error = `Failed to load emails: ${e}`;
      isLoadingEmails = false;
      isSyncing = false;
    }
  }

  async function handleFolderClick(folderName: string) {
    selectedFolderName = folderName;
    await loadEmailsForFolder(folderName);

    // Note: Currently IDLE only monitors INBOX for all accounts
    // TODO: Add support for monitoring non-INBOX folders
  }

  async function handleEmailClick(uid: number) {
      selectedEmailUid = uid;
      isLoadingBody = true;
      emailBody = null;
      attachments = [];
      error = null;

      const selectedConfig = accounts.find(acc => acc.id === selectedAccountId);
      if (!selectedConfig) {
          error = "Could not find selected account configuration.";
          isLoadingBody = false;
          return;
      }

      try {
          // Use cached body if available
          emailBody = await invoke<string>("fetch_email_body_cached", {
            config: selectedConfig,
            uid,
            folder: selectedFolderName
          });

          // Load attachments if available
          if (selectedAccountId) {
              loadAttachmentsForEmail(selectedAccountId, uid);
          }
      } catch (e) {
          error = `Failed to fetch email body: ${e}`;
      } finally {
          isLoadingBody = false;
      }
  }

  async function loadAttachmentsForEmail(accountId: number, uid: number) {
      isLoadingAttachments = true;
      try {
          attachments = await invoke<AttachmentInfo[]>("load_attachments_info", {
              accountId,
              folderName: selectedFolderName,
              uid
          });
      } catch (e) {
          console.error("Failed to load attachments:", e);
      } finally {
          isLoadingAttachments = false;
      }
  }

  async function downloadAttachment(attachmentId: number, filename: string) {
      try {
          // Show save dialog to user
          const filePath = await save({
              defaultPath: filename,
              title: "Save Attachment"
          });

          if (filePath) {
              // Save the attachment directly to the selected path
              await invoke("save_attachment_to_file", {
                  attachmentId,
                  filePath
              });
              console.log("Attachment saved successfully:", filePath);
          }
      } catch (e) {
          console.error("Failed to save attachment:", e);
          error = `Failed to download attachment: ${e}`;
      }
  }

  function formatFileSize(bytes: number): string {
      if (bytes < 1024) return bytes + " B";
      if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
      if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + " MB";
      return (bytes / (1024 * 1024 * 1024)).toFixed(1) + " GB";
  }

  // Manual refresh function
  async function handleManualRefresh() {
      if (!selectedAccountId) {
          error = "Please select an account first.";
          return;
      }

      const selectedConfig = accounts.find(acc => acc.id === selectedAccountId);
      if (!selectedConfig) {
          error = "Could not find selected account configuration.";
          return;
      }

      isSyncing = true;
      error = null;

      try {
          // Sync folders
          console.log("Manual refresh: syncing folders...");
          folders = await invoke<Folder[]>("sync_folders", { config: selectedConfig });

          // Sync emails for current folder
          console.log("Manual refresh: syncing emails...");
          const syncedEmails = await invoke<EmailHeader[]>("sync_emails", {
              config: selectedConfig,
              folder: selectedFolderName
          });

          emails = syncedEmails;
          lastSyncTime = Math.floor(Date.now() / 1000);

          console.log("Manual refresh completed");
      } catch (e) {
          error = `Failed to refresh: ${e}`;
      } finally {
          isSyncing = false;
      }
  }

  async function handleDeleteAccount(email: string, event: MouseEvent) {
      event.stopPropagation();

      if (!confirm(`确定要删除账户 ${email} 吗？`)) {
          return;
      }

      try {
          await invoke("delete_account", { email });
          // Reload accounts
          accounts = await invoke<AccountConfig[]>("load_account_configs");
          // Clear selection if deleted account was selected
          const deletedAccount = accounts.find(acc => acc.email === email);
          if (deletedAccount && deletedAccount.id === selectedAccountId) {
              selectedAccountId = null;
              emails = [];
              emailBody = null;
          }
      } catch (e) {
          error = `Failed to delete account: ${e}`;
      }
  }

  async function handleComposeClick() {
      if (!selectedAccountId) {
          error = "Please select an account first.";
          return;
      }
      showComposeDialog = true;
      isReplyMode = false;
      isForwardMode = false;
      composeTo = "";
      composeSubject = "";
      composeBody = "";
      composeAttachments = [];
      error = null;
      await updateAttachmentSizeLimit();
  }

  async function handleReplyClick() {
      if (!selectedAccountId || !selectedEmailUid) {
          error = "Please select an email first.";
          return;
      }

      const selectedEmail = emails.find(email => email.uid === selectedEmailUid);
      if (!selectedEmail) {
          error = "Could not find selected email.";
          return;
      }

      showComposeDialog = true;
      isReplyMode = true;
      isForwardMode = false;
      composeTo = selectedEmail.from;
      composeSubject = selectedEmail.subject.toLowerCase().startsWith("re:")
          ? selectedEmail.subject
          : `Re: ${selectedEmail.subject}`;
      composeBody = "";
      composeAttachments = [];
      error = null;
      await updateAttachmentSizeLimit();
  }

  async function handleForwardClick() {
      if (!selectedAccountId || !selectedEmailUid) {
          error = "Please select an email first.";
          return;
      }

      const selectedEmail = emails.find(email => email.uid === selectedEmailUid);
      if (!selectedEmail) {
          error = "Could not find selected email.";
          return;
      }

      showComposeDialog = true;
      isReplyMode = false;
      isForwardMode = true;
      composeTo = "";
      composeSubject = selectedEmail.subject.toLowerCase().startsWith("fwd:")
          ? selectedEmail.subject
          : `Fwd: ${selectedEmail.subject}`;
      composeBody = "";
      composeAttachments = [];
      error = null;
      await updateAttachmentSizeLimit();
  }

  function handleCloseCompose() {
      showComposeDialog = false;
      composeTo = "";
      composeSubject = "";
      composeBody = "";
      composeAttachments = [];
      error = null;
  }

  function handleAttachmentSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files) return;

    const newFiles = Array.from(input.files);
    const allFiles = [...composeAttachments, ...newFiles];

    // Check if total size exceeds limit
    const totalSize = allFiles.reduce((sum, file) => sum + file.size, 0);
    if (totalSize > attachmentSizeLimit) {
      const limitMB = (attachmentSizeLimit / (1024 * 1024)).toFixed(2);
      const totalMB = (totalSize / (1024 * 1024)).toFixed(2);
      error = `Total attachment size (${totalMB} MB) exceeds the limit for your email provider (${limitMB} MB)`;
      input.value = ""; // Reset input
      return;
    }

    composeAttachments = allFiles;
    input.value = ""; // Reset input so same file can be selected again
    error = null;
  }

  function removeAttachment(index: number) {
    composeAttachments = composeAttachments.filter((_, i) => i !== index);
  }

  async function updateAttachmentSizeLimit() {
    if (!selectedAccountId) return;

    const selectedConfig = accounts.find(acc => acc.id === selectedAccountId);
    if (!selectedConfig) return;

    try {
      const limit = await invoke<number>("get_attachment_size_limit", {
        email: selectedConfig.email
      });
      attachmentSizeLimit = limit;
    } catch (e) {
      console.error("Failed to get attachment size limit:", e);
      // Keep default limit
    }
  }

  async function handleSendEmail() {
      if (!selectedAccountId) {
          error = "Please select an account first.";
          return;
      }

      if (!composeTo || !composeSubject) {
          error = "Please fill in recipient and subject fields.";
          return;
      }

      const selectedConfig = accounts.find(acc => acc.id === selectedAccountId);
      if (!selectedConfig) {
          error = "Could not find selected account configuration.";
          return;
      }

      isSending = true;
      error = null;

      try {
          // Prepare attachments data
          let attachmentsData: Array<{filename: string, content_type: string, data: number[]}> | null = null;
          if (composeAttachments.length > 0) {
              attachmentsData = [];
              for (const file of composeAttachments) {
                  const arrayBuffer = await file.arrayBuffer();
                  const uint8Array = new Uint8Array(arrayBuffer);
                  const dataArray = Array.from(uint8Array);

                  attachmentsData.push({
                      filename: file.name,
                      content_type: file.type || "application/octet-stream",
                      data: dataArray
                  });
              }
          }

          let result: string;
          if (isReplyMode) {
              result = await invoke<string>("reply_email", {
                  config: selectedConfig,
                  to: composeTo,
                  originalSubject: composeSubject,
                  body: composeBody,
                  attachments: attachmentsData
              });
          } else if (isForwardMode) {
              const selectedEmail = emails.find(email => email.uid === selectedEmailUid);
              if (!selectedEmail) {
                  error = "Could not find selected email.";
                  isSending = false;
                  return;
              }
              result = await invoke<string>("forward_email", {
                  config: selectedConfig,
                  to: composeTo,
                  originalSubject: selectedEmail.subject,
                  originalFrom: selectedEmail.from,
                  originalTo: selectedEmail.to,
                  originalDate: selectedEmail.date,
                  originalBody: emailBody || "",
                  additionalMessage: composeBody,
                  attachments: attachmentsData
              });
          } else {
              if (!composeBody) {
                  error = "Please fill in the message body.";
                  isSending = false;
                  return;
              }
              result = await invoke<string>("send_email", {
                  config: selectedConfig,
                  to: composeTo,
                  subject: composeSubject,
                  body: composeBody,
                  attachments: attachmentsData
              });
          }
          console.log("Send result:", result);
          handleCloseCompose();
          alert("Email sent successfully!");
      } catch (e) {
          error = `Failed to send email: ${e}`;
      } finally {
          isSending = false;
      }
  }

  // Helper function to format time since last sync
  function formatTimeSince(timestamp: number): string {
      const seconds = currentTime - timestamp;

      if (seconds < 60) return "just now";
      const minutes = Math.floor(seconds / 60);
      if (minutes < 60) return `${minutes}m ago`;
      const hours = Math.floor(minutes / 60);
      if (hours < 24) return `${hours}h ago`;
      const days = Math.floor(hours / 24);
      return `${days}d ago`;
  }

  // Helper function to format timestamp to local time (compact for list view)
  function formatLocalDateTime(timestamp: number): string {
      const date = new Date(timestamp * 1000); // Convert seconds to milliseconds
      const now = new Date();
      const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
      const yesterday = new Date(today);
      yesterday.setDate(yesterday.getDate() - 1);
      const emailDate = new Date(date.getFullYear(), date.getMonth(), date.getDate());

      // Format time as HH:MM
      const timeStr = date.toLocaleTimeString(undefined, {
          hour: '2-digit',
          minute: '2-digit',
          hour12: false
      });

      // If today, show time only
      if (emailDate.getTime() === today.getTime()) {
          return timeStr;
      }

      // If yesterday, show "Yesterday HH:MM"
      if (emailDate.getTime() === yesterday.getTime()) {
          return `Yesterday ${timeStr}`;
      }

      // If within this year, show "MMM DD HH:MM"
      if (date.getFullYear() === now.getFullYear()) {
          const monthDay = date.toLocaleDateString(undefined, {
              month: 'short',
              day: 'numeric'
          });
          return `${monthDay} ${timeStr}`;
      }

      // Otherwise show full date "MMM DD, YYYY HH:MM"
      return date.toLocaleDateString(undefined, {
          month: 'short',
          day: 'numeric',
          year: 'numeric'
      }) + ` ${timeStr}`;
  }

  // Helper function to format timestamp to full local time (for detail view)
  function formatFullLocalDateTime(timestamp: number): string {
      const date = new Date(timestamp * 1000); // Convert seconds to milliseconds

      // Format: "Day, Month DD, YYYY at HH:MM:SS"
      return date.toLocaleDateString(undefined, {
          weekday: 'long',
          year: 'numeric',
          month: 'long',
          day: 'numeric'
      }) + ' at ' + date.toLocaleTimeString(undefined, {
          hour: '2-digit',
          minute: '2-digit',
          second: '2-digit',
          hour12: false
      });
  }

  // Helper function to check if current folder is a trash/deleted folder
  function isTrashFolder(folderName: string): boolean {
      const lowerName = folderName.toLowerCase();
      return lowerName.includes("trash") ||
             lowerName.includes("deleted") ||
             lowerName.includes("垃圾") ||
             lowerName.includes("bin");
  }

  async function handleDeleteEmail() {
      if (!selectedAccountId || !selectedEmailUid) {
          error = "Please select an email first.";
          return;
      }

      const selectedEmail = emails.find(email => email.uid === selectedEmailUid);
      if (!selectedEmail) {
          error = "Could not find selected email.";
          return;
      }

      const selectedConfig = accounts.find(acc => acc.id === selectedAccountId);
      if (!selectedConfig) {
          error = "Could not find selected account configuration.";
          return;
      }

      const isInTrash = isTrashFolder(selectedFolderName);

      // Different confirmation messages based on whether we're in trash or not
      const confirmMessage = isInTrash
          ? `Are you sure you want to PERMANENTLY delete this email?\n\nThis action cannot be undone.\n\nSubject: ${selectedEmail.subject}`
          : `Move this email to trash?\n\nSubject: ${selectedEmail.subject}`;

      if (!confirm(confirmMessage)) {
          return;
      }

      error = null;

      try {
          if (isInTrash) {
              // If already in trash, permanently delete
              await invoke("delete_email", {
                  config: selectedConfig,
                  uid: selectedEmailUid,
                  folder: selectedFolderName
              });
              alert("Email permanently deleted!");
          } else {
              // Otherwise, move to trash
              await invoke("move_email_to_trash", {
                  config: selectedConfig,
                  uid: selectedEmailUid,
                  folder: selectedFolderName
              });
              alert("Email moved to trash!");
          }

          // Clear the selected email and body
          selectedEmailUid = null;
          emailBody = null;

          // Reload the email list
          await loadEmailsForFolder(selectedFolderName);
      } catch (e) {
          error = `Failed to delete email: ${e}`;
      }
  }

</script>

<div class="main-layout">
  <!-- ACCOUNTS SIDEBAR -->
  <aside class="sidebar accounts-sidebar">
    <h2>Accounts</h2>
    <ul>
      {#each accounts as account (account.id)}
        <li>
          <div class="account-item-wrapper">
            <button
              class="account-item"
              class:selected={account.id === selectedAccountId}
              onclick={() => handleAccountClick(account.id)}
            >
              {account.email}
            </button>
            <button
              class="delete-button"
              onclick={(e) => handleDeleteAccount(account.email, e)}
              title="删除账户"
              aria-label="删除账户 {account.email}"
            >
              ×
            </button>
          </div>
        </li>
      {/each}
      {#if accounts.length === 0 && !error}
        <li class="no-accounts">No accounts configured.</li>
      {/if}
    </ul>
    <button class="compose-button" onclick={handleComposeClick} disabled={!selectedAccountId}>
      ✉️ Compose
    </button>
    <button class="refresh-button" onclick={handleManualRefresh} disabled={!selectedAccountId || isSyncing}>
      {isSyncing ? "🔄 Syncing..." : "🔄 Refresh"}
    </button>
    {#if selectedAccountId && lastSyncTime > 0}
      <div class="sync-status">
        Last sync: {formatTimeSince(lastSyncTime)}
      </div>
    {/if}
    <a href="/account" class="add-account-link">+ Add Account</a>
    <a href="/settings" class="settings-link">⚙️ Settings</a>
  </aside>

  <!-- FOLDERS SIDEBAR -->
  <aside class="sidebar folders-sidebar">
    <h2>Folders</h2>
    {#if isLoadingFolders}
      <p class="loading-text">Loading folders...</p>
    {:else if folders.length > 0}
      <ul>
        {#each folders as folder (folder.name)}
          <li>
            <button
              class="folder-item"
              class:selected={folder.name === selectedFolderName}
              onclick={() => handleFolderClick(folder.name)}
              title={folder.name}
            >
              📁 {folder.display_name}
            </button>
          </li>
        {/each}
      </ul>
    {:else if selectedAccountId}
      <p class="no-folders">No folders found.</p>
    {:else}
      <p class="no-folders">Select an account to view folders.</p>
    {/if}
  </aside>

  <!-- EMAIL LIST PANE -->
  <div class="email-list-pane">
    {#if isLoadingEmails}
        <p>Loading emails...</p>
    {:else if error && emails.length === 0}
        <p class="error-message">{error}</p>
    {:else if emails.length > 0}
        <ul class="email-list">
            {#each emails as email (email.uid)}
                <li>
                    <button class="email-item" class:selected={email.uid === selectedEmailUid} onclick={() => handleEmailClick(email.uid)}>
                        <div class="email-item-content">
                            {#if email.has_attachments}
                                <span class="attachment-indicator" title="This email has attachments">📎</span>
                            {/if}
                            <div class="email-text">
                                <div class="from">{email.from}</div>
                                <div class="subject">{email.subject}</div>
                            </div>
                        </div>
                        <div class="date">{formatLocalDateTime(email.timestamp)}</div>
                    </button>
                </li>
            {/each}
        </ul>
    {:else if selectedAccountId}
        <p>No emails found in this inbox.</p>
    {/if}
  </div>

  <!-- EMAIL BODY PANE -->
  <main class="content-pane">
    {#if isLoadingBody}
        <p>Loading email content...</p>
    {:else if emailBody && selectedEmailUid}
        {@const selectedEmail = emails.find(email => email.uid === selectedEmailUid)}
        {#if selectedEmail}
            <div class="email-header-section">
                <h2 class="email-subject">{selectedEmail.subject}</h2>
                <div class="email-meta">
                    <div class="meta-row">
                        <span class="meta-label">From:</span>
                        <span class="meta-value">{selectedEmail.from}</span>
                    </div>
                    <div class="meta-row">
                        <span class="meta-label">To:</span>
                        <span class="meta-value">{selectedEmail.to}</span>
                    </div>
                    <div class="meta-row">
                        <span class="meta-label">Date:</span>
                        <span class="meta-value">{formatFullLocalDateTime(selectedEmail.timestamp)}</span>
                    </div>
                </div>
                <div class="email-actions">
                    <button class="action-button reply-button" onclick={handleReplyClick}>
                        ↩ Reply
                    </button>
                    <button class="action-button forward-button" onclick={handleForwardClick}>
                        ➡ Forward
                    </button>
                    <button class="action-button delete-email-button" onclick={handleDeleteEmail}>
                        🗑 Delete
                    </button>
                </div>
            </div>

            <!-- Attachments Section (between header and body) -->
            {#if isLoadingAttachments}
                <div class="attachments-section">
                    <h3>Attachments</h3>
                    <p class="loading-text">Loading attachments...</p>
                </div>
            {:else if attachments.length > 0}
                <div class="attachments-section">
                    <h3>📎 Attachments ({attachments.length})</h3>
                    <div class="attachments-list">
                        {#each attachments as attachment (attachment.id)}
                            <button
                                type="button"
                                class="attachment-item"
                                onclick={() => downloadAttachment(attachment.id, attachment.filename)}
                            >
                                <span class="attachment-icon">📎</span>
                                <div class="attachment-info">
                                    <span class="attachment-name">{attachment.filename}</span>
                                    <span class="attachment-size">{formatFileSize(attachment.size)}</span>
                                </div>
                                <span class="download-icon">⬇</span>
                            </button>
                        {/each}
                    </div>
                </div>
            {/if}

            <div class="email-body">
                {@html emailBody}
            </div>
        {/if}
    {:else if selectedEmailUid}
        <p class="error-message">{error}</p>
    {:else}
        <div class="placeholder">
            <p>Select an email to read its content.</p>
        </div>
    {/if}
  </main>
</div>

<!-- Compose Email Dialog -->
{#if showComposeDialog}
  <div class="modal-overlay" onclick={handleCloseCompose} role="button" tabindex="0" onkeydown={(e) => e.key === 'Escape' && handleCloseCompose()}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.key === 'Escape' && handleCloseCompose()} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-header">
        <h2>{isReplyMode ? "Reply to Email" : isForwardMode ? "Forward Email" : "Compose Email"}</h2>
        <button class="close-button" onclick={handleCloseCompose}>×</button>
      </div>

      <div class="modal-body">
        {#if error}
          <div class="error-banner">{error}</div>
        {/if}

        <div class="form-group">
          <label for="compose-to">To:</label>
          <input
            type="email"
            id="compose-to"
            bind:value={composeTo}
            placeholder="recipient@example.com"
            disabled={isSending}
          />
        </div>

        <div class="form-group">
          <label for="compose-subject">Subject:</label>
          <input
            type="text"
            id="compose-subject"
            bind:value={composeSubject}
            placeholder="Email subject"
            disabled={isSending}
          />
        </div>

        <div class="form-group">
          <label for="compose-body">Body:</label>
          <textarea
            id="compose-body"
            bind:value={composeBody}
            placeholder="Write your message here..."
            rows="10"
            disabled={isSending}
          ></textarea>
        </div>

        <div class="form-group">
          <label for="compose-attachments">
            Attachments:
            <span class="attachment-limit-info">
              (Max: {formatFileSize(attachmentSizeLimit)})
            </span>
          </label>
          <input
            type="file"
            id="compose-attachments"
            multiple
            onchange={handleAttachmentSelect}
            disabled={isSending}
            style="margin-bottom: 0.5rem;"
          />

          {#if composeAttachments.length > 0}
            <div class="attachments-list">
              {#each composeAttachments as file, index}
                <div class="attachment-item">
                  <span class="attachment-name">{file.name}</span>
                  <span class="attachment-size">({formatFileSize(file.size)})</span>
                  <button
                    class="remove-attachment-button"
                    onclick={() => removeAttachment(index)}
                    disabled={isSending}
                    title="Remove attachment"
                  >
                    ×
                  </button>
                </div>
              {/each}
              <div class="attachment-total">
                Total: {formatFileSize(totalAttachmentSize)} / {formatFileSize(attachmentSizeLimit)}
              </div>
            </div>
          {/if}
        </div>
      </div>

      <div class="modal-footer">
        <button class="cancel-button" onclick={handleCloseCompose} disabled={isSending}>
          Cancel
        </button>
        <button class="send-button" onclick={handleSendEmail} disabled={isSending}>
          {isSending ? "Sending..." : "Send"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Global scrollbar hiding for desktop app experience */
  :global(*) {
    scrollbar-width: none; /* Firefox */
    -ms-overflow-style: none; /* IE and Edge */
  }

  :global(*::-webkit-scrollbar) {
    display: none; /* Chrome, Safari, and Opera */
  }

  /* Prevent any global scrolling - desktop app should have fixed layout */
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    width: 100vw;
    height: 100vh;
    overflow: hidden; /* No page-level scrolling */
    position: fixed; /* Lock the viewport */
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
    overflow: hidden; /* Prevent page-level scrolling */
  }

  .sidebar, .email-list-pane, .content-pane {
      height: 100vh;
      overflow: hidden; /* Remove scrollbars, content will be contained */
      display: flex;
      flex-direction: column;
  }

  .accounts-sidebar, .folders-sidebar {
    background-color: var(--sidebar-bg);
    border-right: 1px solid var(--border-color);
    user-select: none;
    padding: 0;
  }

  .accounts-sidebar h2, .folders-sidebar h2 {
    margin: 0;
    border-bottom: 1px solid var(--border-color);
    padding: 1rem;
    font-size: 1rem;
    flex-shrink: 0; /* Fixed header */
  }

  .accounts-sidebar ul, .folders-sidebar ul {
    list-style: none;
    padding: 0.5rem;
    margin: 0;
    flex: 1;
    overflow-y: auto; /* Only the list scrolls */
    min-height: 0; /* Allow flex shrinking */
  }

  .accounts-sidebar li, .folders-sidebar li {
      margin-bottom: 4px;
  }

  .loading-text, .no-folders {
    text-align: center;
    color: #666;
    font-size: 0.875rem;
    padding: 2rem 1rem;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .no-accounts {
    text-align: center;
    color: #666;
    font-size: 0.875rem;
    padding: 1rem;
  }

  .account-item-wrapper {
    display: flex;
    align-items: center;
    gap: 4px;
    position: relative;
  }

  .account-item {
    background: none; border: none; font: inherit; color: inherit; text-align: left;
    flex: 1;
    padding: 0.75rem;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    transition: background-color 0.2s, color 0.2s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .account-item:hover {
    background-color: var(--hover-bg);
  }

  .account-item.selected {
    background-color: var(--selected-bg);
    color: var(--selected-text);
  }

  .folder-item {
    background: none; border: none; font: inherit; color: inherit; text-align: left;
    width: 100%;
    padding: 0.75rem;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    transition: background-color 0.2s, color 0.2s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .folder-item:hover {
    background-color: var(--hover-bg);
  }

  .folder-item.selected {
    background-color: var(--selected-bg);
    color: var(--selected-text);
  }

  .delete-button {
    background: none;
    border: none;
    color: #999;
    font-size: 1.5rem;
    line-height: 1;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    transition: all 0.2s;
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .delete-button:hover {
    background-color: #ff4444;
    color: white;
  }

  .account-item-wrapper:hover .delete-button {
    opacity: 1;
  }

  .compose-button {
      display: block;
      width: calc(100% - 2rem);
      text-align: center;
      padding: 0.75rem;
      margin: 0.5rem 1rem;
      border-radius: 6px;
      background-color: #28a745;
      color: white;
      border: none;
      font-weight: 500;
      flex-shrink: 0;
      cursor: pointer;
      transition: background-color 0.2s;
  }

  .compose-button:hover:not(:disabled) {
      background-color: #218838;
  }

  .compose-button:disabled {
      background-color: #6c757d;
      cursor: not-allowed;
      opacity: 0.6;
  }

  .refresh-button {
      display: block;
      width: calc(100% - 2rem);
      text-align: center;
      padding: 0.75rem;
      margin: 0.5rem 1rem;
      border-radius: 6px;
      background-color: #17a2b8;
      color: white;
      border: none;
      font-weight: 500;
      flex-shrink: 0;
      cursor: pointer;
      transition: background-color 0.2s;
  }

  .refresh-button:hover:not(:disabled) {
      background-color: #138496;
  }

  .refresh-button:disabled {
      background-color: #6c757d;
      cursor: not-allowed;
      opacity: 0.6;
  }

  .sync-status {
      text-align: center;
      font-size: 0.75rem;
      color: #666;
      padding: 0.25rem 1rem;
      margin: 0 1rem 0.5rem 1rem;
      flex-shrink: 0;
  }

  .add-account-link {
      display: block;
      text-align: center;
      padding: 0.75rem;
      margin: 0 1rem 0.5rem 1rem;
      border-radius: 6px;
      background-color: var(--link-bg);
      color: var(--link-text);
      text-decoration: none;
      font-weight: 500;
      flex-shrink: 0;
      transition: background-color 0.2s;
  }

  .add-account-link:hover {
      background-color: var(--link-hover-bg);
  }

  .settings-link {
      display: block;
      text-align: center;
      padding: 0.75rem;
      margin: 0 1rem 1rem 1rem;
      border-radius: 6px;
      background-color: #6c757d;
      color: white;
      text-decoration: none;
      font-weight: 500;
      flex-shrink: 0;
      transition: background-color 0.2s;
  }

  .settings-link:hover {
      background-color: #5a6268;
  }

  .email-list-pane {
      border-right: 1px solid var(--border-color);
      padding: 0;
  }

  .email-list-pane > p {
      padding: 1rem;
      text-align: center;
  }

  .email-list {
      list-style: none;
      padding: 0.5rem;
      margin: 0;
      text-align: left;
      flex: 1;
      overflow-y: auto; /* Only the list scrolls */
      min-height: 0;
  }

  .email-list li {
      margin-bottom: 4px;
  }

  .email-item {
      background: none; border: none; font: inherit; text-align: left;
      width: 100%;
      border: 1px solid var(--border-color);
      border-radius: 8px;
      padding: 0.75rem;
      cursor: pointer;
      transition: background-color 0.2s;
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 0.5rem;
  }

  .email-item:hover {
      background-color: var(--sidebar-bg);
  }

  .email-item.selected {
      border-left: 4px solid var(--selected-bg);
      background-color: var(--sidebar-bg);
  }

  .email-item-content {
      flex: 1;
      display: flex;
      align-items: flex-start;
      gap: 0.5rem;
      min-width: 0;
  }

  .attachment-indicator {
      font-size: 1rem;
      flex-shrink: 0;
      margin-top: 2px;
      opacity: 0.7;
  }

  .email-text {
      flex: 1;
      min-width: 0;
  }

  .email-item .from {
      font-weight: bold;
      font-size: 0.9rem;
  }
  .email-item .subject {
      margin: 0.25rem 0;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
  }
  .email-item .date {
      font-size: 0.8rem;
      color: #666;
      flex-shrink: 0;
      margin-top: 2px;
  }

  .content-pane {
      padding: 0;
  }

  .email-header-section {
      padding: 1.5rem 2rem;
      border-bottom: 1px solid var(--border-color);
      background-color: var(--sidebar-bg);
      flex-shrink: 0; /* Fixed header */
  }

  .email-subject {
      margin: 0 0 1rem 0;
      font-size: 1.5rem;
      font-weight: 600;
      color: var(--text-color);
  }

  .email-meta {
      margin-bottom: 1rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
  }

  .meta-row {
      display: flex;
      align-items: baseline;
      gap: 0.5rem;
      font-size: 0.9rem;
  }

  .meta-label {
      font-weight: 600;
      color: #666;
      min-width: 60px;
  }

  .meta-value {
      color: var(--text-color);
      word-break: break-word;
  }

  .email-actions {
      display: flex;
      gap: 0.5rem;
      margin-top: 1rem;
  }

  .action-button {
      background-color: #007bff;
      color: white;
      border: none;
      padding: 0.5rem 1rem;
      border-radius: 4px;
      cursor: pointer;
      font-weight: 500;
      transition: background-color 0.2s;
      font-size: 0.9rem;
  }

  .action-button:hover {
      background-color: #0056b3;
  }

  .forward-button {
      background-color: #28a745;
  }

  .forward-button:hover {
      background-color: #218838;
  }

  .delete-email-button {
      background-color: #dc3545;
  }

  .delete-email-button:hover {
      background-color: #c82333;
  }

  .email-body {
      padding: 1rem 2rem;
      line-height: 1.6;
      flex: 1;
      overflow-y: auto; /* Only the body scrolls */
      min-height: 0;
  }

  .placeholder, .content-pane > p {
      text-align: center;
      padding: 4rem 2rem;
      color: #666;
  }

  .error-message {
    color: #d9534f;
    text-align: center;
    padding: 2rem;
  }

  /* Modal Styles */
  .modal-overlay {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-color: rgba(0, 0, 0, 0.5);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 1000;
  }

  .modal-content {
      background-color: var(--app-bg);
      border-radius: 8px;
      width: 90%;
      max-width: 600px;
      max-height: 90vh;
      overflow: hidden;
      display: flex;
      flex-direction: column;
      box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .modal-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 1rem 1.5rem;
      border-bottom: 1px solid var(--border-color);
  }

  .modal-header h2 {
      margin: 0;
      font-size: 1.25rem;
  }

  .close-button {
      background: none;
      border: none;
      font-size: 1.5rem;
      color: #999;
      cursor: pointer;
      padding: 0;
      width: 30px;
      height: 30px;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 4px;
      transition: all 0.2s;
  }

  .close-button:hover {
      background-color: #ff4444;
      color: white;
  }

  .modal-body {
      padding: 1.5rem;
      overflow-y: auto;
      flex: 1;
  }

  .error-banner {
      background-color: #f8d7da;
      color: #721c24;
      padding: 0.75rem;
      border-radius: 4px;
      margin-bottom: 1rem;
      border: 1px solid #f5c6cb;
  }

  .form-group {
      margin-bottom: 1rem;
  }

  .form-group label {
      display: block;
      margin-bottom: 0.5rem;
      font-weight: 500;
  }

  .form-group input,
  .form-group textarea {
      width: 100%;
      padding: 0.5rem;
      border: 1px solid var(--border-color);
      border-radius: 4px;
      font-family: inherit;
      font-size: 1rem;
      background-color: var(--app-bg);
      color: var(--text-color);
  }

  .form-group input:focus,
  .form-group textarea:focus {
      outline: none;
      border-color: var(--selected-bg);
  }

  .form-group textarea {
      resize: vertical;
      min-height: 150px;
  }

  .modal-footer {
      padding: 1rem 1.5rem;
      border-top: 1px solid var(--border-color);
      display: flex;
      justify-content: flex-end;
      gap: 0.5rem;
  }

  .cancel-button,
  .send-button {
      padding: 0.5rem 1.5rem;
      border-radius: 4px;
      border: none;
      font-weight: 500;
      cursor: pointer;
      transition: background-color 0.2s;
  }

  .cancel-button {
      background-color: #6c757d;
      color: white;
  }

  .cancel-button:hover:not(:disabled) {
      background-color: #5a6268;
  }

  .send-button {
      background-color: #007bff;
      color: white;
  }

  .send-button:hover:not(:disabled) {
      background-color: #0056b3;
  }

  .cancel-button:disabled,
  .send-button:disabled {
      opacity: 0.6;
      cursor: not-allowed;
  }

  /* Compose Attachments Styles */
  .attachment-limit-info {
      font-size: 0.85rem;
      color: #666;
      font-weight: normal;
  }

  .remove-attachment-button {
      background-color: #dc3545;
      color: white;
      border: none;
      border-radius: 50%;
      width: 24px;
      height: 24px;
      font-size: 18px;
      line-height: 1;
      cursor: pointer;
      padding: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: background-color 0.2s;
  }

  .remove-attachment-button:hover:not(:disabled) {
      background-color: #c82333;
  }

  .remove-attachment-button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
  }

  .attachment-total {
      margin-top: 0.5rem;
      padding: 0.5rem;
      background-color: var(--sidebar-bg);
      border-radius: 4px;
      font-weight: 500;
      font-size: 0.9rem;
      text-align: right;
  }

  /* Attachments Styles */
  .attachments-section {
      padding: 1rem 2rem;
      background-color: var(--sidebar-bg);
      border-bottom: 1px solid var(--border-color);
      flex-shrink: 0;
  }

  .attachments-section h3 {
      margin: 0 0 0.75rem 0;
      font-size: 1rem;
      font-weight: 600;
      color: var(--text-color);
  }

  .attachments-list {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
  }

  .attachment-item {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      padding: 0.75rem;
      background-color: var(--app-bg);
      border: 1px solid var(--border-color);
      border-radius: 6px;
      cursor: pointer;
      transition: background-color 0.2s, border-color 0.2s;
      text-align: left;
      font: inherit;
      color: inherit;
      width: 100%;
      position: relative;
      z-index: 1;
  }

  .attachment-item:hover {
      background-color: var(--hover-bg);
      border-color: var(--selected-bg);
  }

  .attachment-item:active {
      transform: scale(0.98);
  }

  .attachment-icon {
      font-size: 1.5rem;
      flex-shrink: 0;
  }

  .attachment-info {
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
      min-width: 0;
  }

  .attachment-name {
      font-weight: 500;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
  }

  .attachment-size {
      font-size: 0.875rem;
      color: #666;
  }

  .download-icon {
      font-size: 1.25rem;
      color: var(--selected-bg);
      flex-shrink: 0;
  }

</style>