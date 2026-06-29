The DDD aggregate design document has been written to the path above.

**What the document covers:**

1. **Aggregate root definition** — `ModelProviderAccount` as the consistency boundary for a credentialed AI model provider connection within a user/org/team scope.

2. **Identity** — UUID primary key; `(scope, scope_id, provider, account_label)` unique index; partial unique index for the scoped-default invariant.

3. **Value objects** with full Rust struct sketches:
   - `EncryptedCredential` — wraps `auth_type`, `api_key_encrypted` (nullable), `endpoint_url`, `model_path`
   - `AuthType` — `ApiKey | Bearer | CustomHeader | None`
   - `EgressClass` — `External | LocalOnly` with v1 provider defaults table
   - `SpendCap` — `limit_usd` + `used_usd` with `has_budget()` / `remaining()` methods
   - `ValidationStatus` — `Pending | Valid | Invalid | Expired`
   - `ModelProvider` — identity enum with `default_egress_class()` and `requires_api_key()` helpers
   - `Scope` — `User | Org | Team`

4. **Key fields table** — every column, its Rust type, DB type, and notes; includes `extra_config` JSONB shape example.

5. **Lifecycle** — create (validate → encrypt → ping → insert), use (three harness guards before decrypt), rotate (encrypt new key → ping → replace ciphertext), invalidate (set inactive, emit alert).

6. **All six commands** with fields and effects in a table.

7. **Four invariants** with enforcement code:
   - Credential consistency (`api_key_encrypted IS NULL ↔ auth_type = None`)
   - Spend cap with `SELECT ... FOR UPDATE` transaction sketch
   - Egress policy guard (runtime, not DB constraint)
   - Scoped default via partial unique index + `SetDefaultCommand` transaction

8. **Full SeaORM entity** skeleton matching ADR-008's schema decisions.

9. **Full migration** with all indexes including the raw-SQL partial unique index for the default invariant.

10. **API DTO** with `api_key_encrypted` explicitly absent.

11. **Scope resolution algorithm** (team → org → user fallback chain) as used by the remediation harness.

12. **Validation ping strategy table** per provider (Claude, Gemini, Ollama, ONNX, openai_compatible).

13. **Cross-reference table** to ADR-007 through ADR-014 and all implementation file paths.
