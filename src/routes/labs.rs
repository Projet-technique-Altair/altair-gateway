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
        .route("/:id", get(get_lab_by_id))
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
        .unwrap_or_else(|_| json!({ "error": "labs-ms unreachable" }));

    Json(json!({
        "status": "ok",
        "data": resp
    }))
}

async fn get_lab_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<Value> {
    let resp = state.labs
        .get(&format!("/labs/{id}"))
        .await
        .unwrap_or_else(|_| json!({ "error": "lab not found" }));

    Json(json!({
        "status": "ok",
        "data": resp
    }))
}
