use crate::error::{Error, Result};
use crate::translator::TranslationService;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Maximum keys to translate in a single OpenAI request to avoid timeouts
const DEFAULT_CHUNK_SIZE: usize = 15;
/// Minimum chunk size when retrying after timeout
const MIN_CHUNK_SIZE: usize = 5;

pub struct OpenAITranslator {
    client: reqwest::Client,
    api_key: String,
    chunk_size: usize,
    max_retries: u32,
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
    pub fn new(api_key: String, timeout: Duration) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| Error::Config(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            api_key,
            chunk_size: DEFAULT_CHUNK_SIZE,
            max_retries: 3,
        })
    }

    /// Translate a single chunk of texts
    async fn translate_chunk(
        &self,
        texts: &HashMap<String, String>,
        target_lang: &str,
    ) -> Result<HashMap<String, String>> {
        let lang_name = Self::get_language_name(target_lang);

        // Create prompt with chunk of texts
        let texts_json = serde_json::json!(texts);

        let prompt = format!(
            "Translate the following UI text from English to {}. \
            CRITICAL REQUIREMENTS:\n\
            1. Return ONLY a JSON object with the same keys and translated values\n\
            2. PRESERVE ALL PLACEHOLDERS EXACTLY: {{{{count}}}}, {{{{provider}}}}, {{{{variable}}}}, etc.\n\
            3. Keep placeholders in the SAME POSITION in the translated text\n\
            4. Do NOT translate placeholder names - keep them in English\n\
            5. Translate ONLY the surrounding text, not the placeholder variables\n\
            Example: \"{{{{count}}}} items\" in French → \"{{{{count}}}} éléments\"\n\n{}",
            lang_name,
            serde_json::to_string_pretty(&texts_json)
                .map_err(|e| Error::Internal(format!("Failed to serialize texts: {}", e)))?
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
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error response".to_string());
            return Err(Error::Api(format!("OpenAI API error {}: {}", status, body)));
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

        // Validate placeholder preservation
        for (key, translated) in &translations {
            if let Some(original) = texts.get(key) {
                let original_placeholders = extract_placeholders(original);
                let translated_placeholders = extract_placeholders(translated);

                if original_placeholders != translated_placeholders {
                    eprintln!(
                        "⚠️  Warning: Placeholders mismatch in key '{}'\n   Original: {:?}\n   Translated: {:?}",
                        key, original_placeholders, translated_placeholders
                    );
                }
            }
        }

        Ok(translations)
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

/// Extract placeholders like {{count}}, {{provider}}, {{variable}} from text
fn extract_placeholders(text: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume second {
            let mut placeholder = String::new();

            // Extract placeholder content
            while let Some(&ch) = chars.peek() {
                if ch == '}' {
                    chars.next();
                    if chars.peek() == Some(&'}') {
                        chars.next();
                        placeholders.push(format!("{{{{{}}}}}", placeholder));
                        break;
                    }
                } else {
                    placeholder.push(ch);
                    chars.next();
                }
            }
        }
    }

    placeholders.sort();
    placeholders
}

#[async_trait]
impl TranslationService for OpenAITranslator {
    fn provider_name(&self) -> &str {
        "OpenAI"
    }

    fn provider_tier(&self) -> u8 {
        4 // Tier 4: Fallback for specialized content
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

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

        let total_keys = source_texts.len();

        // If batch is small enough, translate in one go
        if total_keys <= self.chunk_size {
            tracing::debug!("Translating {} keys in single chunk", total_keys);
            let chunk_map: HashMap<String, String> = source_texts.into_iter().collect();
            let translated = self.translate_chunk(&chunk_map, target_lang).await?;

            // Convert to expected format
            let mut result = HashMap::new();
            for (key, value) in translated {
                result.insert(key, serde_json::Value::String(value));
            }
            return Ok(result);
        }

        // Large batch - split into chunks
        tracing::info!(
            "Large batch detected ({} keys), splitting into chunks of {}",
            total_keys,
            self.chunk_size
        );

        let mut all_translations = HashMap::new();
        let chunks: Vec<_> = source_texts.chunks(self.chunk_size).collect();
        let total_chunks = chunks.len();

        for (i, chunk) in chunks.into_iter().enumerate() {
            tracing::info!(
                "Translating chunk {}/{} ({} keys)",
                i + 1,
                total_chunks,
                chunk.len()
            );

            let chunk_map: HashMap<String, String> = chunk.iter().cloned().collect();

            // Retry loop with exponential backoff and chunk splitting fallback
            let mut retry_count = 0;
            let mut last_error;

            while retry_count <= self.max_retries {
                match self.translate_chunk(&chunk_map, target_lang).await {
                    Ok(translated) => {
                        // Success - add to results
                        for (key, value) in translated {
                            all_translations.insert(key, serde_json::Value::String(value));
                        }
                        break; // Success - exit retry loop
                    }
                    Err(e) => {
                        last_error = Some(e);
                        retry_count += 1;

                        // Check if it's a timeout error
                        let is_timeout = last_error
                            .as_ref()
                            .map(|err| {
                                let msg = err.to_string();
                                msg.contains("timeout")
                                    || msg.contains("timed out")
                                    || msg.contains("deadline")
                            })
                            .unwrap_or(false);

                        // If we've exhausted retries
                        if retry_count > self.max_retries {
                            // Last resort: if timeout and chunk is large, try splitting
                            if is_timeout && chunk.len() > MIN_CHUNK_SIZE {
                                tracing::warn!(
                                    "Timeout after {} retries on chunk {}/{} ({} keys), splitting into smaller sub-chunks",
                                    self.max_retries,
                                    i + 1,
                                    total_chunks,
                                    chunk.len()
                                );

                                let sub_chunk_size = (chunk.len() / 2).max(MIN_CHUNK_SIZE);
                                let sub_chunks: Vec<_> = chunk.chunks(sub_chunk_size).collect();

                                for (j, sub_chunk) in sub_chunks.into_iter().enumerate() {
                                    tracing::info!(
                                        "  Sub-chunk {}/{} ({} keys)",
                                        j + 1,
                                        chunk.len().div_ceil(sub_chunk_size),
                                        sub_chunk.len()
                                    );

                                    let sub_map: HashMap<String, String> =
                                        sub_chunk.iter().cloned().collect();
                                    let translated =
                                        self.translate_chunk(&sub_map, target_lang).await?;

                                    for (key, value) in translated {
                                        all_translations
                                            .insert(key, serde_json::Value::String(value));
                                    }

                                    // Small delay between sub-chunks
                                    tokio::time::sleep(Duration::from_millis(500)).await;
                                }
                                break; // Successfully handled via sub-chunks
                            } else {
                                // Can't split or not a timeout - propagate error
                                return Err(last_error.unwrap());
                            }
                        } else if retry_count <= self.max_retries {
                            // Retry with exponential backoff
                            let backoff_ms = 1000 * (2_u64.pow(retry_count - 1));
                            tracing::warn!(
                                "Chunk {}/{} failed (attempt {}/{}), retrying in {}ms...",
                                i + 1,
                                total_chunks,
                                retry_count,
                                self.max_retries + 1,
                                backoff_ms
                            );
                            tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                            // Continue to next iteration of while loop
                        }
                    }
                }
            }

            // Small delay between chunks to avoid rate limiting
            if i < total_chunks - 1 {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }

        Ok(all_translations)
    }
}
