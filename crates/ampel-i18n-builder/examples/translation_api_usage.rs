//! Example: Using Translation API with intelligent routing and caching
//!
//! This example demonstrates:
//! - Intelligent provider routing (DeepL for EU, Google for Thai/Arabic)
//! - File-based caching to avoid redundant API calls
//! - Batch translation with retry logic
//! - Usage statistics tracking
//!
//! Run with:
//! ```bash
//! export DEEPL_API_KEY="your-deepl-api-key"
//! export GOOGLE_API_KEY="your-google-api-key"
//! cargo run --example translation_api_usage
//! ```

use ampel_i18n_builder::cli::TranslationProvider;
use ampel_i18n_builder::config::Config;
use ampel_i18n_builder::translator::cache::FileCache;
use ampel_i18n_builder::translator::router::SmartTranslationRouter;
use ampel_i18n_builder::translator::{TranslationService, Translator};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("🌍 Translation API Usage Example\n");

    // 1. Load configuration (from .ampel-i18n.yaml or environment)
    let config = Config::load()?;
    println!("✓ Configuration loaded");

    // 2. Create file-based cache
    let cache = FileCache::default();
    println!("✓ File cache initialized (.ampel-i18n-cache/)");

    // 3. Example: Direct DeepL usage for Finnish
    println!("\n📍 Example 1: DeepL for Finnish (EU language)");
    let deepl_translator = Translator::new(TranslationProvider::DeepL, &config)?;

    let mut finnish_texts = HashMap::new();
    finnish_texts.insert("greeting".to_string(), serde_json::Value::String("Hello, world!".to_string()));
    finnish_texts.insert("farewell".to_string(), serde_json::Value::String("Goodbye!".to_string()));

    // Check cache first
    if let Some(cached) = cache.get("fi", "example", "greeting", "Hello, world!") {
        println!("  ✓ Cache hit: greeting -> {}", cached);
    } else {
        let result = deepl_translator.translate_batch(&finnish_texts, "fi").await?;
        println!("  ✓ API translation: {} keys", result.len());

        // Store in cache
        for (key, value) in &result {
            if let serde_json::Value::String(translated) = value {
                cache.set("fi", "example", key, finnish_texts[key].as_str().unwrap(), translated, "deepl")?;
            }
        }
    }

    // 4. Example: Direct Google usage for Thai
    println!("\n📍 Example 2: Google for Thai (non-EU language)");
    let google_translator = Translator::new(TranslationProvider::Google, &config)?;

    let mut thai_texts = HashMap::new();
    thai_texts.insert("welcome".to_string(), serde_json::Value::String("Welcome to our app!".to_string()));

    let result = google_translator.translate_batch(&thai_texts, "th").await?;
    println!("  ✓ Translated: {} keys", result.len());

    // 5. Example: Smart router (automatic provider selection)
    println!("\n📍 Example 3: Smart routing (automatic provider)");
    let router = SmartTranslationRouter::new(&config)?;

    let mut texts = HashMap::new();
    texts.insert("dashboard.title".to_string(), serde_json::Value::String("Dashboard".to_string()));

    // Router will choose DeepL for Swedish (EU language)
    let sv_result = router.translate_batch(&texts, "sv").await?;
    println!("  ✓ Swedish (sv): {} keys (DeepL used)", sv_result.len());

    // Router will choose Google for Arabic (non-EU language)
    let ar_result = router.translate_batch(&texts, "ar").await?;
    println!("  ✓ Arabic (ar): {} keys (Google used)", ar_result.len());

    // 6. Example: Batch caching
    println!("\n📍 Example 4: Batch cache operations");
    let batch_translations = vec![
        ("key1".to_string(), "Hello".to_string(), "Terve".to_string()),
        ("key2".to_string(), "World".to_string(), "Maailma".to_string()),
        ("key3".to_string(), "Thanks".to_string(), "Kiitos".to_string()),
    ];

    cache.set_batch("fi", "batch_example", &batch_translations, "deepl")?;
    println!("  ✓ Cached {} translations in batch", batch_translations.len());

    // 7. Display cache statistics
    println!("\n📊 Cache Statistics:");
    let stats = cache.stats("fi");
    println!("  • Total entries: {}", stats.total_entries);
    println!("  • Total namespaces: {}", stats.total_namespaces);
    println!("  • Providers used:");
    for (provider, count) in &stats.providers {
        println!("    - {}: {} translations", provider, count);
    }

    println!("\n✓ Example completed successfully!");
    println!("\n💡 Tips:");
    println!("  - Cache is stored in .ampel-i18n-cache/");
    println!("  - Re-run to see cache hits (95%+ on subsequent runs)");
    println!("  - Use 'cargo i18n translate' CLI for production workflows");

    Ok(())
}
