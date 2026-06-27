//! Integration tests for Fleet PR Remediation Phase 3 (Observability & UX):
//! run history/detail, SSE live progress, approve/cancel, and manual trigger.
//! Postgres-gated (early-return on SQLite, matching the Phase-1 convention).

mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::Utc;
use common::{create_test_app, TestDb};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use ampel_db::entities::{
    remediation_policy, remediation_run, remediation_run_pr, repository, user,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn register_and_login(app: &axum::Router) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "runs@example.com",
                "password": "SecurePassword123!",
                "displayName": "Runs User"
            })
            .to_string(),
        ))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    json["data"]["accessToken"].as_str().unwrap().to_string()
}

async fn current_user_id(conn: &DatabaseConnection) -> Uuid {
    user::Entity::find()
        .one(conn)
        .await
        .unwrap()
        .expect("a registered user")
        .id
}

async fn parse_json(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

async fn seed_user(conn: &DatabaseConnection) -> Uuid {
    let id = Uuid::new_v4();
    let now = Utc::now();
    user::ActiveModel {
        id: Set(id),
        email: Set(format!("other-{}@example.com", id.simple())),
        password_hash: Set("x".to_string()),
        display_name: Set(Some("Other".to_string())),
        avatar_url: Set(None),
        language: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(conn)
    .await
    .unwrap();
    id
}

async fn seed_repository(conn: &DatabaseConnection, user_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let now = Utc::now();
    repository::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        provider: Set("github".to_string()),
        provider_id: Set(format!("p-{id}")),
        owner: Set("octocat".to_string()),
        name: Set("repo".to_string()),
        full_name: Set("octocat/repo".to_string()),
        description: Set(None),
        url: Set("https://example.com/octocat/repo".to_string()),
        default_branch: Set("main".to_string()),
        is_private: Set(false),
        is_archived: Set(false),
        poll_interval_seconds: Set(300),
        last_polled_at: Set(None),
        group_id: Set(None),
        provider_account_id: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(conn)
    .await
    .unwrap();
    id
}

#[allow(clippy::too_many_arguments)]
async fn seed_run(conn: &DatabaseConnection, repo_id: Uuid, state: &str, ci_status: &str) -> Uuid {
    let id = Uuid::new_v4();
    let now = Utc::now();
    remediation_run::ActiveModel {
        id: Set(id),
        repository_id: Set(repo_id),
        policy_id: Set(Uuid::nil()),
        triggered_by: Set("system".to_string()),
        triggered_by_user_id: Set(None),
        state: Set(state.to_string()),
        autonomy_level: Set("dry_run_only".to_string()),
        head_sha: Set(None),
        pr_selection_snapshot: Set("[]".to_string()),
        consolidation_plan: Set(None),
        consolidated_pr_number: Set(None),
        merged: Set(false),
        branch_name: Set(format!("ampel/remediation/{id}")),
        ci_status: Set(ci_status.to_string()),
        ci_logs_url: Set(None),
        merge_strategy: Set(None),
        attempts: Set(0),
        error_message: Set(None),
        error_class: Set(None),
        started_at: Set(now),
        completed_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(conn)
    .await
    .unwrap();
    id
}

async fn seed_disposition(conn: &DatabaseConnection, run_id: Uuid, pr_number: i64, json: &str) {
    remediation_run_pr::ActiveModel {
        id: Set(Uuid::new_v4()),
        remediation_run_id: Set(run_id),
        pr_number: Set(pr_number),
        disposition: Set(json.to_string()),
        created_at: Set(Utc::now()),
    }
    .insert(conn)
    .await
    .unwrap();
}

async fn seed_policy(conn: &DatabaseConnection, scope_id: Uuid, enabled: bool) {
    remediation_policy::ActiveModel {
        id: Set(Uuid::new_v4()),
        scope_type: Set("repository".to_string()),
        scope_id: Set(scope_id),
        enabled: Set(enabled),
        min_open_prs: Set(1),
        pr_selection: Set("\"all_open\"".to_string()),
        autonomy_level: Set("dry_run_only".to_string()),
        remediation_tier: Set("consolidate_only".to_string()),
        max_prs_per_run: Set(10),
        allowed_targets: Set("[\"main\"]".to_string()),
        skip_draft: Set(false),
        require_green_before_merge: Set(true),
        air_gapped: Set(false),
        auto_merge_enabled: Set(false),
        auto_merge_rule: Set(None),
        require_human_approval: Set(false),
        agent_budget: Set(None),
        notification_config: Set(None),
        playbook_ref: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(conn)
    .await
    .unwrap();
}

fn get(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

fn post(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_runs_scoped_with_filters() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo_a = seed_repository(&conn, user_id).await;
    let repo_b = seed_repository(&conn, user_id).await;
    seed_run(&conn, repo_a, "selecting", "pending").await;
    seed_run(&conn, repo_a, "completed", "success").await;
    seed_run(&conn, repo_b, "failed", "failed").await;

    // All scoped runs.
    let resp = app
        .clone()
        .oneshot(get("/api/remediation/runs", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let all = parse_json(resp).await;
    assert_eq!(all["data"].as_array().unwrap().len(), 3);

    // Filter by state.
    let resp = app
        .clone()
        .oneshot(get("/api/remediation/runs?state=completed", &token))
        .await
        .unwrap();
    let completed = parse_json(resp).await;
    assert_eq!(completed["data"].as_array().unwrap().len(), 1);
    assert_eq!(completed["data"][0]["state"], "completed");

    // Filter by repository_id.
    let resp = app
        .clone()
        .oneshot(get(
            &format!("/api/remediation/runs?repositoryId={repo_b}"),
            &token,
        ))
        .await
        .unwrap();
    let by_repo = parse_json(resp).await;
    assert_eq!(by_repo["data"].as_array().unwrap().len(), 1);
    assert_eq!(
        by_repo["data"][0]["repositoryId"],
        json!(repo_b.to_string())
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_run_returns_dispositions_and_conflict_report() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo = seed_repository(&conn, user_id).await;
    let run = seed_run(&conn, repo, "completed", "success").await;
    seed_disposition(&conn, run, 1, r#"{"disposition":"consolidated"}"#).await;
    seed_disposition(
        &conn,
        run,
        2,
        r#"{"disposition":"skipped_conflict","reason":"merge conflict in Cargo.lock"}"#,
    )
    .await;
    seed_disposition(
        &conn,
        run,
        3,
        r#"{"disposition":"left_open","reason":"draft"}"#,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(get(&format!("/api/remediation/runs/{run}"), &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let detail = parse_json(resp).await;

    assert_eq!(detail["data"]["prs"].as_array().unwrap().len(), 3);
    assert_eq!(detail["data"]["ciMatrix"]["status"], "success");
    assert_eq!(
        detail["data"]["conflictReport"]["conflicts"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        detail["data"]["conflictReport"]["conflicts"][0]["prNumber"],
        2
    );
    assert_eq!(
        detail["data"]["conflictReport"]["skipped"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_run_cross_scope_returns_404() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;

    // A run owned by a different (real) user.
    let other = seed_user(&conn).await;
    let other_repo = seed_repository(&conn, other).await;
    let run = seed_run(&conn, other_repo, "selecting", "pending").await;

    let resp = app
        .clone()
        .oneshot(get(&format!("/api/remediation/runs/{run}"), &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_cancel_active_run_transitions_to_cancelled() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo = seed_repository(&conn, user_id).await;
    let run = seed_run(&conn, repo, "selecting", "pending").await;

    let resp = app
        .clone()
        .oneshot(post(&format!("/api/remediation/runs/{run}/cancel"), &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = parse_json(resp).await;
    assert_eq!(body["data"]["state"], "cancelled");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_cancel_terminal_run_is_rejected() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo = seed_repository(&conn, user_id).await;
    let run = seed_run(&conn, repo, "completed", "success").await;

    let resp = app
        .clone()
        .oneshot(post(&format!("/api/remediation/runs/{run}/cancel"), &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_approve_run_not_awaiting_is_rejected() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo = seed_repository(&conn, user_id).await;
    let run = seed_run(&conn, repo, "selecting", "pending").await;

    let resp = app
        .clone()
        .oneshot(post(
            &format!("/api/remediation/runs/{run}/approve"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_approve_run_awaiting_approval_transitions_to_merging() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo = seed_repository(&conn, user_id).await;
    // Seed the (currently synthetic) gate state directly.
    let run = seed_run(&conn, repo, "awaiting_approval", "success").await;

    let resp = app
        .clone()
        .oneshot(post(
            &format!("/api/remediation/runs/{run}/approve"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = parse_json(resp).await;
    assert_eq!(body["data"]["state"], "merging");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_manual_trigger_creates_created_run() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo = seed_repository(&conn, user_id).await;
    seed_policy(&conn, repo, true).await;

    let resp = app
        .clone()
        .oneshot(post(
            &format!("/api/remediation/repositories/{repo}/run"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = parse_json(resp).await;
    assert_eq!(body["data"]["state"], "created");
    assert_eq!(body["data"]["triggeredBy"], "manual");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_manual_trigger_without_policy_returns_422() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo = seed_repository(&conn, user_id).await;
    // No enabled policy at any scope.

    let resp = app
        .clone()
        .oneshot(post(
            &format!("/api/remediation/repositories/{repo}/run"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_sse_events_responds_with_event_stream() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo = seed_repository(&conn, user_id).await;
    // Terminal run so the stream finishes promptly without blocking the test.
    let run = seed_run(&conn, repo, "completed", "success").await;

    let resp = app
        .clone()
        .oneshot(get(&format!("/api/remediation/runs/{run}/events"), &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(content_type.starts_with("text/event-stream"));

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_sse_token_then_events_authenticates() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo = seed_repository(&conn, user_id).await;
    let run = seed_run(&conn, repo, "completed", "success").await;

    // Mint a short-lived SSE token.
    let resp = app
        .clone()
        .oneshot(post("/api/remediation/sse-token", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = parse_json(resp).await;
    let sse_token = body["data"]["token"].as_str().unwrap().to_string();

    // Connect with ?token= (no Authorization header).
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/remediation/runs/{run}/events?token={sse_token}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_runs_require_authentication() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/remediation/runs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}
