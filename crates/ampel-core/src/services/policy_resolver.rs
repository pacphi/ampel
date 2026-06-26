//! Resolves the effective remediation policy for a repository by walking the
//! scope hierarchy most-specific-wins (repo -> team -> org -> user default) and
//! applying the ADR-014 org-level air-gapped ceiling.

use crate::errors::{AmpelError, AmpelResult};
use crate::remediation::db;
use crate::remediation::{
    AutonomyLevel, PrSelectionStrategy, RemediationCriteria, RemediationTier,
};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

/// Resolves the effective [`RemediationCriteria`] for a repository.
#[derive(Clone)]
pub struct PolicyResolver {
    db: DatabaseConnection,
}

impl PolicyResolver {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Resolve the effective policy for `repo_id`.
    ///
    /// Returns `None` when the repository has no policy at any scope. When a
    /// policy is found, the ADR-014 org ceiling is applied: if **any** owning org
    /// is air-gapped, the effective `air_gapped` is forced `true` regardless of
    /// the matched policy's value (the ceiling cannot be downgraded).
    pub async fn resolve(&self, repo_id: Uuid) -> AmpelResult<Option<RemediationCriteria>> {
        // The repo anchors the hierarchy; without it nothing is resolvable.
        let Some(repo) = db::repositories::Entity::find_by_id(repo_id)
            .one(&self.db)
            .await
            .map_err(db_err)?
        else {
            return Ok(None);
        };
        let user_id = repo.user_id;

        let (team_ids, orgs) = self.collect_scope(user_id).await?;
        let org_ids: Vec<Uuid> = orgs.iter().map(|o| o.id).collect();
        let org_air_gapped = orgs.iter().any(|o| o.air_gapped);

        // Most-specific-wins: first match provides the base policy.
        let base = match self.find_policy("repository", &[repo_id]).await? {
            Some(p) => Some(p),
            None => match self.find_policy("team", &team_ids).await? {
                Some(p) => Some(p),
                None => match self.find_policy("org", &org_ids).await? {
                    Some(p) => Some(p),
                    None => self.find_policy("user", &[user_id]).await?,
                },
            },
        };

        let Some(base) = base else {
            return Ok(None);
        };

        let mut criteria = to_criteria(base)?;
        // ADR-014 ceiling: org air-gapped is non-overridable and OR-ed up.
        if org_air_gapped {
            criteria.air_gapped = true;
        }

        Ok(Some(criteria))
    }

