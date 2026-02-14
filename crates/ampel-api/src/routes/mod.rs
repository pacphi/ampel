use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use crate::handlers::{
    accounts, analytics, auth, bot_rules, bulk_merge, dashboard, notifications, pr_filters,
    pull_requests, repositories, teams, user_preferences, user_settings,
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
