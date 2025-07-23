// use dashmap::DashMap;
// use std::sync::{Arc, RwLock};
// // use serde_json::Error;
// // use std::collections::HashSet;
// pub mod core;
// pub use core::Value; 


// #[derive(Debug, Clone)]
// pub struct Database {
//     root: Arc<RwLock<core::Node>>,
// }

// impl Database {
//     pub fn new() -> Self {
//         Database {
//             root: Arc::new(RwLock::new(core::Node::new())),
//         }
//     }

//     #[inline]
//     pub fn get_root(&self) -> &Arc<RwLock<core::Node>> {
//         &self.root
//     }

//     pub fn set(&self, key: &str, value: Value) -> Result<(), String> {
//         core::set(&self.root, key, value)
//     }

//     pub fn get(&self, key: &str) -> Result<Option<core::Value>, String> {
//         core::get(&self.root, key)
//     }

//     pub fn delete(&self, key: &str) -> Result<bool, String> {
//         core::delete(&self.root, key)
//     }

//     pub fn drop_all(&self) {
//         let mut root = self.root.write().unwrap();
//         root.v = None;
//         root.t = None;
//         root.c = None;
//     }

//     pub fn memory(&self) -> core::MemoryStats {
//         let (total, count, smallest, largest) = core::node_memory_stats(&self.root);
//         core::MemoryStats {
//             total_bytes: total,
//             node_count: count,
//             smallest_node: smallest,
//             largest_node: largest,
//         }
//     }

//     pub fn size(&self)->usize {
//         core::node_count(&self.root)
//     }

// }


use std::sync::RwLock;
pub mod core;
pub use core::Value;

/// The main handle to your in-memory database
#[derive(Debug)]
pub struct Database {
    root: RwLock<core::Node>,
}

impl Database {
    /// Create a new, empty database.
    pub fn new() -> Self {
        Database {
            root: RwLock::new(core::Node::new()),
        }
    }

    #[inline]
    pub fn get_root(&self) -> &RwLock<core::Node> {
        &self.root
    }

    /// Set a value, e.g. database.set("foo:bar", Value::Text("abc".to_string()))
    pub fn set(&self, key: &str, value: Value) -> Result<(), String> {
        core::set(&self.root, key, value)
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Result<Option<core::Value>, String> {
        core::get(&self.root, key)
    }

    /// Delete a key (or subtree)
    pub fn delete(&self, key: &str) -> Result<bool, String> {
        core::delete(&self.root, key)
    }

    /// Empty the whole database
    pub fn drop_all(&self) {
        let mut root = self.root.write().unwrap();
        root.v = None;
        root.t = None;
        root.c = None;
    }

    /// Memory statistics (total bytes, node count, min/max node size)
    pub fn memory(&self) -> core::MemoryStats {
        let (total, count, smallest, largest) = core::node_memory_stats(&self.root);
        core::MemoryStats {
            total_bytes: total,
            node_count: count,
            smallest_node: smallest,
            largest_node: largest,
        }
    }

    /// Total number of nodes
    pub fn size(&self) -> usize {
        core::node_count(&self.root)
    }
}
