//! Integration tests for translation API clients.

#[cfg(test)]
mod tests {
    use ampel_i18n_builder::api::{DeepLProvider, TranslationProvider, TranslationOptions};

    #[tokio::test]
    #[ignore] // Requires DEEPL_API_KEY environment variable
    async fn test_deepl_translate_single() {
        let api_key = std::env::var("DEEPL_API_KEY")
            .expect("DEEPL_API_KEY not set");

        let provider = DeepLProvider::new(api_key.into());

        let result = provider.translate(
            vec!["Hello world".to_string()],
            "en",
            "de",
            TranslationOptions::default(),
        ).await;

        assert!(result.is_ok());
        let translations = result.unwrap();
        assert_eq!(translations.len(), 1);
        assert!(translations[0].contains("Hallo") || translations[0].contains("Welt"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_deepl_validate_credentials() {
        let api_key = std::env::var("DEEPL_API_KEY")
            .expect("DEEPL_API_KEY not set");

        let provider = DeepLProvider::new(api_key.into());
        let is_valid = provider.validate_credentials().await.unwrap();

        assert!(is_valid);
    }
}
