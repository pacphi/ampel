# Multi-Account Personal Access Token (PAT) Support

## Technical Feature Plan

**Author**: Claude Code (SPARC Analysis)
**Date**: 2025-12-19
**Status**: Draft
**Version**: 1.0

---

## ⚠️ BREAKING CHANGE

**OAuth support has been completely removed in favor of PAT-only authentication.**

This is a clean implementation with no OAuth migration path. All authentication is now exclusively PAT-based.

---

## 1. Executive Summary

This document outlines the technical implementation of Personal Access Token (PAT) authentication for Git providers (GitHub, GitLab, Bitbucket), with support for **multiple accounts per provider per user**.

### Key Changes

- PAT-based authentication only (no OAuth)
- Support multiple accounts per provider (e.g., work GitHub + personal GitHub)
- User-friendly account labeling and management
- Maintain existing encryption infrastructure for token storage
- Clean implementation with simplified codebase

---

## 2. Architecture Overview

### 2.1 Data Model

The `provider_accounts` table supports multiple PAT-based accounts per provider:

```sql
-- Multiple accounts per provider per user
CREATE TABLE provider_accounts (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    provider VARCHAR(50),           -- github, gitlab, bitbucket
    instance_url VARCHAR(255),      -- Self-hosted support
    account_label VARCHAR(100),     -- User-friendly label
    provider_user_id VARCHAR(255),
    provider_username VARCHAR(255),
    access_token_encrypted BYTEA,
    -- See section 4 for complete schema
);
```

### 2.2 Provider Trait

The `GitProvider` trait in `crates/ampel-providers/src/traits.rs` uses PAT-based authentication:

- `validate_credentials()` - Validate PAT and return user info
- `list_repositories()` - List accessible repositories
- `list_pull_requests()` - List PRs for a repository
- All methods accept `ProviderCredentials` parameter

### 2.3 Encryption Infrastructure

Existing AES-256-GCM encryption in `crates/ampel-db/src/encryption.rs`:

- `EncryptionService::encrypt()` - Encrypt tokens with random nonce
- `EncryptionService::decrypt()` - Decrypt stored tokens
- No changes needed for PAT storage

---

## 3. Provider PAT Specifications

### 3.1 GitHub Personal Access Tokens

#### Token Types

| Type                 | Best For       | Repository Scope     | Expiration            |
| -------------------- | -------------- | -------------------- | --------------------- |
| **Fine-grained PAT** | Production use | Per-repository       | Required (max 1 year) |
| **Classic PAT**      | Simpler setup  | All accessible repos | Optional              |

#### Required Permissions (Fine-grained)

```yaml
Repository Access:
  - Contents: Read
  - Metadata: Read
  - Pull requests: Read and write
  - Commit statuses: Read

Account Permissions:
  - Email addresses: Read (optional)
```

#### Required Scopes (Classic)

```
repo                # Full control of private repositories
read:user           # Read user profile data
user:email          # Read user email addresses (optional)
```

#### API Rate Limits

- **Authenticated**: 5,000 requests/hour
- **Rate limit headers**: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

#### Token Validation

```http
GET https://api.github.com/user
Authorization: Bearer <token>
```

- 200 OK: Valid token
- 401 Unauthorized: Invalid/expired token

### 3.2 GitLab Personal Access Tokens

#### Required Scopes

| Scope       | Purpose                                                        |
| ----------- | -------------------------------------------------------------- |
| `api`       | Full API access (read/write projects, merge requests)          |
| `read_user` | Read user profile information                                  |
| `read_api`  | Read-only API access (alternative to `api` for read-only mode) |

**Recommended**: `api` + `read_user` for full functionality

#### Self-Hosted Support

- Change `base_url` from `https://gitlab.com` to self-hosted instance
- Token validation works the same way
- Rate limits may vary by instance configuration

#### API Rate Limits

- **Default**: 2,000 requests/minute (configurable per instance)
- **Headers**: `RateLimit-Limit`, `RateLimit-Remaining`, `RateLimit-Reset`

#### Token Validation

```http
GET https://gitlab.com/api/v4/user
Authorization: Bearer <token>
```

### 3.3 Bitbucket App Passwords

#### Key Differences

- Bitbucket uses "App Passwords" instead of PATs
- Requires **username + app password** (HTTP Basic Auth)
- Created at: Bitbucket Settings > Personal Settings > App passwords

