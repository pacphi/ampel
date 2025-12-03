# OAuth Provider Setup Guide

This guide walks you through setting up OAuth applications for GitHub, GitLab, and Bitbucket to use with Ampel. By the end, you'll have the client IDs, secrets, and redirect URLs configured in your `.env` file.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Understanding OAuth](#understanding-oauth)
- [GitHub OAuth Setup](#github-oauth-setup)
- [GitLab OAuth Setup](#gitlab-oauth-setup)
- [Bitbucket OAuth Setup](#bitbucket-oauth-setup)
- [Environment Configuration](#environment-configuration)
- [Security Best Practices](#security-best-practices)
- [Troubleshooting](#troubleshooting)
- [Additional Resources](#additional-resources)

---

## Prerequisites

Before you begin, make sure you have:

- Accounts on the platforms you want to integrate (GitHub, GitLab, and/or Bitbucket)
- Admin access to create OAuth applications (personal account or organization)
- Ampel running locally or deployed (you'll need the callback URLs)

## Understanding OAuth

OAuth allows Ampel to access your repositories and pull requests without storing your password. Instead, you create an "OAuth application" on each platform, which gives you:

| Term | Also Called | Description |
|------|-------------|-------------|
| **Client ID** | Application ID, Key | Public identifier for your app |
| **Client Secret** | Secret | Private key (keep this secure!) |
| **Redirect URI** | Callback URL | Where users return after authorizing |

---

## GitHub OAuth Setup

### Step 1: Navigate to Developer Settings

1. Log in to [GitHub](https://github.com)
2. Click your **profile picture** (top-right corner)
3. Select **Settings**
4. Scroll down and click **Developer settings** (left sidebar)
5. Click **OAuth Apps**
6. Click **New OAuth App** (or "Register a new application")

### Step 2: Fill in Application Details

| Field | Value |
|-------|-------|
| **Application name** | `Ampel` (or your preferred name) |
| **Homepage URL** | `http://localhost:5173` (development) or your production URL |
| **Application description** | Optional: "PR management dashboard" |
| **Authorization callback URL** | `http://localhost:8080/api/oauth/github/callback` |

> **Note:** GitHub only allows ONE callback URL per OAuth app. For production, you'll need to create a separate OAuth app with your production callback URL.

### Step 3: Register and Get Credentials

1. Click **Register application**
2. You'll see your **Client ID** immediately
3. Click **Generate a new client secret**
4. **Copy the secret immediately** - you won't see it again!

### Step 4: Copy to Your .env File

```bash
GITHUB_CLIENT_ID=your-client-id-here
GITHUB_CLIENT_SECRET=your-client-secret-here
GITHUB_REDIRECT_URI=http://localhost:8080/api/oauth/github/callback
```

### GitHub Scopes Used by Ampel

Ampel requests these scopes during authorization:

| Scope | Purpose |
|-------|---------|
| `repo` | Access repositories and pull requests |
| `read:user` | Read user profile information |
| `read:org` | List organizations the user belongs to |

---

## GitLab OAuth Setup

GitLab allows creating applications at three levels: user, group, or instance-wide.

### Step 1: Navigate to Applications

**For a personal application:**
1. Log in to [GitLab](https://gitlab.com) (or your self-hosted instance)
2. Click your **avatar** (top-left)
3. Select **Edit profile**
4. Click **Applications** (left sidebar)
5. Click **Add new application**

**For a group application:**
1. Go to your group
2. Click **Settings** > **Applications**
3. Click **Add new application**

### Step 2: Fill in Application Details

| Field | Value |
|-------|-------|
| **Name** | `Ampel` |
| **Redirect URI** | `http://localhost:8080/api/oauth/gitlab/callback` |
| **Confidential** | Check this box (recommended for server-side apps) |

### Step 3: Select Scopes

Check the following scopes:

| Scope | Purpose |
|-------|---------|
| `api` | Full API access (for repositories and MRs) |
| `read_user` | Read user profile |
| `read_repository` | Read repository contents |
| `openid` | OpenID Connect authentication |

### Step 4: Save and Get Credentials

1. Click **Save application**
2. Copy the **Application ID** (this is your Client ID)
3. Click **Copy** next to the **Secret** field
4. **Save these immediately** - GitLab hashes secrets after creation

### Step 5: Copy to Your .env File

```bash
GITLAB_CLIENT_ID=your-application-id-here
GITLAB_CLIENT_SECRET=your-secret-here
GITLAB_REDIRECT_URI=http://localhost:8080/api/oauth/gitlab/callback

# For self-hosted GitLab, also add:
# GITLAB_BASE_URL=https://gitlab.yourcompany.com
```

### Renewing a Lost Secret (GitLab 15.9+)

If you lose your secret:
1. Go back to **Applications**
2. Find your application
3. Click **Renew secret**
4. Update your `.env` file with the new secret

---

## Bitbucket OAuth Setup

Bitbucket uses "OAuth consumers" which are created at the workspace level.

### Step 1: Navigate to OAuth Consumers

1. Log in to [Bitbucket](https://bitbucket.org)
2. Click your **avatar** (bottom-left)
3. Select your **workspace** (or choose from "All workspaces")
4. Click the **Settings** cog (top navigation)
5. Select **Workspace settings**
6. Click **OAuth consumers** (left sidebar, under "Apps and features")
7. Click **Add consumer**

**Quick URL:** Go directly to `https://bitbucket.org/{your_workspace}/workspace/settings/api`

### Step 2: Fill in Consumer Details

| Field | Value |
|-------|-------|
| **Name** | `Ampel` |
| **Description** | Optional: "PR management dashboard" |
| **Callback URL** | `http://localhost:8080/api/oauth/bitbucket/callback` |
| **URL** | Optional: Your app homepage |

### Step 3: Select Permissions

Under **Permissions**, check these scopes:

| Category | Permission | Purpose |
|----------|------------|---------|
| **Account** | Read | Read user profile |
| **Repositories** | Read | List repositories |
| **Pull requests** | Read | Access pull request data |

For write access (commenting, approving):
| Category | Permission | Purpose |
|----------|------------|---------|
| **Pull requests** | Write | Comment, approve, merge PRs |

### Step 4: Save and Get Credentials

1. Click **Save**
2. Click on your consumer name to expand it
3. Copy the **Key** (this is your Client ID)
4. Copy the **Secret**

### Step 5: Copy to Your .env File

```bash
BITBUCKET_CLIENT_ID=your-key-here
BITBUCKET_CLIENT_SECRET=your-secret-here
BITBUCKET_REDIRECT_URI=http://localhost:8080/api/oauth/bitbucket/callback
```

---

## Environment Configuration

### Complete .env Example

Here's what your OAuth section should look like:

```bash
# GitHub OAuth
GITHUB_CLIENT_ID=Iv1.abc123def456
GITHUB_CLIENT_SECRET=abc123def456789...
GITHUB_REDIRECT_URI=http://localhost:8080/api/oauth/github/callback

# GitLab OAuth
GITLAB_CLIENT_ID=abc123def456789...
GITLAB_CLIENT_SECRET=gloas-abc123...
GITLAB_REDIRECT_URI=http://localhost:8080/api/oauth/gitlab/callback
# GITLAB_BASE_URL=https://gitlab.yourcompany.com  # Uncomment for self-hosted

# Bitbucket OAuth
BITBUCKET_CLIENT_ID=abc123DEF456
BITBUCKET_CLIENT_SECRET=abc123def456789...
BITBUCKET_REDIRECT_URI=http://localhost:8080/api/oauth/bitbucket/callback
```

### Production Configuration

For production deployments, update your redirect URIs:

```bash
GITHUB_REDIRECT_URI=https://app.example.com/api/oauth/github/callback
GITLAB_REDIRECT_URI=https://app.example.com/api/oauth/gitlab/callback
BITBUCKET_REDIRECT_URI=https://app.example.com/api/oauth/bitbucket/callback
```

Remember: GitHub requires separate OAuth apps for development and production URLs.

---

## Security Best Practices

### Protecting Your Secrets

1. **Never commit secrets to version control**
   - Add `.env` to your `.gitignore` (already done in this project)
   - Use `.env.example` with placeholder values for documentation

2. **Use different credentials per environment**
   - Create separate OAuth apps for development, staging, and production
   - Each environment should have its own client ID/secret

3. **Rotate secrets periodically**
   - GitHub: Generate new secret, update app, delete old secret
   - GitLab: Use "Renew secret" feature (v15.9+)
   - Bitbucket: Delete consumer and recreate

4. **Use environment variables or secret managers**
   - Development: `.env` files
   - Production: Use secret managers (AWS Secrets Manager, HashiCorp Vault, etc.)

### Minimum Required Scopes

Only request the scopes your application actually needs:

| Provider | Minimum Scopes for PR Dashboard |
|----------|----------------------------------|
| GitHub | `repo`, `read:user` |
| GitLab | `api`, `read_user` |
| Bitbucket | Account:Read, Repositories:Read, Pull requests:Read |

### Handling Compromised Secrets

If a secret is exposed:

1. **GitHub**: Generate new secret immediately, revoke any issued tokens
2. **GitLab**: Use "Renew secret" to invalidate the old one
3. **Bitbucket**: Delete the consumer and create a new one
4. Update your `.env` files and redeploy

---

## Troubleshooting

### Common Issues

**"Redirect URI mismatch" error**
- The callback URL in your `.env` must exactly match what's registered in the OAuth app
- Check for trailing slashes, http vs https, and port numbers

**"Invalid client" error**
- Verify your Client ID is correct (copy-paste to avoid typos)
- Ensure the OAuth app hasn't been deleted

**"Invalid client secret" error**
- Secrets are shown only once - if lost, generate a new one
- Check for extra whitespace when copying

**OAuth app not appearing**
- GitHub: Check you're looking at the right account/organization
- GitLab: Ensure you have the correct permissions on the group/instance
- Bitbucket: OAuth consumers are per-workspace, not per-account

### Verifying Your Setup

Test each provider's OAuth flow:
1. Start Ampel with `make dev-api` and `make dev-frontend`
2. Navigate to the login page
3. Click each provider's connect button
4. Complete the authorization flow
5. Verify you return to Ampel successfully

---

## Additional Resources

### Official Documentation

**GitHub:**
- [Creating an OAuth App](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/creating-an-oauth-app)
- [Best Practices for OAuth Apps](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/best-practices-for-creating-an-oauth-app)
- [Authenticating with OAuth Apps](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authenticating-to-the-rest-api-with-an-oauth-app)
- [OAuth Scopes](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/scopes-for-oauth-apps)

**GitLab:**
- [Configure GitLab as OAuth Provider](https://docs.gitlab.com/integration/oauth_provider/)
- [OAuth 2.0 API](https://docs.gitlab.com/api/oauth2/)
- [Application Scopes](https://docs.gitlab.com/ee/integration/oauth_provider.html#configure-as-an-oauth-20-provider)

**Bitbucket:**
- [Use OAuth on Bitbucket Cloud](https://support.atlassian.com/bitbucket-cloud/docs/use-oauth-on-bitbucket-cloud/)
- [OAuth 2.0 Documentation](https://developer.atlassian.com/cloud/bitbucket/oauth-2/)
- [REST API Scopes](https://developer.atlassian.com/cloud/bitbucket/bitbucket-cloud-rest-api-scopes/)

### General OAuth Resources

- [OAuth 2.0 Simplified](https://www.oauth.com/)
- [IETF OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)

---

## Quick Reference Card

| Provider | Settings Location | Callback URL Format |
|----------|-------------------|---------------------|
| GitHub | Settings > Developer settings > OAuth Apps | `/api/oauth/github/callback` |
| GitLab | Profile > Applications | `/api/oauth/gitlab/callback` |
| Bitbucket | Workspace > Settings > OAuth consumers | `/api/oauth/bitbucket/callback` |

| Provider | Client ID Field | Secret Field |
|----------|-----------------|--------------|
| GitHub | Client ID | Client Secret |
| GitLab | Application ID | Secret |
| Bitbucket | Key | Secret |
