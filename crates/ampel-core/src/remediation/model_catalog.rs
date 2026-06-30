//! Curated, checked-in model catalog — the source of truth for the *selectable*
//! models offered in the Remediation "Model" tab.
//!
//! emailibrium-style: a single YAML file (`config/models.yaml`) lists every
//! selectable model with rich, display-ready metadata (context window, cost,
//! tool-use, quality). This module deserializes that YAML and maps each entry
//! onto the existing [`ModelCaps`] so the catalog *populates* capabilities
//! rather than redefining the capability/egress vocabulary.
//!
//! # Purity & resilience
//! Like the rest of `ampel-core`'s remediation abstraction, this module performs
//! no network I/O and loads no model runtime. Parsing is from a string (the
//! pure, fully CI-testable core); a thin file loader is offered for callers that
//! read an on-disk override. BOTH degrade gracefully — a missing or invalid YAML
//! source yields a usable catalog (the embedded default, or an empty one for the
//! pure parser) with a `warn!`, and NEVER panics. The catalog is advisory
//! display/selection data; it is not the credential or spend authority (that
//! stays with the `model_provider_accounts` entity, ADR-008).

use crate::errors::AmpelResult;
use crate::remediation::model_provider::{
    CostModel, Egress, Modality, ModelCaps, ModelKind, OutputContract, ProviderKind,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;

/// The catalog shipped with the binary, embedded at build time. Always available
/// as the baseline even when an on-disk override is missing or unparseable.
pub const DEFAULT_CATALOG_YAML: &str = include_str!("../../../../config/models.yaml");

/// The whole catalog: a map of provider-kind key (`"claude"`, `"ollama"`, …) to
/// its [`CatalogProvider`] block. A [`BTreeMap`] keeps iteration order stable so
/// the rendered list and tests are deterministic.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelCatalog {
    #[serde(default)]
    pub providers: BTreeMap<String, CatalogProvider>,
}

/// One provider's block: a description, an optional egress override, and its
/// list of selectable models.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CatalogProvider {
    #[serde(default)]
    pub description: String,
    /// Optional egress override (`"external"` | `"local_only"`). When absent the
    /// egress is derived from the provider kind.
    #[serde(default)]
    pub egress: Option<String>,
    #[serde(default)]
    pub models: Vec<CatalogModel>,
}

/// A single selectable model. Every field defaults so partial/forward-extended
/// entries deserialize cleanly.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CatalogModel {
    /// Stable identifier used in selection (decoupled from any account's stored
    /// `model_id` — the UI escape hatch may still supply an unlisted id).
    #[serde(default)]
    pub id: String,
    /// Human-friendly display name.
    #[serde(default)]
    pub name: String,
    /// Model family (e.g. `claude`, `qwen`), for grouping/badges.
    #[serde(default)]
    pub family: String,
    /// Context window in tokens → [`ModelCaps::max_context_tokens`].
    #[serde(default)]
    pub context_size: u32,
    /// Whether the model supports tool/function calling.
    #[serde(default)]
    pub tool_calling: bool,
    /// Whether the model is suitable for driving code edits.
    #[serde(default)]
    pub code_edit: bool,
    /// Coarse quality tier (`fair` | `good` | `excellent`), display-only.
    #[serde(default)]
    pub quality: String,
    /// USD per 1k input tokens (exact Decimal). `None` ⇒ treated as free/local.
    #[serde(default)]
    pub cost_per_1k_input: Option<Decimal>,
    /// USD per 1k output tokens (exact Decimal). `None` ⇒ treated as free/local.
    #[serde(default)]
    pub cost_per_1k_output: Option<Decimal>,
    /// Ollama pull tag (e.g. `qwen2.5-coder:7b`) — only meaningful for Ollama.
    #[serde(default)]
    pub ollama_tag: Option<String>,
    /// On-disk ONNX model path hint (not a secret) — only meaningful for ONNX.
    #[serde(default)]
    pub model_path: Option<String>,
}

impl ModelCatalog {
    /// Parse a catalog from a YAML string. **Never panics**: on a deserialization
    /// error it `warn!`s and returns an empty [`ModelCatalog::default`], so a
    /// corrupt override degrades to "no extra models" rather than taking the
    /// service down.
    pub fn parse(yaml: &str) -> Self {
        match serde_yaml::from_str::<Self>(yaml) {
            Ok(catalog) => catalog,
            Err(e) => {
                tracing::warn!(error = %e, "invalid model catalog YAML; falling back to empty catalog");
                Self::default()
            }
        }
    }