#### Required Permissions

```yaml
Account:
  - Read # Read account info

Repositories:
  - Read # List and read repositories

Pull requests:
  - Read # List and view PRs
  - Write # Merge PRs (optional)

Pipelines:
  - Read # View build status
```

#### Authentication Method

```http
GET https://api.bitbucket.org/2.0/user
Authorization: Basic base64(username:app_password)
```

#### API Rate Limits

- **Authenticated**: 1,000 requests/hour
- **Headers**: Standard rate limit headers

---

## 4. Proposed Data Model

### 4.1 New Schema: `provider_accounts`

```sql
-- New table: Multiple accounts per provider per user
CREATE TABLE provider_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Provider identification
    provider VARCHAR(50) NOT NULL,           -- github, gitlab, bitbucket
    instance_url VARCHAR(255),               -- NULL for cloud, URL for self-hosted

    -- Account identification
    account_label VARCHAR(100) NOT NULL,     -- User-friendly label: "Work GitHub", "Personal"
    provider_user_id VARCHAR(255) NOT NULL,  -- Provider's user ID
    provider_username VARCHAR(255) NOT NULL, -- Provider's username
    provider_email VARCHAR(255),             -- Optional: email from provider
    avatar_url VARCHAR(500),                 -- Provider avatar

    -- Authentication
    auth_type VARCHAR(20) NOT NULL DEFAULT 'pat',  -- 'pat' or 'oauth' (for migration)
    access_token_encrypted BYTEA NOT NULL,
    -- For Bitbucket: stores username separately (app passwords need username:password)
    auth_username VARCHAR(255),              -- Bitbucket username for Basic Auth

    -- Token metadata
    scopes TEXT,                             -- JSON array of scopes/permissions
    token_expires_at TIMESTAMPTZ,            -- NULL for non-expiring tokens
    last_validated_at TIMESTAMPTZ,           -- Last successful API call
    validation_status VARCHAR(20) DEFAULT 'pending', -- pending, valid, invalid, expired

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false, -- Default account for this provider

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT unique_user_provider_label
        UNIQUE (user_id, provider, instance_url, account_label),
    CONSTRAINT unique_user_provider_account
        UNIQUE (user_id, provider, instance_url, provider_user_id)
);

-- Indexes
CREATE INDEX idx_provider_accounts_user_id ON provider_accounts(user_id);
CREATE INDEX idx_provider_accounts_provider ON provider_accounts(provider);
CREATE INDEX idx_provider_accounts_active ON provider_accounts(is_active) WHERE is_active = true;

-- Ensure only one default per provider per user
CREATE UNIQUE INDEX idx_provider_accounts_default
    ON provider_accounts(user_id, provider, instance_url)
    WHERE is_default = true;
```

### 4.2 Update Repositories Table

```sql
-- Add foreign key to specific provider account
ALTER TABLE repositories
    ADD COLUMN provider_account_id UUID REFERENCES provider_accounts(id) ON DELETE SET NULL;

-- Migrate data: Link existing repos to their provider accounts
-- (handled in migration script)

-- Index for lookups
CREATE INDEX idx_repositories_provider_account ON repositories(provider_account_id);
```

### 4.3 Entity Model (Rust)

```rust
// crates/ampel-db/src/entities/provider_account.rs

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "provider_accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,

    // Provider identification
    pub provider: String,
    pub instance_url: Option<String>,

    // Account identification
    pub account_label: String,
    pub provider_user_id: String,
    pub provider_username: String,
    pub provider_email: Option<String>,
    pub avatar_url: Option<String>,

    // Authentication (PAT only)
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub access_token_encrypted: Vec<u8>,
    pub auth_username: Option<String>,  // For Bitbucket Basic Auth

    // Token metadata
    pub scopes: Option<String>,
    pub token_expires_at: Option<DateTimeUtc>,
    pub last_validated_at: Option<DateTimeUtc>,
    pub validation_status: String,

    // Status
    pub is_active: bool,
    pub is_default: bool,

    // Timestamps
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pending,
    Valid,
    Invalid,
    Expired,
}
```

---

## 5. Provider Abstraction Updates

### 5.1 New Trait Design

