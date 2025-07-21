use std::sync::{Arc, RwLock};
use dashmap::DashMap;

enum Value {
    Int(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float(f64),
    Text(String),
    Bool(bool),
}

struct Node {
    value: Option<Value>,
    ttl: Option<u64>,
    children: Option<DashMap<String, Arc<RwLock<Node>>>>,
}

fn main() {
    println!("FlashTree Node initialized.");
}
