use dashmap::DashMap;
use std::sync::{Arc, RwLock};
// use serde_json::Error;
// use std::collections::HashSet;
pub mod core;
pub use core::Value; 


#[derive(Debug, Clone)]
pub struct Database {
    root: Arc<RwLock<core::Node>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            root: Arc::new(RwLock::new(core::Node::new())),
        }
    }

    #[inline]
    pub fn get_root(&self) -> &Arc<RwLock<core::Node>> {
        &self.root
    }

    pub fn set(&self, key: &str, value: Value) -> Result<(), String> {
        core::set(&self.root, key, value)
    }

    pub fn get(&self, key: &str) -> Result<Option<core::Value>, String> {
        core::get(&self.root, key)
    }

    pub fn delete(&self, key: &str) -> Result<bool, String> {
        core::delete(&self.root, key)
    }

    pub fn drop_all(&self) {
        let mut root = self.root.write().unwrap();
        root.value = None;
        root.ttl = None;
        root.children = None;
    }
    
}
