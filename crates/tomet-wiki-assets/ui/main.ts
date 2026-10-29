import { mount, unmount } from "svelte";
import EditorModal from "./EditorModal.svelte";
import NewPageModal from "./NewPageModal.svelte";

export interface EditorOptions {
  relPath?: string;
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
}

let activeInstance: Record<string, any> | null = null;

export function mountWikiEditor(target: HTMLElement, options: EditorOptions = {}) {
  closeWikiEditor();

  const config = options.config || (window as any).tmtWikiCms || {};

  if (options.isNew && !options.relPath) {
    activeInstance = mount(NewPageModal, {
      target,
      props: {
        onCreate: (path: string, initialContent: string) => {
          closeWikiEditor();
          activeInstance = mount(EditorModal, {
            target,
            props: {
              relPath: path,
              isNew: true,
              initialContent,
              config,
              onClose: () => closeWikiEditor(),
            },
          });
        },
        onCancel: () => closeWikiEditor(),
      },
    });
  } else {
    const relPath = options.relPath || "index.tmt";
    activeInstance = mount(EditorModal, {
      target,
      props: {
        relPath,
        isNew: options.isNew ?? false,
        initialContent: options.initialContent ?? "",
        config,
        onClose: () => closeWikiEditor(),
      },
    });
  }
}

export function closeWikiEditor() {
  if (activeInstance) {
    try {
      unmount(activeInstance);
    } catch (e) {
      console.warn("Failed to unmount editor instance:", e);
    }
    activeInstance = null;
  }
}

if (typeof window !== "undefined") {
  (window as any).__tmtWikiEditor = {
    mount: mountWikiEditor,
    close: closeWikiEditor,
    open: (target: HTMLElement, options?: EditorOptions) => mountWikiEditor(target, options),
  };
}
