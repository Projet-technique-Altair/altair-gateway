use axum::{Router, routing::get, Json};
use serde_json::json;
use crate::state::AppState;

pub fn users_routes() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

async fn me() -> Json<serde_json::Value> {
    Json(json!({
        "user_id": "user-123",
        "name": "Nikita",
        "role": "student"
    }))
}
