use axum::http::header::{CONNECTION, CONTENT_LENGTH, HOST, ORIGIN, REFERER};

use axum::{
    body::{to_bytes, Body},
    extract::{Path, State},
    http::{Request, StatusCode},
    response::Response,
};
use reqwest::Client;

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

    let client = Client::new();

    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let mut request = client.request(method, url);

    for (name, value) in req.headers().iter() {
        // Skip hop-by-hop / browser headers
        if name == HOST
            || name == CONTENT_LENGTH
            || name == CONNECTION
            || name == ORIGIN
            || name == REFERER
        {
            continue;
        }

        if let Ok(val) = value.to_str() {
            request = request.header(name.as_str(), val);
        }
    }

    println!(
        "[PROXY] → {} {} -> {}",
        method_str,
        uri_str,
        request
            .try_clone()
            .expect("clone req")
            .build()
            .expect("build req")
            .url()
    );

    // Forward body
    let body_bytes = to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let response = request
        .body(body_bytes)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    println!("[PROXY] ← status from {} = {}", service, response.status());

    // Extract data BEFORE consuming response
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

    // Build Axum response
    let mut axum_response = Response::new(Body::from(body));
    *axum_response.status_mut() = status;

    // Forward allowlisted response headers, preserving multi-values (e.g. Set-Cookie).
    for (name, value) in response_headers {
        if let (Ok(header_name), Ok(header_value)) = (
            axum::http::HeaderName::from_bytes(name.as_bytes()),
            axum::http::HeaderValue::from_bytes(&value),
        ) {
            axum_response
                .headers_mut()
                .append(header_name, header_value);
        }
    }

    Ok(axum_response)
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
