//! Remediation playbook value types + YAML parsing + tools-policy ceiling
//! (ADR-006).
//!
//! A playbook describes the agent's role, per-failure-class task templates, the
//! agent loop budget, the tools the agent may use, the context to assemble, the
//! output contract, and per-provider overlays. YAML parsing lives here in the
//! worker (not `ampel-core`) so the domain crate stays serialization-light.
//!
//! ## Tools-policy ceiling (security)
//! [`clamp_tools`] enforces the ADR-006 invariant: a repo-local or DB override
//! may only ever *remove* tools relative to the embedded/org ceiling, never add
//! new ones. It is pure set subtraction (intersection with the ceiling).
#![allow(dead_code)] // wired into the worker binary in slice 3

use std::collections::HashMap;

use ampel_core::errors::{AmpelError, AmpelResult};
use ampel_core::remediation::{AgentBudget, FailureClass, OutputContract};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// A fully-parsed playbook (after YAML decode; ceiling applied by the resolver).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Playbook {
    #[serde(default)]
    pub version: u32,
    /// Trusted role/system framing for the agent.
    pub role: String,
    /// Per-task instruction templates, keyed by task name (e.g. `failed_ci`,
    /// `lockfile_conflict`). Values are minijinja templates over trusted
    /// metadata only.
    pub tasks: HashMap<String, PlaybookTask>,
    #[serde(rename = "loop")]
    pub loop_cfg: LoopConfig,
    pub tools_policy: ToolsPolicy,
    #[serde(default)]
    pub context_spec: ContextSpec,
    /// Default output contract (string form, e.g. `unified_diff`).
    pub output_contract: String,
    /// Per-provider overlays keyed by provider kind (`claude`/`gemini`/
    /// `ollama`/`onnx`).
    #[serde(default)]
    pub provider_overlays: HashMap<String, ProviderOverlay>,
}

/// One task's instruction template.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybookTask {
    /// minijinja template rendered against trusted metadata to produce the
    /// instruction portion of the `system` prompt. NEVER interpolate untrusted
    /// data here — that travels in `context_blocks`.
    pub instructions: String,
}

/// The agent-loop ceiling. Maps to an [`AgentBudget`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoopConfig {
    pub max_iterations: u32,
    pub max_seconds: u64,
    /// Max spend in USD, as a decimal string (exact money; never f64).
    pub max_cost_usd: String,
}

impl LoopConfig {
    /// Convert to a runtime [`AgentBudget`], parsing the decimal cap.
    pub fn to_budget(&self) -> AmpelResult<AgentBudget> {
        let max_cost = Decimal::from_str(&self.max_cost_usd).map_err(|e| {
            AmpelError::ConfigError(format!(
                "playbook: invalid max_cost_usd `{}`: {e}",
                self.max_cost_usd
            ))
        })?;
        Ok(AgentBudget {
            max_iterations: self.max_iterations,
            max_seconds: self.max_seconds,
            max_cost,
        })
    }
}

/// Allow-list of tools the agent may use. Subject to the ceiling clamp.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolsPolicy {
    #[serde(default)]
    pub allowed: Vec<String>,
}

/// Which context blocks to assemble for the model (labels only; the values are
/// gathered at runtime and carried as untrusted data).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ContextSpec {
    #[serde(default)]
    pub blocks: Vec<String>,
}

/// Per-provider overlay (optional field overrides).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProviderOverlay {
    #[serde(default)]
    pub output_contract: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

impl Playbook {
    /// Parse a playbook from YAML.
    pub fn from_yaml(yaml: &str) -> AmpelResult<Self> {
        serde_yaml::from_str(yaml)
            .map_err(|e| AmpelError::ConfigError(format!("playbook: invalid YAML: {e}")))
    }

    /// Pick the task template for a failure class. `lockfile_conflict` maps to
    /// its own task; everything else falls back to `failed_ci`.
    pub fn select_task(&self, class: FailureClass) -> AmpelResult<&PlaybookTask> {
        let key = match class {
            FailureClass::LockfileConflict => "lockfile_conflict",
            _ => "failed_ci",
        };
        self.tasks
            .get(key)
            .or_else(|| self.tasks.get("failed_ci"))
            .ok_or_else(|| {
                AmpelError::ConfigError(format!(
                    "playbook: no task `{key}` and no `failed_ci` fallback"
                ))
            })
    }

    /// Resolve the effective output contract for a provider kind: overlay wins,
    /// else the playbook default.
    pub fn output_contract_for(&self, provider_kind: &str) -> AmpelResult<OutputContract> {
        let raw = self
            .provider_overlays
            .get(provider_kind)
            .and_then(|o| o.output_contract.clone())
            .unwrap_or_else(|| self.output_contract.clone());
        OutputContract::from_str(&raw)
    }
}

/// Enforce the tools-policy ceiling: keep only `requested` tools that are also
/// present in `ceiling`. Pure set subtraction — an override can REMOVE tools but
/// never ADD a tool the ceiling does not grant. Order follows `ceiling` for a
/// deterministic result.
pub fn clamp_tools(ceiling: &[String], requested: &[String]) -> Vec<String> {
    ceiling
        .iter()
        .filter(|t| requested.contains(t))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_clamp_requested_tools_to_ceiling_intersection() {
        let ceiling = vec!["read".to_string(), "write".to_string(), "patch".to_string()];
        let requested = vec!["read".to_string(), "patch".to_string()];
        assert_eq!(clamp_tools(&ceiling, &requested), vec!["read", "patch"]);
    }

    #[test]
    fn should_drop_tools_not_in_ceiling() {
        let ceiling = vec!["read".to_string()];
        // override tries to ADD `shell` beyond the ceiling — must be ignored.
        let requested = vec!["read".to_string(), "shell".to_string()];
        assert_eq!(clamp_tools(&ceiling, &requested), vec!["read"]);
    }

    #[test]
    fn should_return_empty_when_requested_disjoint_from_ceiling() {
        let ceiling = vec!["read".to_string()];
        let requested = vec!["shell".to_string()];
        assert!(clamp_tools(&ceiling, &requested).is_empty());
    }

    #[test]
    fn should_convert_loop_config_to_budget() {
        let cfg = LoopConfig {
            max_iterations: 4,
            max_seconds: 900,
            max_cost_usd: "2.50".to_string(),
        };
        let budget = cfg.to_budget().unwrap();
        assert_eq!(budget.max_iterations, 4);
        assert_eq!(budget.max_seconds, 900);
        assert_eq!(budget.max_cost, Decimal::new(250, 2));
    }

    #[test]
    fn should_reject_non_decimal_cost() {
        let cfg = LoopConfig {
            max_iterations: 1,
            max_seconds: 1,
            max_cost_usd: "free".to_string(),
        };
        assert!(cfg.to_budget().is_err());
    }
}
