//! Concrete persistence adapters that implement the `ampel-core` repository
//! traits over SeaORM entities (dependency-injection seam, ADR write-path).

pub mod remediation_run;

pub use remediation_run::SeaOrmRemediationRunRepository;
