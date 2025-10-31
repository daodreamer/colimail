<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let logFiles = $state<string[]>([]);
  let selectedLogFile = $state<string>('');
  let logContent = $state<string>('');
  let isLoading = $state<boolean>(false);
  let error = $state<string>('');
  let logDirectory = $state<string>('');

  onMount(async () => {
    await loadLogFiles();
    await loadLogDirectory();
  });

  async function loadLogDirectory() {
    try {
      logDirectory = await invoke<string>('get_log_directory');
    } catch (e) {
      console.error('Failed to get log directory:', e);
    }
  }

  async function loadLogFiles() {
    isLoading = true;
    error = '';
    try {
      logFiles = await invoke<string[]>('list_log_files');
      if (logFiles.length > 0 && !selectedLogFile) {
        selectedLogFile = logFiles[0];
        await loadLogContent();
      }
    } catch (e) {
      error = `Failed to load log files: ${e}`;
    } finally {
      isLoading = false;
    }
  }

  async function loadLogContent() {
    if (!selectedLogFile) return;

    isLoading = true;
    error = '';
    try {
      logContent = await invoke<string>('read_log_file', {
        filename: selectedLogFile
      });
    } catch (e) {
      error = `Failed to read log file: ${e}`;
      logContent = '';
    } finally {
      isLoading = false;
    }
  }

  async function loadRecentLogs() {
    isLoading = true;
    error = '';
    try {
      logContent = await invoke<string>('read_recent_logs', { lines: 100 });
      selectedLogFile = 'Recent 100 lines';
    } catch (e) {
      error = `Failed to read recent logs: ${e}`;
      logContent = '';
    } finally {
      isLoading = false;
    }
  }

  function copyToClipboard() {
    navigator.clipboard.writeText(logContent);
  }

  function openLogFolder() {
    if (logDirectory) {
      invoke('shell:open', { path: logDirectory });
    }
  }
</script>

<div class="log-viewer">
  <div class="header">
    <h2>Application Logs</h2>
    <div class="actions">
      <button onclick={loadRecentLogs}>Recent Logs</button>
      <button onclick={loadLogFiles}>Refresh</button>
      {#if logDirectory}
        <button onclick={openLogFolder}>Open Log Folder</button>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="controls">
    <label>
      Select log file:
      <select bind:value={selectedLogFile} onchange={loadLogContent}>
        {#each logFiles as file}
          <option value={file}>{file}</option>
        {/each}
      </select>
    </label>
    <span class="log-location">Location: {logDirectory}</span>
  </div>

  <div class="log-content-wrapper">
    {#if isLoading}
      <div class="loading">Loading logs...</div>
    {:else}
      <pre class="log-content">{logContent || 'No logs available'}</pre>
    {/if}
  </div>

  <div class="footer">
    <button onclick={copyToClipboard} disabled={!logContent}>
      Copy to Clipboard
    </button>
    <span class="info">
      Logs are automatically rotated daily and kept for 7 days
    </span>
  </div>
</div>

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 1rem;
    gap: 1rem;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .header h2 {
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .controls {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .controls label {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .log-location {
    font-size: 0.85rem;
    color: #666;
  }

  .error {
    padding: 0.75rem;
    background-color: #fee;
    color: #c00;
    border: 1px solid #fcc;
    border-radius: 4px;
  }

  .log-content-wrapper {
    flex: 1;
    overflow: auto;
    border: 1px solid #ddd;
    border-radius: 4px;
    background-color: #f9f9f9;
  }

  .log-content {
    margin: 0;
    padding: 1rem;
    font-family: 'Courier New', monospace;
    font-size: 0.85rem;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    color: #666;
  }

  .footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .info {
    font-size: 0.85rem;
    color: #666;
  }

  button {
    padding: 0.5rem 1rem;
    background-color: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  button:hover {
    background-color: #0056b3;
  }

  button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }

  select {
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    background-color: white;
  }
</style>
