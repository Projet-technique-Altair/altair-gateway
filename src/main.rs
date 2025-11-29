use axum::{Router};
use std::env;
use tokio::net::TcpListener;

mod state;
mod routes;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let state = AppState {
        users_url: env::var("USERS_MS_URL").unwrap(),
        labs_url: env::var("LABS_MS_URL").unwrap(),
        sessions_url: env::var("SESSIONS_MS_URL").unwrap(),
    };

    let app = Router::new()
        .merge(routes::labs::labs_routes())
        .merge(routes::users::users_routes())
        .merge(routes::sessions::sessions_routes())
        .merge(routes::webshell::webshell_routes())
        .with_state(state);

    // ➜ Axum 0.7 façon correcte de lancer un serveur
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}
