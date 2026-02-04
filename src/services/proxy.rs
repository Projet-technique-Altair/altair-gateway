use axum::http::header::{
    HOST, CONTENT_LENGTH, CONNECTION, ORIGIN, REFERER,
};

use axum::{
    body::{Body, to_bytes},
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


    let base_url = state
        .services
        .get(&service)
        .ok_or(StatusCode::NOT_FOUND)?;

    let url = format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        rest
    );

    let client = Client::new();

    let method = reqwest::Method::from_bytes(
        req.method().as_str().as_bytes()
    ).map_err(|_| StatusCode::BAD_GATEWAY)?;

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


    println!(
        "[PROXY] ← status from {} = {}",
        service,
        response.status()
    );


    // Extract data BEFORE consuming response
    let status = StatusCode::from_u16(response.status().as_u16())
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

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

    // Forward ONLY content-type (safe)
    if let Some(ct) = content_type {
        axum_response
            .headers_mut()
            .insert("content-type", ct.parse().unwrap());
    }

    Ok(axum_response)
}
