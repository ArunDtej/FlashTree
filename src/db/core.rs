use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};


// For estimating heap usage of Value if it's a String or similar.
// Update this for your own value types if needed!
pub fn value_size(val: &Value) -> usize {
    // If Value is a String:
    // val.capacity()
    // If Value has fields, sum them; if it's Copy, just size_of::<Value>()
    std::mem::size_of_val(val)
}

// Recursively count the total heap size of the tree, starting from a node
pub fn node_size_bytes(node: &Arc<RwLock<Node>>) -> usize {
    let guard = node.read().unwrap();
    let mut size = std::mem::size_of::<Node>();
    size += std::mem::size_of::<Arc<RwLock<Node>>>(); // pointer, atomic, etc.

    // Value
    if let Some(ref value) = guard.value {
        size += value_size(value);
    }
    // Estimate RwLock's own heap (usually a few words)
    size += std::mem::size_of_val(&*node);

    // Children
    if let Some(ref children) = guard.children {
        // Add the DashMap struct (shards pointers etc)
        size += std::mem::size_of_val(children);
        // Optionally, estimate more (e.g., 16K or 32K per default DashMap!)
        // Traverse all entries
        for entry in children.iter() {
            let k = entry.key();
            size += std::mem::size_of_val(k) + k.capacity(); // String struct + heap
            let v = entry.value();
            size += node_size_bytes(v);
        }
        // Highly approximate: add rough per-shard/bucket overhead, e.g.:
        // size += children.len() / 32 * 1024; // 1KB per 32 entries (example)
    }
    size
}


#[derive(Debug, Clone)]
pub enum Value {
    Text(String),
    List(Vec<String>),
    Set(HashSet<String>),
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

#[inline]
pub fn delete(root: &Arc<RwLock<Node>>, key: &str) -> Result<bool, String> {
    Ok(true)
}

pub fn set(root: &Arc<RwLock<Node>>, key: &str, value: Value) -> Result<(), String> {

    let path: Vec<&str> = if key.is_empty() {
        Vec::new()
    } else {
        key.split(':').collect()
    };

    if path.is_empty() {
        let mut guard = root.write().map_err(|_| "Lock poisoned")?;
        guard.value = Some(value);
        return Ok(());
    }

    let mut current = root.clone();

    for (i, part) in path.iter().enumerate() {
        {
            let needs_children = {
                let guard = current.read().map_err(|_| "Lock poisoned")?;
                guard.children.is_none()
            };

            if needs_children {
                let mut guard = current.write().map_err(|_| "Lock poisoned")?;
                if guard.children.is_none() {
                    guard.children = Some(DashMap::new());
                }
            }
        }

        let next = {
            let next;
            {
                let guard = current.read().map_err(|_| "Lock poisoned")?;
                let children = guard.children.as_ref().unwrap();
                next = children
                    .entry(part.to_string())
                    .or_insert_with(|| Arc::new(RwLock::new(Node::new())))
                    .clone();
            }
            next
        };
        current = next;

        if i == path.len() - 1 {
            let mut guard = current.write().map_err(|_| "Lock poisoned")?;
            guard.value = Some(value);
            return Ok(());
        }
    }

    Ok(())
}

pub fn get(root: &Arc<RwLock<Node>>, key: &str) -> Result<Option<Value>, String> {

    let path: Vec<&str> = if key.is_empty() {
        Vec::new()
    } else {
        key.split(':').collect()
    };

    let mut current = root.clone();

    for &part in &path {
        println!("{part}");
        let guard = current.read().map_err(|_| "Lock poisoned")?;
        let children = match guard.children.as_ref() {
            Some(children) => children,
            None => return Ok(None),
        };
        let next = match children.get(part) {
            Some(child_arc) => child_arc.clone(),
            None => return Ok(None),
        };

        drop(guard);
        current = next;
    }

    let guard = current.read().map_err(|_| "Lock poisoned")?;
    Ok(guard.value.clone())
}
