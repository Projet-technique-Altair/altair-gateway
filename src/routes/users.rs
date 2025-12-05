use axum::{
    Router,
    routing::get,
    Json,
    extract::State
};
use serde_json::{json, Value};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/me", get(get_me))
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "gateway-users"
    }))
}

async fn get_me(
    State(state): State<AppState>
) -> Json<Value> {

    // Appel vers users-ms via users_api.rs
    let resp = state.users
        .get("/me")
        .await
        .unwrap_or(json!({
            "error": "users-ms unreachable"
        }));

    Json(json!({
        "status": "ok",
        "data": resp
    }))
}
