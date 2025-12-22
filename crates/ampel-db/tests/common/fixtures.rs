//! Test fixtures for creating test data
//!
//! This module provides builder pattern helpers to create consistent test data
//! across different test suites. Fixtures use the builder pattern to make test
//! data creation flexible and readable.
//!
//! ## Quick Helpers
//!
//! For simple cases, use the convenience functions:
//!
//! ```rust
//! use crate::common::fixtures::{create_test_user, create_test_provider_account};
//!
//! let user = create_test_user(db, "test@example.com", "testuser").await?;
//! let account = create_test_provider_account(db, user.id, "github", "Work", true).await?;
//! ```
//!
//! ## Builder Pattern
//!
//! For more control, use the fixture builders:
//!
//! ```rust
//! use crate::common::fixtures::{UserFixture, ProviderAccountFixture};
//!
//! let user = UserFixture::new("user@example.com", "Test User")
//!     .with_avatar_url("https://example.com/avatar.png")
//!     .create(db)
//!     .await?;
//!
//! let account = ProviderAccountFixture::new(user.id, "github", "Work Account")
//!     .set_default()
//!     .with_scopes(r#"["repo", "read:user"]"#)
//!     .inactive()
//!     .create(db)
//!     .await?;
//! ```
use ampel_db::entities::ci_check::{ActiveModel as CICheckActiveModel, Model as CICheckModel};
use ampel_db::entities::provider_account::{
    ActiveModel as ProviderAccountActiveModel, Model as ProviderAccountModel,
};
use ampel_db::entities::pull_request::{
    ActiveModel as PullRequestActiveModel, Model as PullRequestModel,
};
use ampel_db::entities::repository::{
    ActiveModel as RepositoryActiveModel, Model as RepositoryModel,
};
use ampel_db::entities::review::{ActiveModel as ReviewActiveModel, Model as ReviewModel};
use ampel_db::entities::user::{ActiveModel as UserActiveModel, Model as UserModel};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

/// User fixture builder
pub struct UserFixture {
    email: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
    password_hash: String,
}

#[allow(dead_code)]
impl UserFixture {
    pub fn new(email: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            display_name: Some(display_name.into()),
            avatar_url: None,
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$test$test".to_string(), // Dummy hash
        }
    }

    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn with_avatar_url(mut self, url: impl Into<String>) -> Self {
        self.avatar_url = Some(url.into());
        self
    }

    pub async fn create(self, db: &DatabaseConnection) -> Result<UserModel, sea_orm::DbErr> {
        let now = Utc::now();
        let user = UserActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(self.email),
            display_name: Set(self.display_name),
            avatar_url: Set(self.avatar_url),
            password_hash: Set(self.password_hash),
            created_at: Set(now),
            updated_at: Set(now),
        };

        user.insert(db).await
    }
}

/// Provider account fixture builder
pub struct ProviderAccountFixture {
    user_id: Uuid,
    provider: String,
    account_label: String,
    instance_url: Option<String>,
    provider_user_id: String,
    provider_username: String,
    provider_email: Option<String>,
    avatar_url: Option<String>,
    auth_type: String,
    access_token_encrypted: Vec<u8>,
    auth_username: Option<String>,
    scopes: Option<String>,
    is_active: bool,
    is_default: bool,
    validation_status: String,
}

#[allow(dead_code)]
impl ProviderAccountFixture {
    pub fn new(
        user_id: Uuid,
        provider: impl Into<String>,
        account_label: impl Into<String>,
    ) -> Self {
        let label = account_label.into();
        Self {
            user_id,
            provider: provider.into(),
            account_label: label.clone(),
            instance_url: None,
            provider_user_id: format!("provider_id_{}", label),
            provider_username: format!("username_{}", label),
            provider_email: Some(format!("{}@example.com", label)),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            auth_type: "pat".to_string(),
            access_token_encrypted: vec![1, 2, 3, 4, 5], // Dummy encrypted token
            auth_username: None,
            scopes: Some(r#"["repo","read:user"]"#.to_string()),
            is_active: true,
            is_default: false,
            validation_status: "valid".to_string(),
        }
    }

    pub fn with_instance_url(mut self, url: impl Into<String>) -> Self {
        self.instance_url = Some(url.into());
        self
    }

    pub fn with_provider_user_id(mut self, id: impl Into<String>) -> Self {
        self.provider_user_id = id.into();
        self
    }

    pub fn with_provider_username(mut self, username: impl Into<String>) -> Self {
        self.provider_username = username.into();
        self
    }

    pub fn with_scopes(mut self, scopes: impl Into<String>) -> Self {
        self.scopes = Some(scopes.into());
        self
    }

    pub fn set_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn inactive(mut self) -> Self {
        self.is_active = false;
        self
    }

    pub fn invalid(mut self) -> Self {
        self.validation_status = "invalid".to_string();
        self
    }

    pub fn with_auth_type(mut self, auth_type: impl Into<String>) -> Self {
        self.auth_type = auth_type.into();
        self
    }

    pub async fn create(
        self,
        db: &DatabaseConnection,
    ) -> Result<ProviderAccountModel, sea_orm::DbErr> {
        let now = Utc::now();
        let account = ProviderAccountActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(self.user_id),
            provider: Set(self.provider),
            instance_url: Set(self.instance_url),
            account_label: Set(self.account_label),
            provider_user_id: Set(self.provider_user_id),
            provider_username: Set(self.provider_username),
            provider_email: Set(self.provider_email),
            avatar_url: Set(self.avatar_url),
            auth_type: Set(self.auth_type),
            access_token_encrypted: Set(self.access_token_encrypted),
            auth_username: Set(self.auth_username),
            scopes: Set(self.scopes),
            token_expires_at: Set(None),
            last_validated_at: Set(Some(now)),
            validation_status: Set(self.validation_status),
            is_active: Set(self.is_active),
            is_default: Set(self.is_default),
            created_at: Set(now),
            updated_at: Set(now),
        };

        account.insert(db).await
    }
}

