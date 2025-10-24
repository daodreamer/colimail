<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let syncInterval = $state<number>(300); // Default 5 minutes
  let isSavingSyncSettings = $state(false);
  let notificationEnabled = $state<boolean>(true);
  let soundEnabled = $state<boolean>(true);
  let isSavingNotificationSettings = $state(false);

  // Load settings on mount
  onMount(async () => {
    try {
      syncInterval = await invoke<number>("get_sync_interval");
    } catch (error) {
      console.error("Failed to load sync interval:", error);
    }

    try {
      notificationEnabled = await invoke<boolean>("get_notification_enabled");
    } catch (error) {
      console.error("Failed to load notification setting:", error);
    }

    try {
      soundEnabled = await invoke<boolean>("get_sound_enabled");
    } catch (error) {
      console.error("Failed to load sound setting:", error);
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

  // Save notification settings
  async function saveNotificationSettings() {
    isSavingNotificationSettings = true;
    try {
      await invoke("set_notification_enabled", { enabled: notificationEnabled });
      await invoke("set_sound_enabled", { enabled: soundEnabled });
      alert("通知设置已保存！");
    } catch (error) {
      console.error("保存通知设置失败:", error);
      alert("保存通知设置失败！");
    } finally {
      isSavingNotificationSettings = false;
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

  <!-- Notification Settings Section -->
  <div class="notification-settings-section">
    <h2>通知设置</h2>
    <p class="info-text">
      配置新邮件到达时的提醒方式。
    </p>

    <div class="form-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={notificationEnabled} />
        <span>启用桌面通知</span>
      </label>
      <p class="help-text">
        收到新邮件时在桌面右下角显示浮窗提醒，包含发件人和标题信息。
      </p>
    </div>

    <div class="form-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={soundEnabled} />
        <span>启用声音提醒</span>
      </label>
      <p class="help-text">
        收到新邮件时播放提示音。
      </p>
    </div>

    <button
      class="primary-button"
      onclick={saveNotificationSettings}
      disabled={isSavingNotificationSettings}
      type="button"
    >
      {isSavingNotificationSettings ? "保存中..." : "保存设置"}
    </button>

    <p class="help-text" style="margin-top: 1rem; font-size: 0.85rem; color: #666;">
      💡 <strong>提示:</strong> 启用桌面通知后,收到新邮件时会在屏幕右下角显示浮窗提醒,3秒后自动消失。
    </p>
  </div>
</div>

<style>
  :global(html),
  :global(body) {
    overflow: auto !important;
    position: static !important;
    height: auto !important;
  }

  .container {
    padding: 2rem;
    max-width: 700px;
    margin: 0 auto;
    min-height: 100vh;
    overflow-y: auto;
    box-sizing: border-box;
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
    margin-bottom: 2rem;
  }

  /* Notification Settings Section */
  .notification-settings-section {
    animation: fadeIn 0.3s;
    border-top: 1px solid #e0e0e0;
    padding-top: 2rem;
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

  /* Checkbox styles */
  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
    font-weight: 500;
    color: #333;
    margin-bottom: 0;
  }

  input[type="checkbox"] {
    width: 20px;
    height: 20px;
    cursor: pointer;
    accent-color: #007bff;
  }

  .checkbox-label span {
    user-select: none;
  }
</style>
