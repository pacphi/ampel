use ampel_i18n_builder::api::RateLimiter;
use tokio::time::{sleep, Duration, Instant};
use std::sync::Arc;

#[tokio::test]
async fn test_rate_limiter_basic() {
    let rate_limiter = RateLimiter::new(10, 10); // 10 requests/sec, burst 10

    // First request should be immediate
    let start = Instant::now();
    let result = rate_limiter.acquire().await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert!(elapsed < Duration::from_millis(10),
        "First request should be immediate");
}

#[tokio::test]
async fn test_rate_limiter_burst_then_wait() {
    let rate_limiter = RateLimiter::new(2, 2); // 2 req/s, burst 2

    // Consume burst capacity
    let r1 = rate_limiter.acquire().await;
    let r2 = rate_limiter.acquire().await;

    assert!(r1.is_ok());
    assert!(r2.is_ok());

    // Third request should wait
    let start = Instant::now();
    let r3 = rate_limiter.acquire().await;
    let elapsed = start.elapsed();

    assert!(r3.is_ok());
    assert!(elapsed >= Duration::from_millis(400),
        "Should wait ~500ms for token refill, waited: {:?}", elapsed);
}

#[tokio::test]
async fn test_rate_limiter_token_refill() {
    let rate_limiter = RateLimiter::new(10, 10); // 10 req/s

    // Consume all tokens
    for _ in 0..10 {
        rate_limiter.acquire().await.unwrap();
    }

    // Wait for refill
    sleep(Duration::from_millis(200)).await;

    // Should have ~2 new tokens
    let start = Instant::now();
    let r1 = rate_limiter.acquire().await;
    let r2 = rate_limiter.acquire().await;
    let elapsed = start.elapsed();

    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(elapsed < Duration::from_millis(50),
        "Refilled tokens should be immediately available");
}

#[tokio::test]
async fn test_rate_limiter_concurrent_requests() {
    let rate_limiter = Arc::new(RateLimiter::new(5, 5));

    let mut tasks = vec![];

    // Spawn 10 concurrent requests
    for i in 0..10 {
        let limiter = Arc::clone(&rate_limiter);
        let task = tokio::spawn(async move {
            let start = Instant::now();
            let result = limiter.acquire().await;
            let elapsed = start.elapsed();
            (i, result, elapsed)
        });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    // All should succeed
    for result in &results {
        assert!(result.is_ok());
        let (_, acquire_result, _) = result.as_ref().unwrap();
        assert!(acquire_result.is_ok());
    }

    // Some should have waited
    let max_wait = results.iter()
        .filter_map(|r| r.as_ref().ok())
        .map(|(_, _, elapsed)| *elapsed)
        .max()
        .unwrap();

    assert!(max_wait > Duration::from_millis(100),
        "Some requests should have waited due to rate limiting");
}

#[tokio::test]
async fn test_rate_limiter_precise_timing() {
    let rate_limiter = RateLimiter::new(10, 1); // 10 req/s, burst 1

    let start = Instant::now();

    // Make 5 requests
    for _ in 0..5 {
        rate_limiter.acquire().await.unwrap();
    }

    let elapsed = start.elapsed();

    // Should take approximately 400ms (4 waits of ~100ms each)
    assert!(elapsed >= Duration::from_millis(300),
        "5 requests at 10/s should take ~400ms, took: {:?}", elapsed);
    assert!(elapsed < Duration::from_millis(600),
        "Timing should be reasonably accurate, took: {:?}", elapsed);
}

#[tokio::test]
async fn test_rate_limiter_high_throughput() {
    let rate_limiter = Arc::new(RateLimiter::new(100, 100)); // 100 req/s

    let start = Instant::now();

    let mut tasks = vec![];
    for _ in 0..100 {
        let limiter = Arc::clone(&rate_limiter);
        let task = tokio::spawn(async move {
            limiter.acquire().await
        });
        tasks.push(task);
    }

    futures::future::join_all(tasks).await;

    let elapsed = start.elapsed();

    // 100 requests at 100/s should take ~1 second
    assert!(elapsed < Duration::from_millis(1500),
        "High throughput test should complete quickly, took: {:?}", elapsed);
}

#[tokio::test]
async fn test_rate_limiter_zero_rate() {
    // Edge case: what happens with very low rate?
    let rate_limiter = RateLimiter::new(1, 1); // 1 req/s

    let start = Instant::now();

    rate_limiter.acquire().await.unwrap();
    rate_limiter.acquire().await.unwrap();

    let elapsed = start.elapsed();

    // Should wait approximately 1 second between requests
    assert!(elapsed >= Duration::from_millis(900),
        "Should enforce 1 req/s rate, took: {:?}", elapsed);
}

#[tokio::test]
async fn test_rate_limiter_burst_larger_than_rate() {
    // Burst can be larger than rate for bursty traffic
    let rate_limiter = RateLimiter::new(10, 50); // 10 req/s, burst 50

    let start = Instant::now();

    // Consume entire burst
    for _ in 0..50 {
        rate_limiter.acquire().await.unwrap();
    }

    let initial_elapsed = start.elapsed();

    // Burst should be consumed quickly
    assert!(initial_elapsed < Duration::from_millis(100),
        "Burst should be consumed immediately");

    // Next request should wait
    let wait_start = Instant::now();
    rate_limiter.acquire().await.unwrap();
    let wait_elapsed = wait_start.elapsed();

    assert!(wait_elapsed >= Duration::from_millis(80),
        "Should wait for token refill");
}

#[test]
fn test_rate_limiter_configuration() {
    let limiter = RateLimiter::new(10, 20);

    assert_eq!(limiter.rate(), 10);
    assert_eq!(limiter.burst(), 20);
}

#[tokio::test]
async fn test_rate_limiter_graceful_degradation() {
    let rate_limiter = Arc::new(RateLimiter::new(5, 5));

    // Simulate many concurrent clients
    let mut tasks = vec![];

    for _ in 0..100 {
        let limiter = Arc::clone(&rate_limiter);
        let task = tokio::spawn(async move {
            limiter.acquire().await
        });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    // All should eventually succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_rate_limiter_stats() {
    let rate_limiter = RateLimiter::new(10, 10);

    // Make some requests
    for _ in 0..5 {
        rate_limiter.acquire().await.unwrap();
    }

    // Get statistics
    let stats = rate_limiter.stats();

    assert_eq!(stats.total_requests, 5);
    assert_eq!(stats.granted_immediately, 5); // First 5 from burst
    assert_eq!(stats.throttled, 0);
}

#[tokio::test]
async fn test_rate_limiter_stats_with_throttling() {
    let rate_limiter = RateLimiter::new(2, 2);

    // Make requests that will be throttled
    for _ in 0..5 {
        rate_limiter.acquire().await.unwrap();
    }

    let stats = rate_limiter.stats();

    assert_eq!(stats.total_requests, 5);
    assert!(stats.throttled > 0, "Some requests should be throttled");
}
