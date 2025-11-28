mod routes;
mod state;
mod utils;

use axum::Router;
use crate::routes::{
    users::users_routes,
    labs::labs_routes,
    sessions::sessions_routes,
    webshell::webshell_routes,
};
use state::AppState;
use tower_http::cors::{CorsLayer, Any};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = Router::new()
        .merge(users_routes())
        .merge(labs_routes())
        .merge(sessions_routes())
        .merge(webshell_routes())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("🚀 ALTair Gateway running at http://{addr}");

    axum::serve(listener, app)
        .await
        .unwrap();
}