    /// Collect the team ids the repo owner belongs to and the candidate orgs
    /// (orgs the owner owns, plus orgs reachable via team membership).
    async fn collect_scope(
        &self,
        user_id: Uuid,
    ) -> AmpelResult<(Vec<Uuid>, Vec<db::organizations::Model>)> {
        let team_ids: Vec<Uuid> = db::team_members::Entity::find()
            .filter(db::team_members::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(db_err)?
            .into_iter()
            .map(|m| m.team_id)
            .collect();

        let mut org_ids: Vec<Uuid> = Vec::new();
        if !team_ids.is_empty() {
            let teams = db::teams::Entity::find()
                .filter(db::teams::Column::Id.is_in(team_ids.clone()))
                .all(&self.db)
                .await
                .map_err(db_err)?;
            org_ids.extend(teams.into_iter().map(|t| t.organization_id));
        }

        // Orgs owned directly by the user.
        let mut orgs = db::organizations::Entity::find()
            .filter(db::organizations::Column::OwnerId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(db_err)?;

        // Orgs reached via team membership (excluding ones already collected).
        let known: Vec<Uuid> = orgs.iter().map(|o| o.id).collect();
        let extra: Vec<Uuid> = org_ids
            .into_iter()
            .filter(|id| !known.contains(id))
            .collect();
        if !extra.is_empty() {
            let via_team = db::organizations::Entity::find()
                .filter(db::organizations::Column::Id.is_in(extra))
                .all(&self.db)
                .await
                .map_err(db_err)?;
            orgs.extend(via_team);
        }

        Ok((team_ids, orgs))
    }

    /// Find the first enabled policy for `scope_type` among `scope_ids`,
    /// deterministically ordered by `scope_id`.
    async fn find_policy(
        &self,
        scope_type: &str,
        scope_ids: &[Uuid],
    ) -> AmpelResult<Option<db::remediation_policy::Model>> {
        if scope_ids.is_empty() {
            return Ok(None);
        }
        db::remediation_policy::Entity::find()
            .filter(db::remediation_policy::Column::ScopeType.eq(scope_type))
            .filter(db::remediation_policy::Column::ScopeId.is_in(scope_ids.to_vec()))
            .filter(db::remediation_policy::Column::Enabled.eq(true))
            .order_by_asc(db::remediation_policy::Column::ScopeId)
            .one(&self.db)
            .await
            .map_err(db_err)
    }
}

fn db_err(e: DbErr) -> AmpelError {
    AmpelError::DatabaseError(e.to_string())
}

/// Flatten a raw policy row into typed [`RemediationCriteria`], deserializing the
/// JSON/text value-object columns at this service boundary.
fn to_criteria(p: db::remediation_policy::Model) -> AmpelResult<RemediationCriteria> {
    let autonomy_level: AutonomyLevel = p.autonomy_level.parse()?;
    let remediation_tier: RemediationTier = p.remediation_tier.parse()?;

    let pr_selection: PrSelectionStrategy = serde_json::from_str(&p.pr_selection)
        .map_err(|e| AmpelError::ValidationError(format!("invalid pr_selection: {e}")))?;
    let allowed_targets: Vec<String> = serde_json::from_str(&p.allowed_targets)
        .map_err(|e| AmpelError::ValidationError(format!("invalid allowed_targets: {e}")))?;

    Ok(RemediationCriteria {
        min_open_prs: p.min_open_prs,
        pr_selection,
        max_prs_per_run: p.max_prs_per_run,
        allowed_targets,
        skip_draft: p.skip_draft,
        require_green_before_merge: p.require_green_before_merge,
        air_gapped: p.air_gapped,
        autonomy_level,
        remediation_tier,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remediation::testkit;

    const DRY: &str = "dry_run_only";
    const CONSOLIDATE: &str = "consolidate_only";
    const ALL_OPEN: &str = "\"all_open\"";
    const TARGETS: &str = "[\"main\"]";

    #[tokio::test]
    async fn should_return_none_when_no_policy_exists() {
        // Arrange
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        let resolver = PolicyResolver::new(db);

        // Act
        let resolved = resolver.resolve(repo).await.unwrap();

        // Assert
        assert!(resolved.is_none());
    }

    #[tokio::test]
    async fn should_return_none_when_repo_missing() {
        // Arrange
        let db = testkit::memory_db().await;
        let resolver = PolicyResolver::new(db);

        // Act
        let resolved = resolver.resolve(Uuid::new_v4()).await.unwrap();

        // Assert
        assert!(resolved.is_none());
    }

    #[tokio::test]
    async fn should_fall_back_to_org_policy_when_no_repo_policy() {
        // Arrange
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        let org = testkit::seed_org(&db, user, false).await;
        testkit::seed_policy(
            &db,
            "org",
            org,
            DRY,
            CONSOLIDATE,
            ALL_OPEN,
            TARGETS,
            true,
            5,
            false,
        )
        .await;
        let resolver = PolicyResolver::new(db);

        // Act
        let criteria = resolver.resolve(repo).await.unwrap().expect("resolved");

        // Assert
        assert_eq!(criteria.max_prs_per_run, 5);
    }

    #[tokio::test]
    async fn should_prefer_repo_policy_over_org_policy() {
        // Arrange
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        let org = testkit::seed_org(&db, user, false).await;
        testkit::seed_policy(
            &db,
            "org",
            org,
            DRY,
            CONSOLIDATE,
            ALL_OPEN,
            TARGETS,
            true,
            5,
            false,
        )
        .await;
        testkit::seed_policy(
            &db,
            "repository",
            repo,
            DRY,
            CONSOLIDATE,
            ALL_OPEN,
            TARGETS,
            true,
            99,
            false,
        )
        .await;
        let resolver = PolicyResolver::new(db);

        // Act
        let criteria = resolver.resolve(repo).await.unwrap().expect("resolved");

        // Assert: repo policy (max=99) wins over org policy (max=5).
        assert_eq!(criteria.max_prs_per_run, 99);
    }

    #[tokio::test]
    async fn should_force_air_gapped_when_org_ceiling_set_even_if_policy_disables_it() {
        // Arrange: org is air-gapped, but the winning repo policy says false.
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        let _org = testkit::seed_org(&db, user, true).await;
        testkit::seed_policy(
            &db,
            "repository",
            repo,
            DRY,
            CONSOLIDATE,
            ALL_OPEN,
            TARGETS,
            true,
            5,
            false, // policy.air_gapped = false
        )
        .await;
        let resolver = PolicyResolver::new(db);

        // Act
        let criteria = resolver.resolve(repo).await.unwrap().expect("resolved");

        // Assert: ADR-014 ceiling forces air_gapped = true.
        assert!(criteria.air_gapped);
    }

    #[tokio::test]
    async fn should_not_force_air_gapped_when_org_ceiling_unset() {
        // Arrange
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        let _org = testkit::seed_org(&db, user, false).await;
        testkit::seed_policy(
            &db,
            "repository",
            repo,
            DRY,
            CONSOLIDATE,
            ALL_OPEN,
            TARGETS,
            true,
            5,
            false,
        )
        .await;
        let resolver = PolicyResolver::new(db);

        // Act
        let criteria = resolver.resolve(repo).await.unwrap().expect("resolved");

        // Assert
        assert!(!criteria.air_gapped);
    }

    #[tokio::test]
    async fn should_resolve_team_policy_via_membership() {
        // Arrange: no repo/org policy, but a team the owner belongs to has one.
        let db = testkit::memory_db().await;
        let user = Uuid::new_v4();
        let repo = testkit::seed_repo(&db, user).await;
        let org = testkit::seed_org(&db, user, false).await;
        let team = testkit::seed_team(&db, org).await;
        testkit::seed_team_member(&db, team, user).await;
        testkit::seed_policy(
            &db,
            "team",
            team,
            DRY,
            CONSOLIDATE,
            ALL_OPEN,
            TARGETS,
            true,
            7,
            false,
        )
        .await;
        let resolver = PolicyResolver::new(db);

        // Act
        let criteria = resolver.resolve(repo).await.unwrap().expect("resolved");

        // Assert
        assert_eq!(criteria.max_prs_per_run, 7);
    }
}
