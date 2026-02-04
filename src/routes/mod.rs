use axum::{Router, routing::any};
use axum::middleware::{from_fn, from_fn_with_state};

use crate::state::AppState;
use crate::middleware::{jwt::jwt_middleware, rbac::rbac_middleware};
use crate::services::proxy::proxy;

pub mod health;

pub fn init_routes(state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/health", axum::routing::get(health::health));

    let protected = Router::new()
        .route("/:service/*rest", any(proxy))
        .layer(from_fn(rbac_middleware))
        .layer(from_fn_with_state(state.clone(), jwt_middleware));

    Router::new()
        .merge(public)
        .merge(protected)
}

