pub mod health;
pub mod users;
pub mod labs;
pub mod sessions;
pub mod webshell;

use axum::Router;
use crate::state::AppState;
use axum::routing::get;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .nest("/users", users::routes())
        .nest("/labs", labs::routes())
        .nest("/sessions", sessions::routes())
        .nest("/webshell", webshell::routes())
}