```rust
// crates/ampel-providers/src/traits.rs

use async_trait::async_trait;

/// Authentication credentials for a provider
#[derive(Debug, Clone)]
pub struct ProviderCredentials {
    /// Personal Access Token
    pub token: String,
    /// For Bitbucket: username for Basic Auth
    pub username: Option<String>,
}

/// Token validation result
#[derive(Debug, Clone)]
pub struct TokenValidation {
    pub is_valid: bool,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// Unified Git provider interface
#[async_trait]
pub trait GitProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> Provider;

    /// Get the instance URL (None for cloud providers)
    fn instance_url(&self) -> Option<&str>;

    /// Validate credentials and return user info
    /// This replaces OAuth flow for PAT-based auth
    async fn validate_credentials(
        &self,
        credentials: &ProviderCredentials,
    ) -> ProviderResult<TokenValidation>;

    /// Get authenticated user info
    async fn get_user(
        &self,
        credentials: &ProviderCredentials,
    ) -> ProviderResult<ProviderUser>;

    /// List repositories accessible with these credentials
    async fn list_repositories(
        &self,
        credentials: &ProviderCredentials,
        page: i32,
        per_page: i32,
    ) -> ProviderResult<Vec<DiscoveredRepository>>;

    /// Get a specific repository
    async fn get_repository(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
    ) -> ProviderResult<DiscoveredRepository>;

    /// List pull requests for a repository
    async fn list_pull_requests(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> ProviderResult<Vec<ProviderPullRequest>>;

    /// Get pull request details
    async fn get_pull_request(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        number: i32,
    ) -> ProviderResult<ProviderPullRequest>;

    /// Get CI checks for a pull request
    async fn get_ci_checks(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<ProviderCICheck>>;

    /// Get reviews for a pull request
    async fn get_reviews(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
    ) -> ProviderResult<Vec<ProviderReview>>;

    /// Merge a pull request
    async fn merge_pull_request(
        &self,
        credentials: &ProviderCredentials,
        owner: &str,
        repo: &str,
        pr_number: i32,
        merge_request: &MergeRequest,
    ) -> ProviderResult<MergeResult>;

    /// Get current rate limit status
    async fn get_rate_limit(
        &self,
        credentials: &ProviderCredentials,
    ) -> ProviderResult<RateLimitInfo>;

}
```

### 5.2 Provider Factory Update

```rust
// crates/ampel-providers/src/factory.rs

use std::sync::Arc;

pub struct ProviderFactory {
    // No longer needs OAuth client credentials
}

impl ProviderFactory {
    pub fn new() -> Self {
        Self {}
    }

    /// Create a provider instance
    pub fn create(
        &self,
        provider: Provider,
        instance_url: Option<String>,
    ) -> Arc<dyn GitProvider> {
        match provider {
            Provider::GitHub => Arc::new(GitHubProvider::new(instance_url)),
            Provider::GitLab => Arc::new(GitLabProvider::new(instance_url)),
            Provider::Bitbucket => Arc::new(BitbucketProvider::new(instance_url)),
        }
    }
}
```

### 5.3 GitHub Provider Update

```rust
// crates/ampel-providers/src/github.rs

pub struct GitHubProvider {
    client: Client,
    base_url: String,
}

impl GitHubProvider {
    pub fn new(instance_url: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("Ampel/1.0")
            .build()
            .expect("Failed to create HTTP client");

        // Support GitHub Enterprise
        let base_url = instance_url
            .unwrap_or_else(|| "https://api.github.com".to_string());

        Self { client, base_url }
    }

    fn auth_header(&self, credentials: &ProviderCredentials) -> String {
        format!("Bearer {}", credentials.token)
    }
}

#[async_trait]
impl GitProvider for GitHubProvider {
    async fn validate_credentials(
        &self,
        credentials: &ProviderCredentials,
    ) -> ProviderResult<TokenValidation> {
        let response = self.client
            .get(format!("{}/user", self.base_url))
            .header("Authorization", self.auth_header(credentials))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if response.status() == 401 {
            return Ok(TokenValidation {
                is_valid: false,
                error_message: Some("Invalid or expired token".into()),
                ..Default::default()
            });
        }

        if !response.status().is_success() {
            return Ok(TokenValidation {
                is_valid: false,
                error_message: Some(format!("API error: {}", response.status())),
                ..Default::default()
            });
        }

        let user: GitHubUser = response.json().await?;

        // Get scopes from X-OAuth-Scopes header (for classic PATs)
        // Fine-grained PATs don't expose scopes in headers
        let scopes = vec!["repo".into(), "read:user".into()]; // Assume standard scopes

        Ok(TokenValidation {
            is_valid: true,
            user_id: Some(user.id.to_string()),
            username: Some(user.login),
            email: user.email,
            avatar_url: user.avatar_url,
            scopes,
            expires_at: None, // GitHub doesn't expose in API
            error_message: None,
        })
    }

    // ... rest of implementation updates credentials parameter
}
```

