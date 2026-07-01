//! Model catalog + Ollama discovery/pull over HTTP (Phase 5 — Agentic
//! Remediation Tier).
//!
//! Exposes the checked-in [`ModelCatalog`] (Phase 0) and thin Ollama
//! discovery/pull proxies. Every outbound call reuses the SAME gates that guard
//! `validate_model_account`:
//!
//! - **SSRF guard.** [`assert_endpoint_safe`] is applied to the account endpoint
//!   (passing the account's egress) before ANY tags/pull request leaves the
//!   process. Local-only providers (Ollama) are intentionally exempt inside that
//!   guard — reaching `localhost` is their purpose — so this proxy deliberately
//!   relies on the shared function rather than a bespoke copy.
//! - **Air-gap ceiling (ADR-014).** The catalog endpoint filters out
//!   external-egress providers (Claude/Gemini) when the caller's owned
//!   organization is `air_gapped`.
//! - **Scope isolation.** Ollama endpoints load the authorized account exactly
//!   like `model_accounts`; cross-scope access returns `404` (never leaks).
//!
//! No credential is ever logged, and upstream Ollama error detail is never
//! returned to clients — failures are logged server-side and reduced to a
//! generic `502`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::remediation::{
    CatalogModel, CostModel, Egress, ModelCaps, ModelCatalog, ProviderKind,
};
use ampel_db::entities::organization;

use crate::extractors::AuthUser;
use crate::handlers::model_accounts::load_authorized_account;
use crate::handlers::security::assert_endpoint_safe;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

/// Ollama's default local endpoint (ADR-009) when an account carries none.
const DEFAULT_OLLAMA_ENDPOINT: &str = "http://localhost:11434";

// ============================================================================
// Catalog DTOs (camelCase)
// ============================================================================