    /// The built-in catalog embedded at compile time from `config/models.yaml`.
    pub fn load_default() -> Self {
        Self::parse(DEFAULT_CATALOG_YAML)
    }

    /// Resolve the [`ProviderKind`] for a provider key, or `None` if the key is
    /// not a known provider (such entries are skipped, not fatal).
    fn provider_kind(key: &str) -> Option<ProviderKind> {
        ProviderKind::from_str(key).ok()
    }

    /// Iterate every selectable model paired with its resolved [`ProviderKind`]
    /// and the [`ModelCaps`] it advertises. Entries under unknown provider keys
    /// are skipped.
    pub fn entries(&self) -> Vec<CatalogEntry<'_>> {
        let mut out = Vec::new();
        for (key, provider) in &self.providers {
            let Some(kind) = Self::provider_kind(key) else {
                tracing::warn!(provider = %key, "unknown provider kind in model catalog; skipping");
                continue;
            };
            let egress_override = provider
                .egress
                .as_deref()
                .and_then(|s| Egress::from_str(s).ok());
            for model in &provider.models {
                out.push(CatalogEntry {
                    kind,
                    caps: model.caps(kind, egress_override),
                    model,
                });
            }
        }
        out
    }

    /// The known provider kinds present in the catalog (unknown keys excluded).
    pub fn provider_kinds(&self) -> Vec<ProviderKind> {
        self.providers
            .keys()
            .filter_map(|k| Self::provider_kind(k))
            .collect()
    }
}

/// A catalog model resolved against its provider — what callers (the catalog API
/// in the next phase) actually hand out.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogEntry<'a> {
    pub kind: ProviderKind,
    pub caps: ModelCaps,
    pub model: &'a CatalogModel,
}

impl CatalogModel {
    /// Map this entry onto the existing [`ModelCaps`]. Capability vocabulary is
    /// REUSED (not redefined): modality, output contract, model kind and the
    /// default egress are derived from the provider `kind`; `egress_override`
    /// (from the provider block) wins when present. Cost is metered only when
    /// BOTH per-direction prices are given, otherwise the model is [`CostModel::Free`].
    pub fn caps(&self, kind: ProviderKind, egress_override: Option<Egress>) -> ModelCaps {
        let cost = match (self.cost_per_1k_input, self.cost_per_1k_output) {
            (Some(input_per_1k), Some(output_per_1k)) => CostModel::PerToken {
                input_per_1k,
                output_per_1k,
            },
            _ => CostModel::Free,
        };

        ModelCaps {
            kind: derived_model_kind(kind),
            modality: derived_modality(kind),
            tool_use: self.tool_calling,
            code_edit: self.code_edit,
            max_context_tokens: self.context_size,
            cost,
            egress: egress_override.unwrap_or_else(|| derived_egress(kind)),
            output_contract: derived_output_contract(kind),
        }
    }
}

fn derived_modality(kind: ProviderKind) -> Modality {
    match kind {
        ProviderKind::Claude | ProviderKind::Gemini => Modality::HostedApi,
        ProviderKind::Ollama => Modality::LocalServer,
        ProviderKind::Onnx => Modality::InProcess,
    }
}

fn derived_egress(kind: ProviderKind) -> Egress {
    match kind {
        ProviderKind::Claude | ProviderKind::Gemini => Egress::External,
        ProviderKind::Ollama | ProviderKind::Onnx => Egress::LocalOnly,
    }
}

fn derived_output_contract(kind: ProviderKind) -> OutputContract {
    match kind {
        ProviderKind::Claude | ProviderKind::Gemini => OutputContract::ToolUse,
        ProviderKind::Ollama => OutputContract::UnifiedDiff,
        ProviderKind::Onnx => OutputContract::ClassifyOnly,
    }
}

fn derived_model_kind(kind: ProviderKind) -> ModelKind {
    match kind {
        // ONNX is classify-only and cannot drive the agentic edit loop.
        ProviderKind::Onnx => ModelKind::Inference,
        _ => ModelKind::Agent,
    }
}

