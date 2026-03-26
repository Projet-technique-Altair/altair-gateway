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

    let query = req.uri().query();
    let url = if let Some(q) = query {
        format!("{}/{}?{}", base_url.trim_end_matches('/'), rest, q)
    } else {
        format!("{}/{}", base_url.trim_end_matches('/'), rest)
    };

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
    let request_headers = req.headers().clone();
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

            // =========================
            // ACCESS CONTROL (LABS / STARPATHS)
            // =========================
            if method_str == "GET" && (service == "labs" || service == "starpaths") {
                println!("[ACCESS] ENTER CHECK");

                let parts: Vec<&str> = rest.split('/').collect();
                println!("[ACCESS] parts = {:?}", parts);

                let resource_id = if parts.len() == 1 && is_uuid(parts[0]) {
                    parts[0]
                } else if parts.len() == 2 && is_uuid(parts[1]) {
                    parts[1]
                } else {
                    println!("[ACCESS] skip route: {}", rest);
                    return build_axum_response(service.as_str(), resp).await;
                };

                println!("[ACCESS] resource_id = {}", resource_id);

                let response_headers = resp.headers().clone();
                let body_bytes = resp.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
                let body_str = String::from_utf8_lossy(&body_bytes);

                println!("[ACCESS] resource body = {}", body_str);

                let json: serde_json::Value =
                    serde_json::from_slice(&body_bytes).map_err(|_| StatusCode::BAD_GATEWAY)?;

                let visibility = json
                    .get("data")
                    .and_then(|d| d.get("visibility"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("PUBLIC");

                let creator_id = json
                    .get("data")
                    .and_then(|d| d.get("creator_id"))
                    .and_then(|v| v.as_str());

                let user_id = request_headers
                    .get("x-altair-user-id")
                    .and_then(|v| v.to_str().ok())
                    .ok_or(StatusCode::UNAUTHORIZED)?;

                println!("[ACCESS] creator_id = {:?}", creator_id);
                println!("[ACCESS] user_id = {}", user_id);

                // 👇 bypass creator
                if let Some(cid) = creator_id {
                    if cid == user_id {
                        println!("[ACCESS] creator bypass");
                        
                        // reconstruire réponse direct
                        let mut axum_response = Response::new(Body::from(body_bytes));
                        *axum_response.status_mut() =
                            StatusCode::from_u16(status.as_u16()).map_err(|_| StatusCode::BAD_GATEWAY)?;

                        for (name, value) in response_headers.iter() {
                            if is_allowed_response_header(name) {
                                if let Ok(val) = value.to_str() {
                                    if let (Ok(header_name), Ok(header_value)) = (
                                        HeaderName::from_bytes(name.as_str().as_bytes()),
                                        HeaderValue::from_str(val),
                                    ) {
                                        axum_response.headers_mut().insert(header_name, header_value);
                                    }
                                }
                            }
                        }

                        return Ok(axum_response);
                    }
                }

                let is_private = visibility == "PRIVATE";

                println!("[ACCESS] visibility = {}", visibility);
                println!("[ACCESS] is_private = {}", is_private);

                if is_private {
                    let user_id = request_headers
                        .get("x-altair-user-id")
                        .and_then(|v| v.to_str().ok())
                        .ok_or(StatusCode::UNAUTHORIZED)?;

                    let groups_base = state
                        .services
                        .get("groups")
                        .ok_or(StatusCode::BAD_GATEWAY)?;

                    let access_url = if service == "labs" {
                        format!(
                            "{}/internal/access/lab?user_id={}&lab_id={}",
                            groups_base, user_id, resource_id
                        )
                    } else {
                        format!(
                            "{}/internal/access/starpath?user_id={}&starpath_id={}",
                            groups_base, user_id, resource_id
                        )
                    };

                    println!("[ACCESS] calling {}", access_url);

                    let access_resp = client
                        .get(&access_url)
                        .send()
                        .await
                        .map_err(|_| StatusCode::BAD_GATEWAY)?;

                    let access_status = access_resp.status();
                    let access_json: serde_json::Value = access_resp
                        .json()
                        .await
                        .map_err(|_| StatusCode::BAD_GATEWAY)?;

                    println!("[ACCESS] access status = {}", access_status);
                    println!("[ACCESS] access body = {}", access_json);

                    let allowed = access_json
                        .get("data")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    println!("[ACCESS] allowed = {}", allowed);

                    if !allowed {
                        return Err(StatusCode::FORBIDDEN);
                    }
                }

                let mut axum_response = Response::new(Body::from(body_bytes));
                *axum_response.status_mut() =
                    StatusCode::from_u16(status.as_u16()).map_err(|_| StatusCode::BAD_GATEWAY)?;

                for (name, value) in response_headers.iter() {
                    if is_allowed_response_header(name) {
                        if let Ok(val) = value.to_str() {
                            if let (Ok(header_name), Ok(header_value)) = (
                                HeaderName::from_bytes(name.as_str().as_bytes()),
                                HeaderValue::from_str(val),
                            ) {
                                axum_response.headers_mut().insert(header_name, header_value);
                            }
                        }
                    }
                }

                return Ok(axum_response);
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

fn is_uuid(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok()
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
