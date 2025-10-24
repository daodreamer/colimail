<script lang="ts">
  import type { EmailHeader } from "../lib/types";
  import { formatLocalDateTime } from "../lib/utils";

  // Props
  let {
    emails = [] as EmailHeader[],
    selectedEmailUid = null as number | null,
    isLoading = false,
    error = null as string | null,
    selectedAccountId = null as number | null,
    onEmailClick,
  }: {
    emails?: EmailHeader[];
    selectedEmailUid?: number | null;
    isLoading?: boolean;
    error?: string | null;
    selectedAccountId?: number | null;
    onEmailClick: (uid: number) => void;
  } = $props();
</script>

<div class="email-list-pane">
  {#if isLoading}
    <p>Loading emails...</p>
  {:else if error && emails.length === 0}
    <p class="error-message">{error}</p>
  {:else if emails.length > 0}
    <ul class="email-list">
      {#each emails as email (email.uid)}
        <li>
          <button
            class="email-item"
            class:selected={email.uid === selectedEmailUid}
            onclick={() => onEmailClick(email.uid)}
          >
            <div class="email-item-content">
              {#if email.has_attachments}
                <span class="attachment-indicator" title="This email has attachments">📎</span>
              {/if}
              <div class="email-text">
                <div class="from">{email.from}</div>
                <div class="subject">{email.subject}</div>
              </div>
            </div>
            <div class="date">{formatLocalDateTime(email.timestamp)}</div>
          </button>
        </li>
      {/each}
    </ul>
  {:else if selectedAccountId}
    <p>No emails found in this inbox.</p>
  {/if}
</div>

<style>
  .email-list-pane {
    border-right: 1px solid var(--border-color);
    padding: 0;
    height: 100vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
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
    overflow-y: auto;
    min-height: 0;
  }

  .email-list li {
    margin-bottom: 4px;
  }

  .email-item {
    background: none;
    border: none;
    font: inherit;
    text-align: left;
    width: 100%;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 0.75rem;
    cursor: pointer;
    transition: background-color 0.2s;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.5rem;
  }

  .email-item:hover {
    background-color: var(--sidebar-bg);
  }

  .email-item.selected {
    border-left: 4px solid var(--selected-bg);
    background-color: var(--sidebar-bg);
  }

  .email-item-content {
    flex: 1;
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    min-width: 0;
  }

  .attachment-indicator {
    font-size: 1rem;
    flex-shrink: 0;
    margin-top: 2px;
    opacity: 0.7;
  }

  .email-text {
    flex: 1;
    min-width: 0;
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
    flex-shrink: 0;
    margin-top: 2px;
  }

  .error-message {
    color: #d9534f;
    text-align: center;
    padding: 2rem;
  }
</style>
