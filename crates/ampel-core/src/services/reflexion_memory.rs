//! Reflexion memory — vector-recall self-improvement for the agentic tier
//! (Phase 5b+, feature-flagged).
//!
//! This is the *self-improvement* sibling of the deterministic
//! [`learning_signal`](super::learning_signal) path. Where `learning_signal`
//! feeds an AGGREGATE, deterministic provider-ordering bias (the air-gap-safe,
//! always-on default decision path), `ReflexionMemory` lets a session RECALL the
//! text of *similar prior remediation attempts* and surface them to the model as
//! additional **untrusted** context ("have we seen this failure before, and what
//! happened?"). It is strictly additive and OFF by default.
//!
//! # Layering (mirrors the rest of the remediation feature)
//! `ampel-core` cannot depend on `ampel-db` or on a vector store (dependency
//! cycle + air-gap/CI constraints), so this module owns ONLY:
//!
//! - the [`TrajectoryRecord`] value object (no secrets — see below),
//! - the [`ReflexionMemory`] trait (`Arc<dyn>` for injection),
//! - [`NoopReflexionMemory`] — the always-empty DEFAULT used when the capability
//!   is not configured; recall returns nothing and record is a no-op, so the
//!   harness behaves byte-identically to having no memory at all,
//! - the pure, unit-tested ranking + digest helpers
//!   ([`rank_by_similarity`], [`context_digest_from_logs`]),
//! - an [`InMemoryReflexionMemory`] fake (token-overlap similarity, NO real
//!   embeddings / vector store) for deterministic CI tests.
//!
//! The real vector-backed implementation (`VectorReflexionMemory`, ruvector-core
//! plus a local hashing embedder) lives in `ampel-worker` behind the `reflexion`
//! cargo feature and compiles out entirely when the feature is off.
//!
//! # Security
//! - **No secrets.** A [`TrajectoryRecord`] carries the provider *kind* only
//!   (never a key/endpoint), and its `context_digest` is a token-filtered slice
//!   of the CI logs ([`context_digest_from_logs`]) that drops long, high-entropy
//!   tokens (api keys, hashes) before storage/embedding.
//! - **Prompt-injection-safe.** Recalled trajectories are *untrusted data*. The
//!   harness injects them ONLY as `is_untrusted_data` context blocks, NEVER into
//!   the trusted `system` instruction channel.

use crate::errors::AmpelResult;
use crate::remediation::{FailureClass, ProviderKind};
use crate::services::LearningOutcome;
use async_trait::async_trait;

/// One recalled (or recorded) prior remediation attempt.
///
/// Deliberately carries NO secrets: `provider` is the [`ProviderKind`] only, and
/// `context_digest` is produced by [`context_digest_from_logs`] which strips
/// long high-entropy tokens before the text is ever stored or embedded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrajectoryRecord {
    /// The classified failure this attempt addressed.
    pub failure_class: FailureClass,
    /// Which provider KIND drove the attempt (never a credential).
    pub provider: ProviderKind,
    /// Token-filtered, secret-stripped digest of the failing logs. Used for
    /// similarity matching (and embedding, in the vector-backed impl).
    pub context_digest: String,
    /// Terminal outcome of the attempt (passed / exhausted).
    pub outcome: LearningOutcome,
    /// Short human-readable note (no secrets) describing the attempt.
    pub summary: String,
}

/// Vector-recall self-improvement seam (Phase 5b+). `Arc<dyn>` so the harness is
/// agnostic of the concrete backend (noop / in-memory fake / vector store).
///
/// Implementations MUST be air-gap-safe: no external egress, local embeddings
/// only. The default ([`NoopReflexionMemory`]) recalls nothing.
#[async_trait]
pub trait ReflexionMemory: Send + Sync {
    /// Persist one trajectory for future recall. Best-effort; the caller never
    /// fails a remediation run over a memory write.
    async fn record_trajectory(&self, rec: TrajectoryRecord) -> AmpelResult<()>;

    /// Recall up to `k` prior trajectories for `failure_class`, ranked by
    /// similarity to `query_text` (most similar first).
    async fn recall_similar(
        &self,
        failure_class: FailureClass,
        query_text: &str,
        k: usize,
    ) -> AmpelResult<Vec<TrajectoryRecord>>;
}

