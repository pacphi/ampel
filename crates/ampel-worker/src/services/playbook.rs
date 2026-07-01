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
use std::fmt;

use ampel_core::errors::{AmpelError, AmpelResult};
use ampel_core::remediation::{AgentBudget, FailureClass, OutputContract, ProviderKind};
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

    /// Parse **and schema-validate** a playbook from YAML (ADR-006 schema),
    /// returning a [`PlaybookSchemaError`] whose `field` names the first
    /// offending field path rather than an opaque serde message. This is the
    /// write-time validator behind create/update.
    ///
    /// Checks, in order: the document is a YAML mapping; `role` is a non-empty
    /// string; `tasks` is a non-empty mapping and each task carries a non-empty
    /// `instructions` string; `loop.{max_iterations,max_seconds}` are integers and
    /// `loop.max_cost_usd` is a decimal string (exact money, never a float);
    /// `tools_policy` is a mapping; `output_contract` is a known enum; and every
    /// `provider_overlays` key is a known provider kind (with any overlay
    /// `output_contract` also a known enum). A final strong-typed parse catches
    /// any residual type mismatch.
    pub fn validate_yaml(yaml: &str) -> Result<Self, PlaybookSchemaError> {
        let value: serde_yaml::Value = serde_yaml::from_str(yaml)
            .map_err(|e| field_err("<root>", &format!("invalid YAML: {e}")))?;
        let root = value
            .as_mapping()
            .ok_or_else(|| field_err("<root>", "playbook must be a YAML mapping"))?;

        // role — required, non-empty string.
        if require_str(root, "role")?.trim().is_empty() {
            return Err(field_err("role", "must not be empty"));
        }

        // tasks — required, non-empty mapping; each value carries `instructions`.
        let tasks = require_mapping(root, "tasks")?;
        if tasks.is_empty() {
            return Err(field_err("tasks", "must define at least one task"));
        }
        for (k, v) in tasks {
            let name = k
                .as_str()
                .ok_or_else(|| field_err("tasks", "task keys must be strings"))?;
            let task = v
                .as_mapping()
                .ok_or_else(|| field_err(&format!("tasks.{name}"), "must be a mapping"))?;
            let ok = task
                .get("instructions")
                .and_then(|i| i.as_str())
                .is_some_and(|s| !s.trim().is_empty());
            if !ok {
                return Err(field_err(
                    &format!("tasks.{name}.instructions"),
                    "must be a non-empty string",
                ));
            }
        }

        // loop — required mapping with the three budget fields.
        let loop_cfg = require_mapping(root, "loop")?;
        require_uint(loop_cfg, "loop", "max_iterations")?;
        require_uint(loop_cfg, "loop", "max_seconds")?;
        // max_cost_usd — a quoted decimal string (exact money; never a float).
        let cost = loop_cfg
            .get("max_cost_usd")
            .ok_or_else(|| field_err("loop.max_cost_usd", "is required"))?;
        let cost_str = cost.as_str().ok_or_else(|| {
            field_err(
                "loop.max_cost_usd",
                "must be a decimal string (quote it, e.g. \"2.00\")",
            )
        })?;
        Decimal::from_str(cost_str)
            .map_err(|_| field_err("loop.max_cost_usd", "must be a valid decimal amount"))?;

        // tools_policy — required mapping (its `allowed` list may be empty/absent).
        require_mapping(root, "tools_policy")?;

        // output_contract — required, a known enum.
        let oc = require_str(root, "output_contract")?;
        OutputContract::from_str(oc).map_err(|_| {
            field_err(
                "output_contract",
                "must be one of: tool_use, unified_diff, classify_only",
            )
        })?;

        // provider_overlays — optional; keys must be known provider kinds, and any
        // overlay `output_contract` must also be a known enum.
        if let Some(po) = root.get("provider_overlays") {
            if !po.is_null() {
                let overlays = po
                    .as_mapping()
                    .ok_or_else(|| field_err("provider_overlays", "must be a mapping"))?;
                for (k, v) in overlays {
                    let key = k.as_str().ok_or_else(|| {
                        field_err("provider_overlays", "overlay keys must be strings")
                    })?;
                    ProviderKind::from_str(key).map_err(|_| {
                        field_err(
                            &format!("provider_overlays.{key}"),
                            "unknown provider kind (expected: claude, gemini, ollama, onnx)",
                        )
                    })?;
                    // Each overlay is a typed object (ProviderOverlay), never a bare
                    // scalar — validate that before reaching into its fields.
                    let overlay = v.as_mapping().ok_or_else(|| {
                        field_err(&format!("provider_overlays.{key}"), "must be a mapping")
                    })?;
                    if let Some(overlay_oc) =
                        overlay.get("output_contract").and_then(|o| o.as_str())
                    {
                        OutputContract::from_str(overlay_oc).map_err(|_| {
                            field_err(
                                &format!("provider_overlays.{key}.output_contract"),
                                "must be one of: tool_use, unified_diff, classify_only",
                            )
                        })?;
                    }
                }
            }
        }

        // Final strong-typed parse — catches any residual type mismatch the
        // structural checks above did not cover (the required fields are already
        // guaranteed present, so this surfaces only deeper type errors).
        Self::from_yaml(yaml).map_err(|e| field_err("<root>", &e.to_string()))
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

/// A schema-validation failure that names the offending field path (ADR-006).
///
/// Serde's own parse errors are opaque about *which* field is wrong, so
/// [`Playbook::validate_yaml`] produces this instead: a `field` an operator can
/// act on (e.g. `loop.max_iterations`) plus a human-readable `message`. The API
/// layer maps it onto a `422`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaybookSchemaError {
    /// Dotted path of the offending field, e.g. `loop.max_cost_usd`. `<root>`
    /// denotes a document-level problem (not a mapping, or a residual type error).
    pub field: String,
    /// Human-readable reason the field is invalid.
    pub message: String,
}