/// Load a catalog from an on-disk YAML file, falling back to the embedded
/// [`ModelCatalog::load_default`] when the file is missing/unreadable (and to an
/// empty catalog when it is present but unparseable). Always `Ok`: a bad
/// override must never break startup. The `AmpelResult` return is reserved for
/// future hard-failure modes.
pub fn load_catalog(path: impl AsRef<Path>) -> AmpelResult<ModelCatalog> {
    let path = path.as_ref();
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(ModelCatalog::parse(&contents)),
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "model catalog file unreadable; falling back to embedded default"
            );
            Ok(ModelCatalog::load_default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_catalog() -> ModelCatalog {
        ModelCatalog::load_default()
    }

    #[test]
    fn should_parse_embedded_default_catalog_with_all_known_providers() {
        let catalog = default_catalog();
        for key in ["claude", "gemini", "ollama", "onnx"] {
            let provider = catalog
                .providers
                .get(key)
                .unwrap_or_else(|| panic!("missing provider {key}"));
            assert!(!provider.models.is_empty(), "provider {key} has no models");
        }
    }

    #[test]
    fn should_round_trip_catalog_through_yaml() {
        let catalog = default_catalog();
        let yaml = serde_yaml::to_string(&catalog).unwrap();
        let reparsed = ModelCatalog::parse(&yaml);
        assert_eq!(reparsed, catalog);
    }

    #[test]
    fn should_default_every_missing_field_for_a_bare_model_entry() {
        // Only `id` provided — every other field must take its serde default.
        let yaml = r#"
providers:
  ollama:
    models:
      - id: bare-model
"#;
        let catalog = ModelCatalog::parse(yaml);
        let provider = catalog.providers.get("ollama").unwrap();
        let model = &provider.models[0];
        assert_eq!(model.id, "bare-model");
        assert_eq!(model.name, "");
        assert_eq!(model.family, "");
        assert_eq!(model.context_size, 0);
        assert!(!model.tool_calling);
        assert!(!model.code_edit);
        assert_eq!(model.quality, "");
        assert_eq!(model.cost_per_1k_input, None);
        assert_eq!(model.cost_per_1k_output, None);
        assert_eq!(model.ollama_tag, None);
        assert_eq!(model.model_path, None);
        assert_eq!(provider.egress, None);
        assert_eq!(provider.description, "");
    }

    #[test]
    fn should_fall_back_to_empty_catalog_on_invalid_yaml() {
        let catalog = ModelCatalog::parse("{ this: is not: valid ::: yaml");
        assert_eq!(catalog, ModelCatalog::default());
        assert!(catalog.providers.is_empty());
    }

    #[test]
    fn should_map_claude_model_onto_external_hosted_tooluse_caps() {
        let model = CatalogModel {
            id: "claude-sonnet-4-6".into(),
            context_size: 200_000,
            tool_calling: true,
            code_edit: true,
            cost_per_1k_input: Some(Decimal::new(3, 3)),
            cost_per_1k_output: Some(Decimal::new(15, 3)),
            ..Default::default()
        };
        let caps = model.caps(ProviderKind::Claude, Some(Egress::External));
        assert_eq!(caps.kind, ModelKind::Agent);
        assert_eq!(caps.modality, Modality::HostedApi);
        assert_eq!(caps.egress, Egress::External);
        assert_eq!(caps.output_contract, OutputContract::ToolUse);
        assert_eq!(caps.max_context_tokens, 200_000);
        assert!(caps.tool_use);
        assert!(caps.code_edit);
        assert_eq!(
            caps.cost,
            CostModel::PerToken {
                input_per_1k: Decimal::new(3, 3),
                output_per_1k: Decimal::new(15, 3),
            }
        );
    }

    #[test]
    fn should_map_ollama_model_to_free_local_server_caps() {
        let model = CatalogModel {
            id: "qwen2.5-coder".into(),
            context_size: 32_768,
            tool_calling: true,
            code_edit: true,
            ollama_tag: Some("qwen2.5-coder:7b".into()),
            ..Default::default()
        };
        let caps = model.caps(ProviderKind::Ollama, Some(Egress::LocalOnly));
        assert_eq!(caps.kind, ModelKind::Agent);
        assert_eq!(caps.modality, Modality::LocalServer);
        assert_eq!(caps.egress, Egress::LocalOnly);
        assert_eq!(caps.output_contract, OutputContract::UnifiedDiff);
        assert_eq!(caps.cost, CostModel::Free);
    }

    #[test]
    fn should_map_onnx_model_to_classify_only_inference_caps() {
        let model = CatalogModel {
            id: "failure-classifier".into(),
            context_size: 512,
            model_path: Some("models/x.onnx".into()),
            ..Default::default()
        };
        let caps = model.caps(ProviderKind::Onnx, None);
        assert_eq!(caps.kind, ModelKind::Inference);
        assert_eq!(caps.modality, Modality::InProcess);
        assert_eq!(caps.egress, Egress::LocalOnly);
        assert_eq!(caps.output_contract, OutputContract::ClassifyOnly);
        assert_eq!(caps.cost, CostModel::Free);
    }

    #[test]
    fn should_treat_cost_as_free_when_only_one_price_is_present() {
        let model = CatalogModel {
            id: "half-priced".into(),
            cost_per_1k_input: Some(Decimal::new(3, 3)),
            cost_per_1k_output: None,
            ..Default::default()
        };
        let caps = model.caps(ProviderKind::Claude, None);
        assert_eq!(caps.cost, CostModel::Free);
    }

    #[test]
    fn should_derive_egress_from_kind_when_no_override_given() {
        let model = CatalogModel {
            id: "x".into(),
            ..Default::default()
        };
        assert_eq!(
            model.caps(ProviderKind::Claude, None).egress,
            Egress::External
        );
        assert_eq!(
            model.caps(ProviderKind::Gemini, None).egress,
            Egress::External
        );
        assert_eq!(
            model.caps(ProviderKind::Ollama, None).egress,
            Egress::LocalOnly
        );
        assert_eq!(
            model.caps(ProviderKind::Onnx, None).egress,
            Egress::LocalOnly
        );
    }

    #[test]
    fn should_skip_unknown_provider_kinds_in_entries_and_provider_kinds() {
        let yaml = r#"
providers:
  openai:
    models:
      - id: gpt-4o
  claude:
    models:
      - id: claude-sonnet-4-6
        context_size: 200000
"#;
        let catalog = ModelCatalog::parse(yaml);
        // The unknown key still deserializes into the map...
        assert!(catalog.providers.contains_key("openai"));
        // ...but is excluded from the resolved views.
        let kinds = catalog.provider_kinds();
        assert_eq!(kinds, vec![ProviderKind::Claude]);
        let entries = catalog.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, ProviderKind::Claude);
        assert_eq!(entries[0].model.id, "claude-sonnet-4-6");
    }

    #[test]
    fn should_resolve_embedded_catalog_entries_to_caps() {
        let catalog = default_catalog();
        let entries = catalog.entries();
        assert!(!entries.is_empty());
        // Every embedded entry resolves to a known provider kind.
        assert!(entries.iter().all(|e| matches!(
            e.kind,
            ProviderKind::Claude | ProviderKind::Gemini | ProviderKind::Ollama | ProviderKind::Onnx
        )));
        // A claude entry advertises external egress + tool-use contract.
        let claude = entries
            .iter()
            .find(|e| e.kind == ProviderKind::Claude)
            .expect("a claude entry");
        assert_eq!(claude.caps.egress, Egress::External);
        assert_eq!(claude.caps.output_contract, OutputContract::ToolUse);
    }

    #[test]
    fn should_apply_explicit_provider_egress_override_in_entries() {
        // ONNX defaults to local_only; an (unusual) override is honored.
        let yaml = r#"
providers:
  onnx:
    egress: external
    models:
      - id: weird-onnx
"#;
        let catalog = ModelCatalog::parse(yaml);
        let entries = catalog.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].caps.egress, Egress::External);
    }

    #[test]
    fn should_fall_back_to_embedded_default_when_file_is_missing() {
        let catalog = load_catalog("/nonexistent/path/models.yaml").unwrap();
        // Embedded default has the four known providers.
        assert_eq!(catalog, ModelCatalog::load_default());
        assert!(catalog.providers.contains_key("claude"));
    }

    #[test]
    fn should_parse_decimal_costs_as_exact_values_not_f64() {
        // 0.00125 has no exact f64 representation; Decimal preserves it.
        let yaml = r#"
providers:
  gemini:
    models:
      - id: gemini-2.5-pro
        cost_per_1k_input: "0.00125"
        cost_per_1k_output: "0.010"
"#;
        let catalog = ModelCatalog::parse(yaml);
        let model = &catalog.providers.get("gemini").unwrap().models[0];
        assert_eq!(
            model.cost_per_1k_input,
            Some(Decimal::from_str("0.00125").unwrap())
        );
        assert_eq!(
            model.cost_per_1k_output,
            Some(Decimal::from_str("0.010").unwrap())
        );
    }
}
