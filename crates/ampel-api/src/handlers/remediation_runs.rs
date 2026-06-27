//! Fleet PR Remediation — Phase 3 (Observability & UX) API layer.
//!
//! Run-history reads (`/runs`, `/runs/{id}`) and live progress (`/runs/{id}/events`,
//! SSE) are served directly from `ampel-db`'s canonical `remediation_run` /
//! `remediation_run_pr` entities (the Phase-1 reads-in-handler pattern), avoiding
//! churn on the read-only `ampel-core` repository trait.
//!
//! ## Process topology (ADR-011)
//! `ampel-worker` and `ampel-api` are **separate processes** (separate Fly apps),
//! so an in-process broadcast bus cannot bridge worker→api progress. The SSE
//! stream therefore **polls the `remediation_run` row** every ~2s, emitting an
//! event whenever `state`/`ci_status` changes, a KeepAlive every ~15s, and a
//! terminal `run_finished` event (then closing) once the run reaches a terminal
//! state — or after a 30-minute safety cap.
//!
//! ## SSE auth
//! `EventSource` cannot set an `Authorization` header. A caller first mints a
//! short-lived (30s) SSE token via `POST /api/remediation/sse-token` (a JWT signed
//! with the same `jwt_secret`, scoped `sse-remediation`), then connects to the
//! events endpoint with `?token=…`. For convenience the events endpoint also
//! accepts a normal `Authorization: Bearer <access-token>` header (used by tests
//! and server-to-server callers). No long-lived access token is ever required in
//! the URL.
//!
//! Security: ownership is checked on **every** endpoint — a run whose repository
//! the caller does not own returns 404 (never 403), matching Phase 1. No secrets
//! are logged or returned.

use std::{convert::Infallible, str::FromStr, time::Duration};

