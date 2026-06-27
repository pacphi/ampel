//! Vector-backed [`ReflexionMemory`] (Phase 5b+) — `reflexion` cargo feature.
//!
//! This module is compiled ONLY when the `reflexion` feature is enabled (mirror
//! of how the `onnx` classifier path is gated). With the feature off, the whole
//! module — and the `ruvector-core` dependency — compiles out, so the default
//! build/CI path pulls NO vector store or embedding crate and the deterministic
//! `learning_signal` bias remains the sole decision path.
//!
//! ## Backend decision
//! `ruvector-core` (ruvnet, crates.io) is used with `default-features = false,
//! features = ["hnsw", "memory-only"]`. That selection is **air-gap-safe**:
//! - `hnsw` pulls the pure-Rust `hnsw_rs` ANN index (no native build),
//! - `memory-only` keeps the store in-process (no redb/mmap persistence),
//! - the default `api-embeddings` (reqwest) / `simd` (simsimd) / `onnx-embeddings`
//!   (ort) features are all DISABLED, so there is no external egress and no
//!   native toolchain requirement.
//!
//! ## Local embeddings (air-gap-safe)
//! Embeddings are produced by `ruvector-core`'s [`HashEmbedding`] — a
//! deterministic, local, hashing/bag-of-tokens vectorizer. It is NOT a semantic
//! SOTA embedder; the goal here is the trait + integration + air-gap safety
//! (zero egress), not embedding quality. A stronger LOCAL embedder (e.g. an ONNX
//! sentence model via the same trait) can be swapped in later without touching
//! the [`ReflexionMemory`] seam.
//!
//! ## Security
//! Only the secret-stripped [`TrajectoryRecord`] fields are embedded/stored
//! (see [`ampel_core::services::context_digest_from_logs`]); no credentials,
//! endpoints, or raw high-entropy tokens reach the index.
#![allow(dead_code)] // not yet constructed by the bin; wired via DbAgenticTier in follow-up

use std::collections::HashMap;
use std::str::FromStr;

use ampel_core::errors::{AmpelError, AmpelResult};
use ampel_core::remediation::{FailureClass, ProviderKind};
use ampel_core::services::{LearningOutcome, ReflexionMemory, TrajectoryRecord};
use async_trait::async_trait;
use ruvector_core::embeddings::{EmbeddingProvider, HashEmbedding};
use ruvector_core::{SearchQuery, VectorDB, VectorEntry};
use serde_json::json;

/// Embedding/index dimensionality for the local hashing embedder.
const DIM: usize = 256;

/// Over-fetch factor: HNSW search returns `k` by vector distance, THEN the
/// `failure_class` metadata filter is applied — which can drop matches. Fetch a
/// wider candidate set so the post-filter result can still reach the caller's k.
const SEARCH_FANOUT: usize = 8;

/// A vector-recall [`ReflexionMemory`] backed by `ruvector-core` (in-memory HNSW)
/// + a local hashing embedder. Air-gap-safe: no external egress.
pub struct VectorReflexionMemory {
    db: VectorDB,
    embedder: HashEmbedding,
}

impl VectorReflexionMemory {
    /// Build an in-memory vector memory. Fails only if the index cannot be
    /// initialized.
    pub fn new() -> AmpelResult<Self> {
        let db = VectorDB::with_dimensions(DIM).map_err(map_vec_err)?;
        Ok(Self {
            db,
            embedder: HashEmbedding::new(DIM),
        })
    }

    fn embed(&self, text: &str) -> AmpelResult<Vec<f32>> {
        self.embedder.embed(text).map_err(map_vec_err)
    }
}

