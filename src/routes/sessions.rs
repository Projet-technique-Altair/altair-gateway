use axum::{
    Router,
    routing::{post, get, delete},
    Json,
    extract::{Path, State},
};
use serde::{Serialize, Deserialize};
use serde_json::json;
use uuid::Uuid;


use crate::state::{AppState, LabSession};

pub fn sessions_routes() -> Router<AppState> {
    Router::new()
        .route("/labs/:id/start", post(start_session))
        .route("/sessions/user/:id", get(user_sessions))
        .route("/sessions/:id", get(get_session).delete(delete_session))
}

#[derive(Deserialize)]
pub struct StartBody {
    pub user_id: Option<String>,
}

async fn start_session(
    Path(lab_id): Path<String>,
    State(state): State<AppState>,
    Json(body): Json<StartBody>,
) -> Json<serde_json::Value> {
    let session = LabSession {
        session_id: Uuid::new_v4().to_string(),
        user_id: body.user_id.unwrap_or("user-123".into()),
        lab_id,
        container_id: "mock-container".into(),
        status: "running".into(),
        webshell_url: "ws://localhost:3000/webshell/mock".into(),
    };

    state.sessions.lock().unwrap().push(session.clone());
    Json(json!(session))
}

async fn get_session(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let sessions = state.sessions.lock().unwrap();
    let session = sessions.iter().find(|s| s.session_id == id).cloned();
    Json(json!(session))
}

async fn user_sessions(
    Path(user_id): Path<String>,
    State(state): State<AppState>
) -> Json<serde_json::Value> {
    let sessions = state.sessions.lock().unwrap();
    let result: Vec<_> = sessions.iter().filter(|s| s.user_id == user_id).cloned().collect();
    Json(json!(result))
}

async fn delete_session(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let mut sessions = state.sessions.lock().unwrap();
    sessions.retain(|s| s.session_id != id);
    Json(json!({ "status": "stopped" }))
}
