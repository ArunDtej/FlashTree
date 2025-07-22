use std::sync::{Arc, RwLock};
use dashmap::DashMap;
use serde_json::Error;

#[derive(Debug, Clone)]
pub enum Value {
    Text(String),
    Bool(bool),
}

#[derive(Debug)]
pub struct Node {
    pub value: Option<Value>,
    pub ttl: Option<u64>,
    pub children: Option<DashMap<String, Arc<RwLock<Node>>>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            value: None,
            ttl: None,
            children: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Database {
    root: Arc<RwLock<Node>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            root: Arc::new(RwLock::new(Node::new())),
        }
    }

    pub fn get_root(&self) -> &Arc<RwLock<Node>> {
        &self.root
    }

    // Core API (implement real logic here)
    pub fn set(&self, key: &str, value: Value) -> Result<(), String> {
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Value>, String> {
        Ok(None)
    }

    pub fn delete(&self, key: &str) -> Result<bool, String> {
        todo!("Implement DELETE operation")
    }

    pub fn drop_all(&self){
        let mut root = self.root.write().unwrap();
        root.value = None;
        root.ttl = None;
        root.children = None;
    }

    // ────── String-only adapters for command interface ──────

    /// "String-only" setter for use by commands module
    pub fn set_str(&self, key: &str, value: String) -> Result<(), String> {
        self.set(key, Value::Text(value))
    }

    /// "String-only" getter for use by commands module
    pub fn get_str(&self, key: &str) -> Result<Option<String>, String> {
        match self.get(key)? {
            Some(Value::Text(s)) => Ok(Some(s)),
            Some(Value::Bool(b)) => Ok(Some(b.to_string())),
            None                => Ok(None),
        }
    }
}
