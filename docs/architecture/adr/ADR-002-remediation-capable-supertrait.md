# ADR-002: RemediationCapable Supertrait for Fleet PR Remediation

**Status**: Accepted
**Date**: 2026-06-24
**Deciders**: Architecture Team
**Technical Story**: Fleet PR Remediation Loops require write primitives (branch creation, PR mutation, comment/label authoring) across GitHub, GitLab, and Bitbucket without breaking existing read-only provider integrations or forcing partial implementations onto providers with thin API coverage.

---

## Context

### Problem Statement

Ampel's Fleet PR Remediation Loops feature introduces autonomous triage, consolidation, and remediation of open PRs triggered when a repository accumulates more than three open pull requests. Executing remediation actions — rebasing stale branches, creating consolidation PRs, closing superseded PRs, annotating PRs with triage comments, applying classification labels — requires write access to the underlying Git provider APIs.

The existing `GitProvider` trait covers the read-and-merge surface: credential validation, repository and PR listing, CI check retrieval, review aggregation, and the single merge operation. These semantics fit a read-only PAT scoped to repository data. Remediation actions require a materially different permission scope: branch creation and deletion, PR creation and mutation, comment authoring, and label management.

Extending `GitProvider` directly would force every current and future provider implementation to either stub out unsupported methods or panic, even when the provider is only used for dashboard reads. This is especially problematic for Bitbucket, where the REST API surface for branch and PR write operations is thinner than GitHub's or GitLab's, and for org-level hard-ceiling and air-gapped deployments where write operations are explicitly disabled.

Capabilities also vary per PAT scope: a user-supplied token may have `repo:read` only. The system must degrade gracefully — falling back to sandbox clone-push paths where the provider API does not support a given operation — rather than failing the entire remediation run.

### Technical Context

- `GitProvider` is a `#[async_trait]`-decorated object-safe trait used behind `Arc<dyn GitProvider>` throughout `ampel-providers`; it must remain `dyn`-compatible.
- Provider implementations: `GitHubProvider`, `GitLabProvider`, `BitbucketProvider`, and `MockProvider` (tests). A future `GiteaProvider` is planned.
- PATs are stored AES-256-GCM encrypted in `provider_accounts.access_token_encrypted`; scope metadata is not currently stored.
- Apalis 0.6 background jobs (`RepositoryPollJob`, future `RemediationJob`) run in `ampel-worker` and receive providers from the factory.
- The provider factory (`factory.rs`) constructs concrete types; callers receive `Box<dyn GitProvider>` or `Arc<dyn GitProvider>`.
- Sandbox execution uses rootless Podman/Docker per remediation run; clone-push fallback paths are handled at the job layer, not the provider layer.
- Octopus merges use subprocess `git` commands (not `git2-rs`); the same subprocess approach applies to clone-push fallbacks.
- Playbooks are YAML files (embedded via `rust-embed`, with DB overrides and repo-local `.ampel/remediation.yaml`) rendered by `minijinja`. They declare which operations a remediation policy requires.
- Air-gapped and org-ceiling deployments must be able to load a provider without any write capability being reachable.

---

## Decision

**Introduce `RemediationCapable` as a supertrait of `GitProvider`, implemented only by providers that support write operations, with a `capabilities()` method returning a `RemediationCaps` struct that records which operations are available at runtime.**

This keeps the existing `GitProvider` contract unchanged — all current callers, all read-only PAT flows, and all non-remediation jobs continue to compile and run without modification. Providers opt into remediation by implementing the supertrait. The `RemediationCaps` struct enables the job layer to route unsupported operations to sandbox clone-push fallbacks without panicking or returning opaque errors.

### Implementation Notes

**Trait definition (`crates/ampel-providers/src/remediation.rs`):**

