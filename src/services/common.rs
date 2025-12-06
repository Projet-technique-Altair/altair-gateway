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

    pub async fn get_json(&self, url: &str) -> reqwest::Result<Value> {
        let resp = self.client.get(url).send().await?;
        Ok(resp.json::<Value>().await?)
    }

    pub async fn post_json(&self, url: &str, body: &Value) -> reqwest::Result<Value> {
        let resp = self.client.post(url).json(body).send().await?;
        Ok(resp.json::<Value>().await?)
    }
}
