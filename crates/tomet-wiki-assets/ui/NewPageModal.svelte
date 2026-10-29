<script lang="ts">
  let {
    onCreate,
    onCancel,
  }: {
    onCreate: (path: string, initialContent: string) => void;
    onCancel: () => void;
  } = $props();

  let path = $state("");
  let template = $state<"standard" | "blank">("standard");
  let error = $state<string | null>(null);

  function handleSubmit(e: Event) {
    e.preventDefault();
    let cleanPath = path.trim().replace(/^\/+/, "");
    if (!cleanPath) {
      error = "Path cannot be empty";
      return;
    }
    if (!cleanPath.endsWith(".tmt")) {
      cleanPath += ".tmt";
    }

    const title = cleanPath
      .replace(/\.tmt$/, "")
      .split("/")
      .pop()!
      .replace(/[-_]/g, " ")
      .replace(/\b\w/g, (c) => c.toUpperCase());

    let content = "";
    if (template === "standard") {
      content = `=[ ${title} ]\n\nWrite your notes here...\n`;
    } else {
      content = `=[ ${title} ]\n\n`;
    }

    onCreate(cleanPath, content);
  }
</script>

<div class="tmt-modal-backdrop" onclick={onCancel} role="presentation">
  <div
    class="tmt-new-page-dialog"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key === 'Escape') onCancel(); }}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="tmt-dialog-header">
      <h3>Create New Page</h3>
      <button type="button" class="tmt-close-btn" onclick={onCancel} aria-label="Close">✕</button>
    </div>

    <form onsubmit={handleSubmit} class="tmt-dialog-body">
      {#if error}
        <div class="tmt-error-alert">{error}</div>
      {/if}

      <label class="tmt-field-label" for="tmt-new-path">
        Page Path (relative to repository vault)
      </label>
      <input
        id="tmt-new-path"
        type="text"
        class="tmt-input"
        placeholder="e.g. notes/my-topic.tmt"
        bind:value={path}
      />
      <span class="tmt-field-hint">e.g. <code>guides/getting-started.tmt</code> or <code>concepts/overview.tmt</code></span>

      <label class="tmt-field-label" for="tmt-template-select">
        Initial Template
      </label>
      <select id="tmt-template-select" class="tmt-select" bind:value={template}>
        <option value="standard">Standard Note (Title & Paragraph)</option>
        <option value="blank">Minimal Blank</option>
      </select>

      <div class="tmt-dialog-footer">
        <button type="button" class="tmt-btn tmt-btn-secondary" onclick={onCancel}>
          Cancel
        </button>
        <button type="submit" class="tmt-btn tmt-btn-primary">
          Create & Edit
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .tmt-modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 10001;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
  }

  .tmt-new-page-dialog {
    background: var(--tmt-bg, #ffffff);
    color: var(--tmt-fg, #1f2328);
    width: 100%;
    max-width: 480px;
    border-radius: 12px;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2), 0 10px 10px -5px rgba(0, 0, 0, 0.1);
    overflow: hidden;
    border: 1px solid var(--tmt-border, #d0d7de);
  }

  .tmt-dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--tmt-border, #d0d7de);
  }

  .tmt-dialog-header h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .tmt-close-btn {
    background: transparent;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    color: var(--tmt-fg-muted, #656d76);
  }

  .tmt-dialog-body {
    padding: 1.25rem;
  }

  .tmt-field-label {
    display: block;
    font-size: 0.85rem;
    font-weight: 600;
    margin-bottom: 0.35rem;
    margin-top: 0.75rem;
  }

  .tmt-field-label:first-of-type {
    margin-top: 0;
  }

  .tmt-input, .tmt-select {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--tmt-border, #d0d7de);
    border-radius: 6px;
    font-size: 0.9rem;
    background: var(--tmt-bg, #ffffff);
    color: var(--tmt-fg, #1f2328);
    box-sizing: border-box;
  }

  .tmt-input:focus, .tmt-select:focus {
    outline: none;
    border-color: #0969da;
    box-shadow: 0 0 0 3px rgba(9, 105, 218, 0.15);
  }

  .tmt-field-hint {
    display: block;
    font-size: 0.75rem;
    color: var(--tmt-fg-muted, #656d76);
    margin-top: 0.35rem;
  }

  .tmt-error-alert {
    padding: 0.6rem 0.8rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid #ef4444;
    border-radius: 6px;
    color: #b91c1c;
    font-size: 0.85rem;
    margin-bottom: 1rem;
  }

  .tmt-dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1.5rem;
  }

  .tmt-btn {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid transparent;
  }

  .tmt-btn-secondary {
    background: var(--tmt-bg-subtle, #f6f8fa);
    border-color: var(--tmt-border, #d0d7de);
    color: var(--tmt-fg, #1f2328);
  }

  .tmt-btn-primary {
    background: #0969da;
    color: #ffffff;
  }
</style>
