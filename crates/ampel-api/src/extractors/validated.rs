use axum::{
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

/// JSON extractor that validates the payload
pub struct ValidatedJson<T>(pub T);

#[derive(Debug)]
pub enum ValidatedJsonRejection {
    JsonRejection(JsonRejection),
    ValidationError(validator::ValidationErrors),
}

impl IntoResponse for ValidatedJsonRejection {
    fn into_response(self) -> Response {
        match self {
            ValidatedJsonRejection::JsonRejection(rejection) => rejection.into_response(),
            ValidatedJsonRejection::ValidationError(errors) => {
                let error_messages: Vec<String> = errors
                    .field_errors()
                    .iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |error| {
                            format!(
                                "{}: {}",
                                field,
                                error
                                    .message
                                    .as_ref()
                                    .map(|m| m.to_string())
                                    .unwrap_or_else(|| "Invalid value".to_string())
                            )
                        })
                    })
                    .collect();

                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "Validation failed",
                        "details": error_messages
                    })),
                )
                    .into_response()
            }
        }
    }
}

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ValidatedJsonRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(ValidatedJsonRejection::JsonRejection)?;

        value
            .validate()
            .map_err(ValidatedJsonRejection::ValidationError)?;

        Ok(ValidatedJson(value))
    }
}
