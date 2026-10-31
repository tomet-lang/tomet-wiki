<script lang="ts">
  import { placeholder } from "@codemirror/view";
  import { CodeEditor } from "@telaion/editor";
  import { tometHighlighting } from "@telaion/tomet";
  import { highlight } from "./cms.ts";

  let {
    content = $bindable(""),
    readOnly = false,
    onChange,
  }: {
    content: string;
    readOnly?: boolean;
    onChange?: (val: string) => void;
  } = $props();

  // The colors come from the wiki's own WebAssembly (`tomet::highlight`), asked after every edit.
  // If it fails, the editor keeps working without colors.
  const extensions = [
    placeholder("Write your Tomet markup here..."),
    tometHighlighting({ highlight }, { onError: (error) => console.warn("highlight failed:", error) }),
  ];
</script>

<CodeEditor bind:content {readOnly} {extensions} label="Tomet source" {onChange} />
