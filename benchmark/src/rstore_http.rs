use reqwest::Client;

use crate::KeyValueStore;

#[derive(Clone)]
pub struct RStoreClient {
    pub client: Client,
}

impl RStoreClient {
    pub fn new() -> anyhow::Result<RStoreClient> {
        let client = Client::new();

        // // Check if the server is running
        // client.get("http://localhost:13535/").send().await?;

        Ok(RStoreClient { client })
    }
}

#[async_trait::async_trait]
impl KeyValueStore for RStoreClient {
    async fn set_key_value(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        let request_body = format!("{{\"key\": \"{}\", \"value\": \"{}\"}}", key, value);

        self.client
            .post("http://localhost:13535/value")
            .header("Content-Type", "application/json")
            .body(request_body)
            .send()
            .await?;

        Ok(())
    }

    async fn get_key_value(&mut self, key: &str) -> anyhow::Result<String> {
        let response = self
            .client
            .get(format!("http://localhost:13535/value?key={key}"))
            .send()
            .await?;

        let response_body = response.text().await?;

        let response: RStoreGetResponse = serde_json::from_str(&response_body)?;
        let value = response.value;

        Ok(value)
    }

    async fn clear_all(&mut self) -> anyhow::Result<()> {
        self.client
            .delete("http://localhost:13535/clear")
            .send()
            .await?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
pub struct RStoreGetResponse {
    pub value: String,
}
