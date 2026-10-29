/**
 * Tomet Wiki - Lightweight GitHub OAuth Proxy for Cloudflare Workers
 *
 * Exchanges GitHub OAuth authorization codes for access tokens with zero dependencies.
 *
 * Configuration:
 * - CLIENT_ID: GitHub OAuth App Client ID (set via `wrangler secret put CLIENT_ID`)
 * - CLIENT_SECRET: GitHub OAuth App Client Secret (set via `wrangler secret put CLIENT_SECRET`)
 */

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    // Handle CORS preflight
    if (request.method === "OPTIONS") {
      return new Response(null, {
        status: 204,
        headers: {
          "Access-Control-Allow-Origin": "*",
          "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
          "Access-Control-Allow-Headers": "Content-Type, Authorization",
        },
      });
    }

    if (!env.CLIENT_ID || !env.CLIENT_SECRET) {
      return new Response(
        "Missing CLIENT_ID or CLIENT_SECRET environment secrets in Cloudflare Worker.",
        { status: 500, headers: { "Content-Type": "text/plain" } }
      );
    }

    // Step 2: GitHub redirects back with ?code=...&state=...
    const code = url.searchParams.get("code");
    if (code) {
      try {
        const tokenRes = await fetch("https://github.com/login/oauth/access_token", {
          method: "POST",
          headers: {
            "Accept": "application/json",
            "Content-Type": "application/json",
            "User-Agent": "tomet-auth-proxy",
          },
          body: JSON.stringify({
            client_id: env.CLIENT_ID,
            client_secret: env.CLIENT_SECRET,
            code,
          }),
        });

        const data = await tokenRes.json();

        if (data.error || !data.access_token) {
          const errDesc = data.error_description || data.error || "Failed to exchange token";
          return renderHtml(`
            <h2>Authentication Failed</h2>
            <p>${escapeHtml(errDesc)}</p>
          `);
        }

        const accessToken = data.access_token;

        return renderHtml(`
          <h2>Authorization Successful</h2>
          <p>Connecting back to Tomet Wiki...</p>
          <script>
            (function() {
              const token = ${JSON.stringify(accessToken)};
              const payload = { type: "authorization:github:success", token: token };
              const rawStr = "authorization:github:success:" + JSON.stringify(payload);

              // 1. Post to opener window
              if (window.opener) {
                window.opener.postMessage(rawStr, "*");
                window.opener.postMessage(payload, "*");
              }

              // 2. BroadcastChannel for cross-tab / modern browsers
              try {
                const channel = new BroadcastChannel("tomet_auth");
                channel.postMessage(payload);
                channel.close();
              } catch (e) {}

              // Close popup after notifying opener
              setTimeout(function() {
                window.close();
              }, 400);
            })();
          </script>
        `);
      } catch (err) {
        return renderHtml(`
          <h2>Authentication Error</h2>
          <p>${escapeHtml(err.message || "Unknown error occurred")}</p>
        `);
      }
    }

    // Step 1: Initiate OAuth flow - redirect to GitHub
    const siteId = url.searchParams.get("site_id") || "";
    const state = siteId ? encodeURIComponent(siteId) : Math.random().toString(36).substring(2);
    const redirectUri = `${url.origin}${url.pathname}`;
    const githubAuthUrl = `https://github.com/login/oauth/authorize?client_id=${encodeURIComponent(
      env.CLIENT_ID
    )}&scope=repo&state=${state}&redirect_uri=${encodeURIComponent(redirectUri)}`;

    return Response.redirect(githubAuthUrl, 302);
  },
};

function escapeHtml(str) {
  if (!str) return "";
  return String(str)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function renderHtml(bodyContent) {
  const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Tomet Auth</title>
  <style>
    body {
      font-family: system-ui, -apple-system, sans-serif;
      margin: 0;
      padding: 2rem;
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      background: #0d1117;
      color: #e6edf3;
      box-sizing: border-box;
    }
    .card {
      background: #161b22;
      border: 1px solid #30363d;
      border-radius: 12px;
      padding: 2rem;
      max-width: 420px;
      width: 100%;
      text-align: center;
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    }
    h2 { margin: 0 0 0.5rem; font-size: 1.25rem; }
    p { margin: 0; color: #8b949e; font-size: 0.9rem; line-height: 1.5; }
  </style>
</head>
<body>
  <div class="card">
    ${bodyContent}
  </div>
</body>
</html>`;

  return new Response(html, {
    status: 200,
    headers: {
      "Content-Type": "text/html; charset=utf-8",
      "Cache-Control": "no-store",
    },
  });
}
