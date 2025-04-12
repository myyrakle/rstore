use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct KVEngine {
    kv: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum KVError {
    #[error("Key not found")]
    KeyNotFound,
    #[error("lock failed")]
    LockFailed,
}

pub type KVResult<T> = std::result::Result<T, KVError>;

impl KVEngine {
    pub fn new() -> Self {
        KVEngine {
            kv: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_key_value(&self, key: String, value: String) -> KVResult<()> {
        let Ok(mut kv) = self.kv.lock() else {
            if self.kv.is_poisoned() {
                self.kv.clear_poison();
            }
            return Err(KVError::LockFailed);
        };
        kv.insert(key, value);
        Ok(())
    }

    pub fn get_key_value(&self, key: &str) -> KVResult<String> {
        let Ok(kv) = self.kv.lock() else {
            if self.kv.is_poisoned() {
                self.kv.clear_poison();
            }
            return Err(KVError::LockFailed);
        };
        match kv.get(key) {
            Some(value) => Ok(value.to_owned()),
            None => Err(KVError::KeyNotFound),
        }
    }

    pub fn delete_key_value(&self, key: &str) -> KVResult<()> {
        let Ok(mut kv) = self.kv.lock() else {
            if self.kv.is_poisoned() {
                self.kv.clear_poison();
            }
            return Err(KVError::LockFailed);
        };
        if kv.remove(key).is_none() {
            return Err(KVError::KeyNotFound);
        }
        Ok(())
    }

    pub fn clear_all(&self) -> KVResult<()> {
        let Ok(mut kv) = self.kv.lock() else {
            if self.kv.is_poisoned() {
                self.kv.clear_poison();
            }
            return Err(KVError::LockFailed);
        };
        kv.clear();
        Ok(())
    }
}
