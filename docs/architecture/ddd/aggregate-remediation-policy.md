The DDD aggregate design document has been written to `/Users/cphillipson/Development/active/ai/ampel/docs/architecture/ddd/aggregate-remediation-policy.md`.

The document covers all requested sections:

**Structure:**
1. Aggregate overview with boundary diagram showing `RemediationPolicy` as root and its five embedded value objects
2. Identity — UUID primary key plus `(scope_type, scope_id)` unique index with the DDL comment
3. All five value objects with full Rust struct sketches, field-level doc comments, and `Default` implementations:
   - `RemediationScope` + `ScopeType` (with `PartialOrd` for hierarchy ranking)
   - `PrSelectionCriteria` (author/label/draft/age/review filters)
   - `AgentBudgetConfig` (iterations, wall-clock seconds, `rust_decimal::Decimal` cost)
   - `ModelStrategy` + `StrategyMode` + `RouterConfig` (single/fallback/router modes, tied to ADR-007)
   - `NotificationConfig` (opaque channel list)
4. Aggregate root `RemediationPolicy` struct — field reference table plus full Rust definition with `AutonomyLevel` and `RemediationTier` enums
5. Seven invariants in a table, a `validate()` method, and a typed `RemediationPolicyError` enum using `thiserror`
6. Five command structs (`CreatePolicy`, `UpdatePolicy`, `ToggleEnabled`, `SetAutonomyLevel`, `SetRemediationTier`) plus a representative `apply_set_autonomy_level` handler
7. `PolicyResolver` — `#[async_trait]` trait, `ResolutionContext`, `DbPolicyResolver` stub, `disabled_default()`, air-gapped ceiling projection, and worker usage snippet
8. SeaORM entity sketch with JSON columns for value objects, plus the full PostgreSQL migration DDL including the `CHECK` constraint on `auto_merge + require_human_approval`
9. Crate placement table mapping every artifact to its crate and file path
10. Key design decisions table explaining the rationale for JSON columns, the disabled-stub resolver contract, read-time ceiling application, and the DB constraint as defence-in-depth

All Rust code follows the project's existing conventions: `#[async_trait]`, `sea_orm`, `serde`, `thiserror`, `uuid::Uuid`, `chrono::DateTime<Utc>`, and reuse of the existing `MergeStrategy` type from `ampel-core`.
