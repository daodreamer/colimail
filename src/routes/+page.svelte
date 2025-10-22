<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

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
  }

  interface Folder {
    id: number | null;
    account_id: number;
    name: string;              // Original IMAP folder name (for operations)
    display_name: string;      // User-friendly display name
    delimiter: string | null;
    flags: string | null;
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

  // Compose email state
  let showComposeDialog = $state<boolean>(false);
  let composeTo = $state<string>("");
  let composeSubject = $state<string>("");
  let composeBody = $state<string>("");
  let isSending = $state<boolean>(false);
  let isReplyMode = $state<boolean>(false);

  // --- 生命周期 ---
  onMount(async () => {
    try {
      accounts = await invoke<AccountConfig[]>("load_account_configs");
    } catch (e) {
      error = `Failed to load accounts: ${e}`;
    }
  });

  // --- 事件处理 ---
  async function handleAccountClick(accountId: number) {
    selectedAccountId = accountId;
    selectedFolderName = "INBOX";
    selectedEmailUid = null;
    emailBody = null;
    emails = [];
    folders = [];
    isLoadingFolders = true;
    error = null;

    const selectedConfig = accounts.find(acc => acc.id === accountId);
    if (!selectedConfig) {
        error = "Could not find selected account configuration.";
        isLoadingFolders = false;
        return;
    }

    try {
      // First sync folders from server
      folders = await invoke<Folder[]>("sync_folders", { config: selectedConfig });

      // Then load emails from INBOX
      await loadEmailsForFolder("INBOX");
    } catch (e) {
      error = `Failed to sync folders: ${e}`;
    } finally {
      isLoadingFolders = false;
    }
  }

  async function loadEmailsForFolder(folderName: string) {
    const selectedConfig = accounts.find(acc => acc.id === selectedAccountId);
    if (!selectedConfig) {
        error = "Could not find selected account configuration.";
        return;
    }

    isLoadingEmails = true;
    selectedEmailUid = null;
    emailBody = null;
    emails = [];
    error = null;

    try {
      emails = await invoke<EmailHeader[]>("fetch_emails", {
        config: selectedConfig,
        folder: folderName
      });
    } catch (e) {
      error = `Failed to fetch emails: ${e}`;
    } finally {
      isLoadingEmails = false;
    }
  }

  async function handleFolderClick(folderName: string) {
    selectedFolderName = folderName;
    await loadEmailsForFolder(folderName);
  }

  async function handleEmailClick(uid: number) {
      selectedEmailUid = uid;
      isLoadingBody = true;
      emailBody = null;
      error = null;

      const selectedConfig = accounts.find(acc => acc.id === selectedAccountId);
      if (!selectedConfig) {
          error = "Could not find selected account configuration.";
          isLoadingBody = false;
          return;
      }

      try {
          emailBody = await invoke<string>("fetch_email_body", {
            config: selectedConfig,
            uid,
            folder: selectedFolderName
          });
      } catch (e) {
          error = `Failed to fetch email body: ${e}`;
      } finally {
          isLoadingBody = false;
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

  function handleComposeClick() {
      if (!selectedAccountId) {
          error = "Please select an account first.";
          return;
      }
      showComposeDialog = true;
      isReplyMode = false;
      composeTo = "";
      composeSubject = "";
      composeBody = "";
      error = null;
  }

  function handleReplyClick() {
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
      composeTo = selectedEmail.from;
      composeSubject = selectedEmail.subject.toLowerCase().startsWith("re:")
          ? selectedEmail.subject
          : `Re: ${selectedEmail.subject}`;
      composeBody = "";
      error = null;
  }

  function handleCloseCompose() {
      showComposeDialog = false;
      composeTo = "";
      composeSubject = "";
      composeBody = "";
      error = null;
  }

  async function handleSendEmail() {
      if (!selectedAccountId) {
          error = "Please select an account first.";
          return;
      }

      if (!composeTo || !composeSubject || !composeBody) {
          error = "Please fill in all fields.";
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
          let result: string;
          if (isReplyMode) {
              result = await invoke<string>("reply_email", {
                  config: selectedConfig,
                  to: composeTo,
                  originalSubject: composeSubject,
                  body: composeBody
              });
          } else {
              result = await invoke<string>("send_email", {
                  config: selectedConfig,
                  to: composeTo,
                  subject: composeSubject,
                  body: composeBody
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
    <a href="/settings" class="settings-link">+ Add Account</a>
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
                        <div class="from">{email.from}</div>
                        <div class="subject">{email.subject}</div>
                        <div class="date">{email.date}</div>
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
    {:else if emailBody}
        <div class="email-header-actions">
            <button class="reply-button" onclick={handleReplyClick}>
                ↩ Reply
            </button>
        </div>
        <div class="email-body">
            {@html emailBody}
        </div>
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
        <h2>{isReplyMode ? "Reply to Email" : "Compose Email"}</h2>
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

  .settings-link {
      display: block;
      text-align: center;
      padding: 0.75rem;
      margin: 0 1rem 1rem 1rem;
      border-radius: 6px;
      background-color: var(--link-bg);
      color: var(--link-text);
      text-decoration: none;
      font-weight: 500;
      flex-shrink: 0;
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
  }

  .email-item:hover {
      background-color: var(--sidebar-bg);
  }

  .email-item.selected {
      border-left: 4px solid var(--selected-bg);
      background-color: var(--sidebar-bg);
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
  }

  .content-pane {
      padding: 0;
  }

  .email-header-actions {
      padding: 1rem 2rem;
      border-bottom: 1px solid var(--border-color);
      background-color: var(--sidebar-bg);
      flex-shrink: 0; /* Fixed header */
  }

  .reply-button {
      background-color: #007bff;
      color: white;
      border: none;
      padding: 0.5rem 1rem;
      border-radius: 4px;
      cursor: pointer;
      font-weight: 500;
      transition: background-color 0.2s;
  }

  .reply-button:hover {
      background-color: #0056b3;
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

</style>