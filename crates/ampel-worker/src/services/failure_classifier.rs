//! Worker-side classification cascade (ADR-012).
//!
//! Implements the `ampel_core` [`FailureClassifier`] trait as an L1→L2 cascade:
//!
//! - **L1** — pure [`classify_heuristic`] from `ampel-core` (regex/marker,
//!   confidence 1.0, sub-millisecond). Always available.
//! - **L2** — local ONNX classifier (feature `onnx` only). Consulted only when
//!   L1 returns [`FailureClass::Unknown`] and a classifier model is configured;
//!   accepted only at confidence ≥ 0.7, otherwise the result stays `Unknown`.
//!
//! L3 (model escalation) is the harness's job — the harness sends an `Unknown`
//! run to the model anyway, so the cascade itself stops at L2.
//!
//! With `onnx` OFF (the CI path) the cascade degrades to "heuristic, else
//! Unknown" — fully unit-testable with no runtime.
#![allow(dead_code)] // wired into the worker binary in slice 3

use ampel_core::remediation::{
    classify_heuristic, ClassificationResult, FailureClass, FailureClassifier,
};
use async_trait::async_trait;

#[cfg(feature = "onnx")]
use crate::providers::OnnxClassifierProvider;
#[cfg(feature = "onnx")]
use std::sync::Arc;

/// Minimum confidence to accept an ONNX (L2) classification (ADR-012).
#[cfg(feature = "onnx")]
const ONNX_MIN_CONFIDENCE: f32 = 0.7;

/// The L1→L2 classification cascade.
#[derive(Default)]
pub struct CascadeClassifier {
    #[cfg(feature = "onnx")]
    onnx: Option<Arc<OnnxClassifierProvider>>,
}

impl CascadeClassifier {
    /// Heuristic-only cascade (the CI / no-onnx configuration).
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach an L2 ONNX classifier. Only present under the `onnx` feature.
    #[cfg(feature = "onnx")]
    pub fn with_onnx(mut self, onnx: Arc<OnnxClassifierProvider>) -> Self {
        self.onnx = Some(onnx);
        self
    }
}

#[async_trait]
impl FailureClassifier for CascadeClassifier {
    async fn classify(&self, log_text: &str) -> ClassificationResult {
        // L1: pure heuristic.
        let l1 = classify_heuristic(log_text);
        if l1.class != FailureClass::Unknown {
            return l1;
        }

        // L2: ONNX (feature-gated; only if a model is configured).
        #[cfg(feature = "onnx")]
        if let Some(onnx) = &self.onnx {
            use ampel_core::remediation::{
                ContextBlock, InferenceRequest, ModelCredentials, ModelProvider,
                NormalizedProviderOutput, OutputContract,
            };
            let req = InferenceRequest {
                system: String::new(),
                context_blocks: vec![ContextBlock {
                    label: "ci_log".into(),
                    content: log_text.to_string(),
                    is_untrusted_data: true,
                }],
                max_tokens: 0,
                output_contract: OutputContract::ClassifyOnly,
            };
            if let Ok(resp) = onnx.infer(&ModelCredentials::default(), req).await {
                if let NormalizedProviderOutput::Classification(result) = resp.output {
                    if result.confidence >= ONNX_MIN_CONFIDENCE
                        && result.class != FailureClass::Unknown
                    {
                        return result;
                    }
                }
            }
        }

        // Nothing matched: stay Unknown so the harness escalates to the model.
        ClassificationResult::unknown_heuristic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ampel_core::remediation::ClassifierSource;

    #[tokio::test]
    async fn should_return_heuristic_class_on_l1_match() {
        let cascade = CascadeClassifier::new();
        let r = cascade
            .classify("error[E0432]: unresolved import `crate::foo`")
            .await;
        assert_eq!(r.class, FailureClass::BuildError);
        assert_eq!(r.source, ClassifierSource::Heuristic);
        assert_eq!(r.confidence, 1.0);
    }

    #[tokio::test]
    async fn should_stay_unknown_when_l1_misses_and_no_onnx() {
        let cascade = CascadeClassifier::new();
        let r = cascade.classify("Cloning repository... done.").await;
        assert_eq!(r.class, FailureClass::Unknown);
        assert_eq!(r.source, ClassifierSource::Heuristic);
        assert_eq!(r.confidence, 0.0);
    }

    #[tokio::test]
    async fn should_prefer_specific_heuristic_over_generic_failure() {
        let cascade = CascadeClassifier::new();
        let r = cascade
            .classify("test integration::x ... FAILED (flaky: passed on retry)")
            .await;
        assert_eq!(r.class, FailureClass::FlakyTest);
    }
}