### 5.4 Bitbucket Provider Update (Basic Auth)

```rust
// crates/ampel-providers/src/bitbucket.rs

impl BitbucketProvider {
    fn auth_header(&self, credentials: &ProviderCredentials) -> String {
        // Bitbucket uses Basic Auth with username:app_password
        let username = credentials.username.as_deref().unwrap_or("");
        let auth = base64::encode(format!("{}:{}", username, credentials.token));
        format!("Basic {}", auth)
    }
}
```

---

## 6. API Endpoints

### 6.1 Account Management Endpoints

```text
# List all connected accounts
GET /api/accounts
Response: { accounts: [ProviderAccountResponse] }

# Add new PAT account
POST /api/accounts
Body: {
    provider: "github" | "gitlab" | "bitbucket",
    instance_url?: string,           // For self-hosted
    account_label: string,           // "Work GitHub", "Personal GitLab"
    access_token: string,            // PAT or App Password
    username?: string                // Required for Bitbucket
}
Response: { account: ProviderAccountResponse }

# Get account details
GET /api/accounts/:id
Response: { account: ProviderAccountResponse }

# Update account
PATCH /api/accounts/:id
Body: {
    account_label?: string,
    access_token?: string,           // Update PAT
    is_active?: boolean,
    is_default?: boolean
}
Response: { account: ProviderAccountResponse }

# Delete account
DELETE /api/accounts/:id
Response: 204 No Content

# Validate/refresh account token
POST /api/accounts/:id/validate
Response: {
    is_valid: boolean,
    validation_status: "valid" | "invalid" | "expired",
    error_message?: string
}

# Set as default for provider
POST /api/accounts/:id/set-default
Response: { account: ProviderAccountResponse }
```

### 6.2 Response Types

```typescript
interface ProviderAccountResponse {
  id: string;
  provider: 'github' | 'gitlab' | 'bitbucket';
  instance_url: string | null;
  account_label: string;
  provider_username: string;
  provider_email: string | null;
  avatar_url: string | null;
  scopes: string[];
  token_expires_at: string | null;
  validation_status: 'pending' | 'valid' | 'invalid' | 'expired';
  last_validated_at: string | null;
  is_active: boolean;
  is_default: boolean;
  repository_count: number; // Count of repos using this account
  created_at: string;
}

interface AddAccountRequest {
  provider: 'github' | 'gitlab' | 'bitbucket';
  instance_url?: string;
  account_label: string;
  access_token: string;
  username?: string; // Required for Bitbucket
}
```

### 6.3 Handler Implementation

