<script lang="ts">
  import type { EmailHeader, AttachmentInfo } from "../lib/types";
  import { formatFullLocalDateTime } from "../lib/utils";
  import AttachmentList from "./AttachmentList.svelte";

  // Props
  let {
    email = null as EmailHeader | null,
    body = null as string | null,
    attachments = [] as AttachmentInfo[],
    isLoadingBody = false,
    isLoadingAttachments = false,
    error = null as string | null,
    onReply,
    onForward,
    onDelete,
    onDownloadAttachment,
  }: {
    email?: EmailHeader | null;
    body?: string | null;
    attachments?: AttachmentInfo[];
    isLoadingBody?: boolean;
    isLoadingAttachments?: boolean;
    error?: string | null;
    onReply: () => void;
    onForward: () => void;
    onDelete: () => void;
    onDownloadAttachment: (attachmentId: number, filename: string) => void;
  } = $props();
</script>

<main class="content-pane">
  {#if isLoadingBody}
    <p>Loading email content...</p>
  {:else if email && body}
    <div class="email-header-section">
      <h2 class="email-subject">{email.subject}</h2>
      <div class="email-meta">
        <div class="meta-row">
          <span class="meta-label">From:</span>
          <span class="meta-value">{email.from}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">To:</span>
          <span class="meta-value">{email.to}</span>
        </div>
        {#if email.cc && email.cc.trim()}
          <div class="meta-row">
            <span class="meta-label">CC:</span>
            <span class="meta-value">{email.cc}</span>
          </div>
        {/if}
        <div class="meta-row">
          <span class="meta-label">Date:</span>
          <span class="meta-value">{formatFullLocalDateTime(email.timestamp)}</span>
        </div>
      </div>
      <div class="email-actions">
        <button class="action-button reply-button" onclick={onReply}> ↩ Reply </button>
        <button class="action-button forward-button" onclick={onForward}> ➡ Forward </button>
        <button class="action-button delete-email-button" onclick={onDelete}> 🗑 Delete </button>
      </div>
    </div>

    <AttachmentList
      {attachments}
      isLoading={isLoadingAttachments}
      onDownload={onDownloadAttachment}
    />

    <div class="email-body">
      {@html body}
    </div>
  {:else if error}
    <div class="error-container">
      <p class="error-message">⚠️ Error loading email</p>
      <p class="error-details">{error}</p>
      <p class="error-hint">The email may have been deleted or moved. Please try refreshing the folder.</p>
    </div>
  {:else if email && !body}
    <div class="placeholder">
      <p>Email selected but content not loaded yet...</p>
      <p class="hint">If this persists, try selecting another email.</p>
    </div>
  {:else}
    <div class="placeholder">
      <p>Select an email to read its content.</p>
    </div>
  {/if}
</main>

<style>
  .content-pane {
    padding: 0;
    height: 100vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .content-pane > p {
    text-align: center;
    padding: 4rem 2rem;
    color: #666;
  }

  .email-header-section {
    padding: 1.5rem 2rem;
    border-bottom: 1px solid var(--border-color);
    background-color: var(--sidebar-bg);
    flex-shrink: 0;
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
    overflow-y: auto;
    min-height: 0;
  }

  .placeholder {
    text-align: center;
    padding: 4rem 2rem;
    color: #666;
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  .placeholder .hint {
    font-size: 0.875rem;
    color: #999;
    margin-top: 0.5rem;
  }

  .error-container {
    text-align: center;
    padding: 4rem 2rem;
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  .error-message {
    color: #dc3545;
    font-size: 1.25rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .error-details {
    color: #666;
    margin-bottom: 0.5rem;
  }

  .error-hint {
    color: #999;
    font-size: 0.875rem;
    font-style: italic;
  }
</style>
