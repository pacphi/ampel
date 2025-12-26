use utoipa::OpenApi;

use crate::handlers::{pull_requests_diff, diff_types};
use crate::observability::{HealthChecks, HealthResponse, ReadinessChecks, ReadinessResponse};

/// OpenAPI documentation for Ampel API
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::observability::health_handler,
        crate::observability::readiness_handler,
        pull_requests_diff::get_pull_request_diff,
    ),
    components(
        schemas(
            HealthResponse,
            HealthChecks,
            ReadinessResponse,
            ReadinessChecks,
            diff_types::DiffResponse,
            diff_types::DiffFile,
            diff_types::FileStatus,
            diff_types::DiffError,
            diff_types::DiffApiResponse,
            diff_types::DiffErrorResponse,
            diff_types::DiffMetadata,
            diff_types::DiffQuery,
        )
    ),
    tags(
        (name = "Observability", description = "Health and metrics endpoints"),
        (name = "Pull Requests", description = "Pull request management and diff viewing"),
    ),
    info(
        title = "Ampel API",
        version = "1.0.0",
        description = "Unified PR management dashboard API for GitHub, GitLab, and Bitbucket",
        contact(
            name = "Ampel Team",
            email = "support@ampel.dev"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development"),
        (url = "https://api.ampel.dev", description = "Production")
    )
)]
pub struct ApiDoc;

/// Initialize Swagger UI
pub fn swagger_ui() -> axum::Router {
    use utoipa_swagger_ui::SwaggerUi;
    use axum::Router;

    Router::new().merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
}
