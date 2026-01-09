use std::task::{Context, Poll};

use axum::{
    extract::Request,
    http::header,
    response::Response,
};
use cookie::Cookie;
use futures::future::BoxFuture;
use sea_orm::EntityTrait;
use tower::{Layer, Service};
use uuid::Uuid;

use crate::AppState;
use ampel_db::entities::user;

use super::locale::{normalize_locale, parse_accept_language, DetectedLocale, SUPPORTED_LOCALES};

/// Tower Layer for locale detection middleware
#[derive(Clone)]
pub struct LocaleDetectionLayer;

impl LocaleDetectionLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for LocaleDetectionLayer {
    type Service = LocaleDetectionMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LocaleDetectionMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct LocaleDetectionMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for LocaleDetectionMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            // Extract AppState from request extensions (injected by Axum router)
            let locale = if let Some(state) = req.extensions().get::<AppState>() {
                detect_locale_with_state(&req, state).await
            } else {
                // Fallback if state isn't available
                detect_locale(&req)
            };

            let mut req = req;
            req.extensions_mut().insert(DetectedLocale::new(locale));

            inner.call(req).await
        })
    }
}

/// Detect locale from request sources with database access
async fn detect_locale_with_state(req: &Request, state: &AppState) -> String {
    // 1. Check query parameter (?lang=fi) - explicit override
    if let Some(query) = req.uri().query() {
        if let Some(lang) = extract_query_param(query, "lang") {
            let normalized = normalize_locale(&lang);
            if is_supported_locale(&normalized) {
                return normalized;
            }
        }
    }

    // 2. Check user database preference (if authenticated)
    if let Some(user_id) = try_extract_user_from_jwt(req, state) {
        if let Ok(Some(user)) = user::Entity::find_by_id(user_id).one(&state.db).await {
            if let Some(lang) = user.language {
                let normalized = normalize_locale(&lang);
                if is_supported_locale(&normalized) {
                    return normalized;
                }
            }
        }
    }

    // 3. Check cookie (lang=fi)
    if let Some(cookie_header) = req.headers().get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie_pair in cookie_str.split(';') {
                if let Ok(cookie) = Cookie::parse(cookie_pair.trim()) {
                    if cookie.name() == "lang" {
                        let normalized = normalize_locale(cookie.value());
                        if is_supported_locale(&normalized) {
                            return normalized;
                        }
                    }
                }
            }
        }
    }

    // 4. Check Accept-Language header
    if let Some(accept_lang) = req.headers().get(header::ACCEPT_LANGUAGE) {
        if let Ok(accept_str) = accept_lang.to_str() {
            if let Some(locale) = parse_accept_language(accept_str) {
                return locale;
            }
        }
    }

    // 5. Default fallback
    "en".to_string()
}

/// Detect locale from request sources (without database access - fallback)
fn detect_locale(req: &Request) -> String {
    // 1. Check query parameter (?lang=fi)
    if let Some(query) = req.uri().query() {
        if let Some(lang) = extract_query_param(query, "lang") {
            let normalized = normalize_locale(&lang);
            if is_supported_locale(&normalized) {
                return normalized;
            }
        }
    }

    // 2. Check cookie (lang=fi)
    if let Some(cookie_header) = req.headers().get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie_pair in cookie_str.split(';') {
                if let Ok(cookie) = Cookie::parse(cookie_pair.trim()) {
                    if cookie.name() == "lang" {
                        let normalized = normalize_locale(cookie.value());
                        if is_supported_locale(&normalized) {
                            return normalized;
                        }
                    }
                }
            }
        }
    }

    // 3. Check Accept-Language header
    if let Some(accept_lang) = req.headers().get(header::ACCEPT_LANGUAGE) {
        if let Ok(accept_str) = accept_lang.to_str() {
            if let Some(locale) = parse_accept_language(accept_str) {
                return locale;
            }
        }
    }

    // 4. Default fallback
    "en".to_string()
}

/// Try to extract user ID from JWT token in Authorization header
fn try_extract_user_from_jwt(req: &Request, state: &AppState) -> Option<Uuid> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())?;

    let token = auth_header.strip_prefix("Bearer ")?;
    let claims = state.auth_service.validate_access_token(token).ok()?;

    Some(claims.sub)
}

/// Extract query parameter value
fn extract_query_param(query: &str, param: &str) -> Option<String> {
    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == param {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Check if locale is in supported list
fn is_supported_locale(locale: &str) -> bool {
    SUPPORTED_LOCALES.contains(&locale)
}
