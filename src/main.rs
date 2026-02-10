use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod error;
mod middleware;
mod routes;
mod security;
mod services;
mod state;
mod utils;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let state = state::AppState::new();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT]);

    let app = routes::init_routes(state.clone())
        .with_state(state)
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Cannot bind port 3000");

    println!("Gateway running on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("Failed to start Gateway");
}
