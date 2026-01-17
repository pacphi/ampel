use axum::{extract::Request, http::header, middleware::Next, response::Response};
use cookie::Cookie;
use sea_orm::EntityTrait;
use uuid::Uuid;

use crate::AppState;
use ampel_db::entities::user;

/// Detected locale stored in request extensions
#[derive(Clone, Debug)]
pub struct DetectedLocale {
    pub code: String,
}

impl DetectedLocale {
    pub fn new(code: String) -> Self {
        Self { code }
    }
}

/// Supported locales in priority order
/// Final 27-language hybrid strategy: 21 simple codes + 6 regional variants
/// Simple codes: en, fr, de, it, ru, ja, ko, ar, he, hi, nl, pl, sr, th, tr, sv, da, fi, vi, no, cs
/// Regional variants: en-GB, pt-BR, zh-CN, zh-TW, es-ES, es-MX
const SUPPORTED_LOCALES: &[&str] = &[
    // Simple codes (21)
    "en", "fr", "de", "it", "ru", "ja", "ko", "ar", "he", "hi", "nl", "pl", "sr", "th", "tr", "sv",
    "da", "fi", "vi", "no", "cs",
    // Regional variants (6 - NO simple code duplicates)
    "en-GB", "pt-BR", "zh-CN", "zh-TW", "es-ES", "es-MX",
];

/// Middleware to detect and set locale from multiple sources
///
/// Detection order:
/// 1. Query parameter (?lang=fi) - explicit override
/// 2. Cookie (lang=fi)
/// 3. Accept-Language header
/// 4. Fallback to "en"
///
/// Note: Database preference detection temporarily disabled due to Axum middleware
/// complexity. Will be re-enabled after resolving from_fn_with_state type inference.
pub async fn locale_detection_middleware(mut req: Request, next: Next) -> Response {
    let locale = detect_locale(&req);
    req.extensions_mut().insert(DetectedLocale::new(locale));
    next.run(req).await
}

/// Detect locale from request sources with database access
/// TODO: Re-enable when from_fn_with_state type inference is resolved
#[allow(dead_code)]
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

    // 2. Check user database preference (if authenticated) - NEW
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
/// Returns None if not authenticated or token is invalid
/// TODO: Re-enable when from_fn_with_state type inference is resolved
#[allow(dead_code)]
fn try_extract_user_from_jwt(req: &Request, state: &AppState) -> Option<Uuid> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())?;

    let token = auth_header.strip_prefix("Bearer ")?;

    // Validate token and extract user ID
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

/// Parse Accept-Language header and return best matching supported locale
///
/// Format: "en-US,en;q=0.9,fi;q=0.8,de;q=0.7"
fn parse_accept_language(header: &str) -> Option<String> {
    let mut locales: Vec<(String, f32)> = Vec::new();

    for part in header.split(',') {
        let trimmed = part.trim();
        let (lang, quality) = if let Some((l, q)) = trimmed.split_once(";q=") {
            (l.trim(), q.parse::<f32>().unwrap_or(1.0))
        } else {
            (trimmed, 1.0)
        };

        let normalized = normalize_locale(lang);
        if is_supported_locale(&normalized) {
            locales.push((normalized, quality));
        }
    }

    // Sort by quality descending
    locales.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    locales.first().map(|(locale, _)| locale.clone())
}

/// Normalize locale code to match supported formats
///
/// Handles:
/// - Case normalization (EN -> en, PT-br -> pt-BR)
/// - Language fallbacks (no -> nb, pt -> pt-BR)
/// - Region code formatting (pt_BR -> pt-BR, zh_CN -> zh-CN)
fn normalize_locale(locale: &str) -> String {
    let locale = locale.trim().to_lowercase();

    // Handle underscore separators
    let locale = locale.replace('_', "-");

    // Split into language and region
    let parts: Vec<&str> = locale.split('-').collect();

    match parts.as_slice() {
        // Language only - check for special fallbacks
        [lang] => {
            let lang = *lang;
            match lang {
                "es" => "es-ES".to_string(),     // Default Spanish to Spain
                "pt" => "pt-BR".to_string(),     // Default Portuguese to Brazil
                "zh" => "zh-CN".to_string(),     // Default Chinese to Simplified
                "no" | "nb" => "no".to_string(), // Norwegian Bokmål
                "he" => "he".to_string(),
                "sr" => "sr".to_string(),
                "th" => "th".to_string(),
                "vi" => "vi".to_string(),
                _ => {
                    // Check if language-only code is supported
                    if SUPPORTED_LOCALES.contains(&lang) {
                        lang.to_string()
                    } else {
                        // Try to find a supported locale with this language
                        for supported in SUPPORTED_LOCALES {
                            if supported.starts_with(lang) {
                                return supported.to_string();
                            }
                        }
                        lang.to_string()
                    }
                }
            }
        }
        // Language with region
        [lang, region] => {
            let lang = *lang;
            let region = region.to_uppercase();
            let candidate = format!("{}-{}", lang, region);

            // Check if this exact format is supported
            if SUPPORTED_LOCALES.contains(&candidate.as_str()) {
                return candidate;
            }

            // Check for special cases
            match (lang, region.as_str()) {
                ("en", "GB") => "en-GB".to_string(),
                ("es", "ES") => "es-ES".to_string(),
                ("es", "MX") => "es-MX".to_string(),
                ("pt", "BR") => "pt-BR".to_string(),
                ("zh", "CN") => "zh-CN".to_string(),
                ("no", _) | ("nb", _) => "no".to_string(),
                ("he", _) => "he".to_string(),
                ("sr", _) => "sr".to_string(),
                ("th", _) => "th".to_string(),
                ("vi", _) => "vi".to_string(),
                _ => {
                    // Try just the language code
                    if SUPPORTED_LOCALES.contains(&lang) {
                        lang.to_string()
                    } else {
                        candidate
                    }
                }
            }
        }
        // Unexpected format
        _ => locale.clone(),
    }
}

