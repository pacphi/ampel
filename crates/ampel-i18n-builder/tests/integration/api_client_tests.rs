use ampel_i18n_builder::api::{
    TranslationClient, TranslationRequest, MockTranslationProvider, RateLimiter
};
use mockito::{Mock, Server};
use tokio::time::{sleep, Duration, Instant};

#[tokio::test]
async fn test_translation_client_basic_request() {
    let mut server = Server::new_async().await;

    let mock = server.mock("POST", "/translate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "translations": [
                {"translatedText": "Hola", "confidence": 0.98}
            ],
            "detectedLanguage": "en"
        }"#)
        .create_async()
        .await;

    let client = TranslationClient::new(&server.url(), "test-api-key");

    let request = TranslationRequest {
        texts: vec!["Hello".to_string()],
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
        preserve_formatting: true,
        context_hints: vec![],
    };

    let response = client.translate(request).await;

    assert!(response.is_ok(), "Translation request should succeed");
    let result = response.unwrap();

    assert_eq!(result.translations.len(), 1);
    assert_eq!(result.translations[0], "Hola");
    assert_eq!(result.detected_language, "en");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_translation_client_batch_request() {
    let mut server = Server::new_async().await;

    let mock = server.mock("POST", "/translate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "translations": [
                {"translatedText": "Hola", "confidence": 0.98},
                {"translatedText": "Adiós", "confidence": 0.97},
                {"translatedText": "Por favor", "confidence": 0.99}
            ],
            "detectedLanguage": "en"
        }"#)
        .create_async()
        .await;

    let client = TranslationClient::new(&server.url(), "test-api-key");

    let request = TranslationRequest {
        texts: vec![
            "Hello".to_string(),
            "Goodbye".to_string(),
            "Please".to_string(),
        ],
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
        preserve_formatting: true,
        context_hints: vec![],
    };

    let response = client.translate(request).await;

    assert!(response.is_ok());
    let result = response.unwrap();

    assert_eq!(result.translations.len(), 3);
    assert_eq!(result.translations[0], "Hola");
    assert_eq!(result.translations[1], "Adiós");
    assert_eq!(result.translations[2], "Por favor");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_translation_client_error_handling() {
    let mut server = Server::new_async().await;

    let mock = server.mock("POST", "/translate")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "error": {
                "code": 401,
                "message": "Invalid API key"
            }
        }"#)
        .create_async()
        .await;

    let client = TranslationClient::new(&server.url(), "invalid-key");

    let request = TranslationRequest {
        texts: vec!["Hello".to_string()],
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
        preserve_formatting: true,
        context_hints: vec![],
    };

    let response = client.translate(request).await;

    assert!(response.is_err(), "Invalid API key should fail");

    let error = response.unwrap_err();
    assert!(error.to_string().contains("401") || error.to_string().contains("Invalid"),
        "Error should mention authentication failure");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_translation_client_retry_on_timeout() {
    let mut server = Server::new_async().await;

    // First request times out
    let mock1 = server.mock("POST", "/translate")
        .with_status(504)
        .create_async()
        .await;

    // Second request succeeds
    let mock2 = server.mock("POST", "/translate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "translations": [{"translatedText": "Hola", "confidence": 0.98}],
            "detectedLanguage": "en"
        }"#)
        .create_async()
        .await;

    let mut client = TranslationClient::new(&server.url(), "test-key");
    client.set_max_retries(2);
    client.set_retry_delay(Duration::from_millis(10));

    let request = TranslationRequest {
        texts: vec!["Hello".to_string()],
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
        preserve_formatting: true,
        context_hints: vec![],
    };

    let response = client.translate(request).await;

    assert!(response.is_ok(), "Should succeed after retry");

    mock1.assert_async().await;
    mock2.assert_async().await;
}

