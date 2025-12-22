mod m20250101_000001_initial;
mod m20250102_000002_teams;
mod m20250103_000003_pr_filters;
mod m20250104_000004_merge_notifications;
mod m20250105_000005_skip_review_setting;
mod m20250120_000001_provider_accounts;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_initial::Migration),
            Box::new(m20250102_000002_teams::Migration),
            Box::new(m20250103_000003_pr_filters::Migration),
            Box::new(m20250104_000004_merge_notifications::Migration),
            Box::new(m20250105_000005_skip_review_setting::Migration),
            Box::new(m20250120_000001_provider_accounts::Migration),
        ]
    }
}
