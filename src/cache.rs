use std::collections::HashMap;

pub trait Cache {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn set(&mut self, key: &str, value: &[u8]);
    fn delete(&mut self, key: &str);
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
