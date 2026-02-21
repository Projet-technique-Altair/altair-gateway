use axum::http::header::{CONNECTION, CONTENT_LENGTH, HOST, ORIGIN, REFERER};

use axum::{
    body::{to_bytes, Body},
    extract::{Path, State},
    http::{HeaderName, HeaderValue, Request, StatusCode},
    response::Response,
};
use reqwest::Client;
use tokio::time::{sleep, Duration};

use crate::state::AppState;

pub async fn proxy(
    State(state): State<AppState>,
    Path((service, rest)): Path<(String, String)>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let method_str = req.method().to_string();
    let uri_str = req.uri().to_string();

    let base_url = state.services.get(&service).ok_or(StatusCode::NOT_FOUND)?;

    let url = format!("{}/{}", base_url.trim_end_matches('/'), rest);

    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let forwarded_headers: Vec<(String, String)> = req
        .headers()
        .iter()
        .filter(|(name, _)| {
            !(*name == HOST
                || *name == CONTENT_LENGTH
                || *name == CONNECTION
                || *name == ORIGIN
                || *name == REFERER)
        })
        .filter_map(|(name, value)| {
            Some((name.as_str().to_string(), value.to_str().ok()?.to_string()))
        })
        .collect();

    // Forward body
    let body_bytes = to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let client = Client::new();
    let attempts = state.upstream_retry_max_attempts.max(1);
    let base_delay_ms = state.upstream_retry_base_delay_ms.max(1);

    println!("[PROXY] → {} {} -> {}", method_str, uri_str, url);

    for attempt in 1..=attempts {
        let mut outbound = client.request(method.clone(), &url);
        for (name, value) in &forwarded_headers {
            outbound = outbound.header(name, value);
        }

        let response = outbound.body(body_bytes.clone()).send().await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                if is_retryable_status(status) && attempt < attempts {
                    let delay = backoff_delay(base_delay_ms, attempt);
                    println!(
                        "[PROXY] transient status {} from {} on attempt {}/{}; retry in {}ms",
                        status, service, attempt, attempts, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    continue;
                }

                println!("[PROXY] ← status from {} = {}", service, status);
                return build_axum_response(service.as_str(), resp).await;
            }
            Err(err) => {
                if is_retryable_reqwest_error(&err) && attempt < attempts {
                    let delay = backoff_delay(base_delay_ms, attempt);
                    println!(
                        "[PROXY] transient transport error from {} on attempt {}/{}; retry in {}ms",
                        service, attempt, attempts, delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    continue;
                }
                return Err(StatusCode::BAD_GATEWAY);
            }
        }
    }

    Err(StatusCode::BAD_GATEWAY)
}

fn is_allowed_response_header(name: &reqwest::header::HeaderName) -> bool {
    *name == reqwest::header::CONTENT_TYPE
        || *name == reqwest::header::SET_COOKIE
        || *name == reqwest::header::LOCATION
        || *name == reqwest::header::CACHE_CONTROL
        || *name == reqwest::header::ETAG
        || *name == reqwest::header::LAST_MODIFIED
        || *name == reqwest::header::EXPIRES
        || *name == reqwest::header::VARY
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

async fn build_axum_response(
    service: &str,
    response: reqwest::Response,
) -> Result<Response<Body>, StatusCode> {
    let status =
        StatusCode::from_u16(response.status().as_u16()).map_err(|_| StatusCode::BAD_GATEWAY)?;

    let response_headers: Vec<(String, Vec<u8>)> = response
        .headers()
        .iter()
        .filter(|(name, _)| is_allowed_response_header(name))
        .map(|(name, value)| (name.as_str().to_string(), value.as_bytes().to_vec()))
        .collect();

    let body = response
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    println!(
        "[PROXY] ← body from {} = {}",
        service,
        String::from_utf8_lossy(&body)
    );

    let mut axum_response = Response::new(Body::from(body));
    *axum_response.status_mut() = status;

    for (name, value) in response_headers {
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_bytes(&value),
        ) {
            axum_response
                .headers_mut()
                .append(header_name, header_value);
        }
    }

    Ok(axum_response)
}
