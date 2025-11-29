use axum::{
    Router,
    routing::get,
    extract::State,
    Json,
};
use serde_json::Value;
use crate::state::AppState;

pub fn users_routes() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

async fn me(
    State(state): State<AppState>
) -> Json<Value> {
    let url = format!("{}/me", state.users_url);

    let resp = reqwest::get(url)
        .await
        .expect("Failed to reach Users MS")
        .json::<Value>()
        .await
        .expect("Failed to parse Users MS response");

    Json(resp)
}
