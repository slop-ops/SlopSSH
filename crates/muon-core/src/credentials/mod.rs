pub mod store;

use std::collections::HashMap;

pub struct CredentialCache {
    cache: HashMap<String, String>,
}

impl CredentialCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.cache.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    pub fn remove(&mut self, key: &str) {
        self.cache.remove(key);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for CredentialCache {
    fn default() -> Self {
        Self::new()
    }
}
