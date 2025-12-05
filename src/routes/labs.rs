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
        .route("/", get(list_labs))
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "gateway-labs"
    }))
}

async fn list_labs(
    State(state): State<AppState>
) -> Json<Value> {

    let resp = state.labs
        .get("/labs")
        .await
        .unwrap_or(json!({
            "error": "labs-ms unreachable"
        }));

    Json(resp)
}
