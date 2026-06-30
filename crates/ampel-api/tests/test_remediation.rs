//! Integration tests for the Fleet PR Remediation Phase 1 API (policy CRUD +
//! dry-run preview + fleet). Postgres-gated: these early-return on SQLite because
//! the full Migrator is not SQLite-compatible (matching existing conventions).

mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::Utc;
use common::{create_test_app, TestDb};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use ampel_db::entities::{organization, pull_request, remediation_policy, repository, user};

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
                "email": "remediation@example.com",
                "password": "SecurePassword123!",
                "displayName": "Remediation User"
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

/// Fetch the single registered user's id from the test database.
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

async fn seed_open_pr(conn: &DatabaseConnection, repo_id: Uuid, number: i32) {
    let now = Utc::now();
    pull_request::ActiveModel {
        id: Set(Uuid::new_v4()),
        repository_id: Set(repo_id),
        provider: Set("github".to_string()),
        provider_id: Set(format!("pr-{repo_id}-{number}")),
        number: Set(number),
        title: Set(format!("PR {number}")),
        description: Set(None),
        url: Set(format!("https://example.com/pr/{number}")),
        state: Set("open".to_string()),
        source_branch: Set(format!("feature/{number}")),
        target_branch: Set("main".to_string()),
        author: Set("dependabot[bot]".to_string()),
        author_avatar_url: Set(None),
        is_draft: Set(false),
        is_mergeable: Set(Some(true)),
        has_conflicts: Set(false),
        additions: Set(1),
        deletions: Set(0),
        changed_files: Set(1),
        commits_count: Set(1),
        comments_count: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
        merged_at: Set(None),
        closed_at: Set(None),
        last_synced_at: Set(now),
    }
    .insert(conn)
    .await
    .unwrap();
}

