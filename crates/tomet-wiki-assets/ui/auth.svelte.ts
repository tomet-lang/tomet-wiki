export interface GitHubUser {
  login: string;
  avatar_url: string;
  name?: string;
}

class AuthStore {
  token = $state<string | null>(null);
  user = $state<GitHubUser | null>(null);
  loading = $state<boolean>(false);
  error = $state<string | null>(null);

  constructor() {
    if (typeof window !== "undefined") {
      this.token = localStorage.getItem("tmt_github_token");
      const cached = localStorage.getItem("tmt_github_user");
      if (cached) {
        try {
          this.user = JSON.parse(cached);
        } catch {
          // ignore
        }
      }
      if (this.token) {
        this.verifyToken(this.token).catch(() => {
          this.logout();
        });
      }
    }
  }

  async verifyToken(token: string, apiBaseUrl = "https://api.github.com"): Promise<GitHubUser> {
    this.loading = true;
    this.error = null;
    try {
      const res = await fetch(`${apiBaseUrl}/user`, {
        headers: {
          Authorization: `Bearer ${token}`,
          Accept: "application/vnd.github.v3+json",
        },
      });
      if (!res.ok) {
        throw new Error(`Authentication failed (${res.status} ${res.statusText})`);
      }
      const data = await res.json();
      const user: GitHubUser = {
        login: data.login,
        avatar_url: data.avatar_url,
        name: data.name || data.login,
      };
      this.token = token;
      this.user = user;
      localStorage.setItem("tmt_github_token", token);
      localStorage.setItem("tmt_github_user", JSON.stringify(user));
      if (typeof document !== "undefined") {
        document.documentElement.setAttribute("data-auth", "true");
        window.dispatchEvent(
          new CustomEvent("tmt:auth:change", {
            detail: { token, user, authenticated: true },
          }),
        );
      }
      return user;
    } catch (err: any) {
      this.error = err.message || "Failed to verify GitHub token";
      throw err;
    } finally {
      this.loading = false;
    }
  }

  async loginWithPat(pat: string, apiBaseUrl?: string): Promise<boolean> {
    const cleanToken = pat.trim();
    if (!cleanToken) {
      this.error = "Token cannot be empty";
      return false;
    }
    try {
      await this.verifyToken(cleanToken, apiBaseUrl);
      return true;
    } catch {
      return false;
    }
  }

  loginWithOAuth(authEndpoint?: string, siteDomain?: string): Promise<string> {
    return new Promise((resolve, reject) => {
      this.loading = true;
      this.error = null;

      const endpoint =
        authEndpoint ||
        (typeof window !== "undefined" && (window as any).tmtWikiCms?.authEndpoint) ||
        "https://auth.tomet.dev/github";
      const domain =
        siteDomain ||
        (typeof window !== "undefined" ? window.location.hostname : "");
      const url = `${endpoint}?site_id=${encodeURIComponent(domain)}`;
      const width = 600;
      const height = 700;
      const left = window.screenX + (window.outerWidth - width) / 2;
      const top = window.screenY + (window.outerHeight - height) / 2;

      const popup = window.open(
        url,
        "tomet_oauth_popup",
        `width=${width},height=${height},left=${left},top=${top},toolbar=no,menubar=no`,
      );

      if (!popup) {
        this.loading = false;
        this.error = "Popup blocked by browser. Please allow popups or use a PAT.";
        return reject(new Error(this.error));
      }

      let channel: BroadcastChannel | null = null;
      try {
        channel = new BroadcastChannel("tomet_auth");
      } catch {
        // BroadcastChannel may not be available in some private contexts
      }

      const cleanup = () => {
        window.removeEventListener("message", onMessage);
        if (channel) {
          channel.close();
        }
        clearInterval(pollTimer);
        this.loading = false;
      };

      const handleToken = async (rawToken: string) => {
        cleanup();
        try {
          await this.verifyToken(rawToken);
          resolve(rawToken);
        } catch (e) {
          reject(e);
        }
      };

      const onMessage = (e: MessageEvent) => {
        // Handshake
        if (e.data === "authorizing:github") {
          popup?.postMessage("tomet:handshake", "*");
          return;
        }

        // OAuth success
        if (typeof e.data === "string" && e.data.startsWith("authorization:github:success:")) {
          const jsonStr = e.data.slice("authorization:github:success:".length);
          try {
            const parsed = JSON.parse(jsonStr);
            const token = parsed.token || parsed.bearer;
            if (token) {
              handleToken(token);
            }
          } catch {
            // If raw token string
            handleToken(jsonStr);
          }
        } else if (e.data && typeof e.data === "object") {
          if (e.data.type === "authorization:github:success" && (e.data.token || e.data.bearer)) {
            handleToken(e.data.token || e.data.bearer);
          }
        }
      };

      window.addEventListener("message", onMessage);

      if (channel) {
        channel.onmessage = (e: MessageEvent) => {
          if (e.data && typeof e.data === "object" && (e.data.token || e.data.bearer)) {
            handleToken(e.data.token || e.data.bearer);
          }
        };
      }

      const pollTimer = setInterval(() => {
        if (popup.closed) {
          cleanup();
          if (!this.token) {
            reject(new Error("Authentication window was closed"));
          }
        }
      }, 500);
    });
  }

  logout() {
    this.token = null;
    this.user = null;
    this.error = null;
    if (typeof window !== "undefined") {
      localStorage.removeItem("tmt_github_token");
      localStorage.removeItem("tmt_github_user");
      document.documentElement.removeAttribute("data-auth");
      window.dispatchEvent(
        new CustomEvent("tmt:auth:change", {
          detail: { token: null, user: null, authenticated: false },
        }),
      );
    }
  }
}

export const authStore = new AuthStore();
