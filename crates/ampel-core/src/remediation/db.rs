//! Minimal, read-side SeaORM entity definitions for the tables the remediation
//! services query.
//!
//! ## Why these live here
//!
//! `ampel-db` depends on `ampel-core` (its entities implement
//! `From<Model> for ampel_core::models::*`), so `ampel-core` **cannot** depend on
//! `ampel-db` without creating a cargo dependency cycle. Rather than invert that
//! relationship, the remediation services define the narrow column subsets they
//! need here. Column and table names match the canonical `ampel-db` schema, so
//! these query the same physical tables (Postgres in production, SQLite in
//! tests). Only the columns actually used are declared.

use sea_orm::entity::prelude::*;

pub mod remediation_policy {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "remediation_policy")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub scope_type: String,
        pub scope_id: Uuid,
        pub enabled: bool,
        pub min_open_prs: i32,
        pub pr_selection: String,
        pub autonomy_level: String,
        pub remediation_tier: String,
        pub max_prs_per_run: i32,
        pub allowed_targets: String,
        pub skip_draft: bool,
        pub require_green_before_merge: bool,
        pub air_gapped: bool,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod organizations {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "organizations")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub owner_id: Uuid,
        pub air_gapped: bool,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod teams {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "teams")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub organization_id: Uuid,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod team_members {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "team_members")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub team_id: Uuid,
        pub user_id: Uuid,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod repositories {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "repositories")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub user_id: Uuid,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod pull_requests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "pull_requests")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub repository_id: Uuid,
        pub number: i32,
        pub title: String,
        pub source_branch: String,
        pub target_branch: String,
        pub state: String,
        pub is_draft: bool,
        pub created_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
