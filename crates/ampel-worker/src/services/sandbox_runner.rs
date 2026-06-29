//! Podman/Docker-backed [`SandboxRunner`] (ADR-003 / ADR-005).
//!
//! The mechanical consolidation — clone, sequential `git merge --no-ff` of each
//! source PR branch, lockfile regeneration, push, open consolidating PR — runs
//! inside an isolated OCI container. The **pure** decision logic (runtime
//! detection, lockfile classification + regen command, merge sequencing,
//! conflict parsing) is factored into free functions that are unit-tested here;
//! the actual subprocess/container invocation is a thin wrapper that is *not*
//! exercised in CI (no containers/network on the test runners).
//!
//! Security invariants:
//! - The PAT is passed via env/tmpfs to git, never as a CLI arg, never logged.
//! - `GIT_TERMINAL_PROMPT=0` / `GIT_ASKPASS=echo` prevent interactive prompts.
//! - No force-push primitive exists anywhere in this module.

use std::sync::Arc;
use std::time::Duration;

use ampel_core::errors::{AmpelError, AmpelResult};
use ampel_core::remediation::{
    regen_command_for, HeuristicFingerprinter, LockfileKind, PrRef, RepoFingerprinter,
};
use ampel_core::services::{ConsolidationOutcome, ConsolidationSpec, SandboxRunner};
use async_trait::async_trait;

// ---------------------------------------------------------------------------
// Pure logic (unit-tested) — no I/O, no subprocess, no env reads.
// ---------------------------------------------------------------------------

// The following lockfile/conflict helpers are the pure decision logic the
// container entrypoint + output parser rely on. They are exercised by the unit
// tests below; `#[allow(dead_code)]` covers the window before the (CI-gated)
// container invocation path calls them from non-test code.

/// A recognized lockfile ecosystem requiring deterministic regeneration after a
/// merge touches it.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockfileClass {
    NpmPackageLock,
    PnpmLock,
    YarnLock,
    CargoLock,
    GoModules,
    PoetryLock,
    GemfileLock,
}

impl From<LockfileClass> for LockfileKind {
    fn from(class: LockfileClass) -> Self {
        match class {
            LockfileClass::NpmPackageLock => LockfileKind::PackageLockJson,
            LockfileClass::PnpmLock => LockfileKind::PnpmLock,
            LockfileClass::YarnLock => LockfileKind::YarnLock,
            LockfileClass::CargoLock => LockfileKind::CargoLock,
            LockfileClass::GoModules => LockfileKind::GoSum,
            LockfileClass::PoetryLock => LockfileKind::PoetryLock,
            LockfileClass::GemfileLock => LockfileKind::GemfileLock,
        }
    }
}

impl From<LockfileKind> for LockfileClass {
    fn from(kind: LockfileKind) -> Self {
        match kind {
            LockfileKind::PackageLockJson => LockfileClass::NpmPackageLock,
            LockfileKind::PnpmLock => LockfileClass::PnpmLock,
            LockfileKind::YarnLock => LockfileClass::YarnLock,
            LockfileKind::CargoLock => LockfileClass::CargoLock,
            LockfileKind::GoSum => LockfileClass::GoModules,
            LockfileKind::PoetryLock => LockfileClass::PoetryLock,
            LockfileKind::GemfileLock => LockfileClass::GemfileLock,
        }
    }
}

/// Classify a repo-relative path as a known lockfile, by file name (ADR-005).
/// Returns `None` for non-lockfiles.
///
/// Delegates to the canonical [`ampel_core::remediation::detect_lockfile_kind`]
/// so there is exactly one detection table; this wrapper preserves the worker's
/// historical [`LockfileClass`] surface.
#[allow(dead_code)]
pub fn detect_lockfile_class(path: &str) -> Option<LockfileClass> {
    ampel_core::remediation::detect_lockfile_kind(path).map(LockfileClass::from)
}

/// The deterministic regeneration command for a lockfile class (ADR-005 table).
/// Returned as argv (program first) so the caller never builds a shell string.
///
/// Delegates to the single source-of-truth table in `ampel-core`
/// ([`regen_command_for`]); the worker no longer carries its own copy, so the
/// consolidation path and the [`RepoFingerprinter`] can never diverge.
#[allow(dead_code)]
pub fn regen_command(class: LockfileClass) -> &'static [&'static str] {
    regen_command_for(LockfileKind::from(class))
}

