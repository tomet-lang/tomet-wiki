# Tomet Wiki - Cloudflare Worker OAuth Proxy

A lightweight, zero-dependency Cloudflare Worker that acts as a secure GitHub OAuth gateway for `tomet-wiki`.

By default, `tomet-wiki` points to the public gateway `https://auth.tomet.dev/github`. If you prefer to self-host your own private gateway or use GitHub Enterprise, deploy this Worker to your Cloudflare account.

## How it works

1. The client opens a popup to `https://<your-worker-domain>/github?site_id=<wiki-domain>`.
2. The Worker redirects the user to GitHub's OAuth authorization page.
3. Once authorized by the user, GitHub redirects back to the Worker with an authorization `code`.
4. The Worker securely exchanges `code` + `CLIENT_SECRET` with GitHub for an access token.
5. The Worker returns a lightweight HTML response that transmits the token back to Tomet Wiki via `window.opener.postMessage` and `BroadcastChannel("tomet_auth")`.

## Deployment

### 1. Register a GitHub OAuth App

1. Go to **GitHub Settings** > **Developer settings** > **OAuth Apps** > **New OAuth App**.
2. Set **Application name**: e.g., `My Wiki Auth`.
3. Set **Homepage URL**: Your Wiki's public URL (e.g., `https://wiki.example.com`).
4. Set **Authorization callback URL**:
   `https://<your-worker-subdomain>.workers.dev/github`
5. Click **Register application**.
6. Note the **Client ID**, and click **Generate a new client secret** to generate the **Client Secret**.

### 2. Configure Cloudflare Worker Secrets

From this directory (`tools/auth-proxy`):

```bash
npx wrangler secret put CLIENT_ID
# Enter your GitHub OAuth App Client ID

npx wrangler secret put CLIENT_SECRET
# Enter your GitHub OAuth App Client Secret
```

### 3. Deploy

Deploy the Worker with Wrangler:

```bash
npx wrangler deploy worker.js --name tomet-auth-proxy
```

### 4. Configure `wiki.toml`

Point your wiki configuration to your new worker endpoint:

```toml
[cms]
owner = "your-github-user"
repo = "your-wiki-repo"
branch = "main"
auth_endpoint = "https://tomet-auth-proxy.<your-subdomain>.workers.dev/github"
```
