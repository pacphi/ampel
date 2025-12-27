use crate::error::{Error, Result};
use crate::translator::TranslationService;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

pub struct OpenAITranslator {
    client: reqwest::Client,
    api_key: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

impl OpenAITranslator {
    pub fn new(api_key: String, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, api_key }
    }

    fn get_language_name(code: &str) -> &str {
        match code {
            "fi" => "Finnish",
            "de" => "German",
            "fr" => "French",
            "es" => "Spanish",
            "pt" => "Portuguese",
            "it" => "Italian",
            "nl" => "Dutch",
            "sv" => "Swedish",
            "da" => "Danish",
            "no" => "Norwegian",
            "pl" => "Polish",
            "ru" => "Russian",
            "ja" => "Japanese",
            "zh" => "Chinese",
            "ko" => "Korean",
            _ => code,
        }
    }
}

#[async_trait]
impl TranslationService for OpenAITranslator {
    async fn translate_batch(
        &self,
        texts: &HashMap<String, serde_json::Value>,
        target_lang: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Extract text values
        let source_texts: Vec<(String, String)> = texts
            .iter()
            .filter_map(|(k, v)| match v {
                serde_json::Value::String(s) => Some((k.clone(), s.clone())),
                _ => None,
            })
            .collect();

        if source_texts.is_empty() {
            return Ok(HashMap::new());
        }

        let lang_name = Self::get_language_name(target_lang);

        // Create prompt with all texts
        let texts_json = serde_json::json!(source_texts
            .iter()
            .map(|(k, v)| (k, v))
            .collect::<HashMap<_, _>>());

        let prompt = format!(
            "Translate the following UI text from English to {}. \
            Return ONLY a JSON object with the same keys and translated values. \
            Preserve any placeholders like {{variable}}.\n\n{}",
            lang_name,
            serde_json::to_string_pretty(&texts_json).unwrap()
        );

        let request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a professional translator specializing in UI/UX text. \
                             Return only valid JSON without any markdown formatting."
                        .to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.3,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Api(format!(
                "OpenAI API error {}: {}",
                status, body
            )));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let content = &openai_response
            .choices
            .first()
            .ok_or_else(|| Error::Translation("No response from OpenAI".to_string()))?
            .message
            .content;

        // Parse JSON response
        let translations: HashMap<String, String> = serde_json::from_str(content)
            .map_err(|e| Error::Translation(format!("Failed to parse OpenAI response: {}", e)))?;

        // Convert to expected format
        let mut result = HashMap::new();
        for (key, value) in translations {
            result.insert(key, serde_json::Value::String(value));
        }

        Ok(result)
    }
}