/// A single git step in the consolidation sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GitStep {
    /// `git clone --depth <depth> <url> <dir>` then checkout the default branch.
    Clone { depth: u32 },
    /// `git checkout -b <branch>` for the deterministic consolidation branch.
    CreateBranch { branch: String },
    /// `git merge --no-ff origin/<source_branch>` for one source PR.
    Merge {
        pr_number: i32,
        source_branch: String,
    },
}

/// Build the ordered git step sequence for a consolidation: clone, create the
/// consolidation branch, then `merge --no-ff` each PR in the given (oldest-first)
/// order. Pure — produces the plan, executes nothing.
pub fn build_merge_sequence(branch: &str, prs: &[PrRef], clone_depth: u32) -> Vec<GitStep> {
    let mut steps = Vec::with_capacity(prs.len() + 2);
    steps.push(GitStep::Clone { depth: clone_depth });
    steps.push(GitStep::CreateBranch {
        branch: branch.to_string(),
    });
    for pr in prs {
        steps.push(GitStep::Merge {
            pr_number: pr.number,
            source_branch: pr.branch.clone(),
        });
    }
    steps
}

/// True when git stderr indicates a merge conflict (so the caller records a
/// `SkippedConflict` disposition rather than aborting the whole run).
#[allow(dead_code)]
pub fn parse_merge_conflict(stderr: &str) -> bool {
    let s = stderr.to_ascii_lowercase();
    s.contains("conflict") || s.contains("automatic merge failed") || s.contains("merge conflict")
}

/// Which container runtime to shell out to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxRuntime {
    Podman,
    Docker,
}

impl SandboxRuntime {
    pub fn binary(self) -> &'static str {
        match self {
            SandboxRuntime::Podman => "podman",
            SandboxRuntime::Docker => "docker",
        }
    }
}

/// Resolve the runtime from an explicit env value, falling back to auto-detection
/// against an injected PATH-presence predicate. Pure + testable: the caller
/// supplies `on_path` so no real filesystem/PATH access happens in tests.
pub fn detect_runtime(
    env_value: Option<&str>,
    on_path: impl Fn(&str) -> bool,
) -> AmpelResult<SandboxRuntime> {
    if let Some(v) = env_value {
        return match v.trim().to_ascii_lowercase().as_str() {
            "podman" => Ok(SandboxRuntime::Podman),
            "docker" => Ok(SandboxRuntime::Docker),
            other => Err(AmpelError::ConfigError(format!(
                "unknown AMPEL_SANDBOX_RUNTIME `{other}` (expected podman|docker)"
            ))),
        };
    }
    if on_path("podman") {
        Ok(SandboxRuntime::Podman)
    } else if on_path("docker") {
        Ok(SandboxRuntime::Docker)
    } else {
        Err(AmpelError::ConfigError(
            "no container runtime found on PATH (need podman or docker)".to_string(),
        ))
    }
}

/// Strip a secret value from arbitrary text so it can never reach a log line.
#[allow(dead_code)]
pub fn scrub_secret(text: &str, secret: &str) -> String {
    if secret.is_empty() {
        return text.to_string();
    }
    text.replace(secret, "***redacted***")
}

// ---------------------------------------------------------------------------
// Configuration (env-driven) + the thin container wrapper.
// ---------------------------------------------------------------------------

/// Resolved sandbox configuration. `from_env` reads the ADR-003 env knobs.
#[derive(Clone, Debug)]
pub struct SandboxConfig {
    pub runtime: SandboxRuntime,
    pub image: String,
    pub clone_depth: u32,
    pub subprocess_timeout: Duration,
}

impl SandboxConfig {
    /// Read configuration from the environment (production path).
    pub fn from_env() -> AmpelResult<Self> {
        let runtime = detect_runtime(
            std::env::var("AMPEL_SANDBOX_RUNTIME").ok().as_deref(),
            binary_on_path,
        )?;
        let image = std::env::var("AMPEL_SANDBOX_IMAGE")
            .unwrap_or_else(|_| "ghcr.io/ampel/remediation-sandbox:latest".to_string());
        let clone_depth = parse_env_u32("AMPEL_CLONE_DEPTH", 50);
        let timeout_secs = parse_env_u64("AMPEL_SUBPROCESS_TIMEOUT_SECS", 300);
        Ok(Self {
            runtime,
            image,
            clone_depth,
            subprocess_timeout: Duration::from_secs(timeout_secs),
        })
    }
}