/// The catalog grouped by provider, each model resolved to its capabilities.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogResponse {
    pub providers: Vec<CatalogProviderDto>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProviderDto {
    pub kind: String,
    pub description: String,
    pub egress: String,
    pub models: Vec<CatalogModelDto>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogModelDto {
    pub id: String,
    pub name: String,
    pub family: String,
    pub quality: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ollama_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_path: Option<String>,
    pub context_window: u32,
    pub tool_use: bool,
    pub code_edit: bool,
    pub egress: String,
    pub output_contract: String,
    pub cost: CostDto,
}

/// Cost as exact `Decimal`-as-string (never `f64`).
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CostDto {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_per_1k: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_per_1k: Option<String>,
}

impl CostDto {
    fn from_cost(cost: &CostModel) -> Self {
        match cost {
            CostModel::PerToken {
                input_per_1k,
                output_per_1k,
            } => Self {
                kind: "per_token".to_string(),
                input_per_1k: Some(input_per_1k.to_string()),
                output_per_1k: Some(output_per_1k.to_string()),
            },
            CostModel::Free => Self {
                kind: "free".to_string(),
                input_per_1k: None,
                output_per_1k: None,
            },
        }
    }
}

impl CatalogModelDto {
    fn from_model(model: &CatalogModel, caps: &ModelCaps) -> Self {
        Self {
            id: model.id.clone(),
            name: model.name.clone(),
            family: model.family.clone(),
            quality: model.quality.clone(),
            ollama_tag: model.ollama_tag.clone(),
            model_path: model.model_path.clone(),
            context_window: caps.max_context_tokens,
            tool_use: caps.tool_use,
            code_edit: caps.code_edit,
            egress: caps.egress.to_string(),
            output_contract: caps.output_contract.to_string(),
            cost: CostDto::from_cost(&caps.cost),
        }
    }
}

/// Default egress for a provider kind (ADR-009), used for the provider-level
/// display value. Mirrors the catalog's own kind→egress derivation.
fn egress_for_kind(kind: ProviderKind) -> Egress {
    match kind {
        ProviderKind::Claude | ProviderKind::Gemini => Egress::External,
        ProviderKind::Ollama | ProviderKind::Onnx => Egress::LocalOnly,
    }
}

/// Pure, DB-free projection of a [`ModelCatalog`] into the response DTO. When
/// `air_gapped` is set, any provider/entry whose resolved egress is
/// [`Egress::External`] (Claude/Gemini) is omitted entirely (ADR-014).
fn filter_catalog(catalog: &ModelCatalog, air_gapped: bool) -> ModelCatalogResponse {
    let mut providers = Vec::new();
    for (key, provider) in &catalog.providers {
        // Unknown provider keys are skipped (not fatal), mirroring `entries()`.
        let Ok(kind) = key.parse::<ProviderKind>() else {
            continue;
        };
        let egress_override = provider
            .egress
            .as_deref()
            .and_then(|s| s.parse::<Egress>().ok());
        let provider_egress = egress_override.unwrap_or_else(|| egress_for_kind(kind));

        // Air-gapped orgs cannot use external-egress providers at all.
        if air_gapped && provider_egress == Egress::External {
            continue;
        }

        let mut models = Vec::new();
        for model in &provider.models {
            let caps = model.caps(kind, egress_override);
            if air_gapped && caps.egress == Egress::External {
                continue;
            }
            models.push(CatalogModelDto::from_model(model, &caps));
        }
        if models.is_empty() {
            continue;
        }
        providers.push(CatalogProviderDto {
            kind: kind.to_string(),
            description: provider.description.clone(),
            egress: provider_egress.to_string(),
            models,
        });
    }
    ModelCatalogResponse { providers }
}

// ============================================================================
// Ollama DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogQuery {
    pub organization_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaTagsQuery {
    pub account_id: Uuid,
}

/// Upstream `/api/tags` shape (Ollama emits snake_case field names).
#[derive(Debug, Deserialize)]
struct OllamaTagsUpstream {
    #[serde(default)]
    models: Vec<OllamaTagUpstream>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagUpstream {
    #[serde(default)]
    name: String,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    digest: Option<String>,
    #[serde(default)]
    modified_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaTagsResponse {
    pub models: Vec<OllamaTagDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaTagDto {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaPullRequest {
    pub account_id: Uuid,
    /// The Ollama pull tag (e.g. `qwen2.5-coder:7b`).
    pub model: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaPullResponse {
    pub job_id: Uuid,
    pub status: PullStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullStatusResponse {
    pub job_id: Uuid,
    pub status: PullStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

// ============================================================================
// Pull job registry + state machine
// ============================================================================

/// Lifecycle of an Ollama pull. Terminal states (`Ready`, `Error`) never
/// transition back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PullStatus {
    Queued,
    Downloading,
    Ready,
    Error,
}

impl PullStatus {
    fn is_terminal(self) -> bool {
        matches!(self, PullStatus::Ready | PullStatus::Error)
    }
}

/// Pure transition function. Returns `Some(to)` for a valid transition, else
/// `None`. Valid: `Queued→Downloading`, `Downloading→Ready`, and any
/// non-terminal state `→Error`. Terminal states do not transition.
fn apply_transition(from: PullStatus, to: PullStatus) -> Option<PullStatus> {
    if from.is_terminal() {
        return None;
    }
    let ok = matches!(
        (from, to),
        (PullStatus::Queued, PullStatus::Downloading)
            | (PullStatus::Downloading, PullStatus::Ready)
            | (_, PullStatus::Error)
    );
    ok.then_some(to)
}

/// In-memory (never persisted) record of a single pull job.
#[derive(Debug, Clone)]
struct PullJob {
    #[allow(dead_code)]
    account_id: Uuid,
    owner_user_id: Uuid,
    status: PullStatus,
    detail: Option<String>,
    /// Monotonic insertion order, used for deterministic eviction (NOT
    /// wall-clock, so the bounding policy is unit-testable).
    seq: u64,
}

/// Hard ceiling on the number of retained pull jobs. The registry is advisory
/// progress only, so once past this many entries the oldest are evicted
/// (terminal jobs first) to keep memory bounded.
const MAX_PULL_JOBS: usize = 256;

/// Monotonic sequence source for [`PullJob::seq`]. Sequence-based (not
/// wall-clock) so eviction ordering is deterministic and testable.
static NEXT_SEQ: AtomicU64 = AtomicU64::new(0);

/// Module-level registry, mirroring emailibrium's `DOWNLOADING` static. Keyed by
/// a fresh job [`Uuid`]; contents are advisory progress only.
static PULL_JOBS: LazyLock<Mutex<HashMap<Uuid, PullJob>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Allocate the next monotonic insertion sequence for a new [`PullJob`].
fn next_seq() -> u64 {
    NEXT_SEQ.fetch_add(1, Ordering::Relaxed)
}

/// Bound `jobs` to [`MAX_PULL_JOBS`] after an insert. Eviction is deterministic
/// and sequence-based: the oldest **terminal** (`Ready`/`Error`) jobs are
/// removed first (lowest `seq`), and only if that is still not enough are the
/// oldest jobs overall removed. `keep` (the just-inserted job) is never evicted.
fn evict_over_cap(jobs: &mut HashMap<Uuid, PullJob>, keep: Uuid) {
    if jobs.len() <= MAX_PULL_JOBS {
        return;
    }

    // Pass 1: terminal jobs, oldest sequence first.
    let mut terminal: Vec<(u64, Uuid)> = jobs
        .iter()
        .filter(|(id, job)| **id != keep && job.status.is_terminal())
        .map(|(id, job)| (job.seq, *id))
        .collect();
    terminal.sort_unstable_by_key(|(seq, _)| *seq);
    for (_, id) in terminal {
        if jobs.len() <= MAX_PULL_JOBS {
            break;
        }
        jobs.remove(&id);
    }

    // Pass 2: still over cap (all remaining are non-terminal) — drop the oldest
    // overall, never the job we just inserted.
    if jobs.len() > MAX_PULL_JOBS {
        let mut all: Vec<(u64, Uuid)> = jobs
            .iter()
            .filter(|(id, _)| **id != keep)
            .map(|(id, job)| (job.seq, *id))
            .collect();
        all.sort_unstable_by_key(|(seq, _)| *seq);
        for (_, id) in all {
            if jobs.len() <= MAX_PULL_JOBS {
                break;
            }
            jobs.remove(&id);
        }
    }
}

/// Advance a job through the pure state machine and optionally record a detail
/// string. Invalid transitions are ignored (status is left unchanged).
fn advance_job(job_id: Uuid, to: PullStatus, detail: Option<String>) {
    let mut jobs = PULL_JOBS.lock().expect("pull jobs mutex poisoned");
    if let Some(job) = jobs.get_mut(&job_id) {
        if let Some(next) = apply_transition(job.status, to) {
            job.status = next;
        }
        if let Some(d) = detail {
            job.detail = Some(d);
        }
    }
}

/// Load an authorized account, assert it is an Ollama account, and resolve the
/// `(endpoint, egress)` pair to drive with. Applies the SAME SSRF guard the
/// validate path uses before returning.
async fn resolve_ollama_target(
    state: &AppState,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<(String, Egress), ApiError> {
    let account = load_authorized_account(state, user_id, account_id).await?;
    let kind: ProviderKind = account
        .provider_kind
        .parse()
        .map_err(|_| ApiError::internal("invalid provider_kind in database"))?;
    if kind != ProviderKind::Ollama {
        return Err(ApiError::bad_request(
            "account is not an Ollama provider account",
        ));
    }
    let egress = account
        .egress_class
        .parse::<Egress>()
        .unwrap_or(Egress::LocalOnly);
    let endpoint = account
        .endpoint_url
        .clone()
        .unwrap_or_else(|| DEFAULT_OLLAMA_ENDPOINT.to_string());

    // SAME guard as validate_model_account: local-only egress is exempt inside
    // it, but external egress is fully SSRF-checked before any request.
    assert_endpoint_safe(&endpoint, egress).await?;
    Ok((endpoint, egress))
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/model-catalog — the embedded catalog grouped by provider. When
/// `organization_id` is supplied AND owned AND air-gapped, external-egress
/// providers are omitted (ADR-014).
pub async fn get_model_catalog(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<CatalogQuery>,
) -> Result<Json<ApiResponse<ModelCatalogResponse>>, ApiError> {
    let catalog = ModelCatalog::load_default();

    let air_gapped = match query.organization_id {
        Some(org_id) => organization::Entity::find_by_id(org_id)
            .one(&state.db)
            .await?
            // Only the owner's air-gapped flag filters the view; a missing or
            // unowned org simply applies no filter (returns everything usable).
            .filter(|o| o.owner_id == auth.user_id)
            .map(|o| o.air_gapped)
            .unwrap_or(false),
        None => false,
    };

    Ok(Json(ApiResponse::success(filter_catalog(
        &catalog, air_gapped,
    ))))
}

/// GET /api/model-catalog/ollama/tags — proxy Ollama's `/api/tags`.
pub async fn list_ollama_tags(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<OllamaTagsQuery>,
) -> Result<Json<ApiResponse<OllamaTagsResponse>>, ApiError> {
    let (endpoint, _egress) = resolve_ollama_target(&state, auth.user_id, query.account_id).await?;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| {
            tracing::warn!(error = %e, "failed to build ollama http client");
            ApiError::internal("could not initialize ollama client")
        })?;

    let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
    let resp = client.get(&url).send().await.map_err(|e| {
        tracing::warn!(account_id = %query.account_id, error = %e, "ollama tags request failed");
        ApiError::bad_gateway("could not reach the Ollama server")
    })?;
    if !resp.status().is_success() {
        tracing::warn!(
            account_id = %query.account_id,
            status = %resp.status(),
            "ollama tags returned non-success status"
        );
        return Err(ApiError::bad_gateway("the Ollama server returned an error"));
    }

    let upstream = resp.json::<OllamaTagsUpstream>().await.map_err(|e| {
        tracing::warn!(account_id = %query.account_id, error = %e, "invalid ollama tags response");
        ApiError::bad_gateway("the Ollama server returned an invalid response")
    })?;

    let models = upstream
        .models
        .into_iter()
        .map(|m| OllamaTagDto {
            name: m.name,
            size: m.size,
            digest: m.digest,
            modified_at: m.modified_at,
        })
        .collect();

    Ok(Json(ApiResponse::success(OllamaTagsResponse { models })))
}

/// POST /api/model-catalog/ollama/pull — start a background Ollama pull and
/// return the job id immediately.
pub async fn pull_ollama_model(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<OllamaPullRequest>,
) -> Result<Json<ApiResponse<OllamaPullResponse>>, ApiError> {
    let (endpoint, _egress) = resolve_ollama_target(&state, auth.user_id, req.account_id).await?;

    if req.model.trim().is_empty() {
        return Err(ApiError::bad_request("model tag must not be empty"));
    }

    let job_id = Uuid::new_v4();
    {
        let mut jobs = PULL_JOBS.lock().expect("pull jobs mutex poisoned");
        jobs.insert(
            job_id,
            PullJob {
                account_id: req.account_id,
                owner_user_id: auth.user_id,
                status: PullStatus::Queued,
                detail: Some("queued".to_string()),
                seq: next_seq(),
            },
        );
        // Keep the registry bounded; never evict the job we just inserted.
        evict_over_cap(&mut jobs, job_id);
    }

    let tag = req.model.clone();
    tokio::spawn(async move {
        run_pull(job_id, endpoint, tag).await;
    });

    tracing::info!(user_id = %auth.user_id, job_id = %job_id, "Ollama model pull started");
    Ok(Json(ApiResponse::success(OllamaPullResponse {
        job_id,
        status: PullStatus::Queued,
    })))
}

/// GET /api/model-catalog/ollama/pull/{id}/status — owner-scoped job status.
pub async fn get_pull_status(
    _state: State<AppState>,
    auth: AuthUser,
    Path(job_id): Path<Uuid>,
) -> Result<Json<ApiResponse<PullStatusResponse>>, ApiError> {
    let jobs = PULL_JOBS.lock().expect("pull jobs mutex poisoned");
    // A job owned by another user is treated as absent — never leak existence.
    let job = jobs
        .get(&job_id)
        .filter(|j| j.owner_user_id == auth.user_id)
        .ok_or_else(|| ApiError::not_found("Pull job not found"))?;

    Ok(Json(ApiResponse::success(PullStatusResponse {
        job_id,
        status: job.status,
        detail: job.detail.clone(),
    })))
}

// ============================================================================
// Background pull driver
// ============================================================================

/// Aggregate progress extracted from a batch of NDJSON lines.
enum NdjsonProgress {
    None,
    Status(String),
    Error,
}

/// Drain every COMPLETE newline-delimited JSON object from `buf` (leaving any
/// trailing partial line), returning the aggregate progress. An `{"error": …}`
/// line wins; otherwise the last `{"status": …}` string is reported.
fn drain_ndjson(buf: &mut Vec<u8>) -> NdjsonProgress {
    let Some(last_nl) = buf.iter().rposition(|b| *b == b'\n') else {
        return NdjsonProgress::None;
    };
    let complete: Vec<u8> = buf.drain(..=last_nl).collect();

    let mut had_error = false;
    let mut last_status: Option<String> = None;
    for line in complete.split(|b| *b == b'\n') {
        if line.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(line) {
            if value.get("error").is_some() {
                had_error = true;
            } else if let Some(s) = value.get("status").and_then(|s| s.as_str()) {
                last_status = Some(s.to_string());
            }
        }
    }

    if had_error {
        NdjsonProgress::Error
    } else if let Some(s) = last_status {
        NdjsonProgress::Status(s)
    } else {
        NdjsonProgress::None
    }
}

/// Drive an Ollama `/api/pull` NDJSON stream, updating the job registry. Upstream
/// error detail is logged server-side and reduced to a generic job detail.
async fn run_pull(job_id: Uuid, endpoint: String, tag: String) {
    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(job_id = %job_id, error = %e, "failed to build ollama pull client");
            advance_job(job_id, PullStatus::Error, Some("pull failed".to_string()));
            return;
        }
    };

    let url = format!("{}/api/pull", endpoint.trim_end_matches('/'));
    let body = serde_json::json!({ "name": tag, "stream": true });

    let mut resp = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(job_id = %job_id, error = %e, "ollama pull request failed");
            advance_job(job_id, PullStatus::Error, Some("pull failed".to_string()));
            return;
        }
    };
    if !resp.status().is_success() {
        tracing::warn!(job_id = %job_id, status = %resp.status(), "ollama pull returned non-success");
        advance_job(job_id, PullStatus::Error, Some("pull failed".to_string()));
        return;
    }

    advance_job(
        job_id,
        PullStatus::Downloading,
        Some("downloading".to_string()),
    );

    let mut buf: Vec<u8> = Vec::new();
    loop {
        match resp.chunk().await {
            Ok(Some(bytes)) => {
                buf.extend_from_slice(&bytes);
                match drain_ndjson(&mut buf) {
                    NdjsonProgress::Error => {
                        tracing::warn!(job_id = %job_id, "ollama pull stream reported an error");
                        advance_job(job_id, PullStatus::Error, Some("pull failed".to_string()));
                        return;
                    }
                    NdjsonProgress::Status(s) => {
                        advance_job(job_id, PullStatus::Downloading, Some(s));
                    }
                    NdjsonProgress::None => {}
                }
            }
            Ok(None) => break,
            Err(e) => {
                tracing::warn!(job_id = %job_id, error = %e, "ollama pull stream failed");
                advance_job(job_id, PullStatus::Error, Some("pull failed".to_string()));
                return;
            }
        }
    }

    advance_job(job_id, PullStatus::Ready, Some("ready".to_string()));
    tracing::info!(job_id = %job_id, "Ollama model pull completed");
}

// ============================================================================
// Tests (DB-free, pure functions only — no outbound HTTP is mocked)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(resp: &ModelCatalogResponse) -> Vec<String> {
        resp.providers.iter().map(|p| p.kind.clone()).collect()
    }

    #[test]
    fn should_hide_external_providers_when_org_is_air_gapped() {
        let catalog = ModelCatalog::load_default();

        // Air-gapped: only local_only providers (ollama/onnx) survive.
        let filtered = filter_catalog(&catalog, true);
        let ks = kinds(&filtered);
        assert!(!ks.contains(&"claude".to_string()), "claude must be hidden");
        assert!(!ks.contains(&"gemini".to_string()), "gemini must be hidden");
        assert!(ks.contains(&"ollama".to_string()), "ollama must remain");
        // Every surviving provider AND model advertises local-only egress.
        for provider in &filtered.providers {
            assert_eq!(provider.egress, "local_only");
            for model in &provider.models {
                assert_eq!(model.egress, "local_only");
            }
        }

        // Not air-gapped: external providers are present.
        let unfiltered = filter_catalog(&catalog, false);
        let ks = kinds(&unfiltered);
        assert!(ks.contains(&"claude".to_string()), "claude must be present");
        assert!(ks.contains(&"gemini".to_string()), "gemini must be present");
    }

    #[tokio::test]
    async fn should_reuse_shared_ssrf_guard_on_tags_and_pull() {
        // External egress: the shared guard rejects the cloud-metadata IP. This
        // is the exact guard resolve_ollama_target applies before any request.
        let blocked = assert_endpoint_safe("http://169.254.169.254/latest", Egress::External).await;
        assert!(blocked.is_err(), "metadata IP must be rejected");

        // LocalOnly egress is INTENTIONALLY exempt — Ollama's localhost is
        // legitimate — proving the proxy reuses the same shared guard rather
        // than a bespoke copy.
        let allowed = assert_endpoint_safe("http://localhost:11434", Egress::LocalOnly).await;
        assert!(allowed.is_ok(), "local-only localhost must be allowed");
    }

    #[test]
    fn should_evict_oldest_terminal_jobs_when_registry_is_full() {
        // Arrange: insert well over the cap, all terminal, into a local map so
        // the pure eviction policy is exercised without touching global state.
        let mut jobs: HashMap<Uuid, PullJob> = HashMap::new();
        let total = MAX_PULL_JOBS + 50;
        let mut ids = Vec::with_capacity(total);
        for seq in 0..total {
            let id = Uuid::new_v4();
            ids.push(id);
            jobs.insert(
                id,
                PullJob {
                    account_id: Uuid::new_v4(),
                    owner_user_id: Uuid::new_v4(),
                    status: PullStatus::Ready, // terminal
                    detail: None,
                    seq: seq as u64,
                },
            );
            // Act: bound after every insert, mirroring the handler.
            evict_over_cap(&mut jobs, id);
        }

        // Assert: registry stayed bounded and the newest jobs survived while the
        // oldest were evicted first.
        assert!(
            jobs.len() <= MAX_PULL_JOBS,
            "registry must stay bounded, got {}",
            jobs.len()
        );
        assert!(
            jobs.contains_key(ids.last().unwrap()),
            "the newest job must survive eviction"
        );
        assert!(
            !jobs.contains_key(&ids[0]),
            "the oldest terminal job must be evicted first"
        );
    }

    #[test]
    fn should_keep_the_just_inserted_job_even_when_full() {
        // Non-terminal (Queued) jobs force pass-2 eviction; the just-inserted
        // job must never be the one removed.
        let mut jobs: HashMap<Uuid, PullJob> = HashMap::new();
        let mut last = Uuid::new_v4();
        for seq in 0..(MAX_PULL_JOBS + 10) {
            last = Uuid::new_v4();
            jobs.insert(
                last,
                PullJob {
                    account_id: Uuid::new_v4(),
                    owner_user_id: Uuid::new_v4(),
                    status: PullStatus::Queued, // non-terminal
                    detail: None,
                    seq: seq as u64,
                },
            );
            evict_over_cap(&mut jobs, last);
        }
        assert!(jobs.len() <= MAX_PULL_JOBS);
        assert!(
            jobs.contains_key(&last),
            "the just-inserted job must be retained"
        );
    }

    #[test]
    fn should_allow_valid_pull_status_transitions() {
        assert_eq!(
            apply_transition(PullStatus::Queued, PullStatus::Downloading),
            Some(PullStatus::Downloading)
        );
        assert_eq!(
            apply_transition(PullStatus::Downloading, PullStatus::Ready),
            Some(PullStatus::Ready)
        );
        assert_eq!(
            apply_transition(PullStatus::Queued, PullStatus::Error),
            Some(PullStatus::Error)
        );
        assert_eq!(
            apply_transition(PullStatus::Downloading, PullStatus::Error),
            Some(PullStatus::Error)
        );
    }

    #[test]
    fn should_reject_invalid_and_terminal_pull_status_transitions() {
        // Skipping Downloading is invalid.
        assert_eq!(
            apply_transition(PullStatus::Queued, PullStatus::Ready),
            None
        );
        // Backwards is invalid.
        assert_eq!(
            apply_transition(PullStatus::Downloading, PullStatus::Queued),
            None
        );
        // Terminal states never transition (not even to Error).
        assert_eq!(apply_transition(PullStatus::Ready, PullStatus::Error), None);
        assert_eq!(
            apply_transition(PullStatus::Error, PullStatus::Downloading),
            None
        );
        assert_eq!(apply_transition(PullStatus::Ready, PullStatus::Ready), None);
    }

    #[test]
    fn should_report_ndjson_error_line_over_status() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"{\"status\":\"pulling manifest\"}\n");
        buf.extend_from_slice(b"{\"error\":\"model not found\"}\n");
        assert!(matches!(drain_ndjson(&mut buf), NdjsonProgress::Error));
        // The whole complete portion was drained.
        assert!(buf.is_empty());
    }

    #[test]
    fn should_retain_partial_trailing_ndjson_line() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"{\"status\":\"downloading\"}\n{\"status\":\"verif");
        match drain_ndjson(&mut buf) {
            NdjsonProgress::Status(s) => assert_eq!(s, "downloading"),
            other => panic!(
                "expected Status, got {:?}",
                matches!(other, NdjsonProgress::None)
            ),
        }
        // The incomplete trailing line is preserved for the next chunk.
        assert_eq!(buf, b"{\"status\":\"verif");
    }
}