/// The DEFAULT memory: records nothing, recalls nothing. Wired whenever the
/// `reflexion` capability is not configured so the harness is byte-identical to
/// having no memory at all (zero behavior change).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopReflexionMemory;

#[async_trait]
impl ReflexionMemory for NoopReflexionMemory {
    async fn record_trajectory(&self, _rec: TrajectoryRecord) -> AmpelResult<()> {
        Ok(())
    }

    async fn recall_similar(
        &self,
        _failure_class: FailureClass,
        _query_text: &str,
        _k: usize,
    ) -> AmpelResult<Vec<TrajectoryRecord>> {
        Ok(Vec::new())
    }
}

/// Build a secret-stripped, token-normalized digest of raw CI logs.
///
/// Splits on non-alphanumeric (keeping `_`), lowercases, DROPS tokens longer
/// than [`MAX_TOKEN_LEN`] (api keys / commit hashes / base64 blobs are
/// high-entropy and long — keeping them would both risk a secret leak and add
/// noise to similarity), and caps the result at [`MAX_DIGEST_TOKENS`] tokens.
/// Pure and deterministic.
pub fn context_digest_from_logs(logs: &str) -> String {
    logs.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .filter(|t| t.len() <= MAX_TOKEN_LEN)
        .take(MAX_DIGEST_TOKENS)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Tokens longer than this are dropped from a digest (likely secrets/hashes).
const MAX_TOKEN_LEN: usize = 24;
/// Upper bound on tokens retained in a digest.
const MAX_DIGEST_TOKENS: usize = 64;

/// Pure token-overlap similarity: the count of distinct lowercased tokens shared
/// between `a` and `b`. Higher = more similar. No allocation of intermediate
/// sets beyond the two token sets; deterministic.
pub fn token_overlap(a: &str, b: &str) -> usize {
    use std::collections::BTreeSet;
    let tokenize = |s: &str| -> BTreeSet<String> {
        s.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|t| !t.is_empty())
            .map(|t| t.to_ascii_lowercase())
            .collect()
    };
    let sa = tokenize(a);
    let sb = tokenize(b);
    sa.intersection(&sb).count()
}

/// Rank `candidates` by token-overlap similarity to `query_text`, drop
/// zero-overlap entries, and return the top `k` (most similar first).
///
/// The pure, deterministic core of the in-memory fake's recall (and a reference
/// ranking the vector-backed impl is sanity-checked against). Ties preserve the
/// candidates' original order (stable sort).
pub fn rank_by_similarity(
    query_text: &str,
    candidates: Vec<TrajectoryRecord>,
    k: usize,
) -> Vec<TrajectoryRecord> {
    let mut scored: Vec<(usize, TrajectoryRecord)> = candidates
        .into_iter()
        .map(|rec| (token_overlap(query_text, &rec.context_digest), rec))
        .filter(|(score, _)| *score > 0)
        .collect();
    // Highest overlap first; stable so ties keep insertion order.
    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    scored.into_iter().take(k).map(|(_, rec)| rec).collect()
}

#[cfg(any(test, feature = "test-utils"))]
pub use in_memory::InMemoryReflexionMemory;

#[cfg(any(test, feature = "test-utils"))]
mod in_memory {
    //! In-process fake — token-overlap similarity, NO embeddings / vector store.
    //! Deterministic; safe for default-feature CI.

    use super::*;
    use std::sync::Mutex;

    /// Records trajectories in memory and recalls them by token overlap.
    #[derive(Default)]
    pub struct InMemoryReflexionMemory {
        records: Mutex<Vec<TrajectoryRecord>>,
    }

    impl InMemoryReflexionMemory {
        pub fn new() -> Self {
            Self::default()
        }

