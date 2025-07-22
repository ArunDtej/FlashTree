// use dashmap::DashMap;
// use std::collections::HashSet;
// use std::sync::{Arc, RwLock};

// pub fn node_count(node: &Arc<RwLock<Node>>) -> usize {
//     let guard = node.read().unwrap();
//     let mut count = 1; // Count self

//     if let Some(ref children) = guard.children {
//         for entry in children.iter() {
//             count += node_count(entry.value());
//         }
//     }

//     count
// }


// pub fn value_size(val: &Value) -> usize {
//     use std::mem::size_of_val;
//     match val {
//         Value::Text(s) => size_of_val(s) + s.capacity(),
//         Value::List(vec) => {
//             // Vec allocates heap for strings:
//             let mut total = size_of_val(vec) + vec.capacity() * std::mem::size_of::<String>();
//             // Also count heap for each string element
//             for s in vec {
//                 total += std::mem::size_of_val(s) + s.capacity();
//             }
//             total
//         }
//         Value::Set(set) => {
//             // HashSet allocates a bucket array for pointers (not accessible directly).
//             // We'll fudge a bit: struct, plus buckets (16 bytes per elem), plus inner strings.
//             let mut total = size_of_val(set) + set.capacity() * 16;
//             for s in set {
//                 total += std::mem::size_of_val(s) + s.capacity();
//             }
//             total
//         }
//     }
// }

// pub fn node_size_bytes(node: &Arc<RwLock<Node>>) -> usize {
//     use std::mem::size_of_val;
//     let guard = node.read().unwrap();
//     let mut size = 0;

//     // Arc<RwLock<Node>> overhead (only if you want this included at every call)
//     size += std::mem::size_of::<Arc<RwLock<Node>>>();
//     // RwLock itself
//     size += std::mem::size_of::<RwLock<Node>>();
//     // Node struct
//     size += std::mem::size_of::<Node>();

//     // Value in this node (Option is part of the Node struct already)
//     if let Some(ref value) = guard.value {
//         size += value_size(value);
//     }

//     // Children
//     if let Some(ref children) = guard.children {
//         // DashMap struct header
//         size += size_of_val(children);

//         // DashMap buckets and internal structure (estimate!)
//         size += 512; // fudge for shards/locks/etc.
//         size += children.len() * 32; // per-entry overhead

//         // Keys and recursive child nodes
//         for entry in children.iter() {
//             let key = entry.key();
//             size += size_of_val(key) + key.capacity();
//             size += node_size_bytes(entry.value());
//         }
//     }

//     size
// }



// #[derive(Debug, Clone)]
// pub enum Value {
//     Text(String),
//     List(Vec<String>),
//     Set(HashSet<String>),
// }

// #[derive(Debug)]
// pub struct Node {
//     pub value: Option<Value>,
//     pub ttl: Option<u64>,
//     pub children: Option<DashMap<String, Arc<RwLock<Node>>>>,
// }

// impl Node {
//     pub fn new() -> Self {
//         Node {
//             value: None,
//             ttl: None,
//             children: None,
//         }
//     }
// }

// #[inline]
// pub fn delete(root: &Arc<RwLock<Node>>, key: &str) -> Result<bool, String> {
//     Ok(true)
// }

// pub fn set(root: &Arc<RwLock<Node>>, key: &str, value: Value) -> Result<(), String> {

//     let path: Vec<&str> = if key.is_empty() {
//         Vec::new()
//     } else {
//         key.split(':').collect()
//     };

//     if path.is_empty() {
//         let mut guard = root.write().map_err(|_| "Lock poisoned")?;
//         guard.value = Some(value);
//         return Ok(());
//     }

//     let mut current = root.clone();

//     for (i, part) in path.iter().enumerate() {
//         {
//             let needs_children = {
//                 let guard = current.read().map_err(|_| "Lock poisoned")?;
//                 guard.children.is_none()
//             };

//             if needs_children {
//                 let mut guard = current.write().map_err(|_| "Lock poisoned")?;
//                 if guard.children.is_none() {
//                     guard.children = Some(DashMap::new());
//                 }
//             }
//         }

//         let next = {
//             let next;
//             {
//                 let guard = current.read().map_err(|_| "Lock poisoned")?;
//                 let children = guard.children.as_ref().unwrap();
//                 next = children
//                     .entry(part.to_string())
//                     .or_insert_with(|| Arc::new(RwLock::new(Node::new())))
//                     .clone();
//             }
//             next
//         };
//         current = next;

