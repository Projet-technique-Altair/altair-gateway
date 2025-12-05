use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct UsersApi {
    base: String,
    client: Client,
}

impl UsersApi {
    pub fn new(base: String) -> Self {
        Self { base, client: Client::new() }
    }

    pub async fn get(&self, endpoint: &str) -> Result<Value, reqwest::Error> {
        Ok(self.client
            .get(format!("{}{}", self.base, endpoint))
            .send()
            .await?
            .json::<Value>()
            .await?)
    }
}