        /// All recorded trajectories, in insertion order (for assertions).
        pub fn recorded(&self) -> Vec<TrajectoryRecord> {
            self.records.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ReflexionMemory for InMemoryReflexionMemory {
        async fn record_trajectory(&self, rec: TrajectoryRecord) -> AmpelResult<()> {
            self.records.lock().unwrap().push(rec);
            Ok(())
        }

        async fn recall_similar(
            &self,
            failure_class: FailureClass,
            query_text: &str,
            k: usize,
        ) -> AmpelResult<Vec<TrajectoryRecord>> {
            let candidates: Vec<TrajectoryRecord> = self
                .records
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.failure_class == failure_class)
                .cloned()
                .collect();
            Ok(rank_by_similarity(query_text, candidates, k))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(class: FailureClass, digest: &str, summary: &str) -> TrajectoryRecord {
        TrajectoryRecord {
            failure_class: class,
            provider: ProviderKind::Ollama,
            context_digest: digest.to_string(),
            outcome: LearningOutcome::Passed,
            summary: summary.to_string(),
        }
    }

    #[test]
    fn should_strip_long_high_entropy_tokens_from_digest() {
        let secret = "sk-abcdefghijklmnopqrstuvwxyz0123456789"; // > MAX_TOKEN_LEN
        let logs = format!("error E0001 build failed token={secret} retry");
        let digest = context_digest_from_logs(&logs);
        assert!(
            !digest.contains(secret),
            "secret leaked into digest: {digest}"
        );
        // Short, meaningful tokens survive and are lowercased.
        assert!(digest.contains("error"));
        assert!(digest.contains("e0001"));
        assert!(digest.contains("build"));
        assert!(digest.contains("failed"));
    }

    #[test]
    fn should_cap_digest_token_count() {
        let logs = (0..200)
            .map(|i| format!("t{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let digest = context_digest_from_logs(&logs);
        assert!(digest.split(' ').count() <= MAX_DIGEST_TOKENS);
    }

    #[test]
    fn should_count_token_overlap_case_insensitively() {
        assert_eq!(token_overlap("Build Failed E0001", "build broke e0001"), 2); // build, e0001
        assert_eq!(token_overlap("alpha beta", "gamma delta"), 0);
    }

    #[test]
    fn should_rank_more_similar_trajectory_first() {
        let candidates = vec![
            rec(FailureClass::BuildError, "lint style only", "low overlap"),
            rec(
                FailureClass::BuildError,
                "error e0001 build failed missing import",
                "high overlap",
            ),
        ];
        let ranked = rank_by_similarity("error e0001 build failed", candidates, 5);
        assert_eq!(ranked.len(), 1); // the lint one has zero overlap → dropped
        assert_eq!(ranked[0].summary, "high overlap");
    }

    #[test]
    fn should_truncate_ranking_to_k() {
        let candidates = vec![
            rec(FailureClass::BuildError, "error build a", "a"),
            rec(FailureClass::BuildError, "error build b", "b"),
            rec(FailureClass::BuildError, "error build c", "c"),
        ];
        let ranked = rank_by_similarity("error build", candidates, 2);
        assert_eq!(ranked.len(), 2);
    }

    #[tokio::test]
    async fn should_recall_nothing_from_noop_memory() {
        let mem = NoopReflexionMemory;
        mem.record_trajectory(rec(FailureClass::BuildError, "error build", "x"))
            .await
            .unwrap();
        let recalled = mem
            .recall_similar(FailureClass::BuildError, "error build", 5)
            .await
            .unwrap();
        assert!(recalled.is_empty());
    }

    #[tokio::test]
    async fn should_persist_and_recall_ranked_matches_from_in_memory_fake() {
        let mem = InMemoryReflexionMemory::new();
        mem.record_trajectory(rec(
            FailureClass::BuildError,
            "error e0001 missing import std collections",
            "build fix",
        ))
        .await
        .unwrap();
        mem.record_trajectory(rec(
            FailureClass::TestFailure, // different class → must be filtered out
            "error e0001 missing import std collections",
            "wrong class",
        ))
        .await
        .unwrap();

        assert_eq!(mem.recorded().len(), 2);

        let recalled = mem
            .recall_similar(FailureClass::BuildError, "error e0001 missing import", 5)
            .await
            .unwrap();

        // Only the same-class, overlapping trajectory comes back.
        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].summary, "build fix");
        assert_eq!(recalled[0].failure_class, FailureClass::BuildError);
    }

    #[tokio::test]
    async fn should_recall_empty_when_no_same_class_overlap() {
        let mem = InMemoryReflexionMemory::new();
        mem.record_trajectory(rec(FailureClass::Lint, "lint trailing whitespace", "x"))
            .await
            .unwrap();
        let recalled = mem
            .recall_similar(FailureClass::BuildError, "error build failed", 5)
            .await
            .unwrap();
        assert!(recalled.is_empty());
    }
}
