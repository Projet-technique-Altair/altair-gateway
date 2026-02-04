use axum::{
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use chrono::Utc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct Meta {
    pub request_id: String,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorBody,
    pub meta: Meta,
}

#[allow(dead_code)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}


impl ApiError {
    fn new(
        status: StatusCode,
        code: &str,
        message: String,
    ) -> Self {
        Self {
            status,
            code: code.to_string(),
            message,
            details: None,
        }
    }

    pub fn upstream_unavailable(service: &str) -> Self {
        Self::new(
            StatusCode::BAD_GATEWAY,
            "UPSTREAM_UNAVAILABLE",
            format!("{service} service unreachable"),
        )
    }

    pub fn upstream_timeout(service: &str) -> Self {
        Self::new(
            StatusCode::GATEWAY_TIMEOUT,
            "UPSTREAM_TIMEOUT",
            format!("{service} service timeout"),
        )
    }

    pub fn upstream_invalid_response(service: &str) -> Self {
        Self::new(
            StatusCode::BAD_GATEWAY,
            "UPSTREAM_INVALID_RESPONSE",
            format!("{service} returned invalid response"),
        )
    }

    pub fn from_upstream_status(status: StatusCode) -> Self {
        let code = match status {
            StatusCode::NOT_FOUND => "RESOURCE_NOT_FOUND",
            StatusCode::UNAUTHORIZED => "UNAUTHORIZED",
            StatusCode::FORBIDDEN => "FORBIDDEN",
            StatusCode::BAD_REQUEST => "BAD_REQUEST",
            _ => "UPSTREAM_ERROR",
        };

        Self::new(
            status,
            code,
            format!("Upstream service returned {status}"),
        )
    }

    pub fn forbidden(message: &str) -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            message.to_string(),
        )
    }

    pub fn unauthorized(message: &str) -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED",
            message.to_string(),
        )
    }

}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = ErrorResponse {
            success: false,
            error: ErrorBody {
                code: self.code,
                message: self.message,
                details: None,
            },
            meta: Meta {
                request_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().to_rfc3339(),
            },
        };

        (self.status, Json(body)).into_response()
    }
}