```rust
use async_trait::async_trait;
use crate::traits::GitProvider;

/// Capability flags returned by `RemediationCapable::capabilities()`.
/// All fields default to `false`; providers set only what they support.
#[derive(Debug, Clone, Default)]
pub struct RemediationCaps {
    pub create_branch: bool,
    pub update_branch_from_base: bool,
    pub create_pull_request: bool,
    pub update_pull_request: bool,
    pub close_pull_request: bool,
    pub create_comment: bool,
    pub add_labels: bool,
    pub get_status_for_ref: bool,
    pub delete_branch: bool,
}

#[async_trait]
pub trait RemediationCapable: GitProvider {
    /// Static capability declaration (no async, no provider call).
    fn capabilities(&self) -> RemediationCaps;

    async fn get_default_branch_sha(
        &self,
        repo_id: &str,
    ) -> crate::Result<String>;

    async fn create_branch(
        &self,
        repo_id: &str,
        branch_name: &str,
        from_sha: &str,
    ) -> crate::Result<()>;

    async fn update_branch_from_base(
        &self,
        repo_id: &str,
        branch_name: &str,
        base_branch: &str,
    ) -> crate::Result<()>;

    async fn create_pull_request(
        &self,
        repo_id: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> crate::Result<crate::models::PullRequest>;

    async fn update_pull_request(
        &self,
        repo_id: &str,
        pr_number: u64,
        title: Option<&str>,
        body: Option<&str>,
    ) -> crate::Result<()>;

    async fn close_pull_request(
        &self,
        repo_id: &str,
        pr_number: u64,
    ) -> crate::Result<()>;

    async fn create_comment(
        &self,
        repo_id: &str,
        pr_number: u64,
        body: &str,
    ) -> crate::Result<u64>; // returns comment ID

    async fn add_labels(
        &self,
        repo_id: &str,
        pr_number: u64,
        labels: &[String],
    ) -> crate::Result<()>;

    /// CI/status check for an arbitrary ref (SHA or branch name), not just a PR.
    async fn get_status_for_ref(
        &self,
        repo_id: &str,
        git_ref: &str,
    ) -> crate::Result<crate::models::CiStatus>;

    async fn delete_branch(
        &self,
        repo_id: &str,
        branch_name: &str,
    ) -> crate::Result<()>;
}
```

**Provider implementation matrix:**

| Method | GitHub | GitLab | Bitbucket | Fallback |
|---|---|---|---|---|
| `create_branch` | REST | REST | REST | clone-push |
| `update_branch_from_base` | GraphQL rebase / merge | REST rebase | REST (limited) | clone-push |
| `create_pull_request` | REST | REST | REST | N/A |
| `update_pull_request` | REST | REST | REST | N/A |
| `close_pull_request` | REST | REST | REST | N/A |
| `create_comment` | REST | REST | REST | N/A |
| `add_labels` | REST | REST | not supported | log + skip |
| `get_status_for_ref` | REST | REST | REST | N/A |
| `delete_branch` | REST | REST | REST | clone-push |

Bitbucket's `capabilities()` returns `add_labels: false`; the job layer checks this flag before calling `add_labels` and skips or logs accordingly.

**Factory changes:** The provider factory exposes a separate constructor returning `Option<Arc<dyn RemediationCapable>>` alongside the existing `Arc<dyn GitProvider>` constructor. This makes the capability distinction visible at the call site without downcasting.

```rust
pub fn build_remediation(
    account: &ProviderAccount,
    decrypted_pat: &str,
) -> Option<Arc<dyn RemediationCapable + Send + Sync>> {
    match account.provider {
        ProviderKind::GitHub => Some(Arc::new(GitHubProvider::new(decrypted_pat))),
        ProviderKind::GitLab => Some(Arc::new(GitLabProvider::new(decrypted_pat))),
        ProviderKind::Bitbucket => Some(Arc::new(BitbucketProvider::new(decrypted_pat))),
        ProviderKind::Mock => None, // test callers wire mock directly
    }
}
```

**Remediation job layer:** `RemediationJob` receives `Arc<dyn RemediationCapable + Send + Sync>`. Before each write operation it checks `provider.capabilities()` and routes unsupported operations to the sandbox clone-push executor. This check is synchronous and zero-cost (a struct field comparison).

