<script lang="ts">
  let {
    content = $bindable(""),
    readOnly = false,
    onChange,
  }: {
    content: string;
    readOnly?: boolean;
    onChange?: (val: string) => void;
  } = $props();

  let textareaEl: HTMLTextAreaElement | null = $state(null);
  let lineNumbersEl: HTMLDivElement | null = $state(null);

  const lines = $derived(content.split("\n"));

  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    content = target.value;
    onChange?.(content);
  }

  function handleScroll() {
    if (textareaEl && lineNumbersEl) {
      lineNumbersEl.scrollTop = textareaEl.scrollTop;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Tab") {
      e.preventDefault();
      if (!textareaEl) return;
      const start = textareaEl.selectionStart;
      const end = textareaEl.selectionEnd;
      const val = textareaEl.value;
      content = val.substring(0, start) + "  " + val.substring(end);
      textareaEl.value = content;
      textareaEl.selectionStart = textareaEl.selectionEnd = start + 2;
      onChange?.(content);
    }
  }
</script>

<div class="tmt-raw-editor">
  <div class="tmt-line-numbers" bind:this={lineNumbersEl} aria-hidden="true">
    {#each lines as _, i}
      <div class="tmt-line-number">{i + 1}</div>
    {/each}
  </div>
  <textarea
    bind:this={textareaEl}
    class="tmt-editor-textarea"
    value={content}
    {readOnly}
    placeholder="Write your Tomet markup here..."
    spellcheck="false"
    oninput={handleInput}
    onscroll={handleScroll}
    onkeydown={handleKeyDown}
  ></textarea>
</div>

<style>
  .tmt-raw-editor {
    display: flex;
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: var(--tmt-bg, #ffffff);
    color: var(--tmt-fg, #1f2328);
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .tmt-line-numbers {
    flex-shrink: 0;
    width: 3.5rem;
    padding: 1rem 0.5rem;
    overflow: hidden;
    text-align: right;
    user-select: none;
    color: var(--tmt-fg-muted, #656d76);
    background: var(--tmt-bg-subtle, #f6f8fa);
    border-right: 1px solid var(--tmt-border, #d0d7de);
  }

  .tmt-line-number {
    height: 1.5em;
  }

  .tmt-editor-textarea {
    flex-grow: 1;
    width: 100%;
    height: 100%;
    padding: 1rem;
    margin: 0;
    border: none;
    outline: none;
    resize: none;
    background: transparent;
    color: inherit;
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
    white-space: pre;
    overflow-wrap: normal;
    overflow-x: auto;
    tab-size: 2;
  }
</style>
