use crate::security::jwks_cache::{JwksCache, JwksCacheConfig};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub services: HashMap<String, String>,

    // NOUVEAU
    pub user_cache: DashMap<String, Uuid>,
    pub jwks_cache: Arc<JwksCache>,
}

impl AppState {
    pub fn new() -> Self {
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
            "starpath".into(),
            std::env::var("STARPATH_MS_URL")
                .unwrap_or_else(|_| "http://localhost:3005".to_string()),
        );

        services.insert(
            "groups".into(),
            std::env::var("GROUPS_MS_URL").unwrap_or_else(|_| "http://localhost:3006".to_string()),
        );

        Self {
            services,
            user_cache: DashMap::new(), //
            jwks_cache: Arc::new(JwksCache::new(JwksCacheConfig::from_env())),
        }
    }
}
