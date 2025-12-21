/// Test fixtures for creating test data
///
/// This module provides helper functions to create consistent test data
/// across different test suites.
use ampel_db::entities::provider_account::{
    ActiveModel as ProviderAccountActiveModel, Model as ProviderAccountModel,
};
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
