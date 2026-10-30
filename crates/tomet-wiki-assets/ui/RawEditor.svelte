<script lang="ts">
  import { placeholder } from "@codemirror/view";
  import { CodeEditor } from "@telaion/editor";

  let {
    content = $bindable(""),
    readOnly = false,
    onChange,
  }: {
    content: string;
    readOnly?: boolean;
    onChange?: (val: string) => void;
  } = $props();

  // No syntax colors yet: @tomet/cms does not offer a highlight. When it does, pass
  // `language={tometHighlighting(services)}` from @telaion/tomet here.
  const extensions = [placeholder("Write your Tomet markup here...")];
</script>

<CodeEditor bind:content {readOnly} {extensions} label="Tomet source" {onChange} />