/// Quick fixture creation helpers
pub async fn create_test_user(
    db: &DatabaseConnection,
    email: &str,
    display_name: &str,
) -> Result<UserModel, sea_orm::DbErr> {
    UserFixture::new(email, display_name).create(db).await
}

pub async fn create_test_provider_account(
    db: &DatabaseConnection,
    user_id: Uuid,
    provider: &str,
    label: &str,
    is_default: bool,
) -> Result<ProviderAccountModel, sea_orm::DbErr> {
    let mut fixture = ProviderAccountFixture::new(user_id, provider, label);
    if is_default {
        fixture = fixture.set_default();
    }
    fixture.create(db).await
}

/// Repository fixture builder
pub struct RepositoryFixture {
    user_id: Uuid,
    provider: String,
    provider_id: String,
    owner: String,
    name: String,
    full_name: String,
    description: Option<String>,
    url: String,
    default_branch: String,
    is_private: bool,
    is_archived: bool,
    poll_interval_seconds: i32,
    provider_account_id: Option<Uuid>,
}

#[allow(dead_code)]
impl RepositoryFixture {
    pub fn new(user_id: Uuid, owner: impl Into<String>, name: impl Into<String>) -> Self {
        let owner_str = owner.into();
        let name_str = name.into();
        let full_name = format!("{}/{}", owner_str, name_str);

        Self {
            user_id,
            provider: "github".to_string(),
            provider_id: format!("repo_{}", Uuid::new_v4().simple()),
            owner: owner_str.clone(),
            name: name_str.clone(),
            full_name,
            description: Some(format!("Test repository {}", name_str)),
            url: format!("https://github.com/{}/{}", owner_str, name_str),
            default_branch: "main".to_string(),
            is_private: false,
            is_archived: false,
            poll_interval_seconds: 300,
            provider_account_id: None,
        }
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = provider.into();
        self
    }

