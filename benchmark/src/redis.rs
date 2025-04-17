use std::sync::Arc;

use redis::{Commands, Connection};

use crate::KeyValueStore;

pub struct RedisClient {
    pub redis_client: Arc<redis::Client>,
    pub connection: Connection,
}

unsafe impl Send for RedisClient {}
unsafe impl Sync for RedisClient {}

impl Clone for RedisClient {
    fn clone(&self) -> Self {
        RedisClient {
            redis_client: Arc::clone(&self.redis_client),
            connection: self.redis_client.get_connection().unwrap(),
        }
    }
}

impl RedisClient {
    pub fn new() -> anyhow::Result<RedisClient> {
        // connect to redis
        let client = redis::Client::open("redis://0.0.0.0:16379/")?;
        let mut connection = client.get_connection()?;

        let _: () = connection.ping()?;

        Ok(RedisClient {
            connection,
            redis_client: Arc::new(client),
        })
    }
}

#[async_trait::async_trait]
impl KeyValueStore for RedisClient {
    async fn set_key_value(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        // set key value
        let _: () = self.connection.set(key, value)?;

        Ok(())
    }

    async fn get_key_value(&mut self, key: &str) -> anyhow::Result<String> {
        let value: String = self.connection.get(key)?;

        Ok(value)
    }

    async fn clear_all(&mut self) -> anyhow::Result<()> {
        // clear all
        let all_keys: Vec<String> = self.connection.keys("*")?;
        for key in all_keys {
            let _: () = self.connection.del(key)?;
        }

        Ok(())
    }
}
