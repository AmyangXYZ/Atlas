use std::collections::HashMap;

pub trait Cache {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&mut self, key: &str, value: &[u8]);
    fn delete(&mut self, key: &str);
}

#[derive(Debug, Clone, Copy)]
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

pub struct InMemoryCache {
    map: HashMap<String, Vec<u8>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl Cache for InMemoryCache {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.map.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &[u8]) {
        self.map.insert(key.to_string(), value.to_vec());
    }

    fn delete(&mut self, key: &str) {
        self.map.remove(key);
    }
}
