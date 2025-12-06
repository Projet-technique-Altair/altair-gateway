use axum::{
    Router,
    routing::{get, post},
    Json,
    extract::{State, Json as AxumJson},
};
use serde_json::{json, Value};
use crate::state::AppState;

//
// ─────────────────────────────────────────────
//   ROUTES
// ─────────────────────────────────────────────
//
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/", get(list_sessions))
        .route("/start", post(start_session))
        .route("/stop", post(stop_session))
}

//
// ─────────────────────────────────────────────
//   HEALTHCHECK
// ─────────────────────────────────────────────
//
async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "gateway-sessions"
    }))
}

//
// ─────────────────────────────────────────────
//   INPUT STRUCTURES
// ─────────────────────────────────────────────
//
#[derive(serde::Deserialize, serde::Serialize)]
struct StartInput {
    user_id: String,
    lab_id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct StopInput {
    session_id: String,
}

//
// ─────────────────────────────────────────────
//   HANDLERS
// ─────────────────────────────────────────────
//

/// GET /sessions  → forward au Sessions-MS
async fn list_sessions(
    State(state): State<AppState>
) -> Json<Value> {

    let resp = state.sessions
        .get("/sessions")
        .await
        .unwrap_or_else(|_| json!({
            "error": "sessions-ms unreachable"
        }));

    Json(resp)
}

/// POST /sessions/start  → forward au Sessions-MS
async fn start_session(
    State(state): State<AppState>,
    AxumJson(input): AxumJson<StartInput>
) -> Json<Value> {

    let resp = state.sessions
        .post("/sessions/start", json!(input))
        .await
        .unwrap_or_else(|_| json!({
            "error": "sessions-ms unreachable"
        }));

    Json(resp)
}

/// POST /sessions/stop → forward au Sessions-MS
async fn stop_session(
    State(state): State<AppState>,
    AxumJson(input): AxumJson<StopInput>
) -> Json<Value> {

    let resp = state.sessions
        .post("/sessions/stop", json!(input))
        .await
        .unwrap_or_else(|_| json!({
            "error": "sessions-ms unreachable"
        }));

    Json(resp)
}
