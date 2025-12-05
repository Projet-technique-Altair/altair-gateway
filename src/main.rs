use axum::Router;
use tokio::net::TcpListener;

mod routes;
mod state;
mod services;
mod error;
mod utils;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let state = state::AppState::new();

    let app = routes::init_routes()
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Cannot bind port 3000");

    println!("Gateway running on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("Failed to start Gateway");
}
