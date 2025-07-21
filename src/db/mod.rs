
use std::sync::{Arc, RwLock};
use dashmap::DashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    UInt8(u8),
    UInt16(u16), 
    UInt32(u32),
    UInt64(u64),
    Float(f64),
    Text(String),
    Bool(bool),
}

#[derive(Debug)]
pub struct Node {
    pub value: Option<Value>,
    pub ttl: Option<u64>, // Unix timestamp
    pub children: Option<DashMap<String, Arc<RwLock<Node>>>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            value: None,
            ttl: None,
            children: Some(DashMap::new()),
        }
    }
}

// Database handle that wraps the root node
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

    // TODO: Implement core operations
    pub fn set(&self, key: &str, value: Value) -> Result<(), String> {
        // Implementation will go here
        todo!("Implement SET operation")
    }

    pub fn get(&self, key: &str) -> Result<Option<Value>, String> {
        // Implementation will go here
        todo!("Implement GET operation")
    }

    pub fn delete(&self, key: &str) -> Result<bool, String> {
        // Implementation will go here
        todo!("Implement DELETE operation")
    }
}