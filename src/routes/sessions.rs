use axum::{
    Router,
    routing::{get, post},
    extract::State,
    Json,
};
use serde_json::{json, Value};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/", get(list_sessions))
        .route("/start", post(start_session))
        .route("/stop", post(stop_session))
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "gateway-sessions"
    }))
}

/// GET /sessions
async fn list_sessions(
    State(state): State<AppState>,
) -> Json<Value> {
    let resp = state.sessions
        .get("/sessions")
        .await
        .unwrap_or_else(|_| json!({
            "error": "sessions-ms unreachable"
        }));

    Json(resp)
}

/// POST /sessions/start
async fn start_session(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let resp = state.sessions
        .post("/sessions/start", payload)
        .await
        .unwrap_or_else(|_| json!({
            "error": "sessions-ms unreachable"
        }));

    Json(resp)
}

/// POST /sessions/stop
async fn stop_session(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let resp = state.sessions
        .post("/sessions/stop", payload)
        .await
        .unwrap_or_else(|_| json!({
            "error": "sessions-ms unreachable"
        }));

    Json(resp)
}
