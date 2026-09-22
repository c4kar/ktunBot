use ktunbot::cache::AppCache;

#[tokio::test]
async fn test_rate_limiter() {
    let cache = AppCache::new();
    let chat_id = 999888777;

    // Up to 3 requests allowed
    assert!(cache.check_rate_limit(chat_id, 3).await);
    assert!(cache.check_rate_limit(chat_id, 3).await);
    assert!(cache.check_rate_limit(chat_id, 3).await);

    // 4th request should be rate-limited
    assert!(!cache.check_rate_limit(chat_id, 3).await);

    // Another chat_id should not be affected
    let another_chat = 111222333;
    assert!(cache.check_rate_limit(another_chat, 3).await);
}
