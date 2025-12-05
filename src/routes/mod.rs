use axum::Router;
use crate::state::AppState;

pub mod users;
pub mod labs;
pub mod sessions;
pub mod webshell;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .nest("/users", users::routes())
        .nest("/labs", labs::routes())
        .nest("/sessions", sessions::routes())
        .nest("/webshell", webshell::routes())
}
