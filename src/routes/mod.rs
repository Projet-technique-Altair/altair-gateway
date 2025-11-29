pub mod labs;
pub mod sessions;
pub mod users;
pub mod webshell;

use axum::Router;
use crate::state::AppState;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        .merge(labs::labs_routes())
        .merge(sessions::sessions_routes())
        .merge(users::users_routes())
        .merge(webshell::webshell_routes())
}
