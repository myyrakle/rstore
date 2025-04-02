use reqwest::blocking::Client;

use crate::KeyValueStore;

#[derive(Clone)]
pub struct RStoreClient {
    pub client: Client,
}

impl RStoreClient {
    pub fn new() -> anyhow::Result<RStoreClient> {
        let client = Client::new();

        // Check if the server is running
        client.get("http://localhost:13535/").send()?;

        Ok(RStoreClient { client })
    }
}

impl KeyValueStore for RStoreClient {
    fn set_key_value(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        let request_body = format!("{{\"key\": \"{}\", \"value\": \"{}\"}}", key, value);
        println!("Request body: {}", request_body);

        self.client
            .post("http://localhost:13535/value")
            .header("Content-Type", "application/json")
            .body(request_body)
            .send()?;

        Ok(())
    }

    fn get_key_value(&mut self, key: &str) -> anyhow::Result<String> {
        let response = self
            .client
            .get(format!("http://localhost:13535/value?key={key}"))
            .send()?;

        let response_body = response.text()?;
        println!("Response body: {}", response_body);

        let response: RStoreGetResponse = serde_json::from_str(&response_body)?;
        let value = response.value;

        Ok(value)
    }

    fn clear_all(&mut self) -> anyhow::Result<()> {
        self.client.delete("http://localhost:13535/clear").send()?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
pub struct RStoreGetResponse {
    pub value: String,
}
