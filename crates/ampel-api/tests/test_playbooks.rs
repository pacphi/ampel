//! Integration tests for the Phase 0 remediation-playbook guidance APIs
//! (ADR-006): the read-only embedded-default endpoint and the schema-aware,
//! field-path validation on create/update.
//!
//! Postgres-gated (the full Migrator is not SQLite-compatible), mirroring
//! `test_model_catalog.rs`. Test names contain `playbooks` so the phase gate's
//! `cargo test -p ampel-api playbooks` selects them.

mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use common::{create_test_app, TestDb};
use serde_json::{json, Value};
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helpers (copied from the test_model_catalog.rs harness)
// ---------------------------------------------------------------------------

async fn register_and_login(app: &axum::Router, email: &str) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": email, "password": "SecurePassword123!", "displayName": "U" })
                .to_string(),
        ))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let json = parse_json(response).await;
    json["data"]["accessToken"].as_str().unwrap().to_string()
}

async fn parse_json(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

fn get(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

fn post(uri: &str, token: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// A minimal, well-formed playbook body with one bad field substituted in by the
/// caller. Used to prove the create handler returns a field-path `422`.
fn playbook_yaml(loop_iterations: &str, output_contract: &str) -> String {
    format!(
        r#"
version: 1
role: "You are a remediation engineer."
tasks:
  failed_ci:
    instructions: "Fix {{{{ repo_full_name }}}} on {{{{ base_branch }}}}."
loop:
{loop_iterations}  max_seconds: 900
  max_cost_usd: "2.00"
tools_policy:
  allowed: [read_file]
output_contract: {output_contract}
"#
    )
}

// ---------------------------------------------------------------------------
// Case 1 — the embedded-default endpoint returns the built-in playbook
// ---------------------------------------------------------------------------

#[tokio::test]
async fn playbooks_embedded_endpoint_returns_the_default() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "pb-embedded@example.com").await;

    let resp = app
        .clone()
        .oneshot(get("/api/remediation/playbooks/embedded", &token))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = parse_json(resp).await;
    let data = &body["data"];
    assert_eq!(data["source"].as_str(), Some("builtin"));
    assert_eq!(data["playbookId"].as_str(), Some("default"));
    // The returned content is the real embedded YAML — it must carry the schema's
    // hallmark keys so the editor can prefill from it.
    let content = data["content"].as_str().expect("content string");
    assert!(content.contains("role:"), "content missing role: {content}");
    assert!(
        content.contains("failed_ci"),
        "content missing failed_ci task: {content}"
    );
    assert!(
        content.contains("tools_policy"),
        "content missing tools_policy: {content}"
    );

    test_db.cleanup().await;
}

// ---------------------------------------------------------------------------
// Case 2 — create returns a field-path 422 for a missing required loop field
// ---------------------------------------------------------------------------

#[tokio::test]
async fn playbooks_create_returns_field_path_422_for_missing_loop_field() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "pb-missing-loop@example.com").await;

    // Omit loop.max_iterations entirely.
    let content = playbook_yaml("", "unified_diff");
    let resp = app
        .clone()
        .oneshot(post(
            "/api/remediation/playbooks",
            &token,
            json!({ "playbookId": "custom", "name": "Custom", "content": content }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = parse_json(resp).await;
    let err = body["error"].as_str().expect("error message");
    assert!(
        err.contains("loop.max_iterations"),
        "422 must name the offending field path, got: {err}"
    );

    test_db.cleanup().await;
}

// ---------------------------------------------------------------------------
// Case 3 — create returns a field-path 422 for an unknown output_contract
// ---------------------------------------------------------------------------

#[tokio::test]
async fn playbooks_create_returns_field_path_422_for_unknown_output_contract() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "pb-bad-contract@example.com").await;

    let content = playbook_yaml("  max_iterations: 4\n", "magic_contract");
    let resp = app
        .clone()
        .oneshot(post(
            "/api/remediation/playbooks",
            &token,
            json!({ "playbookId": "custom", "name": "Custom", "content": content }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = parse_json(resp).await;
    let err = body["error"].as_str().expect("error message");
    assert!(
        err.contains("output_contract"),
        "422 must name the output_contract field, got: {err}"
    );

    test_db.cleanup().await;
}

// ---------------------------------------------------------------------------
// Case 4 — a well-formed playbook still creates successfully (no regression)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn playbooks_create_accepts_a_well_formed_playbook() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "pb-valid@example.com").await;

    let content = playbook_yaml("  max_iterations: 4\n", "unified_diff");
    let resp = app
        .clone()
        .oneshot(post(
            "/api/remediation/playbooks",
            &token,
            json!({ "playbookId": "custom", "name": "Custom", "content": content }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    test_db.cleanup().await;
}

// ---------------------------------------------------------------------------
// Case 5 — preview renders the assembled prompt with NO model call
// ---------------------------------------------------------------------------

#[tokio::test]
async fn playbooks_preview_renders_prompt_without_a_model_call() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "pb-preview@example.com").await;

    // Create a valid playbook and capture its id.
    let content = playbook_yaml("  max_iterations: 4\n", "unified_diff");
    let create_resp = app
        .clone()
        .oneshot(post(
            "/api/remediation/playbooks",
            &token,
            json!({ "playbookId": "prev", "name": "Prev", "content": content }),
        ))
        .await
        .unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let created = parse_json(create_resp).await;
    let id = created["data"]["id"].as_str().expect("created playbook id");

    // Preview it. No model-provider account exists in this database, so a
    // successful render proves the endpoint makes NO model call — it only
    // assembles the trusted prompt with minijinja over trusted metadata.
    let resp = app
        .clone()
        .oneshot(post(
            &format!("/api/remediation/playbooks/{id}/preview"),
            &token,
            json!({ "repoFullName": "octo/ampel", "baseBranch": "main" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = parse_json(resp).await;
    let data = &body["data"];
    assert_eq!(
        data["role"].as_str(),
        Some("You are a remediation engineer.")
    );
    // The trusted metadata is rendered into the system instruction...
    let system = data["systemInstruction"]
        .as_str()
        .expect("systemInstruction");
    assert!(
        system.contains("octo/ampel"),
        "rendered prompt missing repo: {system}"
    );
    assert!(
        system.contains("main"),
        "rendered prompt missing branch: {system}"
    );
    // ...and the allow-list is clamped to the embedded ceiling (read_file survives).
    let tools: Vec<&str> = data["allowedTools"]
        .as_array()
        .expect("allowedTools array")
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    assert!(
        tools.contains(&"read_file"),
        "expected read_file in {tools:?}"
    );

    test_db.cleanup().await;
}
