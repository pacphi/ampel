use ampel_i18n_builder::cache::{TranslationCache, CacheKey, CacheBackend};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_cache_basic_set_get() {
    let cache = TranslationCache::new(CacheBackend::Memory);

    let key = CacheKey {
        text: "Hello".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    cache.set(key.clone(), "Hola".to_string()).await;

    let result = cache.get(&key).await;
    assert_eq!(result, Some("Hola".to_string()));
}

#[tokio::test]
async fn test_cache_miss() {
    let cache = TranslationCache::new(CacheBackend::Memory);

    let key = CacheKey {
        text: "Unknown".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    let result = cache.get(&key).await;
    assert_eq!(result, None);
}

#[tokio::test]
async fn test_cache_overwrite() {
    let cache = TranslationCache::new(CacheBackend::Memory);

    let key = CacheKey {
        text: "Hello".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    cache.set(key.clone(), "Hola".to_string()).await;
    cache.set(key.clone(), "Hola!".to_string()).await;

    let result = cache.get(&key).await;
    assert_eq!(result, Some("Hola!".to_string()));
}

#[tokio::test]
async fn test_cache_ttl_expiry() {
    let mut cache = TranslationCache::new(CacheBackend::Memory);
    cache.set_default_ttl(Duration::from_millis(100));

    let key = CacheKey {
        text: "Hello".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    cache.set(key.clone(), "Hola".to_string()).await;

    // Should exist immediately
    assert_eq!(cache.get(&key).await, Some("Hola".to_string()));

    // Wait for expiry
    sleep(Duration::from_millis(150)).await;

    // Should be expired
    assert_eq!(cache.get(&key).await, None);
}

#[tokio::test]
async fn test_cache_lru_eviction() {
    let mut cache = TranslationCache::new(CacheBackend::Memory);
    cache.set_max_entries(3);

    let keys: Vec<CacheKey> = (0..5)
        .map(|i| CacheKey {
            text: format!("Text {}", i),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
        })
        .collect();

    // Add 5 entries (capacity is 3)
    for (i, key) in keys.iter().enumerate() {
        cache.set(key.clone(), format!("Translation {}", i)).await;
    }

    // First 2 entries should be evicted
    assert_eq!(cache.get(&keys[0]).await, None);
    assert_eq!(cache.get(&keys[1]).await, None);

    // Last 3 should exist
    assert!(cache.get(&keys[2]).await.is_some());
    assert!(cache.get(&keys[3]).await.is_some());
    assert!(cache.get(&keys[4]).await.is_some());
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = TranslationCache::new(CacheBackend::Memory);

    for i in 0..10 {
        let key = CacheKey {
            text: format!("Text {}", i),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
        };
        cache.set(key, format!("Translation {}", i)).await;
    }

    cache.clear().await;

    // All entries should be removed
    let key = CacheKey {
        text: "Text 0".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    assert_eq!(cache.get(&key).await, None);
}

#[tokio::test]
async fn test_cache_stats() {
    let cache = TranslationCache::new(CacheBackend::Memory);

    let key1 = CacheKey {
        text: "Hello".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    let key2 = CacheKey {
        text: "Goodbye".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    cache.set(key1.clone(), "Hola".to_string()).await;

    // Hit
    cache.get(&key1).await;
    cache.get(&key1).await;

    // Miss
    cache.get(&key2).await;

    let stats = cache.stats().await;

    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.entries, 1);
}

#[tokio::test]
async fn test_cache_hit_ratio() {
    let cache = TranslationCache::new(CacheBackend::Memory);

    let key = CacheKey {
        text: "Hello".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    cache.set(key.clone(), "Hola".to_string()).await;

    // 8 hits
    for _ in 0..8 {
        cache.get(&key).await;
    }

    // 2 misses
    for i in 0..2 {
        let miss_key = CacheKey {
            text: format!("Unknown {}", i),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
        };
        cache.get(&miss_key).await;
    }

    let stats = cache.stats().await;

    assert_eq!(stats.hits, 8);
    assert_eq!(stats.misses, 2);
    assert_eq!(stats.hit_ratio(), 0.8);
}

#[tokio::test]
async fn test_cache_concurrent_access() {
    use std::sync::Arc;

    let cache = Arc::new(TranslationCache::new(CacheBackend::Memory));

    let key = CacheKey {
        text: "Concurrent".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    cache.set(key.clone(), "Concurrente".to_string()).await;

    let mut tasks = vec![];

    // Spawn 100 concurrent readers
    for _ in 0..100 {
        let cache_clone = Arc::clone(&cache);
        let key_clone = key.clone();

        let task = tokio::spawn(async move {
            cache_clone.get(&key_clone).await
        });

        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("Concurrente".to_string()));
    }
}

#[tokio::test]
async fn test_cache_key_equality() {
    let key1 = CacheKey {
        text: "Hello".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    let key2 = CacheKey {
        text: "Hello".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    let key3 = CacheKey {
        text: "Hello".to_string(),
        source_lang: "en".to_string(),
        target_lang: "fr".to_string(), // Different target
    };

    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
}

#[tokio::test]
async fn test_cache_memory_usage() {
    let mut cache = TranslationCache::new(CacheBackend::Memory);
    cache.set_max_entries(1000);

    // Add 1000 entries
    for i in 0..1000 {
        let key = CacheKey {
            text: format!("Text {}", i),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
        };
        cache.set(key, format!("Translation {}", i)).await;
    }

    let stats = cache.stats().await;

    assert_eq!(stats.entries, 1000);
    assert!(stats.memory_bytes > 0, "Should track memory usage");
}

#[tokio::test]
#[ignore = "Requires Redis server"]
async fn test_cache_redis_backend() {
    let cache = TranslationCache::new(CacheBackend::Redis("redis://localhost:6379".to_string()));

    let key = CacheKey {
        text: "Hello".to_string(),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
    };

    cache.set(key.clone(), "Hola".to_string()).await;

    let result = cache.get(&key).await;
    assert_eq!(result, Some("Hola".to_string()));

    cache.clear().await;
}

#[tokio::test]
async fn test_cache_batch_set() {
    let cache = TranslationCache::new(CacheBackend::Memory);

    let entries = vec![
        (
            CacheKey {
                text: "One".to_string(),
                source_lang: "en".to_string(),
                target_lang: "es".to_string(),
            },
            "Uno".to_string(),
        ),
        (
            CacheKey {
                text: "Two".to_string(),
                source_lang: "en".to_string(),
                target_lang: "es".to_string(),
            },
            "Dos".to_string(),
        ),
    ];

    cache.set_batch(entries.clone()).await;

    for (key, expected_value) in entries {
        let result = cache.get(&key).await;
        assert_eq!(result, Some(expected_value));
    }
}
