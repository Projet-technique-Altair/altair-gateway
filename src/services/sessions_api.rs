use serde_json::Value;
use crate::services::common::HttpClient;

#[derive(Clone)]
pub struct SessionsApi {
    base_url: String,
    http: HttpClient,
}

impl SessionsApi {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            http: HttpClient::new(),
        }
    }

    pub async fn get(&self, path: &str) -> reqwest::Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        self.http.get_json(&url).await
    }

    pub async fn post(
        &self,
        path: &str,
        body: Value,
    ) -> reqwest::Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        self.http.post_json(&url, &body).await
    }
}
