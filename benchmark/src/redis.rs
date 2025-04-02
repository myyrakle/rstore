use redis::{Commands, Connection};

use crate::KeyValueStore;

pub struct RedisClient {
    pub connection: Connection,
}

impl RedisClient {
    pub fn new() -> anyhow::Result<RedisClient> {
        // connect to redis
        let client = redis::Client::open("redis://0.0.0.0:16379/")?;
        let mut connection = client.get_connection()?;

        let _: () = connection.ping()?;

        Ok(RedisClient { connection })
    }
}

impl KeyValueStore for RedisClient {
    fn set_key_value(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        // set key value
        let _: () = self.connection.set(key, value)?;

        Ok(())
    }

    fn get_key_value(&mut self, key: &str) -> anyhow::Result<String> {
        let value: String = self.connection.get(key)?;

        Ok(value)
    }

    fn clear_all(&mut self) -> anyhow::Result<()> {
        // clear all
        let all_keys: Vec<String> = self.connection.keys("*")?;
        for key in all_keys {
            let _: () = self.connection.del(key)?;
        }

        Ok(())
    }
}
