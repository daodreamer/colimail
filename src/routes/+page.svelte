<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // 与 Rust 后端匹配的账户配置类型
  interface AccountConfig {
    id: number;
    email: string;
    // 其他字段暂时不需要
  }

  let accounts = $state<AccountConfig[]>([]);
  let error = $state<string | null>(null);

  // 组件加载时，从后端获取账户列表
  onMount(async () => {
    try {
      const loadedAccounts = await invoke<AccountConfig[]>("load_account_configs");
      accounts = loadedAccounts;
      console.log("Accounts loaded into UI:", loadedAccounts);
    } catch (e) {
      error = `Failed to load accounts: ${e}`;
      console.error(error);
    }
  });

</script>

<div class="main-layout">
  <aside class="sidebar">
    <h2>Accounts</h2>
    {#if error}
      <p class="error-message">{error}</p>
    {/if}
    <ul>
      {#each accounts as account (account.id)}
        <li class="account-item">{account.email}</li>
      {/each}
      {#if accounts.length === 0 && !error}
        <li class="no-accounts">No accounts configured.</li>
      {/if}
    </ul>
    <a href="/settings" class="settings-link">+ Add Account</a>
  </aside>

  <main class="content-pane">
    <h1>Welcome to Mail Desk</h1>
    <p>Select an account from the sidebar to view emails.</p>
  </main>
</div>

<style>
  .main-layout {
    display: flex;
    height: 100vh;
    width: 100vw;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    background-color: #f6f6f6;
    color: #0f0f0f;
  }

  .sidebar {
    width: 280px;
    background-color: #e8e8e8;
    padding: 1rem;
    border-right: 1px solid #dcdcdc;
    display: flex;
    flex-direction: column;
  }

  .sidebar h2 {
    margin-top: 0;
    border-bottom: 1px solid #dcdcdc;
    padding-bottom: 0.5rem;
  }

  .sidebar ul {
    list-style: none;
    padding: 0;
    margin: 0;
    flex-grow: 1;
  }

  .account-item {
    padding: 0.75rem;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    transition: background-color 0.2s;
  }

  .account-item:hover {
    background-color: #dcdcdc;
  }

  .no-accounts {
    padding: 0.75rem;
    color: #666;
  }

  .settings-link {
      display: block;
      text-align: center;
      padding: 0.75rem;
      border-radius: 6px;
      background-color: #007bff;
      color: white;
      text-decoration: none;
      font-weight: 500;
      margin-top: 1rem;
  }

  .settings-link:hover {
      background-color: #0056b3;
  }

  .content-pane {
    flex-grow: 1;
    padding: 2rem;
    text-align: center;
  }

  .error-message {
    color: #d9534f;
  }

  /* Dark mode styles */
  @media (prefers-color-scheme: dark) {
    .main-layout {
      background-color: #2f2f2f;
      color: #f6f6f6;
    }
    .sidebar {
      background-color: #252525;
      border-right: 1px solid #3a3a3a;
    }
    .sidebar h2 {
      border-bottom: 1px solid #3a3a3a;
    }
    .account-item:hover {
      background-color: #3a3a3a;
    }
    .settings-link {
        background-color: #24c8db;
    }
    .settings-link:hover {
        background-color: #1c9aa8;
    }
  }
</style>