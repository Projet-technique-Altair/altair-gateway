use crate::security::jwks_cache::{JwksCache, JwksCacheConfig};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct CachedUserId {
    pub user_id: Uuid,
    pub expires_at: Instant,
}

impl CachedUserId {
    pub fn new(user_id: Uuid, ttl: Duration) -> Self {
        Self {
            user_id,
            expires_at: Instant::now() + ttl,
        }
    }

    pub fn is_expired(&self, now: Instant) -> bool {
        now >= self.expires_at
    }
}

#[derive(Clone)]
pub struct AppState {
    pub services: HashMap<String, String>,
    pub user_cache: Arc<DashMap<String, CachedUserId>>,
    pub user_cache_ttl: Duration,
    pub upstream_retry_max_attempts: u32,
    pub upstream_retry_base_delay_ms: u64,
    pub jwks_cache: Arc<JwksCache>,
}

impl AppState {
    pub fn new() -> Self {
        let user_cache_ttl = parse_u64_env("USER_ID_CACHE_TTL_SECONDS", 300);
        let user_cache_cleanup_interval =
            parse_u64_env("USER_ID_CACHE_CLEANUP_INTERVAL_SECONDS", 60);
        let upstream_retry_max_attempts = parse_u32_env("UPSTREAM_RETRY_MAX_ATTEMPTS", 3);
        let upstream_retry_base_delay_ms = parse_u64_env("UPSTREAM_RETRY_BASE_DELAY_MS", 100);

        let mut services = HashMap::new();

        services.insert(
            "users".into(),
            std::env::var("USERS_MS_URL").unwrap_or_else(|_| "http://localhost:3001".to_string()),
        );

        services.insert(
            "labs".into(),
            std::env::var("LABS_MS_URL").unwrap_or_else(|_| "http://localhost:3002".to_string()),
        );

        services.insert(
            "sessions".into(),
            std::env::var("SESSIONS_MS_URL")
                .unwrap_or_else(|_| "http://localhost:3003".to_string()),
        );

        services.insert(
            "starpaths".into(),
            std::env::var("STARPATH_MS_URL")
                .unwrap_or_else(|_| "http://localhost:3005".to_string()),
        );

        services.insert(
            "groups".into(),
            std::env::var("GROUPS_MS_URL").unwrap_or_else(|_| "http://localhost:3006".to_string()),
        );

        let state = Self {
            services,
            user_cache: Arc::new(DashMap::new()),
            user_cache_ttl: Duration::from_secs(user_cache_ttl),
            upstream_retry_max_attempts: upstream_retry_max_attempts.max(1),
            upstream_retry_base_delay_ms: upstream_retry_base_delay_ms.max(1),
            jwks_cache: Arc::new(JwksCache::new(JwksCacheConfig::from_env())),
        };

        // Periodic cleanup avoids keeping expired entries forever on long-lived instances.
        let cache = Arc::clone(&state.user_cache);
        let cleanup_every = Duration::from_secs(user_cache_cleanup_interval.max(1));
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(cleanup_every);
            loop {
                ticker.tick().await;
                let now = Instant::now();
                let before = cache.len();
                cache.retain(|_, entry| !entry.is_expired(now));
                let removed = before.saturating_sub(cache.len());
                if removed > 0 {
                    println!(
                        "[USER_ID_CACHE] cleanup removed {} expired entries",
                        removed
                    );
                }
            }
        });

        state
    }
}

fn parse_u64_env(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

fn parse_u32_env(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}
