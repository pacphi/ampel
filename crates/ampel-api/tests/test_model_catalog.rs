//! Integration tests for the Phase 5 model-catalog + Ollama discovery APIs.
//!
//! Postgres-gated (the full Migrator is not SQLite-compatible), mirroring
//! `test_model_accounts.rs` exactly. Covers: the ADR-014 air-gap catalog filter
//! (external providers hidden for an owned air-gapped org), the shared SSRF
//! guard rejecting an internal endpoint end-to-end on `/ollama/tags`, and the
//! ADR-008 spend-cap round-trip for a catalog-selected account.

mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::Utc;
use common::{create_test_app, TestDb};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use ampel_db::entities::{model_provider_account, organization, user};

// ---------------------------------------------------------------------------
// Helpers (copied from test_model_accounts.rs harness)
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
    use sea_orm::EntityTrait;
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

/// Insert a model-provider account directly, bypassing the create-handler SSRF
/// guard. Needed to plant an intentionally-misconfigured (external-egress,
/// internal-endpoint) Ollama account so the READ path's guard can be exercised.
async fn seed_ollama_account(
    conn: &DatabaseConnection,
    user_id: Uuid,
    egress_class: &str,
    endpoint_url: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    let now = Utc::now();
    model_provider_account::ActiveModel {
        id: Set(id),
        organization_id: Set(None),
        user_id: Set(Some(user_id)),
        provider_kind: Set("ollama".to_string()),
        display_name: Set("Seeded Ollama".to_string()),
        credentials_encrypted: Set(None),
        endpoint_url: Set(Some(endpoint_url.to_string())),
        egress_class: Set(egress_class.to_string()),
        model_id: Set(None),
        enabled: Set(true),
        auth_type: Set("none".to_string()),
        spend_cap_usd: Set(None),
        spend_used_usd: Set("0".to_string()),
        validation_status: Set("unvalidated".to_string()),
        last_validated_at: Set(None),
        model_path: Set(None),
        is_default: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(conn)
    .await
    .unwrap();
    id
}

fn post(_app: &axum::Router, uri: &str, token: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get(_app: &axum::Router, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

/// Collect the provider `kind` values from a catalog response body.
fn provider_kinds(body: &Value) -> Vec<String> {
    body["data"]["providers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["kind"].as_str().unwrap().to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Case 1 — air-gapped org hides external providers via the catalog (ADR-014)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn should_hide_external_providers_for_air_gapped_org_via_catalog() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "cat-airgap@example.com").await;
    let user_id = current_user_id(test_db.connection()).await;

    let air_gapped_org = seed_org(test_db.connection(), user_id, true).await;
    let open_org = seed_org(test_db.connection(), user_id, false).await;

    // Air-gapped: external providers (claude/gemini) omitted, ollama retained.
    let resp = app
        .clone()
        .oneshot(get(
            &app,
            &format!("/api/model-catalog?organizationId={air_gapped_org}"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let kinds = provider_kinds(&parse_json(resp).await);
    assert!(
        !kinds.contains(&"claude".to_string()),
        "claude must be hidden for an air-gapped org, got {kinds:?}"
    );
    assert!(
        !kinds.contains(&"gemini".to_string()),
        "gemini must be hidden for an air-gapped org, got {kinds:?}"
    );
    assert!(
        kinds.contains(&"ollama".to_string()),
        "ollama must remain for an air-gapped org, got {kinds:?}"
    );

    // Non-air-gapped org: external providers are present.
    let resp = app
        .clone()
        .oneshot(get(
            &app,
            &format!("/api/model-catalog?organizationId={open_org}"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let kinds = provider_kinds(&parse_json(resp).await);
    assert!(
        kinds.contains(&"claude".to_string()),
        "claude must be present for a non-air-gapped org, got {kinds:?}"
    );
    assert!(
        kinds.contains(&"gemini".to_string()),
        "gemini must be present for a non-air-gapped org, got {kinds:?}"
    );

    test_db.cleanup().await;
}

// ---------------------------------------------------------------------------
// Case 2 — SSRF guard blocks an internal endpoint end-to-end on /ollama/tags
// ---------------------------------------------------------------------------

#[tokio::test]
async fn should_reject_ollama_tags_for_external_account_pointing_at_metadata_ip() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "cat-ssrf@example.com").await;
    let user_id = current_user_id(test_db.connection()).await;

    // Seeded directly: an external-egress account pointing at the cloud-metadata
    // IP. The create handler would block this, so we plant it to prove the READ
    // path's shared SSRF guard rejects it before any network call (HTTP 422).
    let account_id = seed_ollama_account(
        test_db.connection(),
        user_id,
        "external",
        "http://169.254.169.254/",
    )
    .await;

    let resp = app
        .clone()
        .oneshot(get(
            &app,
            &format!("/api/model-catalog/ollama/tags?accountId={account_id}"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "external-egress metadata IP must be rejected by the shared SSRF guard"
    );

    test_db.cleanup().await;
}

#[tokio::test]
async fn should_pass_ssrf_guard_for_local_only_ollama_then_fail_upstream() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "cat-local@example.com").await;
    let user_id = current_user_id(test_db.connection()).await;

    // Local-only egress at localhost is the legitimate Ollama target: the guard
    // must let it past. With no server listening the proxy then fails to CONNECT,
    // which surfaces as a 502 (bad gateway) — the "guard passed, upstream
    // unreachable" outcome. The key assertion is that it is NOT a 422.
    let account_id = seed_ollama_account(
        test_db.connection(),
        user_id,
        "local_only",
        DEFAULT_ENDPOINT,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(get(
            &app,
            &format!("/api/model-catalog/ollama/tags?accountId={account_id}"),
            &token,
        ))
        .await
        .unwrap();
    assert_ne!(
        resp.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "local-only localhost must pass the SSRF guard (not a 422)"
    );
    assert_eq!(
        resp.status(),
        StatusCode::BAD_GATEWAY,
        "guard passed but the (absent) upstream is unreachable → 502"
    );

    test_db.cleanup().await;
}

/// A local Ollama endpoint with (almost certainly) nothing listening.
const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:1";

// ---------------------------------------------------------------------------
// Case 3 — spend-cap persists for catalog-selected accounts (ADR-008)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn should_persist_spend_cap_for_catalog_selected_account() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app, "cat-spend@example.com").await;

    // Create an account carrying both a catalog model id and a spend cap.
    let created = app
        .clone()
        .oneshot(post(
            &app,
            "/api/model-accounts",
            &token,
            json!({
                "providerKind": "claude",
                "displayName": "Capped Claude",
                "apiKey": "sk-x",
                "modelId": "claude-sonnet-4-6",
                "spendCapUsd": "12.50"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = parse_json(created).await;
    assert_eq!(created_json["data"]["modelId"], "claude-sonnet-4-6");
    assert_eq!(created_json["data"]["spendCapUsd"], "12.50");
    assert_eq!(created_json["data"]["spendUsedUsd"], "0");
    let account_id = created_json["data"]["id"].as_str().unwrap().to_string();

    // Round-trip via GET: the catalog/model-id path does not bypass spend fields.
    let resp = app
        .clone()
        .oneshot(get(
            &app,
            &format!("/api/model-accounts/{account_id}"),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = parse_json(resp).await;
    assert_eq!(json["data"]["modelId"], "claude-sonnet-4-6");
    assert_eq!(
        json["data"]["spendCapUsd"], "12.50",
        "spend cap must persist intact"
    );
    assert_eq!(
        json["data"]["spendUsedUsd"], "0",
        "spend used must round-trip intact"
    );

    test_db.cleanup().await;
}
