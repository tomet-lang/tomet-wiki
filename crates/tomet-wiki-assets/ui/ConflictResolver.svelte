<script lang="ts">
  import { listConflicts, resolveConflict, type ConflictInfo } from "./cms.ts";

  let {
    content,
    onResolve,
    onManualEdit,
  }: {
    content: string;
    onResolve: (newContent: string) => void;
    onManualEdit: () => void;
  } = $props();

  const conflicts = $derived<ConflictInfo[]>(listConflicts(content));

  function handleChoose(index: number, side: "a" | "b") {
    try {
      const resolved = resolveConflict(content, index, side);
      onResolve(resolved);
    } catch (err) {
      console.error("Failed to resolve conflict:", err);
    }
  }
</script>

<div class="tmt-conflict-resolver">
  {#if conflicts.length > 0}
    <div class="tmt-conflict-banner">
      <span class="tmt-conflict-alert-icon">⚠️</span>
      <strong>{conflicts.length} AST Conflict{conflicts.length > 1 ? "s" : ""} Detected</strong>
      <p>
        Concurrent edits occurred on GitHub. Tomet preserved both versions in the AST.
        Choose which version to keep for each section below, or switch to manual edit.
      </p>
    </div>

    <div class="tmt-conflict-list">
      {#each conflicts as conflict, i}
        <div class="tmt-conflict-card">
          <div class="tmt-conflict-header">
            <span class="tmt-conflict-badge">Conflict #{i + 1} ({conflict.kind})</span>
          </div>

          <div class="tmt-conflict-sides">
            <!-- Side A -->
            <div class="tmt-conflict-side tmt-side-a">
              <div class="tmt-side-header">
                <span class="tmt-side-label">Version A (Current / Base)</span>
                <button
                  type="button"
                  class="tmt-btn tmt-btn-accept tmt-btn-a"
                  onclick={() => handleChoose(conflict.index, "a")}
                >
                  Accept Version A
                </button>
              </div>
              <pre class="tmt-side-content">{conflict.a}</pre>
            </div>

            <!-- Side B -->
            <div class="tmt-conflict-side tmt-side-b">
              <div class="tmt-side-header">
                <span class="tmt-side-label">Version B (Incoming / Remote)</span>
                <button
                  type="button"
                  class="tmt-btn tmt-btn-accept tmt-btn-b"
                  onclick={() => handleChoose(conflict.index, "b")}
                >
                  Accept Version B
                </button>
              </div>
              <pre class="tmt-side-content">{conflict.b}</pre>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="tmt-conflict-empty">
      <div class="tmt-resolved-icon">✅</div>
      <h3>All Conflicts Resolved</h3>
      <p>There are no unresolved merge conflicts remaining in this document.</p>
      <button type="button" class="tmt-btn tmt-btn-primary" onclick={onManualEdit}>
        Return to Raw Editor
      </button>
    </div>
  {/if}
</div>

<style>
  .tmt-conflict-resolver {
    padding: 1.5rem;
    height: 100%;
    overflow-y: auto;
    background: var(--tmt-bg, #ffffff);
    color: var(--tmt-fg, #1f2328);
  }

  .tmt-conflict-banner {
    padding: 1rem 1.25rem;
    margin-bottom: 1.5rem;
    background: rgba(245, 158, 11, 0.1);
    border: 1px solid #f59e0b;
    border-radius: 8px;
    font-size: 0.95rem;
  }

  .tmt-conflict-banner strong {
    color: #d97706;
    display: inline-block;
    margin-bottom: 0.25rem;
  }

  .tmt-conflict-banner p {
    margin: 0.25rem 0 0 0;
    color: var(--tmt-fg, #1f2328);
    font-size: 0.85rem;
  }

  .tmt-conflict-list {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .tmt-conflict-card {
    border: 1px solid var(--tmt-border, #d0d7de);
    border-radius: 8px;
    background: var(--tmt-bg-subtle, #f6f8fa);
    overflow: hidden;
  }

  .tmt-conflict-header {
    padding: 0.6rem 1rem;
    background: var(--tmt-bg, #ffffff);
    border-bottom: 1px solid var(--tmt-border, #d0d7de);
    font-weight: 600;
  }

  .tmt-conflict-badge {
    display: inline-block;
    padding: 0.2rem 0.5rem;
    background: #f59e0b;
    color: #ffffff;
    border-radius: 4px;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .tmt-conflict-sides {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1px;
    background: var(--tmt-border, #d0d7de);
  }

  @media (max-width: 768px) {
    .tmt-conflict-sides {
      grid-template-columns: 1fr;
    }
  }

  .tmt-conflict-side {
    background: var(--tmt-bg, #ffffff);
    padding: 1rem;
    display: flex;
    flex-direction: column;
  }

  .tmt-side-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .tmt-side-label {
    font-size: 0.85rem;
    font-weight: 600;
  }

  .tmt-side-a .tmt-side-label {
    color: #2563eb;
  }

  .tmt-side-b .tmt-side-label {
    color: #16a34a;
  }

  .tmt-side-content {
    flex-grow: 1;
    margin: 0;
    padding: 0.75rem;
    background: var(--tmt-bg-subtle, #f6f8fa);
    border: 1px solid var(--tmt-border, #e5e7eb);
    border-radius: 6px;
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 0.85rem;
    line-height: 1.4;
    white-space: pre-wrap;
    word-break: break-word;
    min-height: 80px;
  }

  .tmt-btn {
    padding: 0.35rem 0.75rem;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid transparent;
    transition: all 0.15s ease;
  }

  .tmt-btn-accept {
    color: #ffffff;
  }

  .tmt-btn-a {
    background: #2563eb;
  }
  .tmt-btn-a:hover {
    background: #1d4ed8;
  }

  .tmt-btn-b {
    background: #16a34a;
  }
  .tmt-btn-b:hover {
    background: #15803d;
  }

  .tmt-btn-primary {
    background: #0969da;
    color: #ffffff;
    padding: 0.5rem 1rem;
    font-size: 0.9rem;
  }
  .tmt-btn-primary:hover {
    background: #0550ae;
  }

  .tmt-conflict-empty {
    text-align: center;
    padding: 3rem 1rem;
  }

  .tmt-resolved-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .tmt-conflict-empty h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
  }

  .tmt-conflict-empty p {
    color: var(--tmt-fg-muted, #656d76);
    margin: 0 0 1.5rem 0;
  }
</style>