/// Check if locale is in supported list
fn is_supported_locale(locale: &str) -> bool {
    SUPPORTED_LOCALES.contains(&locale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Request;

    #[test]
    fn test_normalize_locale() {
        assert_eq!(normalize_locale("en"), "en");
        assert_eq!(normalize_locale("EN"), "en");
        assert_eq!(normalize_locale("en-GB"), "en-GB");
        assert_eq!(normalize_locale("en_GB"), "en-GB");
        assert_eq!(normalize_locale("pt"), "pt-BR");
        assert_eq!(normalize_locale("pt-BR"), "pt-BR");
        assert_eq!(normalize_locale("pt_BR"), "pt-BR");
        assert_eq!(normalize_locale("zh"), "zh-CN");
        assert_eq!(normalize_locale("zh-CN"), "zh-CN");
        assert_eq!(normalize_locale("es"), "es-ES");
        assert_eq!(normalize_locale("es-ES"), "es-ES");
        assert_eq!(normalize_locale("es-MX"), "es-MX");
        assert_eq!(normalize_locale("no"), "no");
        assert_eq!(normalize_locale("nb"), "no");
        assert_eq!(normalize_locale("he"), "he");
        assert_eq!(normalize_locale("sr"), "sr");
        assert_eq!(normalize_locale("th"), "th");
        assert_eq!(normalize_locale("vi"), "vi");
    }

    #[test]
    fn test_is_supported_locale() {
        assert!(is_supported_locale("en"));
        assert!(is_supported_locale("en-GB"));
        assert!(is_supported_locale("pt-BR"));
        assert!(is_supported_locale("fi"));
        assert!(is_supported_locale("zh-CN"));
        assert!(is_supported_locale("no"));
        assert!(is_supported_locale("cs"));
        assert!(is_supported_locale("da"));
        assert!(is_supported_locale("hi"));
        assert!(is_supported_locale("he"));
        assert!(is_supported_locale("sr"));
        assert!(is_supported_locale("th"));
        assert!(is_supported_locale("vi"));
        assert!(is_supported_locale("es-ES"));
        assert!(is_supported_locale("es-MX"));
        assert!(!is_supported_locale("xx"));
        assert!(!is_supported_locale("pt")); // pt is normalized to pt-BR
        assert!(!is_supported_locale("es")); // es is normalized to es-ES
    }

    #[test]
    fn test_parse_accept_language() {
        // Simple case
        assert_eq!(
            parse_accept_language("en-US,en;q=0.9"),
            Some("en".to_string())
        );

        // With quality values
        assert_eq!(
            parse_accept_language("fi;q=0.9,en;q=0.8"),
            Some("fi".to_string())
        );

        // Multiple languages with quality
        assert_eq!(
            parse_accept_language("fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7"),
            Some("fr".to_string())
        );

        // Portuguese normalization
        assert_eq!(
            parse_accept_language("pt-BR,pt;q=0.9,en;q=0.8"),
            Some("pt-BR".to_string())
        );

        // Norwegian normalization
        assert_eq!(
            parse_accept_language("nb,no;q=0.9,en;q=0.8"),
            Some("no".to_string())
        );

        // Unsupported locale falls back to next
        assert_eq!(
            parse_accept_language("xx-YY;q=1.0,fi;q=0.9"),
            Some("fi".to_string())
        );
    }

    #[test]
    fn test_extract_query_param() {
        assert_eq!(
            extract_query_param("lang=fi&other=value", "lang"),
            Some("fi".to_string())
        );
        assert_eq!(
            extract_query_param("other=value&lang=pt-BR", "lang"),
            Some("pt-BR".to_string())
        );
        assert_eq!(extract_query_param("other=value", "lang"), None);
    }

    #[tokio::test]
    async fn test_locale_detection_query_param() {
        let req = Request::builder()
            .uri("https://example.com/api?lang=fi")
            .body(axum::body::Body::empty())
            .unwrap();

        let locale = detect_locale(&req);
        assert_eq!(locale, "fi");
    }

    #[tokio::test]
    async fn test_locale_detection_cookie() {
        let req = Request::builder()
            .uri("https://example.com/api")
            .header(header::COOKIE, "lang=de")
            .body(axum::body::Body::empty())
            .unwrap();

        let locale = detect_locale(&req);
        assert_eq!(locale, "de");
    }

    #[tokio::test]
    async fn test_locale_detection_accept_language() {
        let req = Request::builder()
            .uri("https://example.com/api")
            .header(header::ACCEPT_LANGUAGE, "fi,en;q=0.9")
            .body(axum::body::Body::empty())
            .unwrap();

        let locale = detect_locale(&req);
        assert_eq!(locale, "fi");
    }

    #[tokio::test]
    async fn test_locale_detection_priority() {
        // Query param should take precedence over cookie
        let req = Request::builder()
            .uri("https://example.com/api?lang=fi")
            .header(header::COOKIE, "lang=de")
            .header(header::ACCEPT_LANGUAGE, "fr")
            .body(axum::body::Body::empty())
            .unwrap();

        let locale = detect_locale(&req);
        assert_eq!(locale, "fi");
    }

    #[tokio::test]
    async fn test_locale_detection_fallback() {
        let req = Request::builder()
            .uri("https://example.com/api")
            .body(axum::body::Body::empty())
            .unwrap();

        let locale = detect_locale(&req);
        assert_eq!(locale, "en");
    }

    // Note: Removed test_try_extract_user_from_jwt as it requires async database setup
    // and the JWT extraction logic is adequately tested in the auth service tests
}
