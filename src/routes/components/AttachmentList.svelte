<script lang="ts">
  import type { AttachmentInfo } from "../lib/types";
  import { formatFileSize } from "../lib/utils";

  // Props
  let {
    attachments = [] as AttachmentInfo[],
    isLoading = false,
    onDownload,
  }: {
    attachments?: AttachmentInfo[];
    isLoading?: boolean;
    onDownload: (attachmentId: number, filename: string) => void;
  } = $props();
</script>

{#if isLoading}
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
          onclick={() => onDownload(attachment.id, attachment.filename)}
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

<style>
  .attachments-section {
    padding: 1rem 2rem;
    background-color: var(--sidebar-bg);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  h3 {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-color);
  }

  .loading-text {
    color: #666;
    font-size: 0.875rem;
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