#[tokio::test]
async fn test_rate_limiter_allows_requests() {
    let rate_limiter = RateLimiter::new(10, 20);  // 10 req/s, burst 20

    // First request should succeed immediately
    let result = rate_limiter.acquire().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rate_limiter_enforces_limit() {
    let rate_limiter = RateLimiter::new(2, 2);  // 2 req/s, burst 2

    // Consume all tokens
    rate_limiter.acquire().await.unwrap();
    rate_limiter.acquire().await.unwrap();

    // Next request should wait
    let start = Instant::now();
    let result = rate_limiter.acquire().await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(elapsed >= Duration::from_millis(400),
        "Should wait for token refill, waited: {:?}", elapsed);
}

#[tokio::test]
async fn test_cache_hit() {
    let client = TranslationClient::new("http://mock.api", "test-key");

    // Enable caching
    let cached_client = client.with_cache(1000);

    // This test requires a mock implementation that tracks cache hits
    // For now, we just verify the client configuration
    assert!(cached_client.has_cache_enabled());
}

#[test]
fn test_mock_translation_provider() {
    let provider = MockTranslationProvider::new();

    provider.add_translation("Hello", "en", "es", "Hola");
    provider.add_translation("Goodbye", "en", "es", "Adiós");

    let result = provider.get_translation("Hello", "en", "es");
    assert_eq!(result, Some("Hola".to_string()));

    let result = provider.get_translation("Goodbye", "en", "es");
    assert_eq!(result, Some("Adiós".to_string()));

    let result = provider.get_translation("Unknown", "en", "es");
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_exponential_backoff() {
    let mut client = TranslationClient::new("http://mock.api", "test-key");

    client.set_max_retries(3);
    client.set_initial_retry_delay(Duration::from_millis(100));
    client.set_backoff_multiplier(2.0);

    let delays = client.calculate_retry_delays();

    assert_eq!(delays.len(), 3);
    assert_eq!(delays[0], Duration::from_millis(100));
    assert_eq!(delays[1], Duration::from_millis(200));
    assert_eq!(delays[2], Duration::from_millis(400));
}

#[tokio::test]
async fn test_translation_with_context_hints() {
    let mut server = Server::new_async().await;

    let mock = server.mock("POST", "/translate")
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::JsonString(r#"{
            "q": ["Save"],
            "source": "en",
            "target": "fr",
            "format": "html",
            "context": ["button", "action"]
        }"#.to_string()))
        .with_status(200)
        .with_body(r#"{
            "translations": [{"translatedText": "Enregistrer"}],
            "detectedLanguage": "en"
        }"#)
        .create_async()
        .await;

    let client = TranslationClient::new(&server.url(), "test-key");

    let request = TranslationRequest {
        texts: vec!["Save".to_string()],
        source_lang: "en".to_string(),
        target_lang: "fr".to_string(),
        preserve_formatting: true,
        context_hints: vec!["button".to_string(), "action".to_string()],
    };

    let response = client.translate(request).await;
    assert!(response.is_ok());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_concurrent_requests() {
    let mut server = Server::new_async().await;

    // Mock multiple concurrent successful responses
    for _ in 0..5 {
        server.mock("POST", "/translate")
            .with_status(200)
            .with_body(r#"{
                "translations": [{"translatedText": "Test"}],
                "detectedLanguage": "en"
            }"#)
            .create_async()
            .await;
    }

    let client = TranslationClient::new(&server.url(), "test-key");

    let mut tasks = vec![];

    for i in 0..5 {
        let client_clone = client.clone();
        let task = tokio::spawn(async move {
            let request = TranslationRequest {
                texts: vec![format!("Text {}", i)],
                source_lang: "en".to_string(),
                target_lang: "es".to_string(),
                preserve_formatting: true,
                context_hints: vec![],
            };
            client_clone.translate(request).await
        });
        tasks.push(task);
    }

    // Wait for all tasks
    let results = futures::future::join_all(tasks).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[test]
fn test_api_key_validation() {
    // Valid API keys
    assert!(TranslationClient::is_valid_api_key("sk-1234567890abcdef"));
    assert!(TranslationClient::is_valid_api_key("AIzaSyBxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"));

    // Invalid API keys
    assert!(!TranslationClient::is_valid_api_key(""));
    assert!(!TranslationClient::is_valid_api_key("   "));
    assert!(!TranslationClient::is_valid_api_key("invalid key with spaces"));
}
