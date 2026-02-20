use axum::http::{HeaderName, HeaderValue, Method};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};

mod error;
mod middleware;
mod routes;
mod security;
mod services;
mod state;
mod utils;

const DEFAULT_ALLOWED_ORIGINS: &str = "http://localhost:5173,http://localhost:3000";
const DEFAULT_ALLOWED_METHODS: &str = "GET,POST,PUT,PATCH,DELETE,OPTIONS";
const DEFAULT_ALLOWED_HEADERS: &str = "authorization,content-type";

fn parse_allowed_origins() -> Vec<HeaderValue> {
    let raw =
        std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| DEFAULT_ALLOWED_ORIGINS.to_string());
    let parsed: Vec<HeaderValue> = raw
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();

    if parsed.is_empty() {
        DEFAULT_ALLOWED_ORIGINS
            .split(',')
            .filter_map(|origin| HeaderValue::from_str(origin).ok())
            .collect()
    } else {
        parsed
    }
}

fn parse_allowed_methods() -> Vec<Method> {
    let raw =
        std::env::var("ALLOWED_METHODS").unwrap_or_else(|_| DEFAULT_ALLOWED_METHODS.to_string());
    let parsed: Vec<Method> = raw
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .filter_map(|method| Method::from_bytes(method.as_bytes()).ok())
        .collect();

    if parsed.is_empty() {
        DEFAULT_ALLOWED_METHODS
            .split(',')
            .filter_map(|method| Method::from_bytes(method.as_bytes()).ok())
            .collect()
    } else {
        parsed
    }
}

fn parse_allowed_headers() -> Vec<HeaderName> {
    let raw =
        std::env::var("ALLOWED_HEADERS").unwrap_or_else(|_| DEFAULT_ALLOWED_HEADERS.to_string());
    let parsed: Vec<HeaderName> = raw
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .filter_map(|header| HeaderName::from_bytes(header.to_ascii_lowercase().as_bytes()).ok())
        .collect();

    if parsed.is_empty() {
        DEFAULT_ALLOWED_HEADERS
            .split(',')
            .filter_map(|header| HeaderName::from_bytes(header.as_bytes()).ok())
            .collect()
    } else {
        parsed
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let state = state::AppState::new();

    let allowed_origins = parse_allowed_origins();
    let allowed_methods = parse_allowed_methods();
    let allowed_headers = parse_allowed_headers();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods(allowed_methods)
        .allow_headers(allowed_headers);

    let app = routes::init_routes(state.clone())
        .with_state(state)
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Cannot bind port 3000");

    println!("Gateway running On http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("Failed to start Gateway");
}
