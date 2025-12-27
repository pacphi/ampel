use crate::error::{Error, Result};
use crate::translator::TranslationService;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

pub struct GoogleTranslator {
    client: reqwest::Client,
    api_key: String,
}

#[derive(Serialize)]
struct GoogleRequest {
    q: Vec<String>,
    target: String,
    source: String,
    format: String,
}

#[derive(Deserialize)]
struct GoogleResponse {
    data: GoogleData,
}

#[derive(Deserialize)]
struct GoogleData {
    translations: Vec<GoogleTranslation>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleTranslation {
    translated_text: String,
}

impl GoogleTranslator {
    pub fn new(api_key: String, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, api_key }
    }
}

#[async_trait]
impl TranslationService for GoogleTranslator {
    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Extract text values
        let source_texts: Vec<String> = texts
            .values()
            .filter_map(|v| match v {
                serde_json::Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect();

        if source_texts.is_empty() {
            return Ok(HashMap::new());
        }

        let request = GoogleRequest {
            q: source_texts.clone(),
            target: target_lang.to_string(),
            source: "en".to_string(),
            format: "text".to_string(),
        };

        let url = format!(
            "https://translation.googleapis.com/language/translate/v2?key={}",
            self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Api(format!(
                "Google Translation API error {}: {}",
                status, body
            )));
        }

        let google_response: GoogleResponse = response.json().await?;

        // Map translations back to keys
        let mut result = HashMap::new();
        let keys: Vec<_> = texts.keys().cloned().collect();

        for (i, translation) in google_response.data.translations.iter().enumerate() {
            if let Some(key) = keys.get(i) {
                result.insert(
                    key.clone(),
                    serde_json::Value::String(translation.translated_text.clone()),
                );
            }
        }

        Ok(result)
    }
}