#[async_trait]
impl ReflexionMemory for VectorReflexionMemory {
    async fn record_trajectory(&self, rec: TrajectoryRecord) -> AmpelResult<()> {
        let vector = self.embed(&rec.context_digest)?;
        let mut metadata: HashMap<String, serde_json::Value> = HashMap::new();
        // Typed fields flattened to strings via Display (round-tripped by FromStr
        // on recall). No secrets: provider is the KIND only; digest is stripped.
        metadata.insert("failure_class".into(), json!(rec.failure_class.to_string()));
        metadata.insert("provider".into(), json!(rec.provider.to_string()));
        metadata.insert("outcome".into(), json!(rec.outcome.to_string()));
        metadata.insert("context_digest".into(), json!(rec.context_digest));
        metadata.insert("summary".into(), json!(rec.summary));

        let entry = VectorEntry {
            id: None,
            vector,
            metadata: Some(metadata),
        };
        self.db.insert(entry).map_err(map_vec_err)?;
        Ok(())
    }

    async fn recall_similar(
        &self,
        failure_class: FailureClass,
        query_text: &str,
        k: usize,
    ) -> AmpelResult<Vec<TrajectoryRecord>> {
        if k == 0 {
            return Ok(Vec::new());
        }
        let vector = self.embed(query_text)?;
        // Exact-match metadata filter restricts recall to the same failure class.
        let filter = HashMap::from([(
            "failure_class".to_string(),
            json!(failure_class.to_string()),
        )]);
        let query = SearchQuery {
            vector,
            k: k.saturating_mul(SEARCH_FANOUT).max(k),
            filter: Some(filter),
            ef_search: None,
        };
        let results = self.db.search(query).map_err(map_vec_err)?;

        let mut out = Vec::with_capacity(k);
        for r in results {
            let Some(md) = r.metadata else { continue };
            if let Some(rec) = trajectory_from_metadata(&md) {
                out.push(rec);
                if out.len() >= k {
                    break;
                }
            }
        }
        Ok(out)
    }
}

/// Reconstruct a [`TrajectoryRecord`] from stored metadata. Returns `None` if a
/// required field is absent or unparseable (a corrupt row is skipped, never
/// fatal).
fn trajectory_from_metadata(md: &HashMap<String, serde_json::Value>) -> Option<TrajectoryRecord> {
    let s = |key: &str| md.get(key).and_then(|v| v.as_str());
    Some(TrajectoryRecord {
        failure_class: FailureClass::from_str(s("failure_class")?).ok()?,
        provider: ProviderKind::from_str(s("provider")?).ok()?,
        context_digest: s("context_digest")?.to_string(),
        outcome: LearningOutcome::from_str(s("outcome")?).ok()?,
        summary: s("summary")?.to_string(),
    })
}

/// Map a `ruvector-core` error into an [`AmpelError`] (no secret content).
fn map_vec_err(e: ruvector_core::RuvectorError) -> AmpelError {
    AmpelError::InternalError(format!("reflexion vector store: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_record_and_recall_same_class_trajectory() {
        let mem = VectorReflexionMemory::new().unwrap();
        mem.record_trajectory(TrajectoryRecord {
            failure_class: FailureClass::BuildError,
            provider: ProviderKind::Ollama,
            context_digest: "error e0001 build failed missing import".into(),
            outcome: LearningOutcome::Passed,
            summary: "added missing import".into(),
        })
        .await
        .unwrap();

        let recalled = mem
            .recall_similar(FailureClass::BuildError, "error e0001 build failed", 3)
            .await
            .unwrap();

        assert_eq!(recalled.len(), 1);
        assert_eq!(recalled[0].provider, ProviderKind::Ollama);
        assert_eq!(recalled[0].summary, "added missing import");
    }

    #[tokio::test]
    async fn should_not_recall_other_failure_class() {
        let mem = VectorReflexionMemory::new().unwrap();
        mem.record_trajectory(TrajectoryRecord {
            failure_class: FailureClass::Lint,
            provider: ProviderKind::Ollama,
            context_digest: "lint trailing whitespace".into(),
            outcome: LearningOutcome::Passed,
            summary: "x".into(),
        })
        .await
        .unwrap();

        let recalled = mem
            .recall_similar(FailureClass::BuildError, "error build failed", 3)
            .await
            .unwrap();
        assert!(recalled.is_empty());
    }
}