fn parse_env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn parse_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Best-effort PATH lookup for a binary (no extra deps).
fn binary_on_path(bin: &str) -> bool {
    let Ok(path) = std::env::var("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(bin).is_file())
}

/// Production [`SandboxRunner`]: drives the consolidation inside a Podman/Docker
/// container. Constructed from env via [`SandboxConfig::from_env`].
///
/// NOTE: `run_consolidation` is a thin wrapper around the container invocation
/// and is intentionally not exercised in CI. The decision logic it relies on is
/// unit-tested above as pure functions.
pub struct PodmanSandboxRunner {
    config: SandboxConfig,
    /// Repo fingerprinter used to derive lockfile regen commands (and, later, the
    /// completion command) for a consolidation. Defaults to the pure
    /// [`HeuristicFingerprinter`]; the planned CICD Intelligence engine slots in
    /// here behind the same `Arc<dyn RepoFingerprinter>` with no other changes.
    #[allow(dead_code)] // consumed by the (CI-gated) container path + tests
    fingerprinter: Arc<dyn RepoFingerprinter>,
}

impl PodmanSandboxRunner {
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            fingerprinter: Arc::new(HeuristicFingerprinter::new()),
        }
    }

    /// Override the default heuristic fingerprinter (e.g. with the future CICD
    /// Intelligence engine). The trait is the only seam callers need.
    #[allow(dead_code)] // wired into the worker binary in a later slice
    pub fn with_fingerprinter(mut self, fingerprinter: Arc<dyn RepoFingerprinter>) -> Self {
        self.fingerprinter = fingerprinter;
        self
    }

    /// Construct from the environment; falls back to a Podman/Docker auto-detect.
    pub fn from_env() -> AmpelResult<Self> {
        Ok(Self::new(SandboxConfig::from_env()?))
    }

    /// Fingerprint-aware regen selection: given the repo's file listing and the
    /// set of conflicted lockfile paths a merge touched, resolve each path to its
    /// deterministic regen argv **via the injected fingerprinter** (not a local
    /// hardcoded pattern table). Paths the fingerprint does not recognize as a
    /// lockfile are dropped. Deterministic + side-effect-free.
    #[allow(dead_code)] // consumed by the (CI-gated) container path + tests
    pub async fn resolve_lockfile_regen(
        &self,
        files: &[String],
        conflicted_lockfiles: &[String],
    ) -> AmpelResult<Vec<(String, Vec<String>)>> {
        resolve_lockfile_regen_with(self.fingerprinter.as_ref(), files, conflicted_lockfiles).await
    }
}

/// Pure, injectable form of [`PodmanSandboxRunner::resolve_lockfile_regen`]:
/// fingerprints the repo, then maps each conflicted lockfile path to the
/// fingerprint-derived regen command. Lives free so it is unit-testable against
/// any [`RepoFingerprinter`] (default heuristic or the future engine) with no
/// container.
#[allow(dead_code)] // consumed by the (CI-gated) container path + tests
pub async fn resolve_lockfile_regen_with(
    fingerprinter: &dyn RepoFingerprinter,
    files: &[String],
    conflicted_lockfiles: &[String],
) -> AmpelResult<Vec<(String, Vec<String>)>> {
    let fingerprint = fingerprinter.fingerprint(files, None).await?;
    let mut out = Vec::with_capacity(conflicted_lockfiles.len());
    for path in conflicted_lockfiles {
        if let Some(argv) = fingerprint.regen_command_for_path(path) {
            out.push((
                path.clone(),
                argv.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            ));
        }
    }
    Ok(out)
}

