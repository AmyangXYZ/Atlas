use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait Cache {
    fn get(&mut self, key: &str) -> Option<Vec<u8>>;
    fn set(&mut self, key: &str, value: &[u8]);
    fn delete(&mut self, key: &str);
    fn metadata(&self) -> Vec<CachedDataMeta>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDataMeta {
    name: String,
    size: usize,
    last_updated: u64,
    last_accessed: u64,
    transactions: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CacheOperation {
    Set,
    Get,
    Delete,
}

impl From<u8> for CacheOperation {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Set,
            1 => Self::Get,
            2 => Self::Delete,
            _ => panic!("Invalid cache operation"),
        }
    }
}

#[derive(Debug)]
pub struct InMemoryCache {
    map: HashMap<String, (Vec<u8>, CachedDataMeta)>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl Cache for InMemoryCache {
    fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        if let Some((value, meta)) = self.map.get_mut(key) {
            meta.transactions += 1;
            meta.last_accessed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Some(value.clone())
        } else {
            None
        }
    }

    fn set(&mut self, key: &str, value: &[u8]) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some((existing_value, meta)) = self.map.get_mut(key) {
            // Update existing entry
            *existing_value = value.to_vec();
            meta.transactions += 1;
            meta.size = value.len();
            meta.last_updated = now;
        } else {
            // Insert new entry
            self.map.insert(
                key.to_string(),
                (
                    value.to_vec(),
                    CachedDataMeta {
                        name: key.to_string(),
                        size: value.len(),
                        last_updated: now,
                        last_accessed: 0,
                        transactions: 1,
                    },
                ),
            );
        }
    }

    fn delete(&mut self, key: &str) {
        self.map.remove(key);
    }

    fn metadata(&self) -> Vec<CachedDataMeta> {
        self.map.iter().map(|(_, value)| value.1.clone()).collect()
    }
}
