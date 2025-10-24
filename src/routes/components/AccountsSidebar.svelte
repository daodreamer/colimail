<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { AccountConfig } from "../lib/types";
  import { formatTimeSince } from "../lib/utils";

  // Props
  let {
    accounts = [] as AccountConfig[],
    selectedAccountId = null as number | null,
    isSyncing = false,
    lastSyncTime = 0,
    currentTime = 0,
    onAccountClick,
    onCompose,
    onRefresh,
    onDeleteAccount,
  }: {
    accounts?: AccountConfig[];
    selectedAccountId?: number | null;
    isSyncing?: boolean;
    lastSyncTime?: number;
    currentTime?: number;
    onAccountClick: (accountId: number) => void;
    onCompose: () => void;
    onRefresh: () => void;
    onDeleteAccount: (email: string, event: MouseEvent) => void;
  } = $props();
</script>

<aside class="accounts-sidebar">
  <h2>Accounts</h2>
  <ul>
    {#each accounts as account (account.id)}
      <li>
        <div class="account-item-wrapper">
          <button
            class="account-item"
            class:selected={account.id === selectedAccountId}
            onclick={() => onAccountClick(account.id)}
          >
            <span class="account-email">{account.email}</span>
            {#await invoke("is_idle_active", { accountId: account.id, folderName: "INBOX" })}
              <span class="status-indicator" title="Checking...">⚪</span>
            {:then isActive}
              {#if isActive}
                <span class="status-indicator status-active" title="Real-time sync active">🟢</span>
              {:else}
                <span class="status-indicator status-inactive" title="Offline">🔴</span>
              {/if}
            {:catch}
              <span class="status-indicator" title="Unknown">⚪</span>
            {/await}
          </button>
          <button
            class="delete-button"
            onclick={(e) => onDeleteAccount(account.email, e)}
            title="删除账户"
            aria-label="删除账户 {account.email}"
          >
            ×
          </button>
        </div>
      </li>
    {/each}
    {#if accounts.length === 0}
      <li class="no-accounts">No accounts configured.</li>
    {/if}
  </ul>

  <button class="compose-button" onclick={onCompose} disabled={!selectedAccountId}>
    ✉️ Compose
  </button>

  <button class="refresh-button" onclick={onRefresh} disabled={!selectedAccountId || isSyncing}>
    {isSyncing ? "🔄 Syncing..." : "🔄 Refresh"}
  </button>

  {#if selectedAccountId && lastSyncTime > 0}
    <div class="sync-status">Last sync: {formatTimeSince(lastSyncTime, currentTime)}</div>
  {/if}

  <a href="/account" class="add-account-link">+ Add Account</a>
  <a href="/settings" class="settings-link">⚙️ Settings</a>
</aside>

<style>
  .accounts-sidebar {
    background-color: var(--sidebar-bg);
    border-right: 1px solid var(--border-color);
    user-select: none;
    padding: 0;
    height: 100vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  h2 {
    margin: 0;
    border-bottom: 1px solid var(--border-color);
    padding: 1rem;
    font-size: 1rem;
    flex-shrink: 0;
  }

  ul {
    list-style: none;
    padding: 0.5rem;
    margin: 0;
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  li {
    margin-bottom: 4px;
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
  }

  .account-item {
    background: none;
    border: none;
    font: inherit;
    color: inherit;
    text-align: left;
    flex: 1;
    padding: 0.75rem;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    transition: background-color 0.2s, color 0.2s;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .account-email {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .status-indicator {
    font-size: 0.75rem;
    flex-shrink: 0;
  }

  .status-active {
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }

  .status-inactive {
    opacity: 0.5;
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
</style>
