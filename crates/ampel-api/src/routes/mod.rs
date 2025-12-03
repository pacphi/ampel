use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::handlers::{
    analytics, auth, bot_rules, connections, dashboard, notifications, pull_requests, repositories,
    social_auth, teams,
};
use crate::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Social auth routes (public - for logging into Ampel)
        .route("/api/auth/github/url", post(social_auth::github_auth_url))
        .route("/api/auth/github/callback", get(social_auth::github_auth_callback))
        .route("/api/auth/google/url", post(social_auth::google_auth_url))
        .route("/api/auth/google/callback", get(social_auth::google_auth_callback))
        // Auth routes (protected)
        .route("/api/auth/refresh", post(auth::refresh))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/logout", post(auth::logout))
        // PAT Connection routes
        .route(
            "/api/connections",
            get(connections::list_connections).post(connections::add_connection),
        )
        .route(
            "/api/connections/:id",
            get(connections::get_connection)
                .put(connections::update_connection)
                .delete(connections::delete_connection),
        )
        .route(
            "/api/connections/:id/validate",
            post(connections::validate_connection),
        )
        // Repository routes
        .route("/api/repositories", get(repositories::list_repositories))
        .route("/api/repositories", post(repositories::add_repository))
        .route(
            "/api/repositories/discover",
            get(repositories::discover_repositories),
        )
        .route(
            "/api/repositories/:id",
            get(repositories::get_repository)
                .put(repositories::update_repository)
                .delete(repositories::remove_repository),
        )
        // Pull request routes
        .route("/api/pull-requests", get(pull_requests::list_pull_requests))
        .route(
            "/api/repositories/:repo_id/pull-requests",
            get(pull_requests::list_repository_prs),
        )
        .route(
            "/api/repositories/:repo_id/pull-requests/:pr_id",
            get(pull_requests::get_pull_request),
        )
        .route(
            "/api/repositories/:repo_id/pull-requests/:pr_id/merge",
            post(pull_requests::merge_pull_request),
        )
        .route(
            "/api/repositories/:repo_id/pull-requests/:pr_id/refresh",
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
        .route("/api/teams/:team_id", get(teams::get_team))
        .route("/api/teams/:team_id/members", post(teams::add_member))
        .route(
            "/api/teams/:team_id/members/:user_id",
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
        // Bot/Auto-merge routes
        .route(
            "/api/repositories/:repo_id/auto-merge",
            get(bot_rules::get_auto_merge_rule)
                .put(bot_rules::upsert_auto_merge_rule)
                .delete(bot_rules::delete_auto_merge_rule),
        )
        // Analytics routes
        .route(
            "/api/analytics/summary",
            get(analytics::get_analytics_summary),
        )
        .route("/api/analytics/health", get(analytics::get_health_overview))
        .route(
            "/api/repositories/:repo_id/health",
            get(analytics::get_repository_health),
        )
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}
