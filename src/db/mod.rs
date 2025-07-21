
use std::sync::{Arc, RwLock};
use dashmap::DashMap;
use serde_json::Error;

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

    pub fn set(&self, key: &str, value: Value) -> Result<(), String> {
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Value>, String> {
        Ok((None))
    }

    pub fn delete(&self, key: &str) -> Result<bool, String> {
        todo!("Implement DELETE operation")
    }

    pub fn drop_all(&self){
        let mut root = self.root.write().unwrap();
        root.value = None;
        root.ttl = None;
        root.children = None;
        // Ok(true)
    }
}