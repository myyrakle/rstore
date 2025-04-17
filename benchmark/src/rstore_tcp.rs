use futures::executor::block_on;
use rstore::{
    client::ConnectionConfig,
    protocol::{GetRequest, SetRequest},
};

use crate::KeyValueStore;

#[derive(Clone)]
pub struct RStoreClient {
    pub client: rstore::client::RStoreClient,
}

impl RStoreClient {
    pub fn new() -> anyhow::Result<RStoreClient> {
        let client = rstore::client::RStoreClient::new(ConnectionConfig {
            host: "localhost".to_string(),
            port: 13536,
            ..Default::default()
        });

        block_on(client.connect())?;

        Ok(RStoreClient { client })
    }
}

#[async_trait::async_trait]
impl KeyValueStore for RStoreClient {
    async fn set_key_value(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        self.client
            .set(SetRequest {
                key: key.to_string(),
                value: value.to_string(),
            })
            .await?;

        Ok(())
    }

    async fn get_key_value(&mut self, key: &str) -> anyhow::Result<String> {
        let response = self
            .client
            .get(GetRequest {
                key: key.to_string(),
            })
            .await?;

        Ok(response.value)
    }

    async fn clear_all(&mut self) -> anyhow::Result<()> {
        self.client.clear().await?;

        Ok(())
    }
}