    pub fn with_provider_id(mut self, provider_id: impl Into<String>) -> Self {
        self.provider_id = provider_id.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_default_branch(mut self, branch: impl Into<String>) -> Self {
        self.default_branch = branch.into();
        self
    }

    pub fn private(mut self) -> Self {
        self.is_private = true;
        self
    }

    pub fn archived(mut self) -> Self {
        self.is_archived = true;
        self
    }

    pub fn with_poll_interval(mut self, seconds: i32) -> Self {
        self.poll_interval_seconds = seconds;
        self
    }

    pub fn with_provider_account(mut self, account_id: Uuid) -> Self {
        self.provider_account_id = Some(account_id);
        self
    }

    pub async fn create(self, db: &DatabaseConnection) -> Result<RepositoryModel, sea_orm::DbErr> {
        let now = Utc::now();
        let repo = RepositoryActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(self.user_id),
            provider: Set(self.provider),
            provider_id: Set(self.provider_id),
            owner: Set(self.owner),
            name: Set(self.name),
            full_name: Set(self.full_name),
            description: Set(self.description),
            url: Set(self.url),
            default_branch: Set(self.default_branch),
            is_private: Set(self.is_private),
            is_archived: Set(self.is_archived),
            poll_interval_seconds: Set(self.poll_interval_seconds),
            last_polled_at: Set(None),
            group_id: Set(None),
            provider_account_id: Set(self.provider_account_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        repo.insert(db).await
    }
}

/// Pull request fixture builder
pub struct PullRequestFixture {
    repository_id: Uuid,
    provider: String,
    provider_id: String,
    number: i32,
    title: String,
    description: Option<String>,
    url: String,
    state: String,
    source_branch: String,
    target_branch: String,
    author: String,
    author_avatar_url: Option<String>,
    is_draft: bool,
    is_mergeable: Option<bool>,
    has_conflicts: bool,
    additions: i32,
    deletions: i32,
    changed_files: i32,
    commits_count: i32,
    comments_count: i32,
}

#[allow(dead_code)]
impl PullRequestFixture {
    pub fn new(repository_id: Uuid, number: i32, title: impl Into<String>) -> Self {
        let title_str = title.into();
        Self {
            repository_id,
            provider: "github".to_string(),
            provider_id: format!("pr_{}", number),
            number,
            title: title_str.clone(),
            description: Some(format!("Description for {}", title_str)),
            url: format!("https://github.com/test/repo/pull/{}", number),
            state: "open".to_string(),
            source_branch: "feature-branch".to_string(),
            target_branch: "main".to_string(),
            author: "testuser".to_string(),
            author_avatar_url: Some("https://example.com/avatar.png".to_string()),
            is_draft: false,
            is_mergeable: Some(true),
            has_conflicts: false,
            additions: 100,
            deletions: 50,
            changed_files: 5,
            commits_count: 3,
            comments_count: 2,
        }
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = provider.into();
        self
    }

    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = state.into();
        self
    }

    pub fn with_branches(mut self, source: impl Into<String>, target: impl Into<String>) -> Self {
        self.source_branch = source.into();
        self.target_branch = target.into();
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    pub fn draft(mut self) -> Self {
        self.is_draft = true;
        self
    }

    pub fn with_conflicts(mut self) -> Self {
        self.has_conflicts = true;
        self.is_mergeable = Some(false);
        self
    }

    pub fn with_stats(mut self, additions: i32, deletions: i32, files: i32) -> Self {
        self.additions = additions;
        self.deletions = deletions;
        self.changed_files = files;
        self
    }

    pub async fn create(self, db: &DatabaseConnection) -> Result<PullRequestModel, sea_orm::DbErr> {
        let now = Utc::now();
        let pr = PullRequestActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(self.repository_id),
            provider: Set(self.provider),
            provider_id: Set(self.provider_id),
            number: Set(self.number),
            title: Set(self.title),
            description: Set(self.description),
            url: Set(self.url),
            state: Set(self.state),
            source_branch: Set(self.source_branch),
            target_branch: Set(self.target_branch),
            author: Set(self.author),
            author_avatar_url: Set(self.author_avatar_url),
            is_draft: Set(self.is_draft),
            is_mergeable: Set(self.is_mergeable),
            has_conflicts: Set(self.has_conflicts),
            additions: Set(self.additions),
            deletions: Set(self.deletions),
            changed_files: Set(self.changed_files),
            commits_count: Set(self.commits_count),
            comments_count: Set(self.comments_count),
            created_at: Set(now),
            updated_at: Set(now),
            merged_at: Set(None),
            closed_at: Set(None),
            last_synced_at: Set(now),
        };

        pr.insert(db).await
    }
}

/// CI check fixture builder
pub struct CICheckFixture {
    pull_request_id: Uuid,
    name: String,
    status: String,
    conclusion: Option<String>,
    url: Option<String>,
    duration_seconds: Option<i32>,
}

#[allow(dead_code)]
impl CICheckFixture {
    pub fn new(pull_request_id: Uuid, name: impl Into<String>) -> Self {
        Self {
            pull_request_id,
            name: name.into(),
            status: "completed".to_string(),
            conclusion: Some("success".to_string()),
            url: Some("https://example.com/check".to_string()),
            duration_seconds: Some(120),
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_conclusion(mut self, conclusion: impl Into<String>) -> Self {
        self.conclusion = Some(conclusion.into());
        self
    }

    pub fn queued(mut self) -> Self {
        self.status = "queued".to_string();
        self.conclusion = None;
        self
    }

    pub fn in_progress(mut self) -> Self {
        self.status = "in_progress".to_string();
        self.conclusion = None;
        self
    }

    pub fn failed(mut self) -> Self {
        self.status = "completed".to_string();
        self.conclusion = Some("failure".to_string());
        self
    }

    pub fn with_duration(mut self, seconds: i32) -> Self {
        self.duration_seconds = Some(seconds);
        self
    }

    pub async fn create(self, db: &DatabaseConnection) -> Result<CICheckModel, sea_orm::DbErr> {
        let now = Utc::now();
        let started_at = if self.status != "queued" {
            Some(now - chrono::Duration::seconds(self.duration_seconds.unwrap_or(0) as i64))
        } else {
            None
        };
        let completed_at = if self.status == "completed" {
            Some(now)
        } else {
            None
        };

        let check = CICheckActiveModel {
            id: Set(Uuid::new_v4()),
            pull_request_id: Set(self.pull_request_id),
            name: Set(self.name),
            status: Set(self.status),
            conclusion: Set(self.conclusion),
            url: Set(self.url),
            started_at: Set(started_at),
            completed_at: Set(completed_at),
            duration_seconds: Set(self.duration_seconds),
        };

        check.insert(db).await
    }
}

/// Review fixture builder
pub struct ReviewFixture {
    pull_request_id: Uuid,
    reviewer: String,
    reviewer_avatar_url: Option<String>,
    state: String,
    body: Option<String>,
}

#[allow(dead_code)]
impl ReviewFixture {
    pub fn new(pull_request_id: Uuid, reviewer: impl Into<String>) -> Self {
        Self {
            pull_request_id,
            reviewer: reviewer.into(),
            reviewer_avatar_url: Some("https://example.com/reviewer-avatar.png".to_string()),
            state: "approved".to_string(),
            body: Some("LGTM!".to_string()),
        }
    }

    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = state.into();
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn approved(mut self) -> Self {
        self.state = "approved".to_string();
        self
    }

    pub fn changes_requested(mut self) -> Self {
        self.state = "changes_requested".to_string();
        self
    }

    pub fn commented(mut self) -> Self {
        self.state = "commented".to_string();
        self
    }

    pub async fn create(self, db: &DatabaseConnection) -> Result<ReviewModel, sea_orm::DbErr> {
        let review = ReviewActiveModel {
            id: Set(Uuid::new_v4()),
            pull_request_id: Set(self.pull_request_id),
            reviewer: Set(self.reviewer),
            reviewer_avatar_url: Set(self.reviewer_avatar_url),
            state: Set(self.state),
            body: Set(self.body),
            submitted_at: Set(Utc::now()),
        };

        review.insert(db).await
    }
}

/// Quick fixture creation helpers
/// Create a test repository
#[allow(dead_code)]
pub async fn create_test_repository(
    db: &DatabaseConnection,
    user_id: Uuid,
    owner: &str,
    name: &str,
) -> Result<RepositoryModel, sea_orm::DbErr> {
    RepositoryFixture::new(user_id, owner, name)
        .create(db)
        .await
}

/// Create a test pull request
#[allow(dead_code)]
pub async fn create_test_pull_request(
    db: &DatabaseConnection,
    repository_id: Uuid,
    number: i32,
    title: &str,
) -> Result<PullRequestModel, sea_orm::DbErr> {
    PullRequestFixture::new(repository_id, number, title)
        .create(db)
        .await
}

/// Create a test CI check
#[allow(dead_code)]
pub async fn create_test_ci_check(
    db: &DatabaseConnection,
    pull_request_id: Uuid,
    name: &str,
) -> Result<CICheckModel, sea_orm::DbErr> {
    CICheckFixture::new(pull_request_id, name).create(db).await
}

/// Create a test review
#[allow(dead_code)]
pub async fn create_test_review(
    db: &DatabaseConnection,
    pull_request_id: Uuid,
    reviewer: &str,
) -> Result<ReviewModel, sea_orm::DbErr> {
    ReviewFixture::new(pull_request_id, reviewer)
        .create(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::TestDb;

    #[tokio::test]
    async fn test_user_fixture() {
        if TestDb::skip_if_sqlite() {
            return;
        }

        let test_db = TestDb::new().await.unwrap();
        test_db.run_migrations().await.unwrap();

        let user = UserFixture::new("test@example.com", "Test User")
            .with_avatar_url("https://example.com/avatar.png")
            .create(test_db.connection())
            .await
            .unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.display_name, Some("Test User".to_string()));
        assert_eq!(
            user.avatar_url,
            Some("https://example.com/avatar.png".to_string())
        );
    }

    #[tokio::test]
    async fn test_provider_account_fixture() {
        if TestDb::skip_if_sqlite() {
            return;
        }

        let test_db = TestDb::new().await.unwrap();
        test_db.run_migrations().await.unwrap();

        let user = create_test_user(test_db.connection(), "test@example.com", "testuser")
            .await
            .unwrap();

        let account = ProviderAccountFixture::new(user.id, "github", "Work Account")
            .set_default()
            .create(test_db.connection())
            .await
            .unwrap();

        assert_eq!(account.user_id, user.id);
        assert_eq!(account.provider, "github");
        assert_eq!(account.account_label, "Work Account");
        assert!(account.is_default);
        assert!(account.is_active);
    }
}
