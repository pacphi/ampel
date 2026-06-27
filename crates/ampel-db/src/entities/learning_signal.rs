use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// One strategy-learning observation: how a `(provider, failure_class)` pairing
/// fared on a single completed agentic remediation session (Phase 5b).
///
/// Append-only. Carries the provider *kind* only — never any credential.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "learning_signal")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Provider kind that drove the session (`claude`/`gemini`/`ollama`/`onnx`).
    pub provider: String,
    /// Classified CI failure class (e.g. `build_error`, `lockfile_conflict`).
    pub failure_class: String,
    /// Identifier of the playbook that drove the loop.
    pub playbook_id: String,
    pub playbook_version: i32,
    /// Terminal outcome: `passed` | `exhausted`.
    pub outcome: String,
    /// Wall-clock duration of the session in whole seconds.
    pub duration_secs: i64,
    /// Decimal cost stored as a string for cross-DB safety; parsed at the service
    /// layer. `None` for free (self-hosted) providers.
    pub cost_usd: Option<String>,

    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
