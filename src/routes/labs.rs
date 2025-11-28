use axum::{
    Router,
    routing::get,
    extract::{Path, State},
    Json,
};
use serde_json::json;

use crate::state::AppState;

pub fn labs_routes() -> Router<AppState> {
    Router::new()
        .route("/labs", get(list_labs))
        .route("/labs/:id", get(get_lab))
}

async fn list_labs(
    State(state): State<AppState>
) -> Json<serde_json::Value> {
    let labs = state.labs.lock().unwrap().clone();
    Json(json!(labs))
}

async fn get_lab(
    Path(id): Path<String>,
    State(state): State<AppState>
) -> Json<serde_json::Value> {
    let labs = state.labs.lock().unwrap();
    let lab = labs.iter().find(|l| l.lab_id == id).cloned();
    Json(json!(lab))
}
