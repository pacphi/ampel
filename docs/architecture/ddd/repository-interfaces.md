The repository interfaces document has been written to:

`/Users/cphillipson/Development/active/ai/ampel/docs/architecture/ddd/repository-interfaces.md`

Here is a summary of what was produced:

**Structure**

The document opens with a SeaORM conventions table — entity file location, migration naming (`m20260624_000001_remediation_loops.rs`), UUID primary key pattern, timestamp types, JSON column handling, and encryption responsibilities — so every implementor starts from the same baseline.

Six repository interfaces follow, each with the same four-part shape: purpose, aggregate served, a Rust `#[async_trait]` trait block with fully-typed signatures, and concrete SeaORM implementation notes.

**Key design decisions captured per interface**

- `RemediationPolicyRepository` — exposes `find_all_for_hierarchy(repo_id)` for the `PolicyResolver` four-level walk (repo → team → org → user) and `find_enabled_with_due_schedule(now)` for the sweep job, with a join to `repositories` to avoid N+1 in the outer loop.

- `RemediationRunRepository` — `transition_state(id, from_state, to_state, updates)` is the only sanctioned way to advance state; it uses `WHERE state = from_state` as a CAS guard and returns `DbError::ConcurrentModification` on zero rows affected.

- `RemediationRunPrRepository` — `create_batch` maps to `Entity::insert_many`; rows are append-only (no delete method), mirroring `merge_operation_item`.

- `ModelProviderAccountRepository` — `record_spend` uses a conditional SQL `UPDATE … WHERE cumulative_spend_usd + ? <= monthly_spend_cap_usd` and returns `DbError::SpendCapExceeded` on breach; `create` explicitly delegates encryption to the service layer.

- `RemediationPlaybookRepository` — only DB overrides are stored here; `find_effective` filters on `is_active = true` enforced by a partial unique index.

- `RemediationAgentSessionRepository` — `record_iteration` uses in-database arithmetic (`cost_usd = cost_usd + ?`) to avoid read-modify-write races; `cost_usd` is `NUMERIC(12,6)` / `rust_decimal::Decimal`.

**Supporting sections**

- Two new `DbError` variants (`ConcurrentModification`, `SpendCapExceeded`) with their exact enum definitions for `crates/ampel-db/src/error.rs`.
- A migration checklist table listing every index that must accompany the new tables, including a partial unique index on `remediation_playbooks(scope_type, scope_id) WHERE is_active`.