```rust
// crates/ampel-api/src/handlers/accounts.rs

use axum::{extract::{Path, State}, Json};

/// Add a new PAT-based account
pub async fn add_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<AddAccountRequest>,
) -> Result<Json<ApiResponse<ProviderAccountResponse>>, ApiError> {
    // 1. Create provider instance
    let provider = state.provider_factory.create(
        request.provider.parse()?,
        request.instance_url.clone(),
    );

    // 2. Build credentials
    let credentials = ProviderCredentials {
        token: request.access_token.clone(),
        username: request.username.clone(),
    };

    // 3. Validate token with provider
    let validation = provider.validate_credentials(&credentials).await
        .map_err(|e| ApiError::bad_request(format!("Token validation failed: {}", e)))?;

    if !validation.is_valid {
        return Err(ApiError::bad_request(
            validation.error_message.unwrap_or("Invalid token".into())
        ));
    }

    // 4. Check for duplicate account
    let existing = provider_account::Entity::find()
        .filter(provider_account::Column::UserId.eq(auth.user_id))
        .filter(provider_account::Column::Provider.eq(&request.provider))
        .filter(provider_account::Column::ProviderUserId.eq(&validation.user_id.unwrap()))
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(ApiError::conflict(
            "This account is already connected. Use PATCH to update the token."
        ));
    }

    // 5. Encrypt and store token
    let token_encrypted = state.encryption_service
        .encrypt(&request.access_token)
        .map_err(|e| ApiError::internal(format!("Encryption error: {}", e)))?;

    // 6. Determine if this should be default
    let is_first = provider_account::Entity::find()
        .filter(provider_account::Column::UserId.eq(auth.user_id))
        .filter(provider_account::Column::Provider.eq(&request.provider))
        .count(&state.db)
        .await? == 0;

    // 7. Create account record
    let now = Utc::now();
    let account = provider_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(auth.user_id),
        provider: Set(request.provider.clone()),
        instance_url: Set(request.instance_url),
        account_label: Set(request.account_label),
        provider_user_id: Set(validation.user_id.unwrap()),
        provider_username: Set(validation.username.unwrap()),
        provider_email: Set(validation.email),
        avatar_url: Set(validation.avatar_url),
        access_token_encrypted: Set(token_encrypted),
        auth_username: Set(request.username),
        scopes: Set(Some(serde_json::to_string(&validation.scopes)?)),
        token_expires_at: Set(validation.expires_at),
        last_validated_at: Set(Some(now)),
        validation_status: Set("valid".to_string()),
        is_active: Set(true),
        is_default: Set(is_first),  // First account is default
        created_at: Set(now),
        updated_at: Set(now),
    };

    let account = account.insert(&state.db).await?;

    Ok(Json(ApiResponse::success(account.into())))
}
```

---

## 7. Migration Strategy

### 7.1 Database Migration

```rust
// crates/ampel-db/src/migrations/m20250120_000001_provider_accounts.rs

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create new provider_accounts table
        manager.create_table(
            Table::create()
                .table(ProviderAccounts::Table)
                .if_not_exists()
                .col(ColumnDef::new(ProviderAccounts::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(ProviderAccounts::UserId).uuid().not_null())
                .col(ColumnDef::new(ProviderAccounts::Provider).string().not_null())
                .col(ColumnDef::new(ProviderAccounts::InstanceUrl).string())
                .col(ColumnDef::new(ProviderAccounts::AccountLabel).string().not_null())
                .col(ColumnDef::new(ProviderAccounts::ProviderUserId).string().not_null())
                .col(ColumnDef::new(ProviderAccounts::ProviderUsername).string().not_null())
                .col(ColumnDef::new(ProviderAccounts::ProviderEmail).string())
                .col(ColumnDef::new(ProviderAccounts::AvatarUrl).string())
                .col(ColumnDef::new(ProviderAccounts::AccessTokenEncrypted).binary().not_null())
                .col(ColumnDef::new(ProviderAccounts::AuthUsername).string())
                .col(ColumnDef::new(ProviderAccounts::Scopes).text())
                .col(ColumnDef::new(ProviderAccounts::TokenExpiresAt).timestamp_with_time_zone())
                .col(ColumnDef::new(ProviderAccounts::LastValidatedAt).timestamp_with_time_zone())
                .col(ColumnDef::new(ProviderAccounts::ValidationStatus).string().not_null().default("pending"))
                .col(ColumnDef::new(ProviderAccounts::IsActive).boolean().not_null().default(true))
                .col(ColumnDef::new(ProviderAccounts::IsDefault).boolean().not_null().default(false))
                .col(ColumnDef::new(ProviderAccounts::CreatedAt).timestamp_with_time_zone().not_null())
                .col(ColumnDef::new(ProviderAccounts::UpdatedAt).timestamp_with_time_zone().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .from(ProviderAccounts::Table, ProviderAccounts::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        ).await?;

        // 2. Add indexes
        manager.create_index(
            Index::create()
                .name("idx_provider_accounts_user_id")
                .table(ProviderAccounts::Table)
                .col(ProviderAccounts::UserId)
                .to_owned(),
        ).await?;

        // 3. Add provider_account_id to repositories
        manager.alter_table(
            Table::alter()
                .table(Repositories::Table)
                .add_column(ColumnDef::new(Repositories::ProviderAccountId).uuid())
                .to_owned(),
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_repositories_provider_account")
                .from(Repositories::Table, Repositories::ProviderAccountId)
                .to(ProviderAccounts::Table, ProviderAccounts::Id)
                .on_delete(ForeignKeyAction::SetNull)
                .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_foreign_key(
            ForeignKey::drop()
                .table(Repositories::Table)
                .name("fk_repositories_provider_account")
                .to_owned(),
        ).await?;

        manager.alter_table(
            Table::alter()
                .table(Repositories::Table)
                .drop_column(Repositories::ProviderAccountId)
                .to_owned(),
        ).await?;

        manager.drop_table(
            Table::drop().table(ProviderAccounts::Table).to_owned()
        ).await?;

        Ok(())
    }
}
```

