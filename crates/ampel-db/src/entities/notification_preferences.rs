use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "notification_preferences")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub email_enabled: bool,
    pub slack_enabled: bool,
    pub slack_webhook_url: Option<String>,
    pub push_enabled: bool,
    pub notify_on_pr_ready: bool,
    pub notify_on_pr_failed: bool,
    pub notify_on_review_requested: bool,
    pub digest_frequency: String, // none, daily, weekly
    pub updated_at: DateTimeUtc,
    // Email SMTP settings
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i32>,
    pub smtp_username: Option<String>,
    #[serde(skip_serializing)]
    pub smtp_password_encrypted: Option<Vec<u8>>,
    pub smtp_from_email: Option<String>,
    pub smtp_to_emails: Option<String>, // JSON array stored as text
    pub smtp_use_tls: bool,
    // Merge notification settings
    pub notify_on_merge_success: bool,
    pub notify_on_merge_failure: bool,
    pub slack_channel: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
