mod m20250101_000001_initial;
mod m20250102_000002_teams;
mod m20250103_000003_multi_tenant;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_initial::Migration),
            Box::new(m20250102_000002_teams::Migration),
            Box::new(m20250103_000003_multi_tenant::Migration),
        ]
    }
}