**`MockProvider` for tests:** `MockProvider` implements `RemediationCapable` with all capability flags set to `true` and all methods recording calls to an `Arc<Mutex<Vec<MockCall>>>`. This supports unit tests for the job layer without network access.

---

## Alternatives Considered

### Option A: Add write methods directly to `GitProvider` (Rejected)

Extend the existing `GitProvider` trait with all remediation methods, making every provider implement them.

**Pros**:
- Single trait; no new abstraction.
- Factory and routing code unchanged.

**Cons**:
- Forces `GitHubProvider`, `GitLabProvider`, `BitbucketProvider`, and `MockProvider` to implement or stub ~10 new methods immediately, even when remediaton is disabled or the PAT scope does not permit writes.
- Read-only PAT deployments (dashboard-only orgs) must carry dead method bodies.
- Bitbucket partial coverage is invisible at the type level; callers cannot know which methods are safe to call without runtime probing.
- Future read-only providers (e.g., Gitea in mirror mode) must implement stubs or `unimplemented!()` panics.
- Breaks the single-responsibility principle: a trait used for rate-limit checking now also owns branch deletion.

**Verdict**: Rejected. The method surface explosion and the inability to express partial capability at the type level outweigh the simplicity of a single trait.

### Option B: `RemediationCapable` supertrait (Accepted)

Introduce `pub trait RemediationCapable: GitProvider { ... }` with a `capabilities()` method.

**Pros**:
- Zero breakage to existing `GitProvider` implementations; read-only providers are unaffected.
- Capability introspection is explicit, typed, and synchronous — no runtime probing, no `Option`-returning wrappers on every method.
- Enables graceful degradation to sandbox clone-push fallbacks per-operation.
- Bitbucket partial coverage is expressed as `RemediationCaps { add_labels: false, .. }` — visible to callers without special-casing.
- Air-gapped and org-ceiling deployments simply never acquire an `Arc<dyn RemediationCapable>`; the type system enforces this.
- `MockProvider` implementation is straightforward and supports full job-layer unit testing.

**Cons**:
- New trait and `RemediationCaps` struct add a small amount of surface area to `ampel-providers`.
- Factory needs a second constructor (`build_remediation`); callers must choose the right one.
- `dyn RemediationCapable + Send + Sync` object bounds are slightly more verbose than `dyn GitProvider`.

**Verdict**: Accepted. Clean separation, zero regressions, and typed capability introspection make this the best fit for the feature's requirements and Ampel's deployment diversity.

### Option C: Separate `RemediationService` wrapping providers (Rejected)

Introduce a `RemediationService` struct in `ampel-core` or `ampel-worker` that holds an `Arc<dyn GitProvider>` and implements write operations by calling provider-specific HTTP clients directly (bypassing the trait).

**Pros**:
- No changes to `ampel-providers` trait surface.
- `RemediationService` can aggregate logic across multiple providers.

**Cons**:
- Duplicates provider routing logic (auth headers, base URLs, retry, rate-limit handling) that already lives in provider implementations.
- Provider-specific API details (GraphQL for GitHub rebase, REST for GitLab) leak into `ampel-core` or `ampel-worker`, breaking the provider abstraction.
- Capability introspection still requires per-provider conditionals in the service layer, but without a typed `RemediationCaps` struct — just `if provider_kind == Bitbucket` checks scattered through business logic.
- Testing requires mocking HTTP clients rather than a `MockProvider` that conforms to the trait.
- Harder to add a new provider: must update both the `GitProvider` implementations and the `RemediationService` routing table.

**Verdict**: Rejected. The indirection adds maintenance burden and undermines the provider abstraction that is central to Ampel's multi-platform design.

---

## Trade-off Analysis

