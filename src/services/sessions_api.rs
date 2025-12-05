use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct SessionsApi {
    base: String,
    client: Client,
}

impl SessionsApi {
    pub fn new(base: String) -> Self {
        Self {
            base,
            client: Client::new(),
        }
    }

    pub async fn get(&self, endpoint: &str) -> Result<Value, reqwest::Error> {
        Ok(self.client
            .get(format!("{}{}", self.base, endpoint))
            .send()
            .await?
            .json::<Value>()
            .await?)
    }

    pub async fn post(&self, endpoint: &str, body: Value) -> Result<Value, reqwest::Error> {
        Ok(self.client
            .post(format!("{}{}", self.base, endpoint))
            .json(&body)
            .send()
            .await?
            .json::<Value>()
            .await?)
    }

    pub async fn delete(&self, endpoint: &str) -> Result<Value, reqwest::Error> {
        Ok(self.client
            .delete(format!("{}{}", self.base, endpoint))
            .send()
            .await?
            .json::<Value>()
            .await?)
    }
}
