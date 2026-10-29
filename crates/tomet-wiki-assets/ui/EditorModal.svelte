<script lang="ts">
  import { onMount } from "svelte";
  import { authStore } from "./auth.svelte.ts";
  import {
    fetchFileFromGitHub,
    saveFileViaCms,
    listConflicts,
    type ConflictInfo,
  } from "./cms.ts";
  import RawEditor from "./RawEditor.svelte";
  import ConflictResolver from "./ConflictResolver.svelte";

  let {
    relPath = $bindable(""),
    isNew = false,
    initialContent = "",
    config = {},
    onClose,
  }: {
    relPath: string;
    isNew?: boolean;
    initialContent?: string;
    config?: {
      owner?: string;
      repo?: string;
      branch?: string;
      authEndpoint?: string;
      siteDomain?: string;
      apiBaseUrl?: string;
    };
    onClose: () => void;
  } = $props();

  const owner = $derived(config.owner || "owner");
  const repo = $derived(config.repo || "repo");
  const branch = $derived(config.branch || "main");
  const apiBaseUrl = $derived(config.apiBaseUrl || "https://api.github.com");
  const authEndpoint = $derived(config.authEndpoint || "");

  let content = $state("");
  let originalRemoteContent = $state("");
  let baseRev = $state<string | null>(null);
  let commitMessage = $state("");
  let isLoading = $state(false);
  let isSaving = $state(false);
  let errorMessage = $state<string | null>(null);
  let successMessage = $state<string | null>(null);
  let patInput = $state("");
  let activeTab = $state<"editor" | "conflicts" | "auth">("editor");

  const conflicts = $derived<ConflictInfo[]>(listConflicts(content));
  const isDirty = $derived(content !== originalRemoteContent);

  // Auto-switch to conflicts tab if conflicts exist
  $effect(() => {
    if (conflicts.length > 0 && activeTab !== "conflicts" && activeTab !== "auth") {
      activeTab = "conflicts";
    }
  });

  onMount(() => {
    content = initialContent;
    commitMessage = isNew ? `docs: create ${relPath}` : `docs: update ${relPath}`;
    if (!authStore.token) {
      activeTab = "auth";
    } else {
      loadFile();
    }
  });

  async function loadFile() {
    if (isNew) {
      originalRemoteContent = initialContent;
      content = initialContent;
      return;
    }

    const draftKey = `tmt_draft:${relPath}`;
    const draft = localStorage.getItem(draftKey);

    isLoading = true;
    errorMessage = null;

    try {
      const remote = await fetchFileFromGitHub(
        owner,
        repo,
        relPath,
        branch,
        authStore.token,
        apiBaseUrl,
      );
      originalRemoteContent = remote.content;
      baseRev = remote.sha;

      if (draft && draft !== remote.content) {
        if (confirm("A local unsaved draft was found. Do you want to restore it?")) {
          content = draft;
        } else {
          content = remote.content;
          localStorage.removeItem(draftKey);
        }
      } else {
        content = remote.content;
      }
    } catch (err: any) {
      errorMessage = `Failed to load file: ${err.message}`;
      if (draft) {
        content = draft;
      }
    } finally {
      isLoading = false;
    }
  }

  function handleContentChange(newVal: string) {
    content = newVal;
    if (relPath) {
      localStorage.setItem(`tmt_draft:${relPath}`, newVal);
    }
  }

  async function handleCommit() {
    if (!authStore.token) {
      activeTab = "auth";
      return;
    }

    const message = commitMessage.trim() || (isNew ? `docs: create ${relPath}` : `docs: update ${relPath}`);
    isSaving = true;
    errorMessage = null;
    successMessage = null;

    try {
      const result = await saveFileViaCms(
        owner,
        repo,
        relPath,
        branch,
        content,
        message,
        baseRev,
        authStore.token,
        apiBaseUrl,
      );

      baseRev = result.revision;
      originalRemoteContent = content;
      localStorage.removeItem(`tmt_draft:${relPath}`);

      if (result.hasConflicts) {
        successMessage = "Committed with AST conflicts. Please resolve them below.";
        activeTab = "conflicts";
      } else {
        successMessage = `Successfully committed to ${branch}!`;
        setTimeout(() => {
          successMessage = null;
        }, 4000);
      }
    } catch (err: any) {
      errorMessage = `Commit failed: ${err.message}`;
    } finally {
      isSaving = false;
    }
  }

  async function handlePatLogin() {
    const ok = await authStore.loginWithPat(patInput, apiBaseUrl);
    if (ok) {
      patInput = "";
      activeTab = "editor";
      loadFile();
    }
  }

  async function handleOAuthLogin() {
    if (!authEndpoint) return;
    try {
      await authStore.loginWithOAuth(authEndpoint, config.siteDomain);
      activeTab = "editor";
      loadFile();
    } catch (err: any) {
      errorMessage = err.message;
    }
  }

  function handleCloseModal() {
    if (isDirty) {
      if (!confirm("You have uncommitted changes saved in local draft. Close editor anyway?")) {
        return;
      }
    }
    onClose();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      handleCommit();
    }
    if (e.key === "Escape" && activeTab !== "auth") {
      handleCloseModal();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<div class="tmt-editor-overlay" role="dialog" aria-modal="true">
  <!-- Top Bar -->
  <header class="tmt-editor-topbar">
    <div class="tmt-topbar-left">
      <span class="tmt-doc-icon">📄</span>
      <span class="tmt-doc-path" title={relPath}>{relPath}</span>
      <span class="tmt-branch-pill">{branch}</span>

      {#if isDirty}
        <span class="tmt-status-badge tmt-status-dirty">● Unsaved</span>
      {:else if isSaving}
        <span class="tmt-status-badge tmt-status-saving">● Saving...</span>
      {:else}
        <span class="tmt-status-badge tmt-status-clean">● Clean</span>
      {/if}

      {#if conflicts.length > 0}
        <button
          type="button"
          class="tmt-tab-pill tmt-conflict-pill"
          onclick={() => (activeTab = "conflicts")}
        >
          ⚠️ {conflicts.length} Conflict{conflicts.length > 1 ? "s" : ""}
        </button>
      {/if}
    </div>

    <!-- Center Tab Switcher -->
    <div class="tmt-tab-switcher">
      <button
        type="button"
        class="tmt-tab-btn"
        class:active={activeTab === "editor"}
        onclick={() => (activeTab = "editor")}
      >
        Editor
      </button>
      {#if conflicts.length > 0}
        <button
          type="button"
          class="tmt-tab-btn"
          class:active={activeTab === "conflicts"}
          onclick={() => (activeTab = "conflicts")}
        >
          Conflicts ({conflicts.length})
        </button>
      {/if}
    </div>

    <!-- Right Actions -->
    <div class="tmt-topbar-right">
      <input
        type="text"
        class="tmt-commit-msg-input"
        placeholder="Commit message..."
        bind:value={commitMessage}
      />

      <button
        type="button"
        class="tmt-action-btn tmt-commit-btn"
        disabled={isSaving || !authStore.token}
        onclick={handleCommit}
      >
        {isSaving ? "Saving..." : "Commit"}
      </button>

      <!-- Auth avatar / login trigger -->
      {#if authStore.user}
        <button
          type="button"
          class="tmt-avatar-btn"
          title={`Logged in as ${authStore.user.login}. Click to manage auth.`}
          onclick={() => (activeTab = activeTab === "auth" ? "editor" : "auth")}
        >
          <img src={authStore.user.avatar_url} alt={authStore.user.login} class="tmt-avatar-img" />
        </button>
      {:else}
        <button
          type="button"
          class="tmt-action-btn tmt-login-btn"
          onclick={() => (activeTab = "auth")}
        >
          Log in
        </button>
      {/if}

      <button
        type="button"
        class="tmt-action-btn tmt-close-modal-btn"
        onclick={handleCloseModal}
        title="Close editor"
      >
        ✕
      </button>
    </div>
  </header>

  <!-- Alerts Bar -->
  {#if errorMessage}
    <div class="tmt-alert tmt-alert-error">
      <span>{errorMessage}</span>
      <button type="button" class="tmt-alert-dismiss" onclick={() => (errorMessage = null)}>✕</button>
    </div>
  {/if}

  {#if successMessage}
    <div class="tmt-alert tmt-alert-success">
      <span>{successMessage}</span>
      <button type="button" class="tmt-alert-dismiss" onclick={() => (successMessage = null)}>✕</button>
    </div>
  {/if}

  <!-- Main Body Content -->
  <main class="tmt-editor-content">
    {#if activeTab === "auth"}
      <div class="tmt-auth-view">
        <div class="tmt-auth-card">
          <h2>GitHub Authentication</h2>
          <p>
            Authenticate with GitHub to commit updates directly to
            <code>{owner}/{repo}</code> on branch <code>{branch}</code>.
          </p>

          {#if authStore.user}
            <div class="tmt-current-user">
              <img src={authStore.user.avatar_url} alt="" class="tmt-user-avatar-lg" />
              <div>
                <strong>{authStore.user.name || authStore.user.login}</strong>
                <span class="tmt-user-login">@{authStore.user.login}</span>
              </div>
              <button
                type="button"
                class="tmt-btn tmt-btn-danger"
                onclick={() => authStore.logout()}
              >
                Log out
              </button>
            </div>
            <button
              type="button"
              class="tmt-btn tmt-btn-primary"
              onclick={() => (activeTab = "editor")}
            >
              Continue Editing
            </button>
          {:else}
            {#if authEndpoint}
              <button
                type="button"
                class="tmt-btn tmt-oauth-btn"
                onclick={handleOAuthLogin}
                disabled={authStore.loading}
              >
                Sign in with GitHub (OAuth)
              </button>
              <div class="tmt-auth-divider"><span>OR</span></div>
            {/if}

            <div class="tmt-pat-form">
              <label for="tmt-pat-input">Personal Access Token (PAT)</label>
              <input
                id="tmt-pat-input"
                type="password"
                placeholder="ghp_... or github_pat_..."
                class="tmt-input"
                bind:value={patInput}
              />
              <span class="tmt-hint">
                Requires repository content write permissions.
              </span>
              <button
                type="button"
                class="tmt-btn tmt-btn-primary"
                onclick={handlePatLogin}
                disabled={authStore.loading || !patInput.trim()}
              >
                {authStore.loading ? "Authenticating..." : "Save Token & Continue"}
              </button>
            </div>
          {/if}
        </div>
      </div>
    {:else if isLoading}
      <div class="tmt-loading-view">
        <div class="tmt-spinner"></div>
        <p>Loading document from GitHub...</p>
      </div>
    {:else if activeTab === "conflicts"}
      <ConflictResolver
        {content}
        onResolve={(resolved) => handleContentChange(resolved)}
        onManualEdit={() => (activeTab = "editor")}
      />
    {:else}
      <RawEditor
        bind:content
        onChange={handleContentChange}
      />
    {/if}
  </main>
</div>

<style>
  .tmt-editor-overlay {
    position: fixed;
    inset: 0;
    z-index: 10000;
    display: flex;
    flex-direction: column;
    background: var(--tmt-bg, #ffffff);
    color: var(--tmt-fg, #1f2328);
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  }

  .tmt-editor-topbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background: var(--tmt-bg-subtle, #f6f8fa);
    border-bottom: 1px solid var(--tmt-border, #d0d7de);
    gap: 1rem;
  }

  .tmt-topbar-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .tmt-doc-icon {
    font-size: 1.1rem;
  }

  .tmt-doc-path {
    font-weight: 600;
    font-size: 0.9rem;
    white-space: nowrap;
    overflow: hidden;
    text-stream: ellipsis;
    max-width: 240px;
  }

  .tmt-branch-pill {
    padding: 0.15rem 0.45rem;
    background: var(--tmt-bg, #ffffff);
    border: 1px solid var(--tmt-border, #d0d7de);
    border-radius: 9999px;
    font-size: 0.75rem;
    color: var(--tmt-fg-muted, #656d76);
  }

  .tmt-status-badge {
    font-size: 0.75rem;
    font-weight: 500;
  }

  .tmt-status-clean {
    color: #16a34a;
  }

  .tmt-status-dirty {
    color: #d97706;
  }

  .tmt-status-saving {
    color: #2563eb;
  }

  .tmt-tab-pill {
    background: #fef3c7;
    border: 1px solid #f59e0b;
    color: #b45309;
    padding: 0.15rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    cursor: pointer;
    font-weight: 600;
  }

  .tmt-tab-switcher {
    display: flex;
    gap: 0.25rem;
    background: var(--tmt-bg, #ffffff);
    padding: 0.2rem;
    border-radius: 6px;
    border: 1px solid var(--tmt-border, #d0d7de);
  }

  .tmt-tab-btn {
    padding: 0.25rem 0.75rem;
    font-size: 0.8rem;
    font-weight: 500;
    border: none;
    background: transparent;
    border-radius: 4px;
    cursor: pointer;
    color: var(--tmt-fg-muted, #656d76);
  }

  .tmt-tab-btn.active {
    background: var(--tmt-bg-subtle, #f6f8fa);
    color: var(--tmt-fg, #1f2328);
    font-weight: 600;
  }

  .tmt-topbar-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .tmt-commit-msg-input {
    padding: 0.35rem 0.6rem;
    font-size: 0.85rem;
    border: 1px solid var(--tmt-border, #d0d7de);
    border-radius: 6px;
    width: 220px;
    background: var(--tmt-bg, #ffffff);
    color: var(--tmt-fg, #1f2328);
  }

  .tmt-commit-msg-input:focus {
    outline: none;
    border-color: #0969da;
  }

  .tmt-action-btn {
    padding: 0.35rem 0.8rem;
    border-radius: 6px;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid transparent;
  }

  .tmt-commit-btn {
    background: #1f883d;
    color: #ffffff;
  }
  .tmt-commit-btn:hover:not(:disabled) {
    background: #1a7f37;
  }
  .tmt-commit-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .tmt-login-btn {
    background: #0969da;
    color: #ffffff;
  }

  .tmt-close-modal-btn {
    background: transparent;
    color: var(--tmt-fg-muted, #656d76);
    font-size: 1rem;
    padding: 0.35rem 0.5rem;
  }
  .tmt-close-modal-btn:hover {
    color: var(--tmt-fg, #1f2328);
  }

  .tmt-avatar-btn {
    padding: 0;
    border: none;
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
  }

  .tmt-avatar-img {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 1px solid var(--tmt-border, #d0d7de);
  }

  .tmt-alert {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.6rem 1rem;
    font-size: 0.85rem;
  }

  .tmt-alert-error {
    background: #ffebe9;
    color: #cf222e;
    border-bottom: 1px solid #ff8182;
  }

  .tmt-alert-success {
    background: #dafbe1;
    color: #1a7f37;
    border-bottom: 1px solid #4ac26b;
  }

  .tmt-alert-dismiss {
    background: transparent;
    border: none;
    cursor: pointer;
    color: inherit;
    font-weight: bold;
  }

  .tmt-editor-content {
    flex-grow: 1;
    overflow: hidden;
    position: relative;
  }

  .tmt-loading-view {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--tmt-fg-muted, #656d76);
  }

  .tmt-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(0, 0, 0, 0.1);
    border-radius: 50%;
    border-top-color: #0969da;
    animation: spin 0.8s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .tmt-auth-view {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 2rem;
    background: var(--tmt-bg-subtle, #f6f8fa);
  }

  .tmt-auth-card {
    background: var(--tmt-bg, #ffffff);
    padding: 2rem;
    border-radius: 12px;
    border: 1px solid var(--tmt-border, #d0d7de);
    max-width: 440px;
    width: 100%;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  }

  .tmt-auth-card h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
  }

  .tmt-auth-card p {
    font-size: 0.9rem;
    color: var(--tmt-fg-muted, #656d76);
    margin-bottom: 1.5rem;
  }

  .tmt-oauth-btn {
    width: 100%;
    background: #24292f;
    color: #ffffff;
    padding: 0.6rem;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    border: none;
    margin-bottom: 1rem;
  }

  .tmt-auth-divider {
    display: flex;
    align-items: center;
    text-align: center;
    margin: 1.25rem 0;
  }
  .tmt-auth-divider::before, .tmt-auth-divider::after {
    content: "";
    flex: 1;
    border-bottom: 1px solid var(--tmt-border, #d0d7de);
  }
  .tmt-auth-divider span {
    padding: 0 0.75rem;
    color: var(--tmt-fg-muted, #656d76);
    font-size: 0.75rem;
  }

  .tmt-pat-form label {
    display: block;
    font-size: 0.85rem;
    font-weight: 600;
    margin-bottom: 0.35rem;
  }

  .tmt-input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--tmt-border, #d0d7de);
    border-radius: 6px;
    font-size: 0.9rem;
    box-sizing: border-box;
    background: var(--tmt-bg, #ffffff);
    color: var(--tmt-fg, #1f2328);
  }

  .tmt-hint {
    display: block;
    font-size: 0.75rem;
    color: var(--tmt-fg-muted, #656d76);
    margin: 0.35rem 0 1rem 0;
  }

  .tmt-btn {
    display: inline-block;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-weight: 500;
    font-size: 0.85rem;
    cursor: pointer;
    border: 1px solid transparent;
  }

  .tmt-btn-primary {
    background: #0969da;
    color: #ffffff;
    width: 100%;
  }

  .tmt-btn-danger {
    background: #cf222e;
    color: #ffffff;
  }

  .tmt-current-user {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--tmt-bg-subtle, #f6f8fa);
    border-radius: 8px;
    margin-bottom: 1.5rem;
  }

  .tmt-user-avatar-lg {
    width: 44px;
    height: 44px;
    border-radius: 50%;
  }

  .tmt-user-login {
    display: block;
    font-size: 0.8rem;
    color: var(--tmt-fg-muted, #656d76);
  }
</style>
