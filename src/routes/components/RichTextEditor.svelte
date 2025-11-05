<script lang="ts">
  import { onMount } from "svelte";

  // Props
  let {
    value = $bindable(""),
    disabled = false,
    placeholder = "Write your message here...",
  }: {
    value?: string;
    disabled?: boolean;
    placeholder?: string;
  } = $props();

  let editorElement: HTMLDivElement;
  let currentFontFamily = $state("Arial, sans-serif");
  let currentFontSize = $state("3");

  // Comprehensive font list based on mainstream email clients (Gmail, Outlook, etc.)
  const fontFamilies = [
    // System defaults
    { label: "Sans Serif (Default)", value: "sans-serif", category: "System" },
    { label: "Serif (Default)", value: "serif", category: "System" },
    { label: "Monospace (Default)", value: "monospace", category: "System" },

    // Sans-serif fonts (most popular)
    { label: "Arial", value: "Arial, sans-serif", category: "Sans-serif" },
    { label: "Helvetica", value: "Helvetica, Arial, sans-serif", category: "Sans-serif" },
    { label: "Verdana", value: "Verdana, Geneva, sans-serif", category: "Sans-serif" },
    { label: "Tahoma", value: "Tahoma, Geneva, sans-serif", category: "Sans-serif" },
    { label: "Trebuchet MS", value: "'Trebuchet MS', sans-serif", category: "Sans-serif" },
    { label: "Lucida Sans", value: "'Lucida Sans Unicode', 'Lucida Grande', sans-serif", category: "Sans-serif" },
    { label: "Calibri", value: "Calibri, Candara, Segoe, sans-serif", category: "Sans-serif" },
    { label: "Segoe UI", value: "'Segoe UI', Tahoma, sans-serif", category: "Sans-serif" },
    { label: "Roboto", value: "Roboto, Arial, sans-serif", category: "Sans-serif" },
    { label: "Open Sans", value: "'Open Sans', Arial, sans-serif", category: "Sans-serif" },
    { label: "Lato", value: "Lato, Arial, sans-serif", category: "Sans-serif" },
    { label: "Poppins", value: "Poppins, Arial, sans-serif", category: "Sans-serif" },
    { label: "Raleway", value: "Raleway, Arial, sans-serif", category: "Sans-serif" },
    { label: "Ubuntu", value: "Ubuntu, Arial, sans-serif", category: "Sans-serif" },
    { label: "Rubik", value: "Rubik, Arial, sans-serif", category: "Sans-serif" },
    { label: "Quicksand", value: "Quicksand, Arial, sans-serif", category: "Sans-serif" },
    { label: "Oxygen", value: "Oxygen, Arial, sans-serif", category: "Sans-serif" },
    { label: "Oswald", value: "Oswald, Arial, sans-serif", category: "Sans-serif" },

    // Serif fonts
    { label: "Times New Roman", value: "'Times New Roman', Times, serif", category: "Serif" },
    { label: "Georgia", value: "Georgia, Times, serif", category: "Serif" },
    { label: "Garamond", value: "Garamond, Times, serif", category: "Serif" },
    { label: "Palatino", value: "'Palatino Linotype', Palatino, serif", category: "Serif" },
    { label: "Baskerville", value: "Baskerville, Times, serif", category: "Serif" },
    { label: "Merriweather", value: "Merriweather, Georgia, serif", category: "Serif" },

    // Monospace fonts
    { label: "Courier New", value: "'Courier New', Courier, monospace", category: "Monospace" },
    { label: "Consolas", value: "Consolas, 'Courier New', monospace", category: "Monospace" },
    { label: "Monaco", value: "Monaco, 'Courier New', monospace", category: "Monospace" },
  ];

  // Font sizes: 1=smallest, 3=normal, 7=largest
  const fontSizes = [
    { label: "Small", value: "1" },
    { label: "Normal", value: "3" },
    { label: "Large", value: "5" },
    { label: "Huge", value: "7" },
  ];

  onMount(() => {
    if (editorElement) {
      // Set initial content
      if (value) {
        editorElement.innerHTML = value;
      }

      // Update value when content changes
      const observer = new MutationObserver(() => {
        value = editorElement.innerHTML;
      });

      observer.observe(editorElement, {
        childList: true,
        subtree: true,
        characterData: true,
      });

      return () => observer.disconnect();
    }
  });

  // Watch for external value changes
  $effect(() => {
    if (editorElement && value !== editorElement.innerHTML) {
      const selection = saveSelection();
      editorElement.innerHTML = value;
      restoreSelection(selection);
    }
  });

  function saveSelection() {
    const sel = window.getSelection();
    if (sel && sel.rangeCount > 0) {
      return sel.getRangeAt(0);
    }
    return null;
  }

  function restoreSelection(range: Range | null) {
    if (range) {
      const sel = window.getSelection();
      sel?.removeAllRanges();
      sel?.addRange(range);
    }
  }

  function execCommand(command: string, value: string | boolean = false) {
    document.execCommand(command, false, value as string);
    editorElement.focus();
  }

  function handleFontFamilyChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    currentFontFamily = target.value;
    execCommand("fontName", target.value);
  }

  function handleFontSizeChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    currentFontSize = target.value;
    execCommand("fontSize", target.value);
  }

  function handlePaste(event: ClipboardEvent) {
    event.preventDefault();
    const text = event.clipboardData?.getData("text/plain") || "";
    document.execCommand("insertText", false, text);
  }
