<script lang="ts">
  import { onMount } from "svelte";
  import { authStore } from "./auth.svelte.ts";
  import {
    fetchFileFromGitHub,
    saveFileViaCms,
    initCms,
    resolveRepoPath,
    listConflicts,
    type ConflictInfo,
  } from "./cms.ts";
  import { Alert, Badge, Button, Field, Icon, Input, Modal, Spinner, Tabs } from "@telaion/ui";
  import { TriangleAlert } from "@telaion/ui/icons";
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
      pathPrefix?: string;
      authEndpoint?: string;
      siteDomain?: string;
      apiBaseUrl?: string;
      production?: boolean;
    };
    onClose: () => void;
  } = $props();

  const owner = $derived(config.owner || "owner");
  const repo = $derived(config.repo || "repo");
  const branch = $derived(config.branch || "main");
  const pathPrefix = $derived(config.pathPrefix || "");
  const isProduction = $derived(config.production ?? false);
  const isCmsConfigured = $derived(
    Boolean(config.owner && config.owner !== "owner" && config.repo && config.repo !== "repo"),
  );
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

  // The wiki's wasm must be initialized before it can read a document. Asked before
  // that, it fails, and listConflicts turns the failure into "no conflicts". So wait for it, and
  // look again once it is ready.
  let cmsReady = $state(false);
  const conflicts = $derived<ConflictInfo[]>(cmsReady ? listConflicts(content) : []);
  const isDirty = $derived(content !== originalRemoteContent);

  // Auto-switch to conflicts tab if conflicts exist
  $effect(() => {
    if (conflicts.length > 0 && activeTab !== "conflicts" && activeTab !== "auth") {
      activeTab = "conflicts";
    }
  });

  onMount(() => {
    initCms().then(() => (cmsReady = true));
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
      if (!isCmsConfigured) {
        // CMS owner/repo not configured - use initialContent from page
        originalRemoteContent = initialContent;
        content = draft || initialContent;
        return;
      }

      const fullPath = resolveRepoPath(relPath, pathPrefix);
      const remote = await fetchFileFromGitHub(
        owner,
        repo,
        fullPath,
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
      } else if (initialContent) {
        content = initialContent;
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
    if (!isProduction) {
      errorMessage = "Commits are disabled in development mode. Run with '--production' to enable repository commits.";
      return;
    }

    if (!authStore.token) {
      activeTab = "auth";
      return;
    }

    const message = commitMessage.trim() || (isNew ? `docs: create ${relPath}` : `docs: update ${relPath}`);
    isSaving = true;
    errorMessage = null;
    successMessage = null;

    try {
      const fullPath = resolveRepoPath(relPath, pathPrefix);
      const result = await saveFileViaCms(
        owner,
        repo,
        fullPath,
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
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<!--
  The whole editor is one full-size Modal. Closing it, by Escape, the close button or the backdrop,
  goes through handleCloseModal, which asks first when there are unsaved changes. The modal itself
  never closes: when the person agrees, onClose unmounts this component.
-->
<Modal
  open
  title={relPath}
  size="full"
  onrequestclose={() => {
    handleCloseModal();
    return false;
  }}
>
  <div class="tmt-editor-body">
    <div class="tmt-toolbar">
      <Badge>{branch}</Badge>
      {#if !isProduction}
        <span title="Commits disabled in development mode. Pass --production to enable.">
          <Badge tone="warn">Read-Only</Badge>
        </span>
      {/if}

      {#if isDirty}
        <Badge tone="warn">Unsaved</Badge>
      {:else if isSaving}
        <Badge tone="accent">Saving...</Badge>
      {:else}
        <Badge tone="ok">Clean</Badge>
      {/if}

      {#if conflicts.length > 0}
        <Button variant="quiet" onclick={() => (activeTab = "conflicts")}>
          <Icon icon={TriangleAlert} />
          {conflicts.length} Conflict{conflicts.length > 1 ? "s" : ""}
        </Button>
      {/if}

      <div class="tmt-toolbar-tabs">
        <Tabs
          items={[
            { id: "editor", label: "Editor" },
            ...(conflicts.length > 0
              ? [{ id: "conflicts", label: `Conflicts (${conflicts.length})` }]
              : []),
          ]}
          bind:value={() => activeTab, (id) => (activeTab = id as typeof activeTab)}
        />
      </div>

      <!-- Auth avatar / login trigger -->
      {#if authStore.user}
        <button
          type="button"
          class="tmt-avatar-btn"
          title={`Logged in as ${authStore.user.login}. Click to manage auth.`}
          aria-label={`Logged in as ${authStore.user.login}. Click to manage auth.`}
          onclick={() => (activeTab = activeTab === "auth" ? "editor" : "auth")}
        >
          <img src={authStore.user.avatar_url} alt="" class="tmt-avatar-img" />
        </button>
      {:else}
        <Button onclick={() => (activeTab = "auth")}>Log in</Button>
      {/if}
    </div>

    {#if errorMessage}
      <Alert tone="danger" onDismiss={() => (errorMessage = null)}>{errorMessage}</Alert>
    {/if}
    {#if successMessage}
      <Alert tone="ok" onDismiss={() => (successMessage = null)}>{successMessage}</Alert>
    {/if}

    <main class="tmt-editor-content">
      {#if activeTab === "auth"}
        <div class="tmt-auth-card">
          <h3>GitHub Authentication</h3>
          <p>
            Authenticate with GitHub to commit updates directly to
            <code>{owner}/{repo}</code> on branch <code>{branch}</code>.
          </p>

          {#if authStore.user}
            <div class="tmt-current-user">
              <img src={authStore.user.avatar_url} alt="" class="tmt-user-avatar-lg" />
              <div class="tmt-user-name">
                <strong>{authStore.user.name || authStore.user.login}</strong>
                <span class="tmt-user-login">@{authStore.user.login}</span>
              </div>
              <Button variant="danger" onclick={() => authStore.logout()}>Log out</Button>
            </div>
            <div>
              <Button variant="primary" onclick={() => (activeTab = "editor")}>Continue Editing</Button>
            </div>
          {:else}
            <div>
              <Button variant="primary" onclick={handleOAuthLogin} disabled={authStore.loading}>
                Log in
              </Button>
            </div>

            <details class="tmt-pat-details">
              <summary>Advanced: Use Personal Access Token (PAT)</summary>
              <div class="tmt-pat-form">
                <Field
                  label="Personal Access Token (PAT)"
                  hint="Requires repository content write permissions."
                >
                  <Input type="password" placeholder="ghp_... or github_pat_..." bind:value={patInput} />
                </Field>
                <div>
                  <Button
                    variant="primary"
                    onclick={handlePatLogin}
                    disabled={authStore.loading || !patInput.trim()}
                  >
                    {authStore.loading ? "Authenticating..." : "Save Token & Continue"}
                  </Button>
                </div>
              </div>
            </details>
          {/if}
        </div>
      {:else if isLoading}
        <div class="tmt-loading-view">
          <Spinner label="Loading document from GitHub" />
          <p>Loading document from GitHub...</p>
        </div>
      {:else if activeTab === "conflicts"}
        <ConflictResolver
          {content}
          onResolve={(resolved) => handleContentChange(resolved)}
          onManualEdit={() => (activeTab = "editor")}
        />
      {:else}
        <div class="tmt-editor-frame">
          <RawEditor bind:content onChange={handleContentChange} />
        </div>
      {/if}
    </main>
  </div>

  {#snippet footer()}
    <div class="tmt-commit-message">
      <Input bind:value={commitMessage} placeholder="Commit message..." aria-label="Commit message" />
    </div>
    <Button
      variant="primary"
      disabled={isSaving || !authStore.token || !isProduction}
      onclick={handleCommit}
      title={!isProduction
        ? "Commits disabled in development mode (requires --production)"
        : "Commit changes to GitHub"}
    >
      {isSaving ? "Saving..." : !isProduction ? "Read-Only" : "Commit"}
    </Button>
  {/snippet}
</Modal>

<style>
  /* Only the layout of this editor. Colors, fonts and sizes come from the Telaion tokens. */
  .tmt-editor-body {
    display: flex;
    flex-direction: column;
    gap: calc(var(--tln-space) * 3);
    height: 100%;
    min-height: 0;
  }

  .tmt-toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: calc(var(--tln-space) * 2);
  }

  .tmt-toolbar-tabs {
    flex: 1;
    min-width: 0;
  }

  .tmt-avatar-btn {
    padding: 0;
    line-height: 0;
    background: none;
    border: 1px solid var(--tln-border-strong);
    border-radius: 50%;
    cursor: pointer;
  }

  .tmt-avatar-btn:focus-visible {
    outline: 2px solid var(--tln-accent);
    outline-offset: 2px;
  }

  .tmt-avatar-img {
    width: calc(var(--tln-space) * 7);
    height: calc(var(--tln-space) * 7);
    border-radius: 50%;
  }

  .tmt-editor-content {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .tmt-editor-frame {
    height: 100%;
    border: 1px solid var(--tln-border-strong);
    border-radius: var(--tln-radius);
    overflow: hidden;
  }

  .tmt-loading-view {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: calc(var(--tln-space) * 3);
    height: 100%;
    color: var(--tln-fg-muted);
  }

  .tmt-auth-card {
    display: grid;
    gap: calc(var(--tln-space) * 4);
    max-width: calc(var(--tln-space) * 120);
    margin: calc(var(--tln-space) * 8) auto;
    padding: calc(var(--tln-space) * 6);
    background: var(--tln-bg-subtle);
    border: 1px solid var(--tln-border);
    border-radius: var(--tln-radius);
  }

  .tmt-auth-card h3,
  .tmt-auth-card p {
    margin: 0;
  }

  .tmt-auth-card code {
    font-family: var(--tln-font-mono);
  }

  .tmt-current-user {
    display: flex;
    align-items: center;
    gap: calc(var(--tln-space) * 3);
  }

  .tmt-user-avatar-lg {
    width: calc(var(--tln-space) * 12);
    height: calc(var(--tln-space) * 12);
    border-radius: 50%;
  }

  .tmt-user-name {
    display: grid;
    flex: 1;
    min-width: 0;
  }

  .tmt-user-login {
    color: var(--tln-fg-muted);
    font-size: var(--tln-text-sm);
  }

  .tmt-pat-details summary {
    cursor: pointer;
    color: var(--tln-fg-muted);
    font-size: var(--tln-text-sm);
  }

  .tmt-pat-form {
    display: grid;
    gap: calc(var(--tln-space) * 3);
    margin-top: calc(var(--tln-space) * 3);
  }

  .tmt-commit-message {
    flex: 1 1 calc(var(--tln-space) * 60);
    min-width: 0;
  }
</style>
