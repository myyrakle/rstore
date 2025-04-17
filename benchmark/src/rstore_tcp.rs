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
            port: 13535,
            ..Default::default()
        });

        block_on(client.connect())?;

        Ok(RStoreClient { client })
    }
}

impl KeyValueStore for RStoreClient {
    fn set_key_value(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        block_on(self.client.set(SetRequest {
            key: key.to_string(),
            value: value.to_string(),
        }))?;

        Ok(())
    }

    fn get_key_value(&mut self, key: &str) -> anyhow::Result<String> {
        let response = block_on(self.client.get(GetRequest {
            key: key.to_string(),
        }))?;

        Ok(response.value)
    }

    fn clear_all(&mut self) -> anyhow::Result<()> {
        block_on(self.client.clear())?;

        Ok(())
    }
}
