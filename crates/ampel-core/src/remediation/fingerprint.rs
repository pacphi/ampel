//! Repository fingerprinting for fingerprint-aware remediation (Phase 5c).
//!
//! The mechanical consolidation + agentic harness need two repo-shaped facts:
//!   1. for each conflicted lockfile, which deterministic regen command to run;
//!   2. which build/test command is the "goal" / completion check.
//!
//! Phase 2 expressed (1) as hardcoded file-name → command pattern matches living
//! in the worker's `sandbox_runner`. This module replaces those ad-hoc tables
//! with a [`RepoFingerprint`] abstraction:
//!
//! - [`RepoFingerprinter`] is the seam. The default [`HeuristicFingerprinter`]
//!   reasons purely over a repo file listing (and optionally key file contents),
//!   so it is deterministic and unit-testable with no clone/network/container.
//! - The real "CICD Intelligence engine" is **planned, not built**. When it
//!   ships it becomes another `RepoFingerprinter` impl injected behind the same
//!   `Arc<dyn RepoFingerprinter>` seam — callers do not change.
//!
//! ## Single source of truth for the regen table
//! [`regen_command_for`] is the ONE canonical lockfile → regen-argv mapping
//! (ADR-005). The worker's `sandbox_runner::regen_command` now delegates here via
//! the [`LockfileKind`] conversion, so there is exactly one table and no risk of
//! the consolidation path and the fingerprinter diverging.

use std::collections::HashMap;

use crate::errors::AmpelResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A package-manager / language ecosystem detected in a repository.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ecosystem {
    Cargo,
    Go,
    Poetry,
    Bundler,
    Pnpm,
    Yarn,
    Npm,
}

impl Ecosystem {
    /// Deterministic priority order. When a repo is polyglot, the first
    /// ecosystem in this order supplies the primary build/test "goal" command.
    /// Lower-churn, compile-checked ecosystems lead so the completion condition
    /// is the strongest available signal.
    const PRIORITY: [Ecosystem; 7] = [
        Ecosystem::Cargo,
        Ecosystem::Go,
        Ecosystem::Poetry,
        Ecosystem::Bundler,
        Ecosystem::Pnpm,
        Ecosystem::Yarn,
        Ecosystem::Npm,
    ];

    /// The canonical build command for this ecosystem.
    pub fn build_command(self) -> &'static str {
        match self {
            Ecosystem::Cargo => "cargo build",
            Ecosystem::Go => "go build ./...",
            Ecosystem::Poetry => "poetry build",
            Ecosystem::Bundler => "bundle install",
            Ecosystem::Pnpm => "pnpm run build",
            Ecosystem::Yarn => "yarn build",
            Ecosystem::Npm => "npm run build",
        }
    }

    /// The canonical test command for this ecosystem (the completion check).
    pub fn test_command(self) -> &'static str {
        match self {
            Ecosystem::Cargo => "cargo test",
            Ecosystem::Go => "go test ./...",
            Ecosystem::Poetry => "poetry run pytest",
            Ecosystem::Bundler => "bundle exec rake test",
            Ecosystem::Pnpm => "pnpm test",
            Ecosystem::Yarn => "yarn test",
            Ecosystem::Npm => "npm test",
        }
    }
}

/// A recognized lockfile kind requiring deterministic regeneration after a merge
/// touches it (ADR-005).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockfileKind {
    PackageLockJson,
    PnpmLock,
    YarnLock,
    CargoLock,
    GoSum,
    PoetryLock,
    GemfileLock,
}

impl LockfileKind {
    /// The ecosystem this lockfile belongs to.
    pub fn ecosystem(self) -> Ecosystem {
        match self {
            LockfileKind::PackageLockJson => Ecosystem::Npm,
            LockfileKind::PnpmLock => Ecosystem::Pnpm,
            LockfileKind::YarnLock => Ecosystem::Yarn,
            LockfileKind::CargoLock => Ecosystem::Cargo,
            LockfileKind::GoSum => Ecosystem::Go,
            LockfileKind::PoetryLock => Ecosystem::Poetry,
            LockfileKind::GemfileLock => Ecosystem::Bundler,
        }
    }

    /// The deterministic regen command for this lockfile (delegates to the single
    /// canonical table).
    pub fn regen_command(self) -> &'static [&'static str] {
        regen_command_for(self)
    }
}

