use axum::{
    Router,
    routing::get,
    extract::State,
    Json,
};
use serde_json::Value;
use crate::state::AppState;

pub fn labs_routes() -> Router<AppState> {
    Router::new()
        .route("/labs", get(list_labs))
}

async fn list_labs(
    State(state): State<AppState>
) -> Json<Value> {
    let url = format!("{}/labs", state.labs_url);

    let resp = reqwest::get(url)
        .await
        .expect("Failed to reach Labs MS")
        .json::<Value>()
        .await
        .expect("Failed to parse Labs MS response");

    Json(resp)
}
