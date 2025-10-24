<script lang="ts">
  import { formatFileSize } from "../lib/utils";
  import RichTextEditor from "./RichTextEditor.svelte";

  // Props
  let {
    show = false,
    mode = "compose" as "compose" | "reply" | "forward",
    to = $bindable(""),
    cc = $bindable(""),
    subject = $bindable(""),
    body = $bindable(""),
    attachments = $bindable<File[]>([]),
    attachmentSizeLimit = 10 * 1024 * 1024,
    totalAttachmentSize = 0,
    isSending = false,
    error = null as string | null,
    onSend,
    onCancel,
    onAttachmentAdd,
    onAttachmentRemove,
  }: {
    show?: boolean;
    mode?: "compose" | "reply" | "forward";
    to?: string;
    cc?: string;
    subject?: string;
    body?: string;
    attachments?: File[];
    attachmentSizeLimit?: number;
    totalAttachmentSize?: number;
    isSending?: boolean;
    error?: string | null;
    onSend: () => void;
    onCancel: () => void;
    onAttachmentAdd: (event: Event) => void;
    onAttachmentRemove: (index: number) => void;
  } = $props();

  function getModalTitle(): string {
    switch (mode) {
      case "reply":
        return "Reply to Email";
      case "forward":
        return "Forward Email";
      default:
        return "Compose Email";
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      onCancel();
    }
  }
</script>

{#if show}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-overlay" onclick={onCancel} onkeydown={handleKeyDown}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="modal-content"
      onclick={(e) => e.stopPropagation()}
      onkeydown={handleKeyDown}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="modal-header">
        <h2>{getModalTitle()}</h2>
        <button class="close-button" onclick={onCancel}>×</button>
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
            bind:value={to}
            placeholder="recipient@example.com"
            disabled={isSending}
          />
        </div>

        <div class="form-group">
          <label for="compose-cc">CC:</label>
          <input
            type="text"
            id="compose-cc"
            bind:value={cc}
            placeholder="cc@example.com (separate multiple with commas)"
            disabled={isSending}
          />
        </div>

        <div class="form-group">
          <label for="compose-subject">Subject:</label>
          <input
            type="text"
            id="compose-subject"
            bind:value={subject}
            placeholder="Email subject"
            disabled={isSending}
          />
        </div>

        <div class="form-group">
          <label for="compose-body">Body:</label>
          <RichTextEditor
            bind:value={body}
            disabled={isSending}
            placeholder="Write your message here..."
          />
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
            onchange={onAttachmentAdd}
            disabled={isSending}
            style="margin-bottom: 0.5rem;"
          />

          {#if attachments.length > 0}
            <div class="attachments-list">
              {#each attachments as file, index}
                <div class="attachment-item">
                  <span class="attachment-name">{file.name}</span>
                  <span class="attachment-size">({formatFileSize(file.size)})</span>
                  <button
                    class="remove-attachment-button"
                    onclick={() => onAttachmentRemove(index)}
                    disabled={isSending}
                    title="Remove attachment"
                  >
                    ×
                  </button>
                </div>
              {/each}
              <div class="attachment-total">
                Total: {formatFileSize(totalAttachmentSize)} / {formatFileSize(
                  attachmentSizeLimit
                )}
              </div>
            </div>
          {/if}
        </div>
      </div>

      <div class="modal-footer">
        <button class="cancel-button" onclick={onCancel} disabled={isSending}> Cancel </button>
        <button class="send-button" onclick={onSend} disabled={isSending}>
          {isSending ? "Sending..." : "Send"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
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

  .form-group input {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-family: inherit;
    font-size: 1rem;
    background-color: var(--app-bg);
    color: var(--text-color);
  }

  .form-group input:focus {
    outline: none;
    border-color: var(--selected-bg);
  }

  .attachment-limit-info {
    font-size: 0.85rem;
    color: #666;
    font-weight: normal;
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
    background-color: var(--sidebar-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
  }

  .attachment-name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .attachment-size {
    font-size: 0.875rem;
    color: #666;
    flex-shrink: 0;
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
    flex-shrink: 0;
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