### 7.2 Rollout Plan

This is a clean implementation with no OAuth migration required:

1. **Phase 1**: Database migration (create provider_accounts table)
2. **Phase 2**: Backend API implementation
3. **Phase 3**: Frontend UI for PAT management
4. **Phase 4**: Documentation and user guides
5. **Phase 5**: Production deployment

---

## 8. Frontend Changes

### 8.1 Account Management UI

```typescript
// frontend/src/pages/settings/accounts/AddAccountPage.tsx

interface AddAccountForm {
    provider: 'github' | 'gitlab' | 'bitbucket';
    instanceUrl?: string;
    accountLabel: string;
    accessToken: string;
    username?: string;  // For Bitbucket
}

export function AddAccountPage() {
    const [form, setForm] = useState<AddAccountForm>({
        provider: 'github',
        accountLabel: '',
        accessToken: '',
    });

    const addAccount = useMutation({
        mutationFn: (data: AddAccountForm) =>
            api.post('/api/accounts', data),
        onSuccess: () => {
            toast.success('Account connected successfully');
            navigate('/settings/accounts');
        },
    });

    return (
        <form onSubmit={() => addAccount.mutate(form)}>
            {/* Provider selection */}
            <Select
                label="Provider"
                value={form.provider}
                onChange={(v) => setForm({ ...form, provider: v })}
                options={[
                    { value: 'github', label: 'GitHub' },
                    { value: 'gitlab', label: 'GitLab' },
                    { value: 'bitbucket', label: 'Bitbucket' },
                ]}
            />

            {/* Self-hosted option */}
            {form.provider !== 'github' && (
                <Input
                    label="Instance URL (leave empty for cloud)"
                    placeholder="https://gitlab.company.com"
                    value={form.instanceUrl || ''}
                    onChange={(v) => setForm({ ...form, instanceUrl: v })}
                />
            )}

            {/* Account label */}
            <Input
                label="Account Label"
                placeholder="Work GitHub, Personal GitLab..."
                value={form.accountLabel}
                onChange={(v) => setForm({ ...form, accountLabel: v })}
                required
            />

            {/* PAT instructions based on provider */}
            <TokenInstructions provider={form.provider} />

            {/* Token input */}
            <PasswordInput
                label="Personal Access Token"
                value={form.accessToken}
                onChange={(v) => setForm({ ...form, accessToken: v })}
                required
            />

            {/* Username for Bitbucket */}
            {form.provider === 'bitbucket' && (
                <Input
                    label="Bitbucket Username"
                    value={form.username || ''}
                    onChange={(v) => setForm({ ...form, username: v })}
                    required
                />
            )}

            <Button type="submit" loading={addAccount.isPending}>
                Connect Account
            </Button>
        </form>
    );
}
```

### 8.2 Token Creation Instructions Component

```typescript
// frontend/src/components/settings/TokenInstructions.tsx

export function TokenInstructions({ provider }: { provider: string }) {
    const instructions = {
        github: {
            title: 'Create a GitHub Personal Access Token',
            steps: [
                'Go to GitHub Settings > Developer settings > Personal access tokens',
                'Click "Generate new token (classic)" or "Fine-grained tokens"',
                'Select scopes: repo, read:user',
                'Copy the generated token',
            ],
            link: 'https://github.com/settings/tokens',
        },
        gitlab: {
            title: 'Create a GitLab Personal Access Token',
            steps: [
                'Go to GitLab User Settings > Access Tokens',
                'Enter a name and optional expiration date',
                'Select scopes: api, read_user',
                'Click "Create personal access token"',
            ],
            link: 'https://gitlab.com/-/user_settings/personal_access_tokens',
        },
        bitbucket: {
            title: 'Create a Bitbucket App Password',
            steps: [
                'Go to Bitbucket Settings > Personal Settings > App passwords',
                'Click "Create app password"',
                'Select permissions: Account (Read), Repositories (Read), Pull requests (Read/Write)',
                'Copy the generated password',
            ],
            link: 'https://bitbucket.org/account/settings/app-passwords/',
        },
    };

    const info = instructions[provider];

    return (
        <div className="bg-muted p-4 rounded-lg">
            <h4 className="font-medium">{info.title}</h4>
            <ol className="list-decimal list-inside mt-2 space-y-1">
                {info.steps.map((step, i) => (
                    <li key={i} className="text-sm text-muted-foreground">{step}</li>
                ))}
            </ol>
            <a
                href={info.link}
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-primary mt-2 inline-block"
            >
                Open {provider} settings →
            </a>
        </div>
    );
}
```

