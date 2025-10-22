<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let syncInterval = $state<number>(300); // Default 5 minutes
  let isSavingSyncSettings = $state(false);

  // Load sync interval on mount
  onMount(async () => {
    try {
      syncInterval = await invoke<number>("get_sync_interval");
    } catch (error) {
      console.error("Failed to load sync interval:", error);
    }
  });

  // Save sync interval setting
  async function saveSyncSettings() {
    isSavingSyncSettings = true;
    try {
      await invoke("set_sync_interval", { interval: syncInterval });
      alert("同步设置已保存！");
    } catch (error) {
      console.error("保存同步设置失败:", error);
      alert("保存同步设置失败！");
    } finally {
      isSavingSyncSettings = false;
    }
  }

  // Get interval description
  function getIntervalDescription(interval: number): string {
    if (interval === -1) return "从不同步（仅使用缓存）";
    if (interval === 0) return "手动同步（仅点击刷新时）";
    if (interval < 60) return `${interval} 秒`;
    if (interval < 3600) return `${Math.floor(interval / 60)} 分钟`;
    return `${Math.floor(interval / 3600)} 小时`;
  }
</script>

<div class="container">
  <div class="header">
    <a href="/" class="back-button">← 返回</a>
    <h1>应用设置</h1>
  </div>

  <!-- Sync Settings Section -->
  <div class="sync-settings-section">
    <h2>同步设置</h2>
    <p class="info-text">
      配置邮件同步频率，以平衡实时性和资源消耗。
    </p>

    <div class="form-group">
      <label for="sync-interval">同步间隔</label>
      <select id="sync-interval" bind:value={syncInterval}>
        <option value={0}>手动（仅点击刷新时同步）</option>
        <option value={60}>1 分钟</option>
        <option value={180}>3 分钟</option>
        <option value={300}>5 分钟（推荐）</option>
        <option value={600}>10 分钟</option>
        <option value={900}>15 分钟</option>
        <option value={1800}>30 分钟</option>
        <option value={-1}>从不（仅使用缓存）</option>
      </select>
      <p class="help-text">
        当前设置: <strong>{getIntervalDescription(syncInterval)}</strong>
      </p>
      <p class="help-text">
        {#if syncInterval === 0}
          邮件和文件夹仅在点击"刷新"按钮时同步。
        {:else if syncInterval === -1}
          邮件和文件夹将不会自动同步，仅显示缓存内容。
        {:else}
          当距离上次同步超过 {getIntervalDescription(syncInterval)} 时，切换文件夹或账户将自动同步。
        {/if}
      </p>
    </div>

    <button
      class="primary-button"
      onclick={saveSyncSettings}
      disabled={isSavingSyncSettings}
      type="button"
    >
      {isSavingSyncSettings ? "保存中..." : "保存设置"}
    </button>
  </div>
</div>

<style>
  .container {
    padding: 2rem;
    max-width: 700px;
    margin: 0 auto;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .back-button {
    padding: 0.5rem 1rem;
    background-color: #6c757d;
    color: white;
    text-decoration: none;
    border-radius: 6px;
    font-weight: 500;
    transition: background-color 0.2s;
  }

  .back-button:hover {
    background-color: #5a6268;
  }

  h1 {
    margin: 0;
    color: #333;
  }

  h2 {
    font-size: 1.3rem;
    margin-bottom: 0.5rem;
    color: #444;
  }

  .info-text {
    color: #666;
    font-size: 0.9rem;
    margin-bottom: 1.5rem;
  }

  /* Sync Settings Section */
  .sync-settings-section {
    animation: fadeIn 0.3s;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* Form Elements */
  .form-group {
    margin-bottom: 1.5rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #333;
  }

  select {
    width: 100%;
    padding: 0.75rem;
    font-size: 1rem;
    border-radius: 6px;
    border: 1px solid #ccc;
    box-sizing: border-box;
    background: white;
    cursor: pointer;
    transition: border-color 0.2s;
  }

  select:focus {
    outline: none;
    border-color: #007bff;
  }

  .help-text {
    margin-top: 0.5rem;
    font-size: 0.85rem;
    color: #666;
    line-height: 1.5;
  }

  .help-text strong {
    color: #007bff;
  }

  /* Buttons */
  .primary-button {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    transition: all 0.2s;
    font-weight: 500;
    background-color: #007bff;
    color: white;
    width: 100%;
  }

  .primary-button:hover:not(:disabled) {
    background-color: #0056b3;
  }

  .primary-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }
</style>