| Aspect | Option A: Extend GitProvider | Option B: Supertrait (Chosen) | Option C: RemediationService |
|---|---|---|---|
| Breakage to existing providers | High — all must add ~10 methods | None | None |
| Read-only PAT deployments | Broken — stubs required | Fully supported | Fully supported |
| Capability introspection | Not possible (type-level) | Typed `RemediationCaps` struct | Ad-hoc provider-kind checks |
| Partial coverage (Bitbucket) | Invisible / panic risk | Expressed in `RemediationCaps` | Scattered conditionals |
| Air-gapped / org-ceiling safety | Requires runtime guard | Type-system enforced | Requires runtime guard |
| Provider abstraction integrity | Violated | Preserved | Violated |
| Test ergonomics | Poor (all stubs needed) | Good (`MockProvider` extends supertrait) | Poor (HTTP mock required) |
| Code duplication | Low | Low | High (duplicates provider routing) |
| New provider onboarding | Harder (write stubs required) | Optional (only if write needed) | Harder (two places to update) |
| Verbosity at call sites | Low | Slightly higher (`dyn RemediationCapable`) | Higher (`RemediationService` + `GitProvider`) |

---

## Consequences

### Positive

- All existing `GitProvider` implementations compile and behave identically after this change; no regressions in dashboard, polling, or merge flows.
- Remediation jobs receive a strongly-typed capability declaration before issuing any write call, enabling safe fallback routing with no panics.
- Bitbucket's thinner API coverage is a first-class concern expressed in the type system, not a runtime surprise.
- Air-gapped and org-ceiling deployments can prohibit remediation at the factory layer (`build_remediation` is never called) without any code changes to provider implementations.
- New providers can be added as read-only (`GitProvider` only) and upgraded to write-capable (`RemediationCapable`) independently.

### Negative

- `ampel-providers` gains a new public module (`remediation.rs`) and a new exported struct (`RemediationCaps`); downstream crates importing `ampel-providers` will see additional public items.
- The provider factory gains a second constructor path; callers working with remediation must explicitly request the `RemediationCapable` variant rather than using the existing `build` function.
- `BitbucketProvider` requires the most implementation work: it must implement all supertrait methods even for unsupported operations (returning `Err(ProviderError::NotSupported)`) and must set `add_labels: false` in `capabilities()`. This is more work than a stub but less than a full implementation.

### Neutral

- `MockProvider` grows ~10 new method implementations; these are mechanical and well-covered by the existing mock infrastructure pattern.
- The `RemediationCaps` struct is likely to gain new fields as the feature evolves (e.g., `rebase_pull_request`, `squash_merge`); this is additive and non-breaking as long as `Default` is derived.
- No database schema changes are required for this decision; `RemediationCaps` is a runtime-only struct.

---

## Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| `dyn RemediationCapable + Send + Sync` not object-safe if a new method violates object safety rules | Medium | Review every new method signature against object safety constraints before merging; CI enforces compilation. |
| `capabilities()` returning stale flags if PAT scope changes after provider construction | Low | `capabilities()` reflects API surface, not PAT scope; PAT-scope errors surface as `Err(ProviderError::Forbidden)` at call time and are handled by the fallback router. |
| Bitbucket `update_branch_from_base` REST support is undocumented or removed | Medium | Set `update_branch_from_base: false` in Bitbucket's `RemediationCaps`; fall back to sandbox clone-rebase-push. Add an integration test against Bitbucket's API before enabling in production. |
| `RemediationCaps` struct grows large and becomes hard to maintain | Low | Keep fields boolean and flat; document each flag in the struct definition. Deprecate flags rather than removing them. |
| Two factory constructors cause confusion about which to use | Low | Document the distinction in `factory.rs` doc comments; lint for accidental `build` usage in remediation job code via a Clippy allow-list. |
| Supertrait bound `RemediationCapable: GitProvider` forces boxing at two trait levels in some async contexts | Low | Use `Arc<dyn RemediationCapable + Send + Sync>` consistently; the supertrait bound means the same `Arc` satisfies both `dyn GitProvider` and `dyn RemediationCapable` coercions where needed. |

---

## Related ADRs

- ADR-001: Locale Middleware State Access Pattern — establishes the precedent of adding new Axum middleware/extension patterns without modifying existing request-handling traits.