---

## 9. Security Considerations

### 9.1 Token Storage

- **Encryption**: Continue using AES-256-GCM (existing infrastructure)
- **Key Management**: Encryption key stored in environment variable
- **No plaintext**: Tokens never logged or exposed in API responses

### 9.2 Token Validation

- Validate tokens on creation and periodically
- Mark accounts as invalid when validation fails
- Don't auto-delete: User should manually remove

### 9.3 Audit Logging

```rust
// Log account events for security audit
tracing::info!(
    user_id = %auth.user_id,
    provider = %request.provider,
    account_label = %request.account_label,
    "Provider account created"
);
```

### 9.4 Rate Limiting

- Rate limit account creation: 10 per hour per user
- Rate limit validation calls: 100 per hour per user
- Respect provider rate limits per account

---

## 10. Testing Strategy

### 10.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_pat_auth_header() {
        let credentials = ProviderCredentials {
            token: "ghp_xxxx".to_string(),
            username: None,
        };
        let provider = GitHubProvider::new(None);
        assert_eq!(
            provider.auth_header(&credentials),
            "Bearer ghp_xxxx"
        );
    }

    #[test]
    fn test_bitbucket_basic_auth_header() {
        let credentials = ProviderCredentials {
            token: "app_password".to_string(),
            username: Some("myuser".to_string()),
        };
        let provider = BitbucketProvider::new(None);
        // Should produce: Basic base64(myuser:app_password)
        assert!(provider.auth_header(&credentials).starts_with("Basic "));
    }
}
```

### 10.2 Integration Tests

```rust
#[tokio::test]
async fn test_add_github_account() {
    let app = test_app().await;
    let user = create_test_user(&app.db).await;

    // Mock GitHub API response
    let mock = mock_server();
    mock.expect_get("/user")
        .return_json(json!({
            "id": 12345,
            "login": "testuser",
            "email": "test@example.com"
        }));

    let response = app
        .post("/api/accounts")
        .auth(&user)
        .json(&json!({
            "provider": "github",
            "account_label": "Test Account",
            "access_token": "ghp_test_token"
        }))
        .await;

    assert_eq!(response.status(), 200);

    let account: ProviderAccountResponse = response.json();
    assert_eq!(account.provider_username, "testuser");
    assert_eq!(account.validation_status, "valid");
}
```

---

## 11. Rollout Plan

### 11.1 Deployment Strategy

Simple deployment process:

1. **Database Migration**: Run migration to create provider_accounts table
2. **Backend Deployment**: Deploy PAT-based API endpoints
3. **Frontend Deployment**: Deploy PAT management UI
4. **Documentation**: Update user documentation with PAT setup guides

---

## 12. Success Metrics

| Metric                        | Target                      | Measurement                    |
| ----------------------------- | --------------------------- | ------------------------------ |
| Account creation success rate | >95%                        | API response codes             |
| Token validation latency      | <500ms                      | p95 latency                    |
| User adoption (PAT vs OAuth)  | >80% PAT within 30 days     | Account auth_type distribution |
| Multi-account usage           | >20% users with 2+ accounts | Account count per user         |
| Support tickets               | <10/week for PAT issues     | Support system                 |

---

## 13. Open Questions

1. **Token expiration handling**: Should we proactively notify users before expiration?
2. **GitHub Enterprise**: Need to test with GHE instances
3. **Token scope verification**: How to verify fine-grained PAT has correct permissions?

---

## 14. References

- [GitHub PAT Documentation](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)
- [GitLab PAT Documentation](https://docs.gitlab.com/ee/user/profile/personal_access_tokens.html)
- [Bitbucket App Passwords](https://support.atlassian.com/bitbucket-cloud/docs/app-passwords/)
- [Current Ampel Architecture](./ARCHITECTURE.md)