use axum::{
    extract::{Path, Query, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use chrono::{DateTime, Utc};
use futures::Stream;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ampel_core::remediation::{MergeDisposition, RunState};
use ampel_db::entities::{
    remediation_agent_session, remediation_run, remediation_run_pr, repository,
};

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

// ============================================================================
// Constants
// ============================================================================

/// How often the SSE stream re-reads the run row.
const SSE_POLL_INTERVAL: Duration = Duration::from_secs(2);
/// KeepAlive comment cadence to defeat idle proxies.
const SSE_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(15);
/// Safety cap: never hold an SSE connection open longer than this.
const SSE_MAX_DURATION: Duration = Duration::from_secs(30 * 60);
/// TTL of a minted SSE token.
const SSE_TOKEN_TTL_SECS: i64 = 30;
/// JWT scope claim distinguishing an SSE token from an access token.
const SSE_TOKEN_SCOPE: &str = "sse-remediation";

/// Synthetic gate state that `approve` transitions out of. Not part of
/// [`RunState`] (the current state machine never produces it — see module note),
/// so it is matched as a raw string for forward compatibility.
const AWAITING_APPROVAL: &str = "awaiting_approval";

const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 200;

// ============================================================================
// DTOs
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunSummary {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub policy_id: Uuid,
    pub state: String,
    pub autonomy_level: String,
    pub triggered_by: String,
    pub triggered_by_user_id: Option<Uuid>,
    pub ci_status: String,
    pub consolidated_pr_number: Option<i64>,
    pub merged: bool,
    pub branch_name: String,
    pub attempts: i32,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<remediation_run::Model> for RunSummary {
    fn from(m: remediation_run::Model) -> Self {
        Self {
            id: m.id,
            repository_id: m.repository_id,
            policy_id: m.policy_id,
            state: m.state,
            autonomy_level: m.autonomy_level,
            triggered_by: m.triggered_by,
            triggered_by_user_id: m.triggered_by_user_id,
            ci_status: m.ci_status,
            consolidated_pr_number: m.consolidated_pr_number,
            merged: m.merged,
            branch_name: m.branch_name,
            attempts: m.attempts,
            error_message: m.error_message,
            started_at: m.started_at.to_rfc3339(),
            completed_at: m.completed_at.map(|t| t.to_rfc3339()),
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrDispositionView {
    pub pr_number: i64,
    /// Raw parsed disposition value-object (externally tagged on `disposition`).
    pub disposition: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CiMatrix {
    pub status: String,
    pub logs_url: Option<String>,
    pub head_sha: Option<String>,
    /// Predicted conflicts carried on the consolidation plan, if any.
    pub predicted_conflicts: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictEntry {
    pub pr_number: i64,
    pub reason: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictReport {
    /// PRs skipped because of an unresolved merge conflict.
    pub conflicts: Vec<ConflictEntry>,
    /// PRs intentionally left open (draft / excluded / etc.).
    pub skipped: Vec<ConflictEntry>,
}

/// Agent-session snapshot for the run-detail view (Phase 4). Mirrors the
/// non-secret columns of `remediation_agent_session`; credential/account
/// references (`model_provider_account_id`) are intentionally omitted.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionDto {
    pub iterations: i32,
    pub max_iterations: Option<i32>,
    pub tokens_used: i64,
    /// Decimal cost carried as a string (entity convention), serialized as-is.
    pub cost_usd: Option<String>,
    pub status: String,
    pub failure_class: Option<String>,
    pub classifier_source: Option<String>,
    pub classifier_confidence: Option<f64>,
    pub transcript_ref: Option<String>,
}

impl From<remediation_agent_session::Model> for AgentSessionDto {
    fn from(m: remediation_agent_session::Model) -> Self {
        Self {
            iterations: m.iterations,
            max_iterations: m.max_iterations,
            tokens_used: m.tokens_used,
            cost_usd: m.cost_usd,
            status: m.status,
            failure_class: m.failure_class,
            classifier_source: m.classifier_source,
            classifier_confidence: m.classifier_confidence,
            transcript_ref: m.transcript_ref,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDetail {
    pub run: RunSummary,
    pub prs: Vec<PrDispositionView>,
    pub ci_matrix: CiMatrix,
    pub conflict_report: ConflictReport,
    /// Most-recent agent session for this run, if any (Phase 4).
    pub agent_session: Option<AgentSessionDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRunsQuery {
    pub repository_id: Option<Uuid>,
    pub state: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    /// Short-lived SSE token minted via `POST /api/remediation/sse-token`.
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SseTokenResponse {
    pub token: String,
    pub expires_at: String,
}

/// Claims for the short-lived SSE token (signed with the shared `jwt_secret`).
#[derive(Debug, Serialize, Deserialize)]
struct SseTokenClaims {
    sub: Uuid,
    exp: i64,
    scope: String,
}

// ============================================================================
// Ownership helpers
// ============================================================================

/// Repository ids the caller owns (the run-scoping anchor, matching `preview`).
async fn owned_repo_ids(state: &AppState, user_id: Uuid) -> Result<Vec<Uuid>, ApiError> {
    Ok(repository::Entity::find()
        .filter(repository::Column::UserId.eq(user_id))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect())
}

/// Load a run and assert the caller owns its repository. Returns 404 on miss or
/// cross-scope access (never 403), mirroring Phase 1.
async fn load_authorized_run(
    state: &AppState,
    user_id: Uuid,
    run_id: Uuid,
) -> Result<remediation_run::Model, ApiError> {
    let run = remediation_run::Entity::find_by_id(run_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Run not found"))?;

    let owns = repository::Entity::find_by_id(run.repository_id)
        .one(&state.db)
        .await?
        .map(|r| r.user_id == user_id)
        .unwrap_or(false);

    if owns {
        Ok(run)
    } else {
        Err(ApiError::not_found("Run not found"))
    }
}

// ============================================================================
// GET /api/remediation/runs
// ============================================================================

/// GET /api/remediation/runs — list runs in the caller's scope with filters.
pub async fn list_runs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListRunsQuery>,
) -> Result<Json<ApiResponse<Vec<RunSummary>>>, ApiError> {
    let repo_ids = owned_repo_ids(&state, auth.user_id).await?;
    if repo_ids.is_empty() {
        return Ok(Json(ApiResponse::success(Vec::new())));
    }

    // Optional repository filter must still respect ownership.
    let scoped_ids: Vec<Uuid> = match q.repository_id {
        Some(rid) if repo_ids.contains(&rid) => vec![rid],
        Some(_) => return Ok(Json(ApiResponse::success(Vec::new()))),
        None => repo_ids,
    };

    let mut query = remediation_run::Entity::find()
        .filter(remediation_run::Column::RepositoryId.is_in(scoped_ids));

    if let Some(state_filter) = q.state.as_deref() {
        query = query.filter(remediation_run::Column::State.eq(state_filter));
    }
    if let Some(since) = q.since {
        query = query.filter(remediation_run::Column::CreatedAt.gte(since));
    }
    if let Some(until) = q.until {
        query = query.filter(remediation_run::Column::CreatedAt.lte(until));
    }

    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = q.offset.unwrap_or(0);

    let runs = query
        .order_by_desc(remediation_run::Column::CreatedAt)
        .limit(limit)
        .offset(offset)
        .all(&state.db)
        .await?;

    let summaries = runs.into_iter().map(RunSummary::from).collect();
    Ok(Json(ApiResponse::success(summaries)))
}

// ============================================================================
// GET /api/remediation/runs/{id}
// ============================================================================

/// GET /api/remediation/runs/{id} — run detail with dispositions, CI matrix, and
/// conflict report. 404 when not in the caller's scope.
pub async fn get_run(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(run_id): Path<Uuid>,
) -> Result<Json<ApiResponse<RunDetail>>, ApiError> {
    let run = load_authorized_run(&state, auth.user_id, run_id).await?;

    let pr_rows = remediation_run_pr::Entity::find()
        .filter(remediation_run_pr::Column::RemediationRunId.eq(run_id))
        .order_by_asc(remediation_run_pr::Column::PrNumber)
        .all(&state.db)
        .await?;

    let mut prs = Vec::with_capacity(pr_rows.len());
    let mut conflict_report = ConflictReport::default();

    for row in &pr_rows {
        // Raw value for the API; typed parse drives the conflict report.
        let raw: serde_json::Value =
            serde_json::from_str(&row.disposition).unwrap_or(serde_json::Value::Null);
        prs.push(PrDispositionView {
            pr_number: row.pr_number,
            disposition: raw,
        });

        if let Ok(disp) = serde_json::from_str::<MergeDisposition>(&row.disposition) {
            match disp {
                MergeDisposition::SkippedConflict { reason } => {
                    conflict_report.conflicts.push(ConflictEntry {
                        pr_number: row.pr_number,
                        reason,
                    });
                }
                MergeDisposition::LeftOpen { reason } => {
                    conflict_report.skipped.push(ConflictEntry {
                        pr_number: row.pr_number,
                        reason,
                    });
                }
                _ => {}
            }
        }
    }

    let predicted_conflicts = run
        .consolidation_plan
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| {
            v.get("predicted_conflicts")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
        })
        .unwrap_or_default();

    let ci_matrix = CiMatrix {
        status: run.ci_status.clone(),
        logs_url: run.ci_logs_url.clone(),
        head_sha: run.head_sha.clone(),
        predicted_conflicts,
    };

    // Most-recent agent session for this run (None if the run never ran an agent).
    let agent_session = remediation_agent_session::Entity::find()
        .filter(remediation_agent_session::Column::RemediationRunId.eq(run_id))
        .order_by_desc(remediation_agent_session::Column::CreatedAt)
        .one(&state.db)
        .await?
        .map(AgentSessionDto::from);

    Ok(Json(ApiResponse::success(RunDetail {
        run: RunSummary::from(run),
        prs,
        ci_matrix,
        conflict_report,
        agent_session,
    })))
}

// ============================================================================
// SSE — pure decision helper (unit-tested) + handler
// ============================================================================

/// The change-detection inputs for one SSE poll iteration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunSnapshot {
    pub state: String,
    pub ci_status: String,
}

impl RunSnapshot {
    fn from_model(m: &remediation_run::Model) -> Self {
        Self {
            state: m.state.clone(),
            ci_status: m.ci_status.clone(),
        }
    }
}

/// Outcome of comparing the last-emitted snapshot to the current row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SseDecision {
    pub state_changed: bool,
    pub ci_changed: bool,
    pub terminal: bool,
}

/// Pure decision used by the SSE loop. An unparseable state is treated as
/// terminal so a corrupt row cannot hold a connection open indefinitely.
pub(crate) fn diff_snapshot(last: &RunSnapshot, current: &RunSnapshot) -> SseDecision {
    let terminal = RunState::from_str(&current.state)
        .map(|s| s.is_terminal())
        .unwrap_or(true);
    SseDecision {
        state_changed: last.state != current.state,
        ci_changed: last.ci_status != current.ci_status,
        terminal,
    }
}

fn sse_event(name: &str, payload: serde_json::Value) -> Event {
    Event::default()
        .event(name)
        .data(serde_json::to_string(&payload).unwrap_or_default())
}

fn state_changed_event(m: &remediation_run::Model, previous_state: &str) -> Event {
    sse_event(
        "run_state_changed",
        serde_json::json!({
            "runId": m.id,
            "state": m.state,
            "previousState": previous_state,
            "ciStatus": m.ci_status,
            "ts": Utc::now().to_rfc3339(),
        }),
    )
}

fn ci_status_event(m: &remediation_run::Model) -> Event {
    sse_event(
        "ci_status_updated",
        serde_json::json!({ "runId": m.id, "ciStatus": m.ci_status }),
    )
}

fn run_finished_event(m: &remediation_run::Model) -> Event {
    sse_event(
        "run_finished",
        serde_json::json!({
            "runId": m.id,
            "outcome": m.state,
            "ts": Utc::now().to_rfc3339(),
        }),
    )
}

/// Authenticate an SSE request from either a `Bearer` access token (header) or a
/// short-lived `?token=` SSE token. Returns the authenticated user id.
fn authenticate_sse(
    state: &AppState,
    headers: &HeaderMap,
    token: Option<&str>,
) -> Result<Uuid, ApiError> {
    // Prefer a normal access token when present (tests / server-to-server).
    if let Some(bearer) = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        return state
            .auth_service
            .validate_access_token(bearer)
            .map(|c| c.sub)
            .map_err(|_| ApiError::unauthorized("Invalid or expired token"));
    }

    let token = token.ok_or_else(|| ApiError::unauthorized("Missing SSE token"))?;
    let claims = decode::<SseTokenClaims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| ApiError::unauthorized("Invalid or expired SSE token"))?
    .claims;

    if claims.scope != SSE_TOKEN_SCOPE {
        return Err(ApiError::unauthorized("Wrong token scope"));
    }
    Ok(claims.sub)
}

/// GET /api/remediation/runs/{id}/events — DB-polling SSE live progress (ADR-011).
pub async fn run_events(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
    Query(q): Query<EventsQuery>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let user_id = authenticate_sse(&state, &headers, q.token.as_deref())?;
    let run = load_authorized_run(&state, user_id, run_id).await?;

    let db = state.db.clone();
    let stream = async_stream::stream! {
        let mut last = RunSnapshot::from_model(&run);

        // Initial snapshot covers late-joiners (missed intermediates are OK).
        yield Ok(state_changed_event(&run, &run.state));
        if RunState::from_str(&run.state).map(|s| s.is_terminal()).unwrap_or(true) {
            yield Ok(run_finished_event(&run));
            return;
        }

        let started = std::time::Instant::now();
        loop {
            tokio::time::sleep(SSE_POLL_INTERVAL).await;
            if started.elapsed() >= SSE_MAX_DURATION {
                break;
            }

            let current = match remediation_run::Entity::find_by_id(run_id).one(&db).await {
                Ok(Some(m)) => m,
                // Row gone or DB error: end the stream cleanly.
                _ => break,
            };

            let snapshot = RunSnapshot::from_model(&current);
            let decision = diff_snapshot(&last, &snapshot);

            if decision.state_changed {
                yield Ok(state_changed_event(&current, &last.state));
            } else if decision.ci_changed {
                yield Ok(ci_status_event(&current));
            }
            last = snapshot;

            if decision.terminal {
                yield Ok(run_finished_event(&current));
                break;
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(SSE_KEEPALIVE_INTERVAL)))
}

/// POST /api/remediation/sse-token — mint a short-lived SSE token for the caller.
pub async fn create_sse_token(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<SseTokenResponse>>, ApiError> {
    let expires = Utc::now() + chrono::Duration::seconds(SSE_TOKEN_TTL_SECS);
    let claims = SseTokenClaims {
        sub: auth.user_id,
        exp: expires.timestamp(),
        scope: SSE_TOKEN_SCOPE.to_string(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::internal(format!("mint sse token: {e}")))?;

    Ok(Json(ApiResponse::success(SseTokenResponse {
        token,
        expires_at: expires.to_rfc3339(),
    })))
}

// ============================================================================
// POST /api/remediation/runs/{id}/approve  &  /cancel
// ============================================================================

/// Guarded compare-and-swap on the run's `state` column. Returns the number of
/// affected rows (0 = the run was not in `from`).
async fn cas_state(state: &AppState, run_id: Uuid, from: &str, to: &str) -> Result<u64, ApiError> {
    let res = remediation_run::Entity::update_many()
        .col_expr(remediation_run::Column::State, Expr::value(to))
        .col_expr(remediation_run::Column::UpdatedAt, Expr::value(Utc::now()))
        .filter(remediation_run::Column::Id.eq(run_id))
        .filter(remediation_run::Column::State.eq(from))
        .exec(&state.db)
        .await?;
    Ok(res.rows_affected)
}

/// POST /api/remediation/runs/{id}/approve — `awaiting_approval` → `merging`.
///
/// NOTE: the current state machine never produces `awaiting_approval`, so in the
/// shipped schema this path returns 409 unless a row is seeded into that state.
pub async fn approve_run(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(run_id): Path<Uuid>,
) -> Result<Json<ApiResponse<RunSummary>>, ApiError> {
    let _ = load_authorized_run(&state, auth.user_id, run_id).await?;

    let affected = cas_state(
        &state,
        run_id,
        AWAITING_APPROVAL,
        &RunState::Merging.to_string(),
    )
    .await?;
    if affected == 0 {
        return Err(ApiError::conflict("Run is not awaiting approval"));
    }

    let updated = load_authorized_run(&state, auth.user_id, run_id).await?;
    tracing::info!(user_id = %auth.user_id, run_id = %run_id, "Remediation run approved");
    Ok(Json(ApiResponse::success(RunSummary::from(updated))))
}

/// POST /api/remediation/runs/{id}/cancel — `<active>` → `cancelled`.
pub async fn cancel_run(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(run_id): Path<Uuid>,
) -> Result<Json<ApiResponse<RunSummary>>, ApiError> {
    let run = load_authorized_run(&state, auth.user_id, run_id).await?;

    let current = RunState::from_str(&run.state)
        .map_err(|_| ApiError::internal("invalid run state in database"))?;
    if current.is_terminal() {
        return Err(ApiError::conflict("Run is already in a terminal state"));
    }

    // CAS off the exact observed state to reject a concurrent transition.
    let affected = cas_state(&state, run_id, &run.state, &RunState::Cancelled.to_string()).await?;
    if affected == 0 {
        return Err(ApiError::conflict("Run changed state concurrently"));
    }

    let updated = load_authorized_run(&state, auth.user_id, run_id).await?;
    tracing::info!(user_id = %auth.user_id, run_id = %run_id, "Remediation run cancelled");
    Ok(Json(ApiResponse::success(RunSummary::from(updated))))
}

// ============================================================================
// POST /api/remediation/repositories/{repo_id}/run — manual trigger
// ============================================================================

/// POST /api/remediation/repositories/{repo_id}/run — create a `created` run for
/// the worker sweep to pick up. Rejects unowned repos (404) and repos with no
/// enabled policy (422).
pub async fn trigger_run(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(repo_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<RunSummary>>), ApiError> {
    // Ownership: caller must own the repository.
    let repo = repository::Entity::find_by_id(repo_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("Repository not found"))?;
    if repo.user_id != auth.user_id {
        return Err(ApiError::not_found("Repository not found"));
    }

    // Resolve the effective enabled policy (precedence + ADR-014 ceiling).
    let resolver = ampel_core::services::PolicyResolver::new(state.db.clone());
    let criteria = resolver
        .resolve(repo_id)
        .await?
        .ok_or_else(|| ApiError::unprocessable_entity("No enabled remediation policy for repo"))?;

    let id = Uuid::new_v4();
    let now = Utc::now();
    let model = remediation_run::ActiveModel {
        id: sea_orm::ActiveValue::Set(id),
        repository_id: sea_orm::ActiveValue::Set(repo_id),
        // The run row carries no FK; the canonical `create_run` uses the nil
        // sentinel (the resolver returns criteria, not the matched policy id).
        policy_id: sea_orm::ActiveValue::Set(Uuid::nil()),
        triggered_by: sea_orm::ActiveValue::Set("manual".to_string()),
        triggered_by_user_id: sea_orm::ActiveValue::Set(Some(auth.user_id)),
        state: sea_orm::ActiveValue::Set(RunState::Created.to_string()),
        autonomy_level: sea_orm::ActiveValue::Set(criteria.autonomy_level.to_string()),
        head_sha: sea_orm::ActiveValue::Set(None),
        // Worker's `selecting` phase populates the real selection.
        pr_selection_snapshot: sea_orm::ActiveValue::Set("[]".to_string()),
        consolidation_plan: sea_orm::ActiveValue::Set(None),
        consolidated_pr_number: sea_orm::ActiveValue::Set(None),
        merged: sea_orm::ActiveValue::Set(false),
        branch_name: sea_orm::ActiveValue::Set(format!("ampel/remediation/{id}")),
        ci_status: sea_orm::ActiveValue::Set("pending".to_string()),
        ci_logs_url: sea_orm::ActiveValue::Set(None),
        merge_strategy: sea_orm::ActiveValue::Set(None),
        attempts: sea_orm::ActiveValue::Set(0),
        error_message: sea_orm::ActiveValue::Set(None),
        error_class: sea_orm::ActiveValue::Set(None),
        started_at: sea_orm::ActiveValue::Set(now),
        completed_at: sea_orm::ActiveValue::Set(None),
        created_at: sea_orm::ActiveValue::Set(now),
        updated_at: sea_orm::ActiveValue::Set(now),
    };
    remediation_run::Entity::insert(model)
        .exec(&state.db)
        .await?;

    let created = load_authorized_run(&state, auth.user_id, id).await?;
    tracing::info!(user_id = %auth.user_id, repo_id = %repo_id, run_id = %id, "Remediation run manually triggered");
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(RunSummary::from(created))),
    ))
}

// ============================================================================
// Unit tests — pure SSE decision helper
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(state: &str, ci: &str) -> RunSnapshot {
        RunSnapshot {
            state: state.to_string(),
            ci_status: ci.to_string(),
        }
    }

    #[test]
    fn should_detect_no_change_when_identical() {
        let d = diff_snapshot(&snap("selecting", "pending"), &snap("selecting", "pending"));
        assert!(!d.state_changed);
        assert!(!d.ci_changed);
        assert!(!d.terminal);
    }

    #[test]
    fn should_detect_state_change() {
        let d = diff_snapshot(
            &snap("selecting", "pending"),
            &snap("consolidating", "pending"),
        );
        assert!(d.state_changed);
        assert!(!d.ci_changed);
        assert!(!d.terminal);
    }

    #[test]
    fn should_detect_ci_change_without_state_change() {
        let d = diff_snapshot(&snap("verifying", "pending"), &snap("verifying", "success"));
        assert!(!d.state_changed);
        assert!(d.ci_changed);
    }

    #[test]
    fn should_flag_terminal_for_completed_state() {
        let d = diff_snapshot(
            &snap("finalizing", "success"),
            &snap("completed", "success"),
        );
        assert!(d.state_changed);
        assert!(d.terminal);
    }

    #[test]
    fn should_treat_unknown_state_as_terminal() {
        let d = diff_snapshot(&snap("merging", "pending"), &snap("garbage", "pending"));
        assert!(d.terminal);
    }
}
