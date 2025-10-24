<script lang="ts">
  import { onMount } from "svelte";

  // Props
  let {
    title = "",
    body = "",
    from = "",
    subject = "",
    onClose,
  }: {
    title?: string;
    body?: string;
    from?: string;
    subject?: string;
    onClose: () => void;
  } = $props();

  let visible = $state(false);

  onMount(() => {
    // Fade in
    setTimeout(() => {
      visible = true;
    }, 10);

    // Auto close after 3 seconds
    setTimeout(() => {
      visible = false;
      setTimeout(onClose, 300); // Wait for fade out animation
    }, 3000);
  });
</script>

<div class="toast-container" class:visible>
  <div class="toast">
    <div class="toast-header">
      <span class="toast-icon">📧</span>
      <span class="toast-title">{title}</span>
      <button class="close-button" onclick={onClose} aria-label="关闭">×</button>
    </div>
    <div class="toast-body">
      <div class="toast-from">发件人: {from}</div>
      <div class="toast-subject">主题: {subject}</div>
    </div>
  </div>
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 10000;
    opacity: 0;
    transform: translateY(20px);
    transition: all 0.3s ease-out;
    pointer-events: none;
  }

  .toast-container.visible {
    opacity: 1;
    transform: translateY(0);
    pointer-events: auto;
  }

  .toast {
    min-width: 320px;
    max-width: 400px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    overflow: hidden;
    color: white;
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  .toast-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 1rem 0.75rem 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.2);
  }

  .toast-icon {
    font-size: 1.5rem;
    line-height: 1;
  }

  .toast-title {
    flex: 1;
    font-weight: 600;
    font-size: 1rem;
  }

  .close-button {
    background: none;
    border: none;
    color: white;
    font-size: 1.5rem;
    line-height: 1;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .close-button:hover {
    background-color: rgba(255, 255, 255, 0.2);
  }

  .toast-body {
    padding: 0.75rem 1rem 1rem 1rem;
  }

  .toast-from {
    font-size: 0.9rem;
    margin-bottom: 0.5rem;
    opacity: 0.95;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .toast-subject {
    font-size: 0.85rem;
    opacity: 0.9;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
