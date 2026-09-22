use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AppCache {
    announcements: Cache<String, Arc<Vec<u8>>>,
    menus: Cache<String, Arc<Vec<u8>>>,
    rate_limits: Cache<i64, u32>,
}

impl Default for AppCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AppCache {
    pub fn new() -> Self {
        Self {
            announcements: Cache::builder()
                .time_to_live(Duration::from_secs(900))
                .build(),
            menus: Cache::builder()
                .time_to_live(Duration::from_secs(7200))
                .build(),
            rate_limits: Cache::builder()
                .time_to_live(Duration::from_secs(5))
                .max_capacity(10_000)
                .build(),
        }
    }

    /// Check if chat_id has exceeded rate limit (returns true if allowed, false if rate limited)
    pub async fn check_rate_limit(&self, chat_id: i64, max_per_5s: u32) -> bool {
        let current = self.rate_limits.get(&chat_id).await.unwrap_or(0);
        if current >= max_per_5s {
            false
        } else {
            self.rate_limits.insert(chat_id, current + 1).await;
            true
        }
    }

    pub async fn get_announcements(&self, n: usize) -> Option<Vec<u8>> {
        self.announcements
            .get(&n.to_string())
            .await
            .map(|arc| arc.as_ref().clone())
    }

    pub async fn set_announcements(&self, n: usize, bytes: Vec<u8>) {
        self.announcements
            .insert(n.to_string(), Arc::new(bytes))
            .await;
    }

    pub async fn get_menu(&self, month: &str) -> Option<Vec<u8>> {
        self.menus.get(month).await.map(|arc| arc.as_ref().clone())
    }

    pub async fn set_menu(&self, month: &str, bytes: Vec<u8>) {
        self.menus.insert(month.to_string(), Arc::new(bytes)).await;
    }
}