impl fmt::Display for PlaybookSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for PlaybookSchemaError {}

fn field_err(field: &str, message: &str) -> PlaybookSchemaError {
    PlaybookSchemaError {
        field: field.to_string(),
        message: message.to_string(),
    }
}

/// Fetch a required string field from a YAML mapping, else a field-scoped error.
fn require_str<'a>(
    map: &'a serde_yaml::Mapping,
    key: &str,
) -> Result<&'a str, PlaybookSchemaError> {
    match map.get(key) {
        None => Err(field_err(key, "is required")),
        Some(v) => v.as_str().ok_or_else(|| field_err(key, "must be a string")),
    }
}

/// Fetch a required sub-mapping from a YAML mapping, else a field-scoped error.
fn require_mapping<'a>(
    map: &'a serde_yaml::Mapping,
    key: &str,
) -> Result<&'a serde_yaml::Mapping, PlaybookSchemaError> {
    match map.get(key) {
        None => Err(field_err(key, "is required")),
        Some(v) => v
            .as_mapping()
            .ok_or_else(|| field_err(key, "must be a mapping")),
    }
}

/// Assert `parent.key` is present and a non-negative integer.
fn require_uint(
    map: &serde_yaml::Mapping,
    parent: &str,
    key: &str,
) -> Result<(), PlaybookSchemaError> {
    let path = format!("{parent}.{key}");
    match map.get(key) {
        None => Err(field_err(&path, "is required")),
        Some(v) if v.as_u64().is_some() => Ok(()),
        Some(_) => Err(field_err(&path, "must be a non-negative integer")),
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

    // ---- validate_yaml: schema-aware, field-path validation (ADR-006) --------

    /// A minimal, well-formed playbook. Tests tweak/remove fields from this.
    const VALID_YAML: &str = r#"
version: 1
role: "You are a remediation engineer."
tasks:
  failed_ci:
    instructions: "Fix {{ repo_full_name }} on {{ base_branch }}."
loop:
  max_iterations: 4
  max_seconds: 900
  max_cost_usd: "2.00"
tools_policy:
  allowed: [read_file, apply_patch]
output_contract: unified_diff
provider_overlays:
  claude:
    output_contract: tool_use
    model: claude-sonnet-4-6
"#;

    #[test]
    fn should_accept_a_well_formed_playbook() {
        let pb = Playbook::validate_yaml(VALID_YAML).expect("valid playbook");
        assert_eq!(pb.role, "You are a remediation engineer.");
        assert!(pb.tasks.contains_key("failed_ci"));
    }

    #[test]
    fn should_validate_the_shipped_embedded_default() {
        // The real embedded default must always satisfy the write-time validator,
        // otherwise the editor's "load built-in default" round-trip would break.
        let default_yaml = include_str!("../../playbooks/default.yaml");
        assert!(Playbook::validate_yaml(default_yaml).is_ok());
    }

    #[test]
    fn should_report_field_path_for_missing_role() {
        let yaml = VALID_YAML.replace("role: \"You are a remediation engineer.\"", "");
        let err = Playbook::validate_yaml(&yaml).unwrap_err();
        assert_eq!(err.field, "role");
    }

    #[test]
    fn should_report_field_path_for_empty_tasks() {
        let yaml = r#"
role: "r"
tasks: {}
loop:
  max_iterations: 1
  max_seconds: 1
  max_cost_usd: "0.10"
tools_policy:
  allowed: []
output_contract: unified_diff
"#;
        let err = Playbook::validate_yaml(yaml).unwrap_err();
        assert_eq!(err.field, "tasks");
    }

    #[test]
    fn should_report_field_path_for_task_missing_instructions() {
        let yaml = r#"
role: "r"
tasks:
  failed_ci:
    notes: "no instructions here"
loop:
  max_iterations: 1
  max_seconds: 1
  max_cost_usd: "0.10"
tools_policy:
  allowed: []
output_contract: unified_diff
"#;
        let err = Playbook::validate_yaml(yaml).unwrap_err();
        assert_eq!(err.field, "tasks.failed_ci.instructions");
    }

    #[test]
    fn should_report_field_path_for_missing_loop_max_iterations() {
        let yaml = VALID_YAML.replace("  max_iterations: 4\n", "");
        let err = Playbook::validate_yaml(&yaml).unwrap_err();
        assert_eq!(err.field, "loop.max_iterations");
    }

    #[test]
    fn should_report_field_path_for_non_decimal_cost() {
        let yaml = VALID_YAML.replace("max_cost_usd: \"2.00\"", "max_cost_usd: \"free\"");
        let err = Playbook::validate_yaml(&yaml).unwrap_err();
        assert_eq!(err.field, "loop.max_cost_usd");
    }

    #[test]
    fn should_report_field_path_for_unquoted_float_cost() {
        // A bare float (not a quoted string) is exactly the money-precision
        // footgun the schema forbids — must be flagged on the field, not panic.
        let yaml = VALID_YAML.replace("max_cost_usd: \"2.00\"", "max_cost_usd: 2.00");
        let err = Playbook::validate_yaml(&yaml).unwrap_err();
        assert_eq!(err.field, "loop.max_cost_usd");
    }

    #[test]
    fn should_report_field_path_for_unknown_output_contract() {
        let yaml = VALID_YAML.replace("output_contract: unified_diff", "output_contract: magic");
        let err = Playbook::validate_yaml(&yaml).unwrap_err();
        assert_eq!(err.field, "output_contract");
    }

    #[test]
    fn should_report_field_path_for_unknown_provider_overlay_kind() {
        let yaml = VALID_YAML.replace("  claude:", "  banana:");
        let err = Playbook::validate_yaml(&yaml).unwrap_err();
        assert_eq!(err.field, "provider_overlays.banana");
    }

    #[test]
    fn should_report_field_path_for_unknown_overlay_output_contract() {
        let yaml = VALID_YAML.replace(
            "  claude:\n    output_contract: tool_use",
            "  claude:\n    output_contract: bogus",
        );
        let err = Playbook::validate_yaml(&yaml).unwrap_err();
        assert_eq!(err.field, "provider_overlays.claude.output_contract");
    }

    #[test]
    fn should_report_tasks_field_for_non_string_task_key() {
        // A non-string task key must fail on `tasks` with a clear message, not
        // defer to the opaque final serde parse.
        let yaml = r#"
role: "r"
tasks:
  123:
    instructions: "do it"
loop:
  max_iterations: 1
  max_seconds: 1
  max_cost_usd: "0.10"
tools_policy:
  allowed: []
output_contract: unified_diff
"#;
        let err = Playbook::validate_yaml(yaml).unwrap_err();
        assert_eq!(err.field, "tasks");
    }

    #[test]
    fn should_report_overlay_field_for_non_mapping_overlay_value() {
        // A scalar where an overlay object is expected must fail on the overlay's
        // own field path, not the opaque final parse.
        let yaml = VALID_YAML.replace(
            "  claude:\n    output_contract: tool_use\n    model: claude-sonnet-4-6",
            "  claude: 42",
        );
        let err = Playbook::validate_yaml(&yaml).unwrap_err();
        assert_eq!(err.field, "provider_overlays.claude");
    }

    #[test]
    fn should_report_root_for_non_mapping_document() {
        // A YAML sequence (or any non-mapping) is not a playbook — flagged at the
        // document level, never a spurious per-field error.
        let err = Playbook::validate_yaml("- a\n- b\n").unwrap_err();
        assert_eq!(err.field, "<root>");
    }

    #[test]
    fn should_report_root_for_unparseable_yaml() {
        // Genuinely malformed YAML (unbalanced flow mapping) fails the parse.
        let err = Playbook::validate_yaml("role: {unclosed").unwrap_err();
        assert_eq!(err.field, "<root>");
    }
}