#[async_trait]
impl SandboxRunner for PodmanSandboxRunner {
    async fn run_consolidation(
        &self,
        spec: ConsolidationSpec,
    ) -> AmpelResult<ConsolidationOutcome> {
        // Build the deterministic, oldest-first git plan (pure).
        let steps = build_merge_sequence(&spec.branch_name, &spec.prs, self.config.clone_depth);
        tracing::info!(
            run_id = %spec.run_id,
            runtime = self.config.runtime.binary(),
            image = %self.config.image,
            steps = steps.len(),
            "preparing sandboxed consolidation"
        );

        // The full container orchestration (write an entrypoint that performs the
        // git steps + lockfile regen, mount a tmpfs env-file carrying the PAT,
        // apply the egress allowlist, parse a final JSON result line) is a later
        // wiring step. It is deliberately gated out of the CI test path; the
        // CI-safe tests drive the orchestrator with `FakeSandboxRunner` instead.
        //
        // Returning an explicit error here keeps the production binary honest
        // until the container entrypoint ships, rather than silently "succeeding".
        let _ = (&self.config.subprocess_timeout, spec.credential.expose());
        Err(AmpelError::InternalError(
            "PodmanSandboxRunner container execution is not yet wired (Phase 2 slice 2 ships the \
             pure consolidation logic + entrypoint image; runtime invocation lands next)"
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pr(n: i32, branch: &str) -> PrRef {
        PrRef {
            number: n,
            title: format!("PR {n}"),
            branch: branch.to_string(),
        }
    }

    #[test]
    fn should_detect_each_lockfile_class_by_filename() {
        assert_eq!(
            detect_lockfile_class("frontend/package-lock.json"),
            Some(LockfileClass::NpmPackageLock)
        );
        assert_eq!(
            detect_lockfile_class("pnpm-lock.yaml"),
            Some(LockfileClass::PnpmLock)
        );
        assert_eq!(
            detect_lockfile_class("app/yarn.lock"),
            Some(LockfileClass::YarnLock)
        );
        assert_eq!(
            detect_lockfile_class("Cargo.lock"),
            Some(LockfileClass::CargoLock)
        );
        assert_eq!(
            detect_lockfile_class("go.sum"),
            Some(LockfileClass::GoModules)
        );
        assert_eq!(
            detect_lockfile_class("go.mod"),
            Some(LockfileClass::GoModules)
        );
        assert_eq!(
            detect_lockfile_class("poetry.lock"),
            Some(LockfileClass::PoetryLock)
        );
        assert_eq!(
            detect_lockfile_class("Gemfile.lock"),
            Some(LockfileClass::GemfileLock)
        );
    }

    #[test]
    fn should_return_none_for_non_lockfile_paths() {
        assert_eq!(detect_lockfile_class("src/main.rs"), None);
        assert_eq!(detect_lockfile_class("package.json"), None);
        assert_eq!(detect_lockfile_class("README.md"), None);
    }

    #[test]
    fn should_map_each_class_to_its_regen_command() {
        assert_eq!(
            regen_command(LockfileClass::NpmPackageLock),
            ["npm", "install", "--package-lock-only"]
        );
        assert_eq!(
            regen_command(LockfileClass::PnpmLock),
            ["pnpm", "install", "--frozen-lockfile=false"]
        );
        assert_eq!(
            regen_command(LockfileClass::YarnLock),
            ["yarn", "install", "--mode", "update-lockfile"]
        );
        assert_eq!(
            regen_command(LockfileClass::CargoLock),
            ["cargo", "generate-lockfile"]
        );
        assert_eq!(
            regen_command(LockfileClass::GoModules),
            ["go", "mod", "tidy"]
        );
        assert_eq!(
            regen_command(LockfileClass::PoetryLock),
            ["poetry", "lock", "--no-update"]
        );
        assert_eq!(
            regen_command(LockfileClass::GemfileLock),
            ["bundle", "lock", "--update"]
        );
    }

    #[test]
    fn should_build_oldest_first_merge_sequence_with_clone_and_branch() {
        // Arrange
        let prs = [pr(1, "feature/a"), pr(2, "feature/b")];

        // Act
        let steps = build_merge_sequence("ampel/remediation/run", &prs, 50);

        // Assert: clone, create-branch, then one --no-ff merge per PR in order.
        assert_eq!(
            steps,
            vec![
                GitStep::Clone { depth: 50 },
                GitStep::CreateBranch {
                    branch: "ampel/remediation/run".to_string()
                },
                GitStep::Merge {
                    pr_number: 1,
                    source_branch: "feature/a".to_string()
                },
                GitStep::Merge {
                    pr_number: 2,
                    source_branch: "feature/b".to_string()
                },
            ]
        );
    }

    #[test]
    fn should_detect_git_merge_conflict_in_stderr() {
        assert!(parse_merge_conflict(
            "CONFLICT (content): Merge conflict in Cargo.lock"
        ));
        assert!(parse_merge_conflict(
            "Automatic merge failed; fix conflicts"
        ));
        assert!(!parse_merge_conflict("Updating 1234..5678\nFast-forward"));
    }

    #[test]
    fn should_prefer_explicit_runtime_env_over_autodetect() {
        // Arrange + Act: env wins even if neither binary is "on PATH".
        let podman = detect_runtime(Some("podman"), |_| false).unwrap();
        let docker = detect_runtime(Some("docker"), |_| false).unwrap();

        // Assert
        assert_eq!(podman, SandboxRuntime::Podman);
        assert_eq!(docker, SandboxRuntime::Docker);
    }

    #[test]
    fn should_reject_unknown_runtime_env_value() {
        assert!(detect_runtime(Some("containerd"), |_| true).is_err());
    }

    #[test]
    fn should_autodetect_podman_then_docker_from_path() {
        // podman present -> podman.
        assert_eq!(
            detect_runtime(None, |b| b == "podman").unwrap(),
            SandboxRuntime::Podman
        );
        // only docker present -> docker.
        assert_eq!(
            detect_runtime(None, |b| b == "docker").unwrap(),
            SandboxRuntime::Docker
        );
        // neither present -> error.
        assert!(detect_runtime(None, |_| false).is_err());
    }

    #[test]
    fn should_scrub_secret_from_text() {
        // Arrange
        let out = "fatal: could not read Password for 'https://ghp_supersecret@github.com'";

        // Act
        let scrubbed = scrub_secret(out, "ghp_supersecret");

        // Assert
        assert!(!scrubbed.contains("ghp_supersecret"));
        assert!(scrubbed.contains("***redacted***"));
    }

    // --- Phase 5c: fingerprint-aware regen selection -----------------------

    /// Parity guard: routing a class through the fingerprint API yields the exact
    /// same argv as the legacy `regen_command`, for all 7 package managers — i.e.
    /// the worker delegate and the single source-of-truth table cannot diverge.
    #[test]
    fn should_match_legacy_regen_command_for_all_seven_package_managers() {
        for class in [
            LockfileClass::NpmPackageLock,
            LockfileClass::PnpmLock,
            LockfileClass::YarnLock,
            LockfileClass::CargoLock,
            LockfileClass::GoModules,
            LockfileClass::PoetryLock,
            LockfileClass::GemfileLock,
        ] {
            // legacy worker surface vs canonical core table — must be identical.
            assert_eq!(
                regen_command(class),
                regen_command_for(LockfileKind::from(class)),
                "regen table diverged for {class:?}"
            );
        }
    }

    #[tokio::test]
    async fn should_select_regen_command_via_fingerprinter_for_conflicted_lockfile() {
        // Arrange: a polyglot repo listing + two conflicted lockfiles.
        let runner = PodmanSandboxRunner::new(SandboxConfig {
            runtime: SandboxRuntime::Podman,
            image: "img".into(),
            clone_depth: 1,
            subprocess_timeout: Duration::from_secs(1),
        });
        let files = vec![
            "Cargo.toml".to_string(),
            "Cargo.lock".to_string(),
            "frontend/package.json".to_string(),
            "frontend/package-lock.json".to_string(),
        ];
        let conflicted = vec![
            "Cargo.lock".to_string(),
            "frontend/package-lock.json".to_string(),
        ];

        // Act: the consolidation path resolves regen commands through the
        // injected fingerprinter (NOT a local hardcoded pattern match).
        let resolved = runner
            .resolve_lockfile_regen(&files, &conflicted)
            .await
            .unwrap();

        // Assert: each conflicted lockfile mapped to its fingerprint-derived argv.
        assert_eq!(
            resolved,
            vec![
                (
                    "Cargo.lock".to_string(),
                    vec!["cargo".to_string(), "generate-lockfile".to_string()]
                ),
                (
                    "frontend/package-lock.json".to_string(),
                    vec![
                        "npm".to_string(),
                        "install".to_string(),
                        "--package-lock-only".to_string()
                    ]
                ),
            ]
        );
    }

    #[tokio::test]
    async fn should_drop_non_lockfile_paths_from_regen_selection() {
        // A path the fingerprint does not recognize as a lockfile is skipped.
        let runner = PodmanSandboxRunner::new(SandboxConfig {
            runtime: SandboxRuntime::Podman,
            image: "img".into(),
            clone_depth: 1,
            subprocess_timeout: Duration::from_secs(1),
        });
        let files = vec!["Cargo.toml".to_string(), "Cargo.lock".to_string()];
        let conflicted = vec!["Cargo.lock".to_string(), "src/main.rs".to_string()];

        let resolved = runner
            .resolve_lockfile_regen(&files, &conflicted)
            .await
            .unwrap();

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].0, "Cargo.lock");
    }
}
