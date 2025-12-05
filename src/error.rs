use axum::{Json, response::IntoResponse};
use serde_json::json;

pub struct ApiError {
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(json!({ "error": self.message }));
        (axum::http::StatusCode::BAD_GATEWAY, body).into_response()
    }
}
