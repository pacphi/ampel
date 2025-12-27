use axum::{extract::State, Json};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};

use ampel_db::entities::user;

use crate::extractors::AuthUser;
use crate::handlers::{ApiError, ApiResponse};
use crate::AppState;

/// List of supported language codes (20 languages)
const SUPPORTED_LANGUAGES: &[&str] = &[
    "en", // English
    "es", // Spanish
    "fr", // French
    "de", // German
    "it", // Italian
    "pt", // Portuguese
    "ru", // Russian
    "ja", // Japanese
    "zh", // Chinese
    "ko", // Korean
    "ar", // Arabic
    "hi", // Hindi
    "nl", // Dutch
    "pl", // Polish
    "tr", // Turkish
    "sv", // Swedish
    "da", // Danish
    "fi", // Finnish
    "no", // Norwegian
    "cs", // Czech
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageResponse {
    pub language: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLanguageRequest {
    pub language: String,
}

/// GET /api/v1/user/preferences/language
/// Get user's current language preference
pub async fn get_language_preference(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<ApiResponse<LanguageResponse>>, ApiError> {
    let user = user::Entity::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("User not found"))?;

    let language = user.language.unwrap_or_else(|| "en".to_string());

    Ok(Json(ApiResponse::success(LanguageResponse { language })))
}

/// PUT /api/v1/user/preferences/language
/// Update user's language preference
pub async fn update_language_preference(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateLanguageRequest>,
) -> Result<Json<ApiResponse<LanguageResponse>>, ApiError> {
    // Validate language code
    if !SUPPORTED_LANGUAGES.contains(&req.language.as_str()) {
        return Err(ApiError::bad_request(format!(
            "Invalid language code '{}'. Supported languages: {}",
            req.language,
            SUPPORTED_LANGUAGES.join(", ")
        )));
    }

    // Find the user
    let user = user::Entity::find_by_id(auth.user_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| ApiError::not_found("User not found"))?;

    // Update the language
    let mut user_active: user::ActiveModel = user.into();
    user_active.language = Set(Some(req.language.clone()));

    let updated_user = user_active.update(&state.db).await?;

    Ok(Json(ApiResponse::success(LanguageResponse {
        language: updated_user.language.unwrap_or_else(|| "en".to_string()),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_languages_count() {
        assert_eq!(
            SUPPORTED_LANGUAGES.len(),
            20,
            "Expected 20 supported languages"
        );
    }

    #[test]
    fn test_supported_languages_valid() {
        for lang in SUPPORTED_LANGUAGES {
            assert!(!lang.is_empty(), "Language code should not be empty");
            assert!(
                lang.len() <= 10,
                "Language code should be at most 10 characters"
            );
        }
    }
}
