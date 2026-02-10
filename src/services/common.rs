use crate::error::ApiError;
use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_json(&self, url: &str) -> Result<Value, ApiError> {
        let resp = self.client.get(url).send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::upstream_timeout("upstream")
            } else {
                ApiError::upstream_unavailable("upstream")
            }
        })?;
        Self::handle_response(resp).await
    }

    pub async fn get_json_with_headers(
        &self,
        url: &str,
        headers: &[(&str, &str)],
    ) -> Result<Value, ApiError> {
        let mut req = self.client.get(url);

        for (k, v) in headers {
            req = req.header(*k, *v);
        }

        let resp = req.send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::upstream_timeout("upstream")
            } else {
                ApiError::upstream_unavailable("upstream")
            }
        })?;

        Self::handle_response(resp).await
    }

    async fn handle_response(resp: reqwest::Response) -> Result<Value, ApiError> {
        let status = axum::http::StatusCode::from_u16(resp.status().as_u16())
            .unwrap_or(axum::http::StatusCode::BAD_GATEWAY);

        if status.is_success() {
            resp.json::<Value>()
                .await
                .map_err(|_| ApiError::upstream_invalid_response("upstream"))
        } else {
            Err(ApiError::from_upstream_status(status))
        }
    }

    pub async fn post_json(&self, url: &str, body: &Value) -> Result<Value, ApiError> {
        let resp = self.client.post(url).json(body).send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::upstream_timeout("upstream")
            } else {
                ApiError::upstream_unavailable("upstream")
            }
        })?;

        Self::handle_response(resp).await
    }

    pub async fn put_json(&self, url: &str, body: &Value) -> Result<Value, ApiError> {
        let resp = self.client.put(url).json(body).send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::upstream_timeout("upstream")
            } else {
                ApiError::upstream_unavailable("upstream")
            }
        })?;

        Self::handle_response(resp).await
    }

    pub async fn delete_json(&self, url: &str) -> Result<Value, ApiError> {
        let resp = self.client.delete(url).send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::upstream_timeout("upstream")
            } else {
                ApiError::upstream_unavailable("upstream")
            }
        })?;

        Self::handle_response(resp).await
    }
}
