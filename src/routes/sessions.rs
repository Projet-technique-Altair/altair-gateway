use axum::{
    Router,
    routing::{post, get},
    extract::{Path, State},
    Json,
};
use serde_json::Value;
use crate::state::AppState;

pub fn sessions_routes() -> Router<AppState> {
    Router::new()
        .route("/labs/:id/start", post(start_session))
        .route("/sessions", get(list_sessions))
}

async fn start_session(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Json<Value> {
    let url = format!("{}/labs/{}/start", state.sessions_url, id);

    let resp = reqwest::Client::new()
        .post(url)
        .json(&serde_json::json!({ "user_id": "user-123" }))
        .send()
        .await
        .expect("Failed to reach Sessions MS")
        .json::<Value>()
        .await
        .expect("Failed to parse Sessions MS response");

    Json(resp)
}

async fn list_sessions(
    State(state): State<AppState>
) -> Json<Value> {
    let url = format!("{}/sessions", state.sessions_url);

    let resp = reqwest::get(url)
        .await
        .expect("Failed to reach Sessions MS")
        .json::<Value>()
        .await
        .expect("Failed to parse Sessions MS response");

    Json(resp)
}