#[allow(clippy::too_many_arguments)]
async fn seed_policy(
    conn: &DatabaseConnection,
    scope_type: &str,
    scope_id: Uuid,
    enabled: bool,
    min_open_prs: i32,
    autonomy_level: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    let now = Utc::now();
    remediation_policy::ActiveModel {
        id: Set(id),
        scope_type: Set(scope_type.to_string()),
        scope_id: Set(scope_id),
        enabled: Set(enabled),
        min_open_prs: Set(min_open_prs),
        pr_selection: Set("\"all_open\"".to_string()),
        autonomy_level: Set(autonomy_level.to_string()),
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
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(conn)
    .await
    .unwrap();
    id
}

async fn seed_air_gapped_org(conn: &DatabaseConnection, owner_id: Uuid) {
    let now = Utc::now();
    organization::ActiveModel {
        id: Set(Uuid::new_v4()),
        owner_id: Set(owner_id),
        name: Set("AirGapped Org".to_string()),
        slug: Set(format!("org-{}", Uuid::new_v4().simple())),
        description: Set(None),
        logo_url: Set(None),
        air_gapped: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(conn)
    .await
    .unwrap();
}

fn auth_post(uri: &str, token: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(body.to_string()))
        .unwrap()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_policy_crud_lifecycle() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(test_db.connection()).await;

    // Create (user-scoped, so the caller is authorized for their own scope).
    let create_body = json!({
        "scopeType": "user",
        "scopeId": user_id,
        "minOpenPrs": 2,
        "autonomyLevel": "dry_run_only",
        "remediationTier": "consolidate_only",
        "maxPrsPerRun": 5,
        "prSelection": "all_open"
    });
    let resp = app
        .clone()
        .oneshot(auth_post("/api/remediation/policies", &token, create_body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = parse_json(resp).await;
    assert_eq!(created["data"]["enabled"], true);
    let policy_id = created["data"]["id"].as_str().unwrap().to_string();

    // Get.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/remediation/policies/{policy_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let got = parse_json(resp).await;
    assert_eq!(got["data"]["minOpenPrs"], 2);

    // List.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/remediation/policies")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list = parse_json(resp).await;
    assert_eq!(list["data"].as_array().unwrap().len(), 1);

    // Patch.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/remediation/policies/{policy_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::from(json!({ "maxPrsPerRun": 9 }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let patched = parse_json(resp).await;
    assert_eq!(patched["data"]["maxPrsPerRun"], 9);

    // Toggle (true -> false).
    let resp = app
        .clone()
        .oneshot(auth_post(
            &format!("/api/remediation/policies/{policy_id}/toggle"),
            &token,
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let toggled = parse_json(resp).await;
    assert_eq!(toggled["data"]["enabled"], false);

    // Delete.
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/remediation/policies/{policy_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_create_policy_rejects_conflicting_invariant() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(test_db.connection()).await;

    let body = json!({
        "scopeType": "user",
        "scopeId": user_id,
        "minOpenPrs": 1,
        "autonomyLevel": "dry_run_only",
        "remediationTier": "consolidate_only",
        "maxPrsPerRun": 5,
        "autoMergeEnabled": true,
        "requireHumanApproval": true
    });
    let resp = app
        .oneshot(auth_post("/api/remediation/policies", &token, body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_create_fully_autonomous_requires_explicit_tier() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(test_db.connection()).await;

    let body = json!({
        "scopeType": "user",
        "scopeId": user_id,
        "minOpenPrs": 1,
        "autonomyLevel": "fully_autonomous",
        "maxPrsPerRun": 5
    });
    let resp = app
        .oneshot(auth_post("/api/remediation/policies", &token, body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_preview_returns_plan_and_performs_no_writes() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo_id = seed_repository(&conn, user_id).await;
    seed_open_pr(&conn, repo_id, 1).await;
    seed_open_pr(&conn, repo_id, 2).await;
    seed_policy(&conn, "repository", repo_id, true, 1, "dry_run_only").await;

    let pr_count_before = pull_request::Entity::find()
        .filter(pull_request::Column::RepositoryId.eq(repo_id))
        .all(&conn)
        .await
        .unwrap()
        .len();

    let resp = app
        .oneshot(auth_post(
            &format!("/api/remediation/repositories/{repo_id}/preview"),
            &token,
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let plan = parse_json(resp).await;
    // ConsolidationPlan comes from ampel-core and serializes snake_case.
    assert_eq!(plan["data"]["pr_count"], 2);

    // Read-only: PR rows unchanged after preview.
    let pr_count_after = pull_request::Entity::find()
        .filter(pull_request::Column::RepositoryId.eq(repo_id))
        .all(&conn)
        .await
        .unwrap()
        .len();
    assert_eq!(pr_count_before, pr_count_after);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_fleet_reflects_threshold_and_air_gap() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let conn = test_db.connection().clone();
    let app = create_test_app(conn.clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(&conn).await;

    let repo_id = seed_repository(&conn, user_id).await;
    seed_open_pr(&conn, repo_id, 1).await; // 1 open PR
    seed_air_gapped_org(&conn, user_id).await;
    // Threshold of 2 with only 1 open PR -> not eligible.
    seed_policy(&conn, "repository", repo_id, true, 2, "dry_run_only").await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/remediation/fleet")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let fleet = parse_json(resp).await;
    let rows = fleet["data"].as_array().unwrap();
    let row = rows
        .iter()
        .find(|r| r["repositoryId"] == json!(repo_id.to_string()))
        .expect("fleet row for seeded repo");

    assert_eq!(row["openPrCount"], 1);
    assert_eq!(row["eligible"], false); // 1 < min_open_prs(2)
    assert_eq!(row["airGapped"], true); // ADR-014 org ceiling

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_list_policies_requires_authentication() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/remediation/policies")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_list_scopes_returns_caller_scopes_with_labels() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;
    let user_id = current_user_id(test_db.connection()).await;
    let repo_id = seed_repository(test_db.connection(), user_id).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/remediation/scopes")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = parse_json(resp).await;

    // User scope is the caller themselves, labelled by display name.
    assert_eq!(body["data"]["user"]["id"], user_id.to_string());
    assert_eq!(body["data"]["user"]["label"], "Remediation User");

    // The seeded repository is offered, labelled by its full name.
    let repos = body["data"]["repositories"].as_array().unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0]["id"], repo_id.to_string());
    assert_eq!(repos[0]["label"], "octocat/repo");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_create_policy_rejects_non_uuid_scope_id() {
    if TestDb::skip_if_sqlite() {
        return;
    }
    let test_db = TestDb::new().await.expect("create test DB");
    test_db.run_migrations().await.expect("run migrations");
    let app = create_test_app(test_db.connection().clone()).await;
    let token = register_and_login(&app).await;

    // Regression: a non-UUID scope id (e.g. a typed username) must be rejected
    // by the JSON extractor, never silently accepted. This is the original
    // "Failed to save the policy" failure mode.
    let body = json!({
        "scopeType": "user",
        "scopeId": "pacphi",
        "minOpenPrs": 2,
        "autonomyLevel": "dry_run_only",
        "remediationTier": "consolidate_only",
        "maxPrsPerRun": 5,
        "prSelection": "all_open"
    });
    let resp = app
        .clone()
        .oneshot(auth_post("/api/remediation/policies", &token, body))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    test_db.cleanup().await;
}