/// **The** canonical lockfile → regeneration argv table (ADR-005). Program
/// first; callers never build a shell string. This is the single source of truth
/// the worker's consolidation path and the fingerprinter both route through.
pub fn regen_command_for(kind: LockfileKind) -> &'static [&'static str] {
    match kind {
        LockfileKind::PackageLockJson => &["npm", "install", "--package-lock-only"],
        LockfileKind::PnpmLock => &["pnpm", "install", "--frozen-lockfile=false"],
        LockfileKind::YarnLock => &["yarn", "install", "--mode", "update-lockfile"],
        LockfileKind::CargoLock => &["cargo", "generate-lockfile"],
        LockfileKind::GoSum => &["go", "mod", "tidy"],
        LockfileKind::PoetryLock => &["poetry", "lock", "--no-update"],
        LockfileKind::GemfileLock => &["bundle", "lock", "--update"],
    }
}

/// Classify a repo-relative path as a known lockfile, by file name (ADR-005).
/// Returns `None` for non-lockfiles. Single source of truth for lockfile
/// detection — the worker's `detect_lockfile_class` delegates here.
pub fn detect_lockfile_kind(path: &str) -> Option<LockfileKind> {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    match name {
        "package-lock.json" => Some(LockfileKind::PackageLockJson),
        "pnpm-lock.yaml" => Some(LockfileKind::PnpmLock),
        "yarn.lock" => Some(LockfileKind::YarnLock),
        "Cargo.lock" => Some(LockfileKind::CargoLock),
        "go.sum" | "go.mod" => Some(LockfileKind::GoSum),
        "poetry.lock" => Some(LockfileKind::PoetryLock),
        "Gemfile.lock" => Some(LockfileKind::GemfileLock),
        _ => None,
    }
}

/// A repository's inferred build/dependency shape.
///
/// `ecosystems` is ordered by [`Ecosystem::PRIORITY`]; `build_command` /
/// `test_command` reflect the highest-priority ecosystem present (the "goal").
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RepoFingerprint {
    pub ecosystems: Vec<Ecosystem>,
    pub lockfiles: Vec<LockfileKind>,
    pub build_command: Option<String>,
    pub test_command: Option<String>,
}

impl RepoFingerprint {
    /// The regen command for a specific conflicted lockfile path, derived from
    /// this fingerprint. Returns `None` when the path is not a recognized
    /// lockfile. This is the fingerprint-aware replacement for the old
    /// file-name → command pattern match in the consolidation path.
    pub fn regen_command_for_path(&self, path: &str) -> Option<&'static [&'static str]> {
        detect_lockfile_kind(path).map(regen_command_for)
    }

    /// The fingerprint-derived completion command (the test/goal command), if any.
    pub fn completion_command(&self) -> Option<&str> {
        self.test_command.as_deref()
    }
}

/// Resolve the effective completion (build/test) command: an explicit
/// playbook-supplied command always wins; otherwise fall back to the
/// fingerprint-derived one. Optional + backward-compatible — when the playbook
/// specifies a goal nothing changes; when it does not, the fingerprint enriches
/// it. Pure and deterministic.
pub fn effective_completion_command(
    playbook_supplied: Option<&str>,
    fingerprint: &RepoFingerprint,
) -> Option<String> {
    match playbook_supplied {
        Some(cmd) if !cmd.trim().is_empty() => Some(cmd.to_string()),
        _ => fingerprint.completion_command().map(str::to_string),
    }
}

/// Infers a [`RepoFingerprint`] from a repository's shape.
///
/// The input is a file listing (repo-relative paths) plus optionally the
/// contents of a few key manifest files, so implementations stay pure/testable
/// with no real clone. The default [`HeuristicFingerprinter`] ignores
/// `file_contents`; the future CICD Intelligence engine may consult them.
#[async_trait]
pub trait RepoFingerprinter: Send + Sync {
    async fn fingerprint(
        &self,
        files: &[String],
        file_contents: Option<&HashMap<String, String>>,
    ) -> AmpelResult<RepoFingerprint>;
}

/// The default, pure heuristic fingerprinter.
///
/// Infers ecosystems + lockfiles from the presence of well-known marker files
/// and derives build/test commands per ecosystem. Deterministic: the same file
/// listing always yields the same fingerprint (ecosystems + lockfiles emitted in
/// [`Ecosystem::PRIORITY`] order).
///
/// The real CICD Intelligence engine (planned) will be a separate
/// `RepoFingerprinter` impl that can consult `file_contents`, CI config, and a
/// learned model; it slots in behind the same `Arc<dyn RepoFingerprinter>`.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeuristicFingerprinter;