</script>

<div class="rich-text-editor" class:disabled>
  <div class="toolbar">
    <!-- Font family selector -->
    <div class="toolbar-group">
      <select
        class="font-family-select"
        value={currentFontFamily}
        onchange={handleFontFamilyChange}
        {disabled}
        aria-label="Font family"
        style="font-family: {currentFontFamily};"
      >
        {#each fontFamilies as font (font.value)}
          <option value={font.value} style="font-family: {font.value};">
            {font.label}
          </option>
        {/each}
      </select>
    </div>

    <!-- Font size selector -->
    <div class="toolbar-group">
      <select
        class="font-size-select"
        value={currentFontSize}
        onchange={handleFontSizeChange}
        {disabled}
        aria-label="Font size"
      >
        {#each fontSizes as size (size.value)}
          <option value={size.value}>{size.label}</option>
        {/each}
      </select>
    </div>

    <div class="toolbar-divider"></div>

    <!-- Text formatting -->
    <div class="toolbar-group">
      <button
        class="toolbar-button"
        onclick={() => execCommand("bold")}
        {disabled}
        title="Bold (Ctrl+B)"
        aria-label="Bold"
      >
        <strong>B</strong>
      </button>
      <button
        class="toolbar-button"
        onclick={() => execCommand("italic")}
        {disabled}
        title="Italic (Ctrl+I)"
        aria-label="Italic"
      >
        <em>I</em>
      </button>
      <button
        class="toolbar-button"
        onclick={() => execCommand("underline")}
        {disabled}
        title="Underline (Ctrl+U)"
        aria-label="Underline"
      >
        <span style="text-decoration: underline;">U</span>
      </button>
    </div>

    <div class="toolbar-divider"></div>

    <!-- Text alignment -->
    <div class="toolbar-group">
      <button
        class="toolbar-button"
        onclick={() => execCommand("justifyLeft")}
        {disabled}
        title="Align left"
        aria-label="Align left"
      >
        ☰
      </button>
      <button
        class="toolbar-button"
        onclick={() => execCommand("justifyCenter")}
        {disabled}
        title="Align center"
        aria-label="Align center"
      >
        ☷
      </button>
      <button
        class="toolbar-button"
        onclick={() => execCommand("justifyRight")}
        {disabled}
        title="Align right"
        aria-label="Align right"
      >
        ≡
      </button>
    </div>

    <div class="toolbar-divider"></div>

    <!-- Lists -->
    <div class="toolbar-group">
      <button
        class="toolbar-button"
        onclick={() => execCommand("insertUnorderedList")}
        {disabled}
        title="Bulleted list"
        aria-label="Bulleted list"
      >
        • •
      </button>
      <button
        class="toolbar-button"
        onclick={() => execCommand("insertOrderedList")}
        {disabled}
        title="Numbered list"
        aria-label="Numbered list"
      >
        1. 2.
      </button>
    </div>

    <div class="toolbar-divider"></div>

    <!-- Indent/Outdent -->
    <div class="toolbar-group">
      <button
        class="toolbar-button"
        onclick={() => execCommand("outdent")}
        {disabled}
        title="Decrease indent"
        aria-label="Decrease indent"
      >
        ◁
      </button>
      <button
        class="toolbar-button"
        onclick={() => execCommand("indent")}
        {disabled}
        title="Increase indent"
        aria-label="Increase indent"
      >
        ▷
      </button>
    </div>
  </div>

  <!-- Editor content area -->
  <div
    bind:this={editorElement}
    class="editor-content"
    contenteditable={!disabled}
    data-placeholder={placeholder}
    onpaste={handlePaste}
    role="textbox"
    aria-multiline="true"
    aria-label="Email body"
  ></div>
</div>

<style>
  .rich-text-editor {
    display: flex;
    flex-direction: column;
    flex: 1;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background-color: var(--app-bg);
    overflow: hidden;
  }

  .rich-text-editor.disabled {
    opacity: 0.6;
    pointer-events: none;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.5rem;
    border-bottom: 1px solid var(--border-color);
    background-color: var(--sidebar-bg);
    flex-wrap: wrap;
  }

  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .toolbar-divider {
    width: 1px;
    height: 24px;
    background-color: var(--border-color);
    margin: 0 0.25rem;
  }

  .font-family-select,
  .font-size-select {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background-color: var(--app-bg);
    color: var(--text-color);
    font-size: 0.875rem;
    cursor: pointer;
    min-width: 140px;
    max-width: 180px;
  }

  .font-size-select {
    min-width: 80px;
    max-width: 100px;
  }

  .font-family-select:hover,
  .font-size-select:hover {
    background-color: var(--hover-bg);
  }

  .font-family-select:focus,
  .font-size-select:focus {
    outline: none;
    border-color: var(--selected-bg);
  }

  /* Style options in the dropdown to show their actual font */
  .font-family-select option {
    padding: 0.5rem;
    font-size: 0.95rem;
  }

  .toolbar-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 1px solid transparent;
    border-radius: 4px;
    background-color: transparent;
    color: var(--text-color);
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .toolbar-button:hover:not(:disabled) {
    background-color: var(--hover-bg);
    border-color: var(--border-color);
  }

  .toolbar-button:active:not(:disabled) {
    background-color: var(--selected-bg);
  }

  .toolbar-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .editor-content {
    min-height: 150px;
    flex: 1;
    padding: 0.75rem;
    overflow-y: auto;
    font-family: sans-serif;
    font-size: 1rem;
    line-height: 1.5;
    color: var(--text-color);
    background-color: var(--app-bg);
  }

  .editor-content:focus {
    outline: none;
  }

  .editor-content[contenteditable="true"]:empty:before {
    content: attr(data-placeholder);
    color: #999;
    pointer-events: none;
  }

  /* Ensure proper styling for formatted content */
  .editor-content :global(p) {
    margin: 0 0 0.5rem 0;
  }

  .editor-content :global(p:last-child) {
    margin-bottom: 0;
  }

  .editor-content :global(ul),
  .editor-content :global(ol) {
    margin: 0.5rem 0;
    padding-left: 2rem;
  }

  .editor-content :global(li) {
    margin: 0.25rem 0;
  }

  .editor-content :global(strong) {
    font-weight: bold;
  }

  .editor-content :global(em) {
    font-style: italic;
  }

  .editor-content :global(u) {
    text-decoration: underline;
  }
</style>
