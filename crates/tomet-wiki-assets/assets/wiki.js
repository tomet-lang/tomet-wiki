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

  // 3. Edit & New Page button event dispatchers for inline Svelte editor
  const editBtn = document.getElementById('tmt-wiki-edit-btn');
  if (editBtn) {
    editBtn.addEventListener('click', () => {
      const relPath = editBtn.getAttribute('data-rel-path');
      window.dispatchEvent(new CustomEvent('tmt:wiki:edit', { detail: { relPath } }));
    });
  }

  const newBtn = document.getElementById('tmt-wiki-new-btn');
  if (newBtn) {
    newBtn.addEventListener('click', () => {
      window.dispatchEvent(new CustomEvent('tmt:wiki:new'));
    });
  }
})();
