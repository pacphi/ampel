# Personal Access Token (PAT) Setup Guide

This guide walks you through creating Personal Access Tokens (PATs) for GitHub, GitLab, and Bitbucket to use with Ampel.
By the end, you'll be able to add provider accounts directly through the Ampel UI without needing OAuth configuration.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Understanding PATs](#understanding-pats)
- [GitHub PAT Setup](#github-pat-setup)
- [GitLab PAT Setup](#gitlab-pat-setup)
- [Bitbucket PAT Setup](#bitbucket-app-password-setup)
- [Adding Accounts to Ampel](#adding-accounts-to-ampel)
- [Security Best Practices](#security-best-practices)
- [Troubleshooting](#troubleshooting)
- [Additional Resources](#additional-resources)

---

## Prerequisites

Before you begin, make sure you have:

- Accounts on the platforms you want to integrate (GitHub, GitLab, and/or Bitbucket)
- Access to create PATs/app passwords (available on all account types)
- Ampel running and accessible (local or deployed)

## Understanding PATs

Personal Access Tokens (PATs) are a secure way to authenticate with Git providers without sharing your password.
They offer several advantages:

| Advantage                    | Description                                             |
| ---------------------------- | ------------------------------------------------------- |
| **No OAuth Setup**           | No need to register applications or configure callbacks |
| **Fine-grained Permissions** | Grant only the access your application needs            |
| **Easily Revocable**         | Revoke a token instantly if compromised                 |
| **Multiple Tokens**          | Create separate tokens for different purposes           |
| **Expiration Options**       | Set automatic expiration for enhanced security          |

---

## GitHub PAT Setup

### Step 1: Navigate to Token Settings

1. Log in to [GitHub](https://github.com)
2. Click your **profile picture** (top-right corner)
3. Select **Settings**
4. Scroll down and click **Developer settings** (left sidebar)
5. Click **Personal access tokens**
6. Choose **Tokens (classic)** or **Fine-grained tokens** (recommended)

### Step 2: Create a Fine-Grained Token (Recommended)

1. Click **Generate new token** > **Generate new token (fine-grained)**
2. Fill in the token details:

| Field                 | Value                                                       |
| --------------------- | ----------------------------------------------------------- |
| **Token name**        | `Ampel PR Dashboard`                                        |
| **Expiration**        | 90 days (or custom - recommended for security)              |
| **Resource owner**    | Select your account or organization                         |
| **Repository access** | Select repositories (choose repos you want Ampel to access) |

3. Under **Permissions**, configure:

**Repository permissions:**
| Permission | Access Level | Purpose |
| ---------- | ------------ | ------- |
| **Pull requests** | Read and write | View and manage pull requests |
| **Contents** | Read-only | Access repository files |
| **Metadata** | Read-only | Access repository metadata |
| **Issues** | Read-only | View linked issues |

**Account permissions:**
| Permission | Access Level | Purpose |
| ---------- | ------------ | ------- |
| **Email addresses** | Read-only | Read user email |

### Step 3: Create a Classic Token (Alternative)

If you prefer classic tokens:

1. Click **Generate new token** > **Generate new token (classic)**
2. Fill in the details:

| Field          | Value                 |
| -------------- | --------------------- |
| **Note**       | `Ampel PR Dashboard`  |
| **Expiration** | 90 days (recommended) |

3. Select scopes:

| Scope        | Purpose                                                       |
| ------------ | ------------------------------------------------------------- |
| `repo`       | Full control of private repositories (includes pull requests) |
| `read:org`   | Read organization membership                                  |
| `read:user`  | Read user profile data                                        |
| `user:email` | Read user email addresses                                     |

### Step 4: Generate and Copy Token

1. Click **Generate token**
2. **Copy the token immediately** - you won't see it again!
3. Store it securely (you'll paste it into Ampel in the next section)

**Quick URL:** `https://github.com/settings/tokens/new`

---

## GitLab PAT Setup

### Step 1: Navigate to Access Tokens

1. Log in to [GitLab](https://gitlab.com) (or your self-hosted instance)
2. Click your **avatar** (top-right)
3. Select **Edit profile**
4. Click **Access Tokens** (left sidebar)
5. Scroll to **Add new token**

**Quick URL:** `https://gitlab.com/-/profile/personal_access_tokens`

### Step 2: Create the Token

Fill in the token details:

| Field               | Value                          |
| ------------------- | ------------------------------ |
| **Token name**      | `Ampel PR Dashboard`           |
| **Expiration date** | 90 days from now (recommended) |

### Step 3: Select Scopes

Check the following scopes:

| Scope             | Purpose                                                                |
| ----------------- | ---------------------------------------------------------------------- |
| `api`             | Full API access (required for repositories and merge requests)         |
| `read_user`       | Read user profile information                                          |
| `read_api`        | Read-only API access (alternative to `api` if read-only is sufficient) |
| `read_repository` | Read repository contents                                               |

**For read-only access:** Use `read_api` and `read_repository` instead of `api`.

**For write access (merge, comment):** Include `api` scope.

### Step 4: Generate and Copy Token

1. Click **Create personal access token**
2. **Copy the token immediately** - GitLab shows it only once!
3. Store it securely

### Self-Hosted GitLab

For self-hosted GitLab instances, the process is identical. You'll also need to configure the base URL in Ampel when adding the account.

---

## Bitbucket App Password Setup

Bitbucket uses "App Passwords" instead of PATs. They work similarly but are created differently.

### Step 1: Navigate to App Passwords

1. Log in to [Bitbucket](https://bitbucket.org)
2. Click your **avatar** (bottom-left)
3. Select **Personal settings**
4. Click **App passwords** (left sidebar, under "Access management")
5. Click **Create app password**

**Quick URL:** `https://bitbucket.org/account/settings/app-passwords/new`

### Step 2: Create the App Password

| Field     | Value                |
| --------- | -------------------- |
| **Label** | `Ampel PR Dashboard` |

### Step 3: Select Permissions

Check the following permissions:

**Account:**
| Permission | Purpose |
| ---------- | ------- |
| **Read** | Read user profile information |

**Workspace membership:**
| Permission | Purpose |
| ---------- | ------- |
| **Read** | Access workspace details |

**Repositories:**
| Permission | Purpose |
| ---------- | ------- |
| **Read** | View repositories |

**Pull requests:**
| Permission | Purpose |
| ---------- | ------- |
| **Read** | View pull requests |
| **Write** | Comment, approve, merge PRs (optional) |

### Step 4: Generate and Copy Password

1. Click **Create**
2. **Copy the app password immediately** - you won't see it again!
3. Store it securely

**Note:** You'll use your Bitbucket username along with the app password when adding the account to Ampel.

---

## Adding Accounts to Ampel

Once you have your tokens/app passwords, add them to Ampel:

### Using the Ampel UI

1. Log in to Ampel
2. Navigate to **Settings** > **Provider Accounts**
3. Click **Add Account**
4. Select the provider (GitHub, GitLab, or Bitbucket)
5. Fill in the form:

**GitHub:**
| Field | Value |
| ----- | ----- |
| **Account Name** | A friendly name (e.g., "My GitHub") |
| **Personal Access Token** | Paste your GitHub token |

**GitLab:**
| Field | Value |
| ----- | ----- |
| **Account Name** | A friendly name (e.g., "My GitLab") |
| **Personal Access Token** | Paste your GitLab token |
| **Base URL** | Leave default for GitLab.com, or enter your self-hosted URL |

**Bitbucket:**
| Field | Value |
| ----- | ----- |
| **Account Name** | A friendly name (e.g., "My Bitbucket") |
| **Username** | Your Bitbucket username |
| **App Password** | Paste your Bitbucket app password |

6. Click **Test Connection** to verify
7. Click **Save**

### Multiple Accounts

You can add multiple accounts from the same provider:

- Personal GitHub + Work GitHub
- GitLab.com + Self-hosted GitLab
- Multiple Bitbucket workspaces

Each account is stored securely with encrypted tokens.

---

## Security Best Practices

### Token Management

1. **Use expiration dates**
   - Set tokens to expire in 90 days or less
   - Rotate tokens periodically
   - Remove unused tokens

2. **Grant minimum permissions**
   - Only enable the scopes you actually need
   - Use read-only access when write isn't required
   - Review permissions quarterly

3. **One token per application**
   - Create separate tokens for different tools
   - Easier to identify and revoke if compromised
   - Better audit trail

4. **Never commit tokens to version control**
   - Tokens are stored encrypted in Ampel's database
   - Don't share tokens via email or chat
   - Use secure password managers for temporary storage

### Token Storage in Ampel

Ampel stores tokens securely:

- **Encryption**: AES-256-GCM encryption at rest
- **Encryption key**: Stored separately in environment variables
- **Database**: Encrypted tokens stored in PostgreSQL
- **Memory**: Tokens only decrypted when actively used

### Handling Compromised Tokens

If a token is exposed:

1. **Revoke immediately:**
   - **GitHub**: Settings > Developer settings > Personal access tokens > Delete
   - **GitLab**: Profile > Access Tokens > Revoke
   - **Bitbucket**: Personal settings > App passwords > Delete

2. **Create a new token** following the steps above
3. **Update Ampel** with the new token via Settings > Provider Accounts
4. **Review access logs** on the provider platform for suspicious activity

### Monitoring Token Usage

**GitHub:**

- View token usage at `https://github.com/settings/tokens`
- Shows last used date and IP address

**GitLab:**

- View active tokens at `https://gitlab.com/-/profile/personal_access_tokens`
- Shows last used date and expiration

**Bitbucket:**

- View app passwords at `https://bitbucket.org/account/settings/app-passwords/`
- Shows creation date (no usage tracking)

---

## Troubleshooting

### Common Issues

**"Invalid token" error**

- Verify you copied the entire token (no truncation)
- Check for extra whitespace when pasting
- Ensure the token hasn't expired
- Confirm the token has the required scopes

**"Insufficient permissions" error**

- Review the token scopes/permissions
- For GitHub: Ensure `repo` scope is enabled
- For GitLab: Ensure `api` or `read_api` scope is enabled
- For Bitbucket: Ensure repositories and pull requests have read permission

**"Token expired" error**

- Create a new token (tokens can't be renewed)
- Update the account in Ampel settings
- Consider using a longer expiration next time

**Can't see repositories after connecting**

- **GitHub (fine-grained)**: Ensure you selected repository access
- **GitHub (classic)**: Token may only have access to public repos
- **GitLab**: Check project visibility and token scopes
- **Bitbucket**: Verify workspace membership and repository permissions

### Testing Your Token

You can test tokens manually using curl:

**GitHub:**

```bash
curl -H "Authorization: token YOUR_TOKEN" https://api.github.com/user
```

**GitLab:**

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" https://gitlab.com/api/v4/user
```

**Bitbucket:**

```bash
curl -u YOUR_USERNAME:YOUR_APP_PASSWORD https://api.bitbucket.org/2.0/user
```

If these commands fail, the token itself is invalid.

---

## Additional Resources

### Official Documentation

**GitHub:**

- [Creating a personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)
- [Fine-grained personal access tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token)
- [Token scopes](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/scopes-for-oauth-apps)

**GitLab:**

- [Personal access tokens](https://docs.gitlab.com/ee/user/profile/personal_access_tokens.html)
- [Token scopes](https://docs.gitlab.com/ee/user/profile/personal_access_tokens.html#personal-access-token-scopes)

**Bitbucket:**

- [App passwords](https://support.atlassian.com/bitbucket-cloud/docs/app-passwords/)
- [Repository access tokens](https://support.atlassian.com/bitbucket-cloud/docs/repository-access-tokens/)

### Security Resources

- [OWASP Token Best Practices](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [Git Provider Security Guides](https://docs.github.com/en/code-security)

---

## Quick Reference Card

| Provider  | Token Creation URL                                                                                       | Token Type                                      | Key Scopes                                          |
| --------- | -------------------------------------------------------------------------------------------------------- | ----------------------------------------------- | --------------------------------------------------- |
| GitHub    | [github.com/settings/tokens](https://github.com/settings/tokens)                                         | Personal Access Token (classic or fine-grained) | `repo`, `read:user`, `read:org`                     |
| GitLab    | [gitlab.com/-/profile/personal_access_tokens](https://gitlab.com/-/profile/personal_access_tokens)       | Personal Access Token                           | `api`, `read_user`, `read_repository`               |
| Bitbucket | [bitbucket.org/account/settings/app-passwords](https://bitbucket.org/account/settings/app-passwords/new) | App Password                                    | Account:Read, Repositories:Read, Pull requests:Read |

### Token Format Examples

| Provider  | Format                                                  | Example                                    |
| --------- | ------------------------------------------------------- | ------------------------------------------ |
| GitHub    | `ghp_` prefix (classic) or `github_pat_` (fine-grained) | `ghp_abcd1234efgh5678ijkl9012mnop3456qrst` |
| GitLab    | `glpat-` prefix                                         | `glpat-abc123xyz789`                       |
| Bitbucket | Alphanumeric string                                     | `ATBBxyz123abc456`                         |

---

## Migration from OAuth

If you previously used OAuth configuration:

1. Create PATs for each provider following the guides above
2. Add accounts via Ampel UI (Settings > Provider Accounts)
3. Existing OAuth environment variables are no longer needed
4. The `.env` file no longer requires OAuth credentials
5. All authentication is now handled through provider accounts in the database

This PAT-based approach is simpler, more flexible, and doesn't require OAuth application registration.
