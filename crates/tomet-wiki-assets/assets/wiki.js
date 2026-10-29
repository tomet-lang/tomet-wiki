// Tomet Wiki client script
(function() {
  function escapeHtml(str) {
    if (!str) return '';
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

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

  // 4. Header Authentication & User Menu
  const authContainer = document.getElementById('tmt-wiki-auth-container');

  function renderLoggedOut() {
    if (!authContainer) return;
    authContainer.innerHTML = '<button type="button" class="tmt-wiki-btn" id="tmt-wiki-login-btn">Log in</button>';
    const loginBtn = document.getElementById('tmt-wiki-login-btn');
    loginBtn?.addEventListener('click', () => {
      startOAuthLogin();
    });
  }

  function renderLoggedIn(user) {
    if (!authContainer) return;
    const avatarUrl = user?.avatar_url || '';
    const login = user?.login || 'user';
    const name = user?.name || login;

    authContainer.innerHTML = `
      <div class="tmt-wiki-user-wrapper">
        <button type="button" class="tmt-wiki-avatar-btn" id="tmt-wiki-user-btn" aria-haspopup="true" aria-expanded="false" title="@${escapeHtml(login)}">
          <img src="${escapeHtml(avatarUrl)}" alt="@${escapeHtml(login)}" class="tmt-wiki-avatar-img" />
        </button>
        <div class="tmt-user-dropdown" id="tmt-wiki-user-menu" role="menu" hidden>
          <div class="tmt-user-dropdown-header">
            <span class="tmt-user-name">${escapeHtml(name)}</span>
            <span class="tmt-user-handle">@${escapeHtml(login)}</span>
          </div>
          <div class="tmt-user-dropdown-divider"></div>
          <button type="button" class="tmt-user-dropdown-item tmt-logout-btn" id="tmt-wiki-logout-btn" role="menuitem">
            <svg class="tmt-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
            <span>Log out</span>
          </button>
        </div>
      </div>
    `;

    const userBtn = document.getElementById('tmt-wiki-user-btn');
    const userMenu = document.getElementById('tmt-wiki-user-menu');
    const logoutBtn = document.getElementById('tmt-wiki-logout-btn');

    function closeMenu() {
      if (userMenu && !userMenu.hidden) {
        userMenu.hidden = true;
        userBtn?.setAttribute('aria-expanded', 'false');
      }
    }

    function toggleMenu(e) {
      e.stopPropagation();
      if (!userMenu) return;
      const isHidden = userMenu.hidden;
      userMenu.hidden = !isHidden;
      userBtn?.setAttribute('aria-expanded', isHidden ? 'true' : 'false');
    }

    userBtn?.addEventListener('click', toggleMenu);

    document.addEventListener('click', (e) => {
      if (!authContainer.contains(e.target)) {
        closeMenu();
      }
    });

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        closeMenu();
      }
    });

    logoutBtn?.addEventListener('click', () => {
      logout();
    });
  }

  function logout() {
    localStorage.removeItem('tmt_github_token');
    localStorage.removeItem('tmt_github_user');
    document.documentElement.removeAttribute('data-auth');
    renderLoggedOut();
    window.dispatchEvent(
      new CustomEvent('tmt:auth:change', {
        detail: { token: null, user: null, authenticated: false },
      })
    );
    if (window.__tmtWikiEditor?.close) {
      window.__tmtWikiEditor.close();
    }
  }

  async function verifyAndRender(token) {
    const cms = window.tmtWikiCms || {};
    const apiBaseUrl = cms.apiBaseUrl || 'https://api.github.com';
    try {
      const res = await fetch(`${apiBaseUrl}/user`, {
        headers: {
          Authorization: `Bearer ${token}`,
          Accept: 'application/vnd.github.v3+json',
        },
      });
      if (res.ok) {
        const data = await res.json();
        const user = {
          login: data.login,
          avatar_url: data.avatar_url,
          name: data.name || data.login,
        };
        localStorage.setItem('tmt_github_user', JSON.stringify(user));
        renderLoggedIn(user);
        return user;
      } else if (res.status === 401 || res.status === 403) {
        logout();
      }
    } catch (err) {
      console.warn('Could not verify GitHub user:', err);
    }
    return null;
  }

  function startOAuthLogin() {
    const cms = window.tmtWikiCms || {};
    const authEndpoint = cms.authEndpoint || 'https://auth.tomet.dev/github';
    const domain = cms.siteDomain || window.location.hostname;
    const url = `${authEndpoint}?site_id=${encodeURIComponent(domain)}`;
    const width = 600;
    const height = 700;
    const left = window.screenX + (window.outerWidth - width) / 2;
    const top = window.screenY + (window.outerHeight - height) / 2;

    const popup = window.open(
      url,
      'tomet_oauth_popup',
      `width=${width},height=${height},left=${left},top=${top},toolbar=no,menubar=no`
    );

    if (!popup) {
      alert('Popup blocked by browser. Please allow popups for this site.');
      return;
    }

    let channel = null;
    try {
      channel = new BroadcastChannel('tomet_auth');
    } catch (e) {}

    function cleanup() {
      window.removeEventListener('message', onMessage);
      if (channel) channel.close();
      clearInterval(pollTimer);
    }

    async function onTokenReceived(token) {
      cleanup();
      localStorage.setItem('tmt_github_token', token);
      document.documentElement.setAttribute('data-auth', 'true');
      const user = await verifyAndRender(token);
      window.dispatchEvent(
        new CustomEvent('tmt:auth:change', {
          detail: { token, user, authenticated: true },
        })
      );
    }

    function onMessage(e) {
      if (e.data === 'authorizing:github') {
        popup?.postMessage('tomet:handshake', '*');
        return;
      }
      if (typeof e.data === 'string' && e.data.startsWith('authorization:github:success:')) {
        const jsonStr = e.data.slice('authorization:github:success:'.length);
        try {
          const parsed = JSON.parse(jsonStr);
          const token = parsed.token || parsed.bearer;
          if (token) onTokenReceived(token);
        } catch {
          onTokenReceived(jsonStr);
        }
      } else if (e.data && typeof e.data === 'object') {
        if (e.data.type === 'authorization:github:success' && (e.data.token || e.data.bearer)) {
          onTokenReceived(e.data.token || e.data.bearer);
        }
      }
    }

    window.addEventListener('message', onMessage);
    if (channel) {
      channel.onmessage = (e) => {
        if (e.data && typeof e.data === 'object' && (e.data.token || e.data.bearer)) {
          onTokenReceived(e.data.token || e.data.bearer);
        }
      };
    }

    const pollTimer = setInterval(() => {
      if (popup.closed) {
        cleanup();
      }
    }, 500);
  }

  // Cross-component auth synchronization
  window.addEventListener('tmt:auth:change', (e) => {
    if (e.detail?.authenticated && e.detail.token) {
      document.documentElement.setAttribute('data-auth', 'true');
      if (e.detail.user) {
        renderLoggedIn(e.detail.user);
      } else {
        verifyAndRender(e.detail.token);
      }
    } else if (e.detail && !e.detail.authenticated) {
      document.documentElement.removeAttribute('data-auth');
      renderLoggedOut();
    }
  });

  // Initialize auth UI on page load
  if (authContainer) {
    const token = localStorage.getItem('tmt_github_token');
    if (token) {
      document.documentElement.setAttribute('data-auth', 'true');
      let cachedUser = null;
      try {
        const raw = localStorage.getItem('tmt_github_user');
        if (raw) cachedUser = JSON.parse(raw);
      } catch (e) {}
      if (cachedUser) {
        renderLoggedIn(cachedUser);
      } else {
        // Fallback placeholder while verifying
        authContainer.innerHTML = '<div class="tmt-wiki-avatar-btn" style="opacity:0.5;background:var(--tmt-bg-subtle)"></div>';
      }
      verifyAndRender(token);
    } else {
      document.documentElement.removeAttribute('data-auth');
      renderLoggedOut();
    }
  }
})();
