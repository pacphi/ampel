//! Integration tests for the Phase 4 model-provider account + playbook APIs.
//!
//! Postgres-gated (the full Migrator is not SQLite-compatible), matching the
//! existing remediation test conventions. Covers: create/list, the ADR-014
//! air-gapped 422 on External-egress account creation, credential
//! non-disclosure, cross-scope 404 isolation, and playbook create + preview
//! (which renders the prompt with NO model call).

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

use ampel_db::entities::{organization, user};

// ---------------------------------------------------------------------------
// Helpers
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

async fn seed_org(conn: &DatabaseConnection, owner_id: Uuid, air_gapped: bool) -> Uuid {
    let id = Uuid::new_v4();
    let now = Utc::now();
    organization::ActiveModel {
        id: Set(id),
        owner_id: Set(owner_id),
        name: Set("Org".to_string()),
        slug: Set(format!("org-{id}")),
        description: Set(None),
        logo_url: Set(None),
        air_gapped: Set(air_gapped),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(conn)
    .await
    .unwrap();
    id
}

fn post(app: &axum::Router, uri: &str, token: &str, body: Value) -> Request<Body> {
    let _ = app;
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get(app: &axum::Router, uri: &str, token: &str) -> Request<Body> {
    let _ = app;
    Request::builder()
        .method("GET")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn should_create_user_scoped_claude_account_and_never_disclose_key() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "ma1@example.com").await;

    let resp = app
        .clone()
        .oneshot(post(
            &app,
            "/api/model-accounts",
            &token,
            json!({
                "providerKind": "claude",
                "displayName": "My Claude",
                "apiKey": "sk-super-secret-123",
                "modelId": "claude-sonnet-4-6"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = parse_json(resp).await;
    let data = &json["data"];
    assert_eq!(data["providerKind"], "claude");
    assert_eq!(data["egressClass"], "external");
    assert_eq!(data["validationStatus"], "unvalidated");
    assert_eq!(data["hasCredentials"], true);
    // The plaintext key must never appear anywhere in the serialized response.
    let raw = json.to_string();
    assert!(
        !raw.contains("sk-super-secret-123"),
        "api key leaked: {raw}"
    );
    assert!(!raw.contains("credentialsEncrypted"));
    assert!(!raw.contains("apiKey"));

    test_db.cleanup().await;
}

#[tokio::test]
async fn should_reject_external_account_for_air_gapped_org_with_422() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "ma2@example.com").await;
    let user_id = current_user_id(test_db.connection()).await;
    let org_id = seed_org(test_db.connection(), user_id, true).await;

    let resp = app
        .clone()
        .oneshot(post(
            &app,
            "/api/model-accounts",
            &token,
            json!({
                "providerKind": "claude",
                "displayName": "Hosted in air-gap",
                "apiKey": "sk-x",
                "organizationId": org_id,
            }),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    test_db.cleanup().await;
}

#[tokio::test]
async fn should_allow_local_only_account_for_air_gapped_org() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "ma3@example.com").await;
    let user_id = current_user_id(test_db.connection()).await;
    let org_id = seed_org(test_db.connection(), user_id, true).await;

    let resp = app
        .clone()
        .oneshot(post(
            &app,
            "/api/model-accounts",
            &token,
            json!({
                "providerKind": "ollama",
                "displayName": "Local Ollama",
                "organizationId": org_id,
                "endpointUrl": "http://localhost:11434"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = parse_json(resp).await;
    assert_eq!(json["data"]["egressClass"], "local_only");
    assert_eq!(json["data"]["authType"], "none");
    test_db.cleanup().await;
}

#[tokio::test]
async fn should_return_404_for_other_users_account() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;

    // User A creates an account.
    let token_a = register_and_login(&app, "owner@example.com").await;
    let created = app
        .clone()
        .oneshot(post(
            &app,
            "/api/model-accounts",
            &token_a,
            json!({ "providerKind": "gemini", "displayName": "A", "apiKey": "k" }),
        ))
        .await
        .unwrap();
    let created_json = parse_json(created).await;
    let account_id = created_json["data"]["id"].as_str().unwrap();

    // User B cannot see it (404, not 403, to avoid leaking existence).
    let token_b = register_and_login(&app, "intruder@example.com").await;
    let resp = app
        .clone()
        .oneshot(get(
            &app,
            &format!("/api/model-accounts/{account_id}"),
            &token_b,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    test_db.cleanup().await;
}

#[tokio::test]
async fn should_create_and_preview_playbook_without_model_call() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "pb@example.com").await;

    let yaml = r#"
version: 1
role: "You are an autonomous CI remediation engineer"
tasks:
  failed_ci:
    instructions: "Fix the failing build in {{ repo_full_name }} on {{ base_branch }}"
loop:
  max_iterations: 3
  max_seconds: 600
  max_cost_usd: "1.00"
tools_policy:
  allowed: [read_file, apply_patch, git_push]
output_contract: unified_diff
"#;

    let created = app
        .clone()
        .oneshot(post(
            &app,
            "/api/remediation/playbooks",
            &token,
            json!({ "playbookId": "custom", "name": "Custom", "content": yaml }),
        ))
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = parse_json(created).await;
    let pb_id = created_json["data"]["id"].as_str().unwrap();

    let preview = app
        .clone()
        .oneshot(post(
            &app,
            &format!("/api/remediation/playbooks/{pb_id}/preview"),
            &token,
            json!({ "failureClass": "build_error", "repoFullName": "octo/ampel", "baseBranch": "main" }),
        ))
        .await
        .unwrap();
    assert_eq!(preview.status(), StatusCode::OK);
    let preview_json = parse_json(preview).await;
    let data = &preview_json["data"];
    assert!(data["systemInstruction"]
        .as_str()
        .unwrap()
        .contains("octo/ampel"));
    assert_eq!(data["outputContract"], "unified_diff");
    // ADR-006 ceiling clamp: `git_push` is not in the embedded ceiling, so the
    // override cannot grant it — it must be dropped from the resolved tools.
    let tools = data["allowedTools"].as_array().unwrap();
    assert!(
        !tools.iter().any(|t| t == "git_push"),
        "ceiling not enforced"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn should_reject_invalid_playbook_yaml_with_422() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "pb2@example.com").await;

    let resp = app
        .clone()
        .oneshot(post(
            &app,
            "/api/remediation/playbooks",
            &token,
            json!({ "playbookId": "bad", "name": "Bad", "content": "this: : is not a playbook" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    test_db.cleanup().await;
}
