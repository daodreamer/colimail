<script lang="ts">
  import type { Folder } from "../lib/types";

  // Props
  let {
    folders = $bindable<Folder[]>([]),
    selectedFolderName = $bindable<string>("INBOX"),
    isLoading = false,
    selectedAccountId = null as number | null,
    onFolderClick,
  }: {
    folders?: Folder[];
    selectedFolderName?: string;
    isLoading?: boolean;
    selectedAccountId?: number | null;
    onFolderClick: (folderName: string) => void;
  } = $props();
</script>

<aside class="folders-sidebar">
  <h2>Folders</h2>
  {#if isLoading}
    <p class="loading-text">Loading folders...</p>
  {:else if folders.length > 0}
    <ul>
      {#each folders as folder (folder.name)}
        <li>
          <button
            class="folder-item"
            class:selected={folder.name === selectedFolderName}
            onclick={() => onFolderClick(folder.name)}
            title={folder.name}
          >
            📁 {folder.display_name}
          </button>
        </li>
      {/each}
    </ul>
  {:else if selectedAccountId}
    <p class="no-folders">No folders found.</p>
  {:else}
    <p class="no-folders">Select an account to view folders.</p>
  {/if}
</aside>

<style>
  .folders-sidebar {
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

  .folder-item {
    background: none;
    border: none;
    font: inherit;
    color: inherit;
    text-align: left;
    width: 100%;
    padding: 0.75rem;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    transition: background-color 0.2s, color 0.2s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .folder-item:hover {
    background-color: var(--hover-bg);
  }

  .folder-item.selected {
    background-color: var(--selected-bg);
    color: var(--selected-text);
  }

  .loading-text,
  .no-folders {
    text-align: center;
    color: #666;
    font-size: 0.875rem;
    padding: 2rem 1rem;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