impl HeuristicFingerprinter {
    pub fn new() -> Self {
        Self
    }

    /// Marker files (by base name) that imply an ecosystem is present, beyond the
    /// lockfiles themselves (which are detected via [`detect_lockfile_kind`]).
    fn ecosystem_for_marker(name: &str) -> Option<Ecosystem> {
        match name {
            "package-lock.json" => Some(Ecosystem::Npm),
            "pnpm-lock.yaml" => Some(Ecosystem::Pnpm),
            "yarn.lock" => Some(Ecosystem::Yarn),
            "Cargo.toml" | "Cargo.lock" => Some(Ecosystem::Cargo),
            "go.mod" | "go.sum" => Some(Ecosystem::Go),
            "pyproject.toml" | "poetry.lock" => Some(Ecosystem::Poetry),
            "Gemfile" | "Gemfile.lock" => Some(Ecosystem::Bundler),
            _ => None,
        }
    }
}

#[async_trait]
impl RepoFingerprinter for HeuristicFingerprinter {
    async fn fingerprint(
        &self,
        files: &[String],
        _file_contents: Option<&HashMap<String, String>>,
    ) -> AmpelResult<RepoFingerprint> {
        let base_names: Vec<&str> = files
            .iter()
            .map(|p| p.rsplit(['/', '\\']).next().unwrap_or(p.as_str()))
            .collect();

        // Ecosystems: emit in canonical priority order, de-duplicated.
        let ecosystems: Vec<Ecosystem> = Ecosystem::PRIORITY
            .into_iter()
            .filter(|eco| {
                base_names
                    .iter()
                    .any(|n| Self::ecosystem_for_marker(n) == Some(*eco))
            })
            .collect();

        // Lockfiles: emit in ecosystem-priority order, de-duplicated.
        let mut lockfiles: Vec<LockfileKind> = Vec::new();
        for eco in &ecosystems {
            for name in &base_names {
                if let Some(kind) = detect_lockfile_kind(name) {
                    if kind.ecosystem() == *eco && !lockfiles.contains(&kind) {
                        lockfiles.push(kind);
                    }
                }
            }
        }

        // Build/test "goal" come from the highest-priority ecosystem present.
        let primary = ecosystems.first().copied();
        let build_command = primary.map(|e| e.build_command().to_string());
        let test_command = primary.map(|e| e.test_command().to_string());

        Ok(RepoFingerprint {
            ecosystems,
            lockfiles,
            build_command,
            test_command,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|s| s.to_string()).collect()
    }

    async fn fp(paths: &[&str]) -> RepoFingerprint {
        HeuristicFingerprinter::new()
            .fingerprint(&files(paths), None)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn should_fingerprint_cargo_repo() {
        let f = fp(&["Cargo.toml", "Cargo.lock", "src/main.rs"]).await;
        assert_eq!(f.ecosystems, vec![Ecosystem::Cargo]);
        assert_eq!(f.lockfiles, vec![LockfileKind::CargoLock]);
        assert_eq!(f.build_command.as_deref(), Some("cargo build"));
        assert_eq!(f.test_command.as_deref(), Some("cargo test"));
        assert_eq!(
            f.regen_command_for_path("Cargo.lock"),
            Some(&["cargo", "generate-lockfile"][..])
        );
    }

    #[tokio::test]
    async fn should_fingerprint_pnpm_repo() {
        let f = fp(&["package.json", "pnpm-lock.yaml"]).await;
        assert_eq!(f.ecosystems, vec![Ecosystem::Pnpm]);
        assert_eq!(f.lockfiles, vec![LockfileKind::PnpmLock]);
        assert_eq!(f.test_command.as_deref(), Some("pnpm test"));
        assert_eq!(
            f.regen_command_for_path("frontend/pnpm-lock.yaml"),
            Some(&["pnpm", "install", "--frozen-lockfile=false"][..])
        );
    }

    #[tokio::test]
    async fn should_fingerprint_polyglot_repo_with_per_lockfile_regen() {
        // npm + cargo polyglot: both ecosystems detected; Cargo wins the goal
        // (higher priority); each lockfile resolves its own regen command.
        let f = fp(&[
            "Cargo.toml",
            "Cargo.lock",
            "frontend/package.json",
            "frontend/package-lock.json",
        ])
        .await;
        assert_eq!(f.ecosystems, vec![Ecosystem::Cargo, Ecosystem::Npm]);
        assert_eq!(
            f.lockfiles,
            vec![LockfileKind::CargoLock, LockfileKind::PackageLockJson]
        );
        // Goal comes from the highest-priority ecosystem (Cargo).
        assert_eq!(f.build_command.as_deref(), Some("cargo build"));
        assert_eq!(f.test_command.as_deref(), Some("cargo test"));
        // Per-lockfile regen is fingerprint-derived, not hardcoded by the caller.
        assert_eq!(
            f.regen_command_for_path("Cargo.lock"),
            Some(&["cargo", "generate-lockfile"][..])
        );
        assert_eq!(
            f.regen_command_for_path("frontend/package-lock.json"),
            Some(&["npm", "install", "--package-lock-only"][..])
        );
    }

    #[tokio::test]
    async fn should_emit_empty_fingerprint_for_unknown_repo() {
        let f = fp(&["README.md", "src/lib.rs"]).await;
        assert!(f.ecosystems.is_empty());
        assert!(f.lockfiles.is_empty());
        assert_eq!(f.build_command, None);
        assert_eq!(f.test_command, None);
    }

    #[test]
    fn should_map_every_lockfile_kind_to_its_canonical_regen_command() {
        // Parity: the single source-of-truth table.
        assert_eq!(
            regen_command_for(LockfileKind::PackageLockJson),
            ["npm", "install", "--package-lock-only"]
        );
        assert_eq!(
            regen_command_for(LockfileKind::PnpmLock),
            ["pnpm", "install", "--frozen-lockfile=false"]
        );
        assert_eq!(
            regen_command_for(LockfileKind::YarnLock),
            ["yarn", "install", "--mode", "update-lockfile"]
        );
        assert_eq!(
            regen_command_for(LockfileKind::CargoLock),
            ["cargo", "generate-lockfile"]
        );
        assert_eq!(
            regen_command_for(LockfileKind::GoSum),
            ["go", "mod", "tidy"]
        );
        assert_eq!(
            regen_command_for(LockfileKind::PoetryLock),
            ["poetry", "lock", "--no-update"]
        );
        assert_eq!(
            regen_command_for(LockfileKind::GemfileLock),
            ["bundle", "lock", "--update"]
        );
    }

    #[test]
    fn should_detect_each_lockfile_kind_by_filename() {
        assert_eq!(
            detect_lockfile_kind("frontend/package-lock.json"),
            Some(LockfileKind::PackageLockJson)
        );
        assert_eq!(detect_lockfile_kind("go.mod"), Some(LockfileKind::GoSum));
        assert_eq!(detect_lockfile_kind("go.sum"), Some(LockfileKind::GoSum));
        assert_eq!(detect_lockfile_kind("src/main.rs"), None);
    }

    #[test]
    fn should_prefer_playbook_completion_command_over_fingerprint() {
        let f = RepoFingerprint {
            test_command: Some("cargo test".into()),
            ..Default::default()
        };
        assert_eq!(
            effective_completion_command(Some("make verify"), &f).as_deref(),
            Some("make verify")
        );
    }

    #[test]
    fn should_fall_back_to_fingerprint_completion_command_when_playbook_silent() {
        let f = RepoFingerprint {
            test_command: Some("cargo test".into()),
            ..Default::default()
        };
        assert_eq!(
            effective_completion_command(None, &f).as_deref(),
            Some("cargo test")
        );
        // Empty/whitespace playbook value is treated as "unspecified".
        assert_eq!(
            effective_completion_command(Some("   "), &f).as_deref(),
            Some("cargo test")
        );
    }

    #[test]
    fn should_round_trip_fingerprint_json_snake_case() {
        let f = RepoFingerprint {
            ecosystems: vec![Ecosystem::Cargo, Ecosystem::Npm],
            lockfiles: vec![LockfileKind::CargoLock, LockfileKind::PackageLockJson],
            build_command: Some("cargo build".into()),
            test_command: Some("cargo test".into()),
        };
        let json = serde_json::to_string(&f).unwrap();
        assert!(json.contains("\"cargo\""));
        assert!(json.contains("\"package_lock_json\""));
        assert_eq!(serde_json::from_str::<RepoFingerprint>(&json).unwrap(), f);
    }
}
