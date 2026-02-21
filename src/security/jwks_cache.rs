use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::header::CACHE_CONTROL;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::error::ApiError;

#[derive(Debug, Clone, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Jwk {
    pub kid: String,
    pub kty: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Clone)]
struct CachedJwks {
    jwks: Jwks,
    expires_at: Instant,
    stale_until: Instant,
}

#[derive(Debug, Clone)]
pub struct JwksCacheConfig {
    ttl_seconds: u64,
    stale_if_error_seconds: u64,
    min_ttl_seconds: u64,
    max_ttl_seconds: u64,
}

impl JwksCacheConfig {
    pub fn from_env() -> Self {
        let ttl_seconds = std::env::var("JWKS_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(300);
        let stale_if_error_seconds = std::env::var("JWKS_STALE_IF_ERROR_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(120);
        let min_ttl_seconds = std::env::var("JWKS_CACHE_MIN_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(30);
        let max_ttl_seconds = std::env::var("JWKS_CACHE_MAX_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(3600);

        Self {
            ttl_seconds,
            stale_if_error_seconds,
            min_ttl_seconds,
            max_ttl_seconds,
        }
    }

    fn clamp_ttl(&self, ttl: u64) -> u64 {
        ttl.max(self.min_ttl_seconds).min(self.max_ttl_seconds)
    }
}

#[derive(Clone)]
pub struct JwksCache {
    client: reqwest::Client,
    config: JwksCacheConfig,
    inner: Arc<RwLock<Option<CachedJwks>>>,
}

impl JwksCache {
    pub fn new(config: JwksCacheConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
            inner: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_jwk_for_kid(&self, jwks_url: &str, kid: &str) -> Result<Jwk, ApiError> {
        let jwks = self.get_jwks(jwks_url).await?;
        if let Some(jwk) = find_rsa_key(&jwks, kid) {
            return Ok(jwk);
        }

        // Security mitigation: if kid is unknown, force immediate refresh once.
        println!("[JWKS] unknown kid, forcing refresh: {kid}");
        let refreshed = self.refresh_jwks(jwks_url, true).await?;
        find_rsa_key(&refreshed, kid)
            .ok_or_else(|| ApiError::unauthorized("Unknown signing key (kid)"))
    }

    async fn get_jwks(&self, jwks_url: &str) -> Result<Jwks, ApiError> {
        let now = Instant::now();
        if let Some(cached) = self.inner.read().await.as_ref() {
            if cached.expires_at > now {
                println!("[JWKS] cache hit");
                return Ok(cached.jwks.clone());
            }
        }

        println!("[JWKS] cache miss/expired");
        self.refresh_jwks(jwks_url, false).await
    }

    async fn refresh_jwks(&self, jwks_url: &str, force: bool) -> Result<Jwks, ApiError> {
        if !force {
            let now = Instant::now();
            if let Some(cached) = self.inner.read().await.as_ref() {
                if cached.expires_at > now {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        let mut write_guard = self.inner.write().await;
        if !force {
            let now = Instant::now();
            if let Some(cached) = write_guard.as_ref() {
                if cached.expires_at > now {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        let stale_snapshot = write_guard.clone();
        match self.fetch_jwks(jwks_url).await {
            Ok((jwks, ttl_from_cache_control)) => {
                let ttl = self
                    .config
                    .clamp_ttl(ttl_from_cache_control.unwrap_or(self.config.ttl_seconds));
                let now = Instant::now();
                println!("[JWKS] refresh success (ttl={ttl}s)");
                let cached = CachedJwks {
                    jwks: jwks.clone(),
                    expires_at: now + Duration::from_secs(ttl),
                    stale_until: now
                        + Duration::from_secs(ttl + self.config.stale_if_error_seconds),
                };
                *write_guard = Some(cached);
                Ok(jwks)
            }
            Err(err) => {
                let now = Instant::now();
                if let Some(stale) = stale_snapshot {
                    if stale.stale_until > now {
                        println!("[JWKS] refresh failed, using stale cache");
                        return Ok(stale.jwks);
                    }
                }
                println!("[JWKS] refresh failed, no stale cache available");
                Err(err)
            }
        }
    }

    async fn fetch_jwks(&self, jwks_url: &str) -> Result<(Jwks, Option<u64>), ApiError> {
        let response = self
            .client
            .get(jwks_url)
            .send()
            .await
            .map_err(|_| ApiError::upstream_unavailable("keycloak"))?;

        if !response.status().is_success() {
            return Err(ApiError::upstream_invalid_response("keycloak"));
        }

        let cache_control_max_age = response
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_max_age_seconds);

        let jwks = response
            .json::<Jwks>()
            .await
            .map_err(|_| ApiError::upstream_invalid_response("keycloak"))?;

        Ok((jwks, cache_control_max_age))
    }
}

fn find_rsa_key(jwks: &Jwks, kid: &str) -> Option<Jwk> {
    jwks.keys
        .iter()
        .find(|k| k.kid == kid && k.kty == "RSA")
        .cloned()
}

fn parse_max_age_seconds(cache_control: &str) -> Option<u64> {
    cache_control
        .split(',')
        .map(str::trim)
        .find_map(|part| part.strip_prefix("max-age="))
        .and_then(|v| v.parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::parse_max_age_seconds;

    #[test]
    fn parse_max_age_from_cache_control() {
        assert_eq!(parse_max_age_seconds("public, max-age=600"), Some(600));
        assert_eq!(
            parse_max_age_seconds("max-age=90, must-revalidate"),
            Some(90)
        );
        assert_eq!(parse_max_age_seconds("no-cache"), None);
    }
}
