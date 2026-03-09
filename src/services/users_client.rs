use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::error::ApiError;

pub async fn resolve_user_id(
    users_ms_url: &str,
    keycloak_id: &str,
    email: &str,
    name: &str,
    pseudo: &str,
    roles: &str,
    max_attempts: u32,
    base_delay_ms: u64,
) -> Result<Uuid, ApiError> {
    let attempts = max_attempts.max(1);
    let url = format!("{users_ms_url}/me");
    let client = Client::new();

    for attempt in 1..=attempts {
        let res = client
            .get(&url)
            .header("x-altair-keycloak-id", keycloak_id)
            .header("x-altair-email", email)
            .header("x-altair-name", name)
            .header("x-altair-pseudo", pseudo)
            .header("x-altair-roles", roles)
            .send()
            .await;

        match res {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    let json: Value = response
                        .json()
                        .await
                        .map_err(|_| ApiError::upstream_invalid_response("users"))?;

                    let user_id = json
                        .get("data")
                        .and_then(|d| d.get("user_id"))
                        .and_then(|id| id.as_str())
                        .ok_or_else(|| ApiError::upstream_invalid_response("users"))?;

                    return Uuid::parse_str(user_id)
                        .map_err(|_| ApiError::upstream_invalid_response("users"));
                }

                if is_retryable_status(status) && attempt < attempts {
                    let delay = backoff_delay(base_delay_ms, attempt);
                    println!(
                        "[USERS_CLIENT] transient status {} on attempt {}/{}; retry in {}ms",
                        status, attempt, attempts, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    continue;
                }

                return Err(ApiError::upstream_invalid_response("users"));
            }
            Err(err) => {
                if is_retryable_reqwest_error(&err) && attempt < attempts {
                    let delay = backoff_delay(base_delay_ms, attempt);
                    println!(
                        "[USERS_CLIENT] transient transport error on attempt {}/{}; retry in {}ms",
                        attempt, attempts, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    continue;
                }

                return if err.is_timeout() {
                    Err(ApiError::upstream_timeout("users"))
                } else {
                    Err(ApiError::upstream_unavailable("users"))
                };
            }
        }
    }

    Err(ApiError::upstream_unavailable("users"))
}

fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status == reqwest::StatusCode::BAD_GATEWAY
        || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
        || status == reqwest::StatusCode::GATEWAY_TIMEOUT
}

fn is_retryable_reqwest_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect() || err.is_request()
}

fn backoff_delay(base_delay_ms: u64, attempt: u32) -> u64 {
    let exp = attempt.saturating_sub(1).min(5);
    let factor = 1_u64 << exp;
    base_delay_ms.saturating_mul(factor).min(2_000)
}
