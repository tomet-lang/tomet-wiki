// Tomet Wiki client script
(function() {
  // 1. Theme toggle
  const themeToggle = document.getElementById('tmt-wiki-theme-toggle');
  if (themeToggle) {
    themeToggle.addEventListener('click', () => {
      const current = document.documentElement.getAttribute('data-theme') || 'light';
      const next = current === 'dark' ? 'light' : 'dark';
      document.documentElement.setAttribute('data-theme', next);
      localStorage.setItem('tmt-wiki-theme', next);
    });
  }

  // 2. Search trigger & key shortcut ("/")
  const searchTrigger = document.getElementById('tmt-wiki-search-trigger');
  if (searchTrigger) {
    searchTrigger.addEventListener('click', () => {
      window.dispatchEvent(new CustomEvent('tmt:wiki:search'));
    });
  }

  window.addEventListener('keydown', (e) => {
    if (e.key === '/' && !['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName)) {
      e.preventDefault();
      searchTrigger?.click();
    }
  });

  // 3. Edit & New Page button handlers: dynamically load & mount Svelte editor
  async function loadAndMountEditor(options) {
    const root = document.getElementById('tmt-wiki-editor-root');
    if (!root) return;

    try {
      if (!window.__tmtWikiEditor) {
        const basePath = window.tmtBasePath || '';
        const scriptUrl = `${basePath}/wiki-editor.js`.replace(/\/+/g, '/');
        await import(scriptUrl);
      }
      if (window.__tmtWikiEditor) {
        window.__tmtWikiEditor.open(root, options);
      }
    } catch (err) {
      console.error('Failed to load wiki editor:', err);
    }
  }

  const editBtn = document.getElementById('tmt-wiki-edit-btn');
  if (editBtn) {
    editBtn.addEventListener('click', () => {
      const relPath = editBtn.getAttribute('data-rel-path') ||
                      document.querySelector('main[data-tmt-rel-path]')?.getAttribute('data-tmt-rel-path');
      window.dispatchEvent(new CustomEvent('tmt:wiki:edit', { detail: { relPath } }));
      loadAndMountEditor({ relPath, isNew: false });
    });
  }

  const newBtn = document.getElementById('tmt-wiki-new-btn');
  if (newBtn) {
    newBtn.addEventListener('click', () => {
      window.dispatchEvent(new CustomEvent('tmt:wiki:new'));
      loadAndMountEditor({ isNew: true });
    });
  }

  // Hotkey: Ctrl+E / Cmd+E to edit page
  window.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && (e.key === 'e' || e.key === 'E') && !e.shiftKey) {
      const tag = document.activeElement?.tagName;
      if (tag !== 'INPUT' && tag !== 'TEXTAREA') {
        e.preventDefault();
        editBtn?.click();
      }
    }
  });
})();
