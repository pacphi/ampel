//! In-process ONNX failure classifier (ADR-009/012). FEATURE-GATED.
//!
//! Compiled **only** under `--features onnx`. The `ort` native runtime is not
//! available on CI runners, so this entire file (and the cascade's L2 stage that
//! references it) compiles out by default. No network: local, in-process,
//! `classify_only`.
//!
//! The model is expected to consume a bag-of-tokens / TF feature vector and emit
//! one logit per [`FailureClass`] (excluding `Unknown`) in declaration order.
//! Feature extraction is kept deliberately simple and pure; the `ort` call is a
//! thin wrapper.
#![allow(dead_code)] // wired into the worker binary in slice 3

use ampel_core::errors::{AmpelError, AmpelResult};
use ampel_core::remediation::{
    AgentBudget, AgentOutcome, AgentTask, ClassificationResult, ClassifierSource, CostModel,
    Egress, FailureClass, InferenceRequest, InferenceResponse, Modality, ModelCaps,
    ModelCredentials, ModelKind, ModelProvider, NormalizedProviderOutput, OutputContract,
};
use async_trait::async_trait;
use ort::session::Session;
use std::sync::Mutex;

/// The classes the model scores, in output-logit order. `Unknown` is never a
/// model output — it is the cascade's fallback when confidence is too low.
const LABELS: [FailureClass; 7] = [
    FailureClass::BuildError,
    FailureClass::TestFailure,
    FailureClass::TypeError,
    FailureClass::Lint,
    FailureClass::LockfileConflict,
    FailureClass::FlakyTest,
    FailureClass::MissingDependency,
];

/// Local ONNX classifier provider (`classify_only`).
pub struct OnnxClassifierProvider {
    session: Mutex<Session>,
}

impl OnnxClassifierProvider {
    /// Load an ONNX model from `model_path`.
    pub fn from_path(model_path: &str) -> AmpelResult<Self> {
        let session = Session::builder()
            .and_then(|b| b.commit_from_file(model_path))
            .map_err(|e| AmpelError::ProviderError(format!("onnx: load `{model_path}`: {e}")))?;
        Ok(Self {
            session: Mutex::new(session),
        })
    }

    /// Map the highest-scoring logit to a [`ClassificationResult`]. Confidence is
    /// the softmax probability of the argmax class.
    pub fn label_from_logits(logits: &[f32]) -> ClassificationResult {
        let (idx, &max) = logits
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap_or((0, &0.0));
        let sum_exp: f32 = logits.iter().map(|l| (l - max).exp()).sum();
        let confidence = if sum_exp > 0.0 { 1.0 / sum_exp } else { 0.0 };
        let class = LABELS.get(idx).copied().unwrap_or(FailureClass::Unknown);
        ClassificationResult {
            class,
            source: ClassifierSource::Onnx,
            confidence,
        }
    }
}

/// Extract a fixed-width term-frequency feature vector from log text. Pure.
pub fn extract_features(log_text: &str, width: usize) -> Vec<f32> {
    let mut features = vec![0.0f32; width];
    for token in log_text.split(|c: char| !c.is_alphanumeric()) {
        if token.is_empty() {
            continue;
        }
        let h = token.bytes().fold(0usize, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as usize)
        });
        features[h % width] += 1.0;
    }
    features
}

#[async_trait]
impl ModelProvider for OnnxClassifierProvider {
    async fn infer(
        &self,
        _creds: &ModelCredentials,
        req: InferenceRequest,
    ) -> AmpelResult<InferenceResponse> {
        // Classify over the concatenated untrusted blocks.
        let text: String = req
            .context_blocks
            .iter()
            .map(|b| b.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let features = extract_features(&text, 256);

        let mut session = self
            .session
            .lock()
            .map_err(|_| AmpelError::ProviderError("onnx: session lock poisoned".into()))?;
        let input = ort::value::Tensor::from_array(([1usize, features.len()], features))
            .map_err(|e| AmpelError::ProviderError(format!("onnx: tensor: {e}")))?;
        let outputs = session
            .run(ort::inputs!["input" => input])
            .map_err(|e| AmpelError::ProviderError(format!("onnx: run: {e}")))?;
        let (_, logits) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| AmpelError::ProviderError(format!("onnx: extract: {e}")))?;

        let result = Self::label_from_logits(logits);
        Ok(InferenceResponse {
            output: NormalizedProviderOutput::Classification(result),
            tokens_used: 0,
            cost: rust_decimal::Decimal::ZERO,
        })
    }

    async fn run_agent(
        &self,
        _creds: &ModelCredentials,
        _task: AgentTask,
        _budget: AgentBudget,
    ) -> AmpelResult<AgentOutcome> {
        Err(AmpelError::ProviderError(
            "onnx: classify-only provider cannot run an agent".into(),
        ))
    }

    fn capabilities(&self) -> ModelCaps {
        ModelCaps {
            kind: ModelKind::Inference,
            modality: Modality::InProcess,
            tool_use: false,
            code_edit: false,
            max_context_tokens: 0,
            cost: CostModel::Free,
            egress: Egress::LocalOnly,
            output_contract: OutputContract::ClassifyOnly,
        }
    }

    async fn validate(&self, _creds: &ModelCredentials) -> AmpelResult<()> {
        // Loaded successfully at construction; nothing external to ping.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_extract_fixed_width_features() {
        let f = extract_features("error error build", 16);
        assert_eq!(f.len(), 16);
        assert!(f.iter().any(|&v| v >= 2.0)); // "error" counted twice
    }

    #[test]
    fn should_pick_argmax_label_with_confidence() {
        // index 2 (TypeError) dominates.
        let logits = [0.1, 0.2, 5.0, 0.1, 0.0, 0.0, 0.0];
        let r = super::OnnxClassifierProvider::label_from_logits(&logits);
        assert_eq!(r.class, FailureClass::TypeError);
        assert_eq!(r.source, ClassifierSource::Onnx);
        assert!(r.confidence > 0.7);
    }
}
