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
    date: string;
  }

  // --- 状态管理 ---
  let accounts = $state<AccountConfig[]>([]);
  let emails = $state<EmailHeader[]>([]);
  let emailBody = $state<string | null>(null);
  let error = $state<string | null>(null);
  let selectedAccountId = $state<number | null>(null);
  let selectedEmailUid = $state<number | null>(null);
  let isLoadingEmails = $state<boolean>(false);
  let isLoadingBody = $state<boolean>(false);

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
    selectedEmailUid = null;
    emailBody = null;
    emails = [];
    isLoadingEmails = true;
    error = null;

    const selectedConfig = accounts.find(acc => acc.id === accountId);
    if (!selectedConfig) {
        error = "Could not find selected account configuration.";
        isLoadingEmails = false;
        return;
    }

    try {
      emails = await invoke<EmailHeader[]>("fetch_emails", { config: selectedConfig });
    } catch (e) {
      error = `Failed to fetch emails: ${e}`;
    } finally {
      isLoadingEmails = false;
    }
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
          emailBody = await invoke<string>("fetch_email_body", { config: selectedConfig, uid });
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

</script>

<div class="main-layout">
  <!-- ACOUNTS SIDEBAR -->
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
    <a href="/settings" class="settings-link">+ Add Account</a>
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

<style>
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
    grid-template-columns: 240px 320px 1fr;
    height: 100vh;
    width: 100vw;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    background-color: var(--app-bg);
    color: var(--text-color);
  }

  .sidebar, .email-list-pane, .content-pane {
      height: 100vh;
      overflow-y: auto;
      padding: 1rem;
  }

  .accounts-sidebar {
    background-color: var(--sidebar-bg);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    user-select: none;
    padding: 1rem;
  }

  .accounts-sidebar h2 {
    margin-top: 0;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 0.5rem;
  }

  .accounts-sidebar ul {
    list-style: none;
    padding: 0;
    margin: 0;
    flex-grow: 1;
  }
  
  .accounts-sidebar li {
      margin-bottom: 4px;
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

  .settings-link {
      display: block;
      text-align: center;
      padding: 0.75rem;
      border-radius: 6px;
      background-color: var(--link-bg);
      color: var(--link-text);
      text-decoration: none;
      font-weight: 500;
      margin-top: 1rem;
      flex-shrink: 0;
  }

  .email-list-pane {
      border-right: 1px solid var(--border-color);
  }

  .email-list {
      list-style: none;
      padding: 0;
      margin: 0;
      text-align: left;
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

  .email-body {
      padding: 1rem 2rem;
      line-height: 1.6;
  }

  .placeholder, .content-pane > p {
      text-align: center;
      margin-top: 4rem;
      color: #666;
  }

  .error-message {
    color: #d9534f;
    text-align: center;
    padding: 2rem;
  }

</style>