<script lang="ts">
  import { onMount } from "svelte";
  import BoldIcon from "@lucide/svelte/icons/bold";
  import ItalicIcon from "@lucide/svelte/icons/italic";
  import UnderlineIcon from "@lucide/svelte/icons/underline";
  import TextAlignStartIcon from "@lucide/svelte/icons/text-align-start";
  import TextAlignCenterIcon from "@lucide/svelte/icons/text-align-center";
  import TextAlignEndIcon from "@lucide/svelte/icons/text-align-end";
  import ListIcon from "@lucide/svelte/icons/list";
  import ListOrderedIcon from "@lucide/svelte/icons/list-ordered";
  import IndentDecreaseIcon from "@lucide/svelte/icons/indent-decrease";
  import IndentIncreaseIcon from "@lucide/svelte/icons/indent-increase";
  import PaletteIcon from "@lucide/svelte/icons/palette";
  import HighlighterIcon from "@lucide/svelte/icons/highlighter";

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
  let currentTextColor = $state("#000000");
  let currentHighlightColor = $state("#ffffff");
  let lastSelection = $state<Range | null>(null);
  let formattingState = $state({
    bold: false,
    italic: false,
    underline: false,
    alignStart: false,
    alignCenter: false,
    alignEnd: false,
    unorderedList: false,
    orderedList: false,
  });

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
    if (!editorElement) {
      return;
    }

    if (value) {
      editorElement.innerHTML = value;
    }

    const observer = new MutationObserver(() => {
      value = editorElement.innerHTML;
    });

    observer.observe(editorElement, {
      childList: true,
      subtree: true,
      characterData: true,
    });

    const selectionHandler = () => updateFormattingState();

    document.addEventListener("selectionchange", selectionHandler);
    editorElement.addEventListener("keyup", updateFormattingState);
    editorElement.addEventListener("mouseup", updateFormattingState);

    updateFormattingState();

    return () => {
      observer.disconnect();
      document.removeEventListener("selectionchange", selectionHandler);
      editorElement.removeEventListener("keyup", updateFormattingState);
      editorElement.removeEventListener("mouseup", updateFormattingState);
    };
  });

  // Watch for external value changes
  $effect(() => {
    if (editorElement && value !== editorElement.innerHTML) {
      const selection = saveSelection();
      editorElement.innerHTML = value;
      restoreSelection(selection);
      updateFormattingState();
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
    if (!range) return;
    const sel = window.getSelection();
    sel?.removeAllRanges();
    sel?.addRange(range);
  }

  function execCommand(command: string, value: string | boolean = false) {
    restoreSelection(lastSelection);
    document.execCommand(command, false, value as string);
    editorElement.focus();
    updateFormattingState();
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

  function handleTextColorChange(event: Event) {
    const target = event.target as HTMLInputElement;
    currentTextColor = target.value;
    execCommand("foreColor", target.value);
  }

  function handleHighlightColorChange(event: Event) {
    const target = event.target as HTMLInputElement;
    currentHighlightColor = target.value;
    restoreSelection(lastSelection);
    if (!document.execCommand("hiliteColor", false, target.value)) {
      document.execCommand("backColor", false, target.value);
    }
    editorElement.focus();
    updateFormattingState();
  }

  function cacheSelection() {
    const selection = saveSelection();
    if (!selection || !editorElement) return;
    if (isNodeInsideEditor(selection.commonAncestorContainer)) {
      lastSelection = selection;
    }
  }

  function updateFormattingState() {
    if (!editorElement) return;
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) return;
    const range = sel.getRangeAt(0);
    if (!isNodeInsideEditor(range.commonAncestorContainer)) return;

    lastSelection = range;

    formattingState = {
      bold: document.queryCommandState("bold"),
      italic: document.queryCommandState("italic"),
      underline: document.queryCommandState("underline"),
      alignStart: document.queryCommandState("justifyLeft"),
      alignCenter: document.queryCommandState("justifyCenter"),
      alignEnd: document.queryCommandState("justifyRight"),
      unorderedList: document.queryCommandState("insertUnorderedList"),
      orderedList: document.queryCommandState("insertOrderedList"),
    };

    const fontName = document.queryCommandValue("fontName");
    if (fontName) {
      currentFontFamily = normalizeFontName(fontName);
    }

    const fontSize = document.queryCommandValue("fontSize");
    if (fontSize) {
      currentFontSize = fontSize;
    }

    const foreColor = document.queryCommandValue("foreColor");
    if (foreColor) {
      currentTextColor = normalizeColor(foreColor);
    } else {
      currentTextColor = "#000000";
    }

    const highlightColor = document.queryCommandValue("hiliteColor") || document.queryCommandValue("backColor");
    if (highlightColor) {
      currentHighlightColor = normalizeColor(highlightColor, "#ffffff");
    } else {
      currentHighlightColor = "#ffffff";
    }
  }

  function isNodeInsideEditor(node: Node) {
    if (!editorElement) return false;
    const elementNode = node instanceof Element ? node : node.parentElement ?? node;
    return editorElement.contains(elementNode);
  }

  function normalizeFontName(name: string) {
    const cleaned = name.replace(/"/g, "").toLowerCase();
    const match = fontFamilies.find(
      (font) => font.value.replace(/"/g, "").toLowerCase() === cleaned
    );
    return match ? match.value : name;
  }

  function normalizeColor(value: string, fallback = "#000000") {
    if (!value) return fallback;
    if (value === "transparent") return fallback;
    if (value.startsWith("#")) {
      if (value.length === 4) {
        return (
          "#" +
          value[1] + value[1] +
          value[2] + value[2] +
          value[3] + value[3]
        ).toLowerCase();
      }
      return value.toLowerCase();
    }

    const rgbaMatch = value.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([0-9.]+))?/i);
    if (rgbaMatch) {
      const r = Number(rgbaMatch[1]);
      const g = Number(rgbaMatch[2]);
      const b = Number(rgbaMatch[3]);
      const alpha = rgbaMatch[4] !== undefined ? Number(rgbaMatch[4]) : 1;
      if (alpha === 0) {
        return fallback;
      }
      return rgbToHex(r, g, b);
    }

    const temp = document.createElement("div");
    temp.style.color = value;
    document.body.appendChild(temp);
    const computedColor = getComputedStyle(temp).color;
    document.body.removeChild(temp);

    const computedMatch = computedColor.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([0-9.]+))?/i);
    if (computedMatch) {
      const r = Number(computedMatch[1]);
      const g = Number(computedMatch[2]);
      const b = Number(computedMatch[3]);
      const alpha = computedMatch[4] !== undefined ? Number(computedMatch[4]) : 1;
      if (alpha === 0) {
        return fallback;
      }
      return rgbToHex(r, g, b);
    }

    return fallback;
  }

  function rgbToHex(r: number, g: number, b: number) {
    const toHex = (num: number) => num.toString(16).padStart(2, "0");
    return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
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

    <div class="toolbar-group color-group">
      <label class="color-button" aria-label="Text color" class:disabled={disabled}>
        <PaletteIcon size={18} />
        <input
          type="color"
          class="color-input"
          value={currentTextColor}
          onchange={handleTextColorChange}
          onfocus={cacheSelection}
          onpointerdown={cacheSelection}
          {disabled}
          aria-label="Choose text color"
        />
      </label>
      <label class="color-button" aria-label="Highlight color" class:disabled={disabled}>
        <HighlighterIcon size={18} />
        <input
          type="color"
          class="color-input"
          value={currentHighlightColor}
          onchange={handleHighlightColorChange}
          onfocus={cacheSelection}
          onpointerdown={cacheSelection}
          {disabled}
          aria-label="Choose highlight color"
        />
      </label>
    </div>

    <div class="toolbar-divider"></div>

    <!-- Text formatting -->
    <div class="toolbar-group">
      <button
        class="toolbar-button"
        class:active={formattingState.bold}
        onclick={() => execCommand("bold")}
        {disabled}
        title="Bold (Ctrl+B)"
        aria-label="Bold"
      >
        <BoldIcon size={18} />
      </button>
      <button
        class="toolbar-button"
        class:active={formattingState.italic}
        onclick={() => execCommand("italic")}
        {disabled}
        title="Italic (Ctrl+I)"
        aria-label="Italic"
      >
        <ItalicIcon size={18} />
      </button>
      <button
        class="toolbar-button"
        class:active={formattingState.underline}
        onclick={() => execCommand("underline")}
        {disabled}
        title="Underline (Ctrl+U)"
        aria-label="Underline"
      >
        <UnderlineIcon size={18} />
      </button>
    </div>

    <div class="toolbar-divider"></div>

    <!-- Text alignment -->
    <div class="toolbar-group">
      <button
        class="toolbar-button"
        class:active={formattingState.alignStart && !formattingState.alignCenter && !formattingState.alignEnd}
        onclick={() => execCommand("justifyLeft")}
        {disabled}
        title="Align start"
        aria-label="Align start"
      >
        <TextAlignStartIcon size={18} />
      </button>
      <button
        class="toolbar-button"
        class:active={formattingState.alignCenter}
        onclick={() => execCommand("justifyCenter")}
        {disabled}
        title="Align center"
        aria-label="Align center"
      >
        <TextAlignCenterIcon size={18} />
      </button>
      <button
        class="toolbar-button"
        class:active={formattingState.alignEnd && !formattingState.alignCenter && !formattingState.alignStart}
        onclick={() => execCommand("justifyRight")}
        {disabled}
        title="Align end"
        aria-label="Align end"
      >
        <TextAlignEndIcon size={18} />
      </button>
    </div>

    <div class="toolbar-divider"></div>

    <!-- Lists -->
    <div class="toolbar-group">
      <button
        class="toolbar-button"
        class:active={formattingState.unorderedList}
        onclick={() => execCommand("insertUnorderedList")}
        {disabled}
        title="Bulleted list"
        aria-label="Bulleted list"
      >
        <ListIcon size={18} />
      </button>
      <button
        class="toolbar-button"
        class:active={formattingState.orderedList}
        onclick={() => execCommand("insertOrderedList")}
        {disabled}
        title="Numbered list"
        aria-label="Numbered list"
      >
        <ListOrderedIcon size={18} />
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
        <IndentDecreaseIcon size={18} />
      </button>
      <button
        class="toolbar-button"
        onclick={() => execCommand("indent")}
        {disabled}
        title="Increase indent"
        aria-label="Increase indent"
      >
        <IndentIncreaseIcon size={18} />
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

  .color-group {
    margin-left: 0.25rem;
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

  .toolbar-button.active {
    background-color: var(--selected-bg);
    border-color: var(--border-color);
    color: var(--accent-foreground, var(--text-color));
  }

  .toolbar-button :global(svg) {
    width: 18px;
    height: 18px;
  }

  .color-button {
    position: relative;
    width: 32px;
    height: 32px;
    border: 1px solid transparent;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: transparent;
    color: var(--text-color);
    cursor: pointer;
    transition: all 0.2s;
  }

  .color-button:hover {
    background-color: var(--hover-bg);
    border-color: var(--border-color);
  }

  .color-button:focus-within {
    border-color: var(--border-color);
    background-color: var(--hover-bg);
  }

  .color-button.disabled {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  .color-input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
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

  .editor-content :global(ul) {
    list-style: disc outside;
  }

  .editor-content :global(ol) {
    list-style: decimal outside;
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
