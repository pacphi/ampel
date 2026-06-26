//! Fleet PR Remediation — Phase 1 (Policy CRUD + Dry-Run) domain layer.
//!
//! Value objects and enums live here; the orchestrating services live under
//! [`crate::services`]. The DB entity column subsets the services query are
//! defined in [`db`] to avoid an `ampel-db -> ampel-core` dependency cycle.

mod consolidation;
mod policy;

pub(crate) mod db;

pub use consolidation::{ConsolidationPlan, PrRef};
pub use policy::{
    AutonomyLevel, PrSelectionStrategy, RemediationCriteria, RemediationTier, ScopeType,
};

#[cfg(test)]
pub(crate) mod testkit {
    //! In-memory SQLite fixtures for service unit tests. The schema is built
    //! directly from the local [`super::db`] entities (no `ampel-db` dependency).

    use super::db;
    use sea_orm::{
        ActiveValue::Set, ConnectionTrait, Database, DatabaseConnection, EntityTrait, Schema,
    };
    use uuid::Uuid;

    pub async fn memory_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("connect sqlite");
        create_schema(&db).await;
        db
    }

    async fn create_schema(db: &DatabaseConnection) {
        let backend = db.get_database_backend();
        let schema = Schema::new(backend);

        macro_rules! create {
            ($entity:expr) => {{
                let stmt = schema.create_table_from_entity($entity);
                db.execute(backend.build(&stmt))
                    .await
                    .expect("create table");
            }};
        }

        create!(db::remediation_policy::Entity);
        create!(db::organizations::Entity);
        create!(db::teams::Entity);
        create!(db::team_members::Entity);
        create!(db::repositories::Entity);
        create!(db::pull_requests::Entity);
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn seed_policy(
        db: &DatabaseConnection,
        scope_type: &str,
        scope_id: Uuid,
        autonomy_level: &str,
        remediation_tier: &str,
        pr_selection_json: &str,
        allowed_targets_json: &str,
        skip_draft: bool,
        max_prs_per_run: i32,
        air_gapped: bool,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let model = db::remediation_policy::ActiveModel {
            id: Set(id),
            scope_type: Set(scope_type.to_string()),
            scope_id: Set(scope_id),
            enabled: Set(true),
            min_open_prs: Set(1),
            pr_selection: Set(pr_selection_json.to_string()),
            autonomy_level: Set(autonomy_level.to_string()),
            remediation_tier: Set(remediation_tier.to_string()),
            max_prs_per_run: Set(max_prs_per_run),
            allowed_targets: Set(allowed_targets_json.to_string()),
            skip_draft: Set(skip_draft),
            require_green_before_merge: Set(true),
            air_gapped: Set(air_gapped),
        };
        db::remediation_policy::Entity::insert(model)
            .exec(db)
            .await
            .expect("insert policy");
        id
    }

    pub async fn seed_repo(db: &DatabaseConnection, user_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        let model = db::repositories::ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
        };
        db::repositories::Entity::insert(model)
            .exec(db)
            .await
            .expect("insert repo");
        id
    }

    pub async fn seed_org(db: &DatabaseConnection, owner_id: Uuid, air_gapped: bool) -> Uuid {
        let id = Uuid::new_v4();
        let model = db::organizations::ActiveModel {
            id: Set(id),
            owner_id: Set(owner_id),
            air_gapped: Set(air_gapped),
        };
        db::organizations::Entity::insert(model)
            .exec(db)
            .await
            .expect("insert org");
        id
    }

    pub async fn seed_team(db: &DatabaseConnection, org_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        let model = db::teams::ActiveModel {
            id: Set(id),
            organization_id: Set(org_id),
        };
        db::teams::Entity::insert(model)
            .exec(db)
            .await
            .expect("insert team");
        id
    }

    pub async fn seed_team_member(db: &DatabaseConnection, team_id: Uuid, user_id: Uuid) {
        let model = db::team_members::ActiveModel {
            id: Set(Uuid::new_v4()),
            team_id: Set(team_id),
            user_id: Set(user_id),
        };
        db::team_members::Entity::insert(model)
            .exec(db)
            .await
            .expect("insert team_member");
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn seed_pr(
        db: &DatabaseConnection,
        repo_id: Uuid,
        number: i32,
        target_branch: &str,
        state: &str,
        is_draft: bool,
        created_offset_secs: i64,
    ) {
        let created_at = chrono::Utc::now() - chrono::Duration::seconds(created_offset_secs.max(0));
        let model = db::pull_requests::ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(repo_id),
            number: Set(number),
            title: Set(format!("PR {number}")),
            source_branch: Set(format!("feature/{number}")),
            target_branch: Set(target_branch.to_string()),
            state: Set(state.to_string()),
            is_draft: Set(is_draft),
            created_at: Set(created_at),
        };
        db::pull_requests::Entity::insert(model)
            .exec(db)
            .await
            .expect("insert pr");
    }
}
