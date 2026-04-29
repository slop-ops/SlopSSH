//! Credential storage and in-memory caching.

pub mod store;

use std::collections::HashMap;

/// Simple in-memory credential cache for ephemeral lookups.
pub struct CredentialCache {
    cache: HashMap<String, String>,
}

impl CredentialCache {
    /// Creates a new empty credential cache.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Retrieves a cached value by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.cache.get(key).map(|s| s.as_str())
    }

    /// Stores a value in the cache, replacing any existing entry for the same key.
    pub fn set(&mut self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    /// Removes a single entry from the cache.
    pub fn remove(&mut self, key: &str) {
        self.cache.remove(key);
    }

    /// Removes all entries from the cache.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for CredentialCache {
    fn default() -> Self {
        Self::new()
    }
}