//         if i == path.len() - 1 {
//             let mut guard = current.write().map_err(|_| "Lock poisoned")?;
//             guard.value = Some(value);
//             return Ok(());
//         }
//     }

//     Ok(())
// }

// pub fn get(root: &Arc<RwLock<Node>>, key: &str) -> Result<Option<Value>, String> {

//     let path: Vec<&str> = if key.is_empty() {
//         Vec::new()
//     } else {
//         key.split(':').collect()
//     };

//     let mut current = root.clone();

//     for &part in &path {
//         println!("{part}");
//         let guard = current.read().map_err(|_| "Lock poisoned")?;
//         let children = match guard.children.as_ref() {
//             Some(children) => children,
//             None => return Ok(None),
//         };
//         let next = match children.get(part) {
//             Some(child_arc) => child_arc.clone(),
//             None => return Ok(None),
//         };

//         drop(guard);
//         current = next;
//     }

//     let guard = current.read().map_err(|_| "Lock poisoned")?;
//     Ok(guard.value.clone())
// }

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum Value {
    Text(String),
    List(Vec<String>),
    Set(HashSet<String>),
}

#[derive(Debug)]
pub struct Node {
    pub v: Option<Value>,
    pub t: Option<u64>,
    pub c: Option<HashMap<String, Arc<RwLock<Node>>>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            v: None,
            t: None,
            c: None,
        }
    }
}

pub fn node_count(node: &Arc<RwLock<Node>>) -> usize {
    let guard = node.read().unwrap();
    let mut count = 1; // Count self

    if let Some(ref children) = guard.c {
        for child in children.values() {
            count += node_count(child);
        }
    }
    count
}

pub fn value_size(val: &Value) -> usize {
    use std::mem::size_of_val;
    match val {
        Value::Text(s) => size_of_val(s) + s.capacity(),
        Value::List(vec) => {
            let mut total = size_of_val(vec) + vec.capacity() * std::mem::size_of::<String>();
            for s in vec {
                total += size_of_val(s) + s.capacity();
            }
            total
        }
        Value::Set(set) => {
            let mut total = size_of_val(set) + set.capacity() * 16;
            for s in set {
                total += size_of_val(s) + s.capacity();
            }
            total
        }
    }
}

pub fn node_size_bytes(node: &Arc<RwLock<Node>>) -> usize {
    use std::mem::size_of_val;
    let guard = node.read().unwrap();
    let mut size = 0;
    size += std::mem::size_of::<Arc<RwLock<Node>>>();
    size += std::mem::size_of::<RwLock<Node>>();
    size += std::mem::size_of::<Node>();
    if let Some(ref value) = guard.v {
        size += value_size(value);
    }
    if let Some(ref children) = guard.c {
        size += size_of_val(children);
        size += children.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<Arc<RwLock<Node>>>());
        for (key, child) in children.iter() {
            size += size_of_val(key) + key.capacity();
            size += node_size_bytes(child);
        }
    }
    size
}

pub fn set(root: &Arc<RwLock<Node>>, key: &str, value: Value) -> Result<(), String> {
    let path: Vec<&str> = if key.is_empty() {
        Vec::new()
    } else {
        key.split(':').collect()
    };
    if path.is_empty() {
        let mut guard = root.write().map_err(|_| "Lock poisoned")?;
        guard.v = Some(value);
        return Ok(());
    }
    let mut current = root.clone();
    for (i, part) in path.iter().enumerate() {
        {
            let needs_children = {
                let guard = current.read().map_err(|_| "Lock poisoned")?;
                guard.c.is_none()
            };
            if needs_children {
                let mut guard = current.write().map_err(|_| "Lock poisoned")?;
                if guard.c.is_none() {
                    guard.c = Some(HashMap::new());
                }
            }
        }
        let next = {
            let mut guard = current.write().map_err(|_| "Lock poisoned")?;
            let children = guard.c.as_mut().unwrap();
            children
                .entry(part.to_string())
                .or_insert_with(|| Arc::new(RwLock::new(Node::new())))
                .clone()
        };
        current = next;
        if i == path.len() - 1 {
            let mut guard = current.write().map_err(|_| "Lock poisoned")?;
            guard.v = Some(value);
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
        let guard = current.read().map_err(|_| "Lock poisoned")?;
        let children = match guard.c.as_ref() {
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
    Ok(guard.v.clone())
}

#[inline]
pub fn delete(_root: &Arc<RwLock<Node>>, _key: &str) -> Result<bool, String> {
    Ok(true)
}
