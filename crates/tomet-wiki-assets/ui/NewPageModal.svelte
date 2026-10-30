<script lang="ts">
  import { Button, Field, Input, Modal, Select } from "@telaion/ui";

  let {
    onCreate,
    onCancel,
  }: {
    onCreate: (path: string, initialContent: string) => void;
    onCancel: () => void;
  } = $props();

  const uid = $props.id();
  const formId = `${uid}-form`;
  const templates = [
    { value: "standard", label: "Standard Note (Title & Paragraph)" },
    { value: "blank", label: "Minimal Blank" },
  ];

  let path = $state("");
  let template = $state("standard");
  let error = $state<string | undefined>();

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

    const content =
      template === "standard" ? `=[ ${title} ]\n\nWrite your notes here...\n` : `=[ ${title} ]\n\n`;

    onCreate(cleanPath, content);
  }
</script>

<Modal open title="Create New Page" size="md" onclose={onCancel}>
  <form id={formId} onsubmit={handleSubmit}>
    <Field label="Page Path (relative to repository vault)" {error}>
      {#snippet hint()}
        e.g. <code>guides/getting-started.tmt</code> or <code>concepts/overview.tmt</code>
      {/snippet}
      <Input bind:value={path} autofocus placeholder="e.g. notes/my-topic.tmt" />
    </Field>
    <Field label="Initial Template">
      <Select bind:value={template} options={templates} />
    </Field>
  </form>
  {#snippet footer()}
    <Button onclick={onCancel}>Cancel</Button>
    <Button variant="primary" type="submit" form={formId}>Create & Edit</Button>
  {/snippet}
</Modal>

<style>
  code {
    font-family: var(--tln-font-mono);
  }

  form {
    display: grid;
    gap: calc(var(--tln-space) * 4);
  }
</style>
