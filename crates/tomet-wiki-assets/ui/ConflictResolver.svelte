<script lang="ts">
  import {
    ConflictResolver as Resolver,
    utf8ToUtf16Spans,
    type Conflict,
    type TometServices,
  } from "@telaion/tomet";
  import { initCms, listConflicts, resolveConflict } from "./cms.ts";

  let {
    content,
    onResolve,
    onManualEdit,
  }: {
    content: string;
    onResolve: (newContent: string) => void;
    onManualEdit: () => void;
  } = $props();

  // Telaion carries no Tomet engine. The wiki ships its own WebAssembly (crates/tomet-wiki-wasm) and describes it here.
  // The engine must be initialized before it answers; asking it too early would look like
  // "no conflicts", so every call waits for it.
  const services: Pick<TometServices, "listConflicts" | "resolveConflict"> = {
    async listConflicts(source) {
      await initCms();
      const found = listConflicts(source);
      // The cms functions count UTF-8 bytes; Telaion counts UTF-16 code units, as JavaScript does.
      const spans = utf8ToUtf16Spans(
        source,
        found.map((c) => ({ from: c.span.start, to: c.span.end, tag: c.kind })),
      );
      return found.map(
        (c, i): Conflict => ({
          index: c.index,
          kind: c.kind,
          from: spans[i].from,
          to: spans[i].to,
          a: c.a,
          b: c.b,
        }),
      );
    },
    async resolveConflict(source, index, side) {
      await initCms();
      return resolveConflict(source, index, side);
    },
  };
</script>

<Resolver {content} {services} {onResolve} {onManualEdit} />
