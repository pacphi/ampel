use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use crate::handlers::{
    accounts, analytics, auth, bot_rules, bulk_merge, dashboard, model_accounts, model_catalog,
    notifications, pr_filters, pull_requests, remediation, remediation_playbooks, remediation_runs,
    repositories, teams, user_preferences, user_settings,
};
use crate::{
    health_handler, metrics_handler,
    middleware::{locale_detection_middleware, track_metrics},
    readiness_handler, AppState,
};

pub fn create_router(state: AppState) -> Router {
    // Create the main application router with all routes
    let app = Router::new()
        // Observability routes (no auth required)
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .route("/metrics", get(metrics_handler))
        // Auth routes (public)
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/refresh", post(auth::refresh))
        // Auth routes (protected)
        .route("/api/auth/me", get(auth::me).put(auth::update_me))
        .route("/api/auth/logout", post(auth::logout))
        // Account management routes (PAT-based multi-account support)
        .route(
            "/api/accounts",
            get(accounts::list_accounts).post(accounts::add_account),
        )
        .route(
            "/api/accounts/{id}",
            get(accounts::get_account)
                .patch(accounts::update_account)
                .delete(accounts::delete_account),
        )
        .route(
            "/api/accounts/{id}/validate",
            post(accounts::validate_account),
        )
        .route(
            "/api/accounts/{id}/set-default",
            post(accounts::set_default_account),
        )
        // Repository routes
        .route("/api/repositories", get(repositories::list_repositories))
        .route("/api/repositories", post(repositories::add_repository))
        .route(
            "/api/repositories/discover",
            get(repositories::discover_repositories),
        )
        .route(
            "/api/repositories/refresh-all",
            post(repositories::refresh_all_repositories),
        )
        .route(
            "/api/repositories/refresh-status/{job_id}",
            get(repositories::get_refresh_status),
        )
        .route(
            "/api/repositories/{id}",
            get(repositories::get_repository)
                .patch(repositories::update_repository)
                .delete(repositories::remove_repository),
        )
        // Pull request routes
        .route("/api/pull-requests", get(pull_requests::list_pull_requests))
        .route(
            "/api/repositories/{repo_id}/pull-requests",
            get(pull_requests::list_repository_prs),
        )
        .route(
            "/api/repositories/{repo_id}/pull-requests/{pr_id}",
            get(pull_requests::get_pull_request),
        )
        .route(
            "/api/repositories/{repo_id}/pull-requests/{pr_id}/merge",
            post(pull_requests::merge_pull_request),
        )
        .route(
            "/api/repositories/{repo_id}/pull-requests/{pr_id}/refresh",
            post(pull_requests::refresh_pull_request),
        )
        // Dashboard routes
        .route("/api/dashboard/summary", get(dashboard::get_summary))
        .route("/api/dashboard/grid", get(dashboard::get_grid))
        // Team routes
        .route(
            "/api/teams",
            get(teams::list_teams).post(teams::create_team),
        )
        .route("/api/teams/{team_id}", get(teams::get_team))
        .route("/api/teams/{team_id}/members", post(teams::add_member))
        .route(
            "/api/teams/{team_id}/members/{user_id}",
            delete(teams::remove_member),
        )
        // Notification routes
        .route(
            "/api/notifications/preferences",
            get(notifications::get_preferences).put(notifications::update_preferences),
        )
        .route(
            "/api/notifications/test-slack",
            post(notifications::test_slack_webhook),
        )
        .route(
            "/api/notifications/test-email",
            post(notifications::test_email_smtp),
        )
        // User settings routes (behavior config)
        .route(
            "/api/settings/behavior",
            get(user_settings::get_settings).put(user_settings::update_settings),
        )
        // User preferences routes (language)
        .route(
            "/api/v1/user/preferences/language",
            get(user_preferences::get_language_preference)
                .put(user_preferences::update_language_preference),
        )
        // Bulk merge routes
        .route("/api/merge/bulk", post(bulk_merge::bulk_merge))
        .route("/api/merge/operations", get(bulk_merge::list_operations))
        .route(
            "/api/merge/operations/{operation_id}",
            get(bulk_merge::get_operation),
        )
        // Bot/Auto-merge routes
        .route(
            "/api/repositories/{repo_id}/auto-merge",
            get(bot_rules::get_auto_merge_rule)
                .put(bot_rules::upsert_auto_merge_rule)
                .delete(bot_rules::delete_auto_merge_rule),
        )
        // PR Filters routes (global user settings)
        .route(
            "/api/pr-filters",
            get(pr_filters::get_pr_filters).put(pr_filters::update_pr_filters),
        )
        .route("/api/pr-filters/reset", post(pr_filters::reset_pr_filters))
        // Remediation routes (Fleet PR Remediation — Phase 1)
        .route("/api/remediation/scopes", get(remediation::list_scopes))
        .route(
            "/api/remediation/policies",
            get(remediation::list_policies).post(remediation::create_policy),
        )
        .route(
            "/api/remediation/policies/{id}",
            get(remediation::get_policy)
                .patch(remediation::update_policy)
                .delete(remediation::delete_policy),
        )
        .route(
            "/api/remediation/policies/{id}/toggle",
            post(remediation::toggle_policy),
        )
        .route(
            "/api/remediation/repositories/{repo_id}/preview",
            post(remediation::preview_repository),
        )
        .route("/api/remediation/fleet", get(remediation::get_fleet))
        // Model provider accounts (Phase 4 — Agentic Remediation Tier)
        .route(
            "/api/model-accounts",
            get(model_accounts::list_model_accounts).post(model_accounts::create_model_account),
        )
        .route(
            "/api/model-accounts/{id}",
            get(model_accounts::get_model_account)
                .patch(model_accounts::update_model_account)
                .delete(model_accounts::delete_model_account),
        )
        .route(
            "/api/model-accounts/{id}/validate",
            post(model_accounts::validate_model_account),
        )
        // Model catalog + Ollama discovery/pull (Phase 5 — Agentic Remediation Tier)
        .route("/api/model-catalog", get(model_catalog::get_model_catalog))
        .route(
            "/api/model-catalog/ollama/tags",
            get(model_catalog::list_ollama_tags),
        )
        .route(
            "/api/model-catalog/ollama/pull",
            post(model_catalog::pull_ollama_model),
        )
        .route(
            "/api/model-catalog/ollama/pull/{id}/status",
            get(model_catalog::get_pull_status),
        )
        // Remediation playbooks (Phase 4 — ADR-006)
        .route(
            "/api/remediation/playbooks",
            get(remediation_playbooks::list_playbooks).post(remediation_playbooks::create_playbook),
        )
        // Built-in default (read-only, static segment before `{id}`).
        .route(
            "/api/remediation/playbooks/embedded",
            get(remediation_playbooks::get_embedded_playbook),
        )
        .route(
            "/api/remediation/playbooks/{id}",
            get(remediation_playbooks::get_playbook)
                .patch(remediation_playbooks::update_playbook)
                .delete(remediation_playbooks::delete_playbook),
        )
        .route(
            "/api/remediation/playbooks/{id}/preview",
            post(remediation_playbooks::preview_playbook),
        )
        // Remediation runs (Phase 3 — Observability & UX)
        .route("/api/remediation/runs", get(remediation_runs::list_runs))
        .route("/api/remediation/runs/{id}", get(remediation_runs::get_run))
        .route(
            "/api/remediation/runs/{id}/events",
            get(remediation_runs::run_events),
        )
        .route(
            "/api/remediation/runs/{id}/approve",
            post(remediation_runs::approve_run),
        )
        .route(
            "/api/remediation/runs/{id}/cancel",
            post(remediation_runs::cancel_run),
        )
        .route(
            "/api/remediation/sse-token",
            post(remediation_runs::create_sse_token),
        )
        .route(
            "/api/remediation/repositories/{repo_id}/run",
            post(remediation_runs::trigger_run),
        )
        // Analytics routes
        .route(
            "/api/analytics/summary",
            get(analytics::get_analytics_summary),
        )
        .route("/api/analytics/health", get(analytics::get_health_overview))
        .route(
            "/api/repositories/{repo_id}/health",
            get(analytics::get_repository_health),
        );

    // Apply middleware layers and state
    app.layer(middleware::from_fn(locale_detection_middleware))
        .layer(middleware::from_fn(track_metrics))
        .with_state(state)
}
