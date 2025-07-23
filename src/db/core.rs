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


// hash map implementation - no dash map


// use std::collections::{HashMap, HashSet};
// use std::sync::{Arc, RwLock};

// #[derive(Debug, Clone)]
// pub enum Value {
//     Text(String),
//     List(Vec<String>),
//     Set(HashSet<String>),
// }

// #[derive(Debug)]
// pub struct Node {
//     pub v: Option<Value>,
//     pub t: Option<u64>,
//     pub c: Option<HashMap<String, Arc<RwLock<Node>>>>,
// }

// #[derive(Debug)]
// pub struct MemoryStats {
//     pub total_bytes: usize,
//     pub node_count: usize,
//     pub smallest_node: usize,
//     pub largest_node: usize,
// }


// impl Node {
//     pub fn new() -> Self {
//         Node {
//             v: None,
//             t: None,
//             c: None,
//         }
//     }
// }

// pub fn node_count(node: &Arc<RwLock<Node>>) -> usize {
//     let guard = node.read().unwrap();
//     let mut count = 1; // Count self

//     if let Some(ref children) = guard.c {
//         for child in children.values() {
//             count += node_count(child);
//         }
//     }
//     count
// }

// /// Returns (total_bytes, node_count, smallest_node, largest_node)
// pub fn node_memory_stats(node: &Arc<RwLock<Node>>) -> (usize, usize, usize, usize) {
//     use std::mem::size_of_val;
//     let guard = node.read().unwrap();
//     let mut size = 0;
//     size += std::mem::size_of::<Arc<RwLock<Node>>>();
//     size += std::mem::size_of::<RwLock<Node>>();
//     size += std::mem::size_of::<Node>();
//     if let Some(ref value) = guard.v {
//         size += value_size(value);
//     }
//     if let Some(ref children) = guard.c {
//         size += size_of_val(children);
//         size += children.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<Arc<RwLock<Node>>>());
//     }

//     // Initially, the smallest and largest are this node's size.
//     let mut smallest = size;
//     let mut largest = size;
//     let mut total = size;
//     let mut count = 1;

//     if let Some(ref children) = guard.c {
//         for (_key, child) in children.iter() {
//             let (child_total, child_count, child_smallest, child_largest) = node_memory_stats(child);
//             total += child_total;
//             count += child_count;
//             if child_smallest < smallest {
//                 smallest = child_smallest;
//             }
//             if child_largest > largest {
//                 largest = child_largest;
//             }
//         }
//     }
//     (total, count, smallest, largest)
// }


// pub fn value_size(val: &Value) -> usize {
//     use std::mem::size_of_val;
//     match val {
//         Value::Text(s) => size_of_val(s) + s.capacity(),
//         Value::List(vec) => {
//             let mut total = size_of_val(vec) + vec.capacity() * std::mem::size_of::<String>();
//             for s in vec {
//                 total += size_of_val(s) + s.capacity();
//             }
//             total
//         }
//         Value::Set(set) => {
//             let mut total = size_of_val(set) + set.capacity() * 16;
//             for s in set {
//                 total += size_of_val(s) + s.capacity();
//             }
//             total
//         }
//     }
// }

// pub fn set(root: &Arc<RwLock<Node>>, key: &str, value: Value) -> Result<(), String> {
//     let path: Vec<&str> = if key.is_empty() {
//         Vec::new()
//     } else {
//         key.split(':').collect()
//     };
//     if path.is_empty() {
//         let mut guard = root.write().map_err(|_| "Lock poisoned")?;
//         guard.v = Some(value);
//         return Ok(());
//     }
//     let mut current = root.clone();
//     for (i, part) in path.iter().enumerate() {
//         {
//             let needs_children = {
//                 let guard = current.read().map_err(|_| "Lock poisoned")?;
//                 guard.c.is_none()
//             };
//             if needs_children {
//                 let mut guard = current.write().map_err(|_| "Lock poisoned")?;
//                 if guard.c.is_none() {
//                     guard.c = Some(HashMap::new());
//                 }
//             }
//         }
//         let next = {
//             let mut guard = current.write().map_err(|_| "Lock poisoned")?;
//             let children = guard.c.as_mut().unwrap();
//             children
//                 .entry(part.to_string())
//                 .or_insert_with(|| Arc::new(RwLock::new(Node::new())))
//                 .clone()
//         };
//         current = next;
//         if i == path.len() - 1 {
//             let mut guard = current.write().map_err(|_| "Lock poisoned")?;
//             guard.v = Some(value);
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
//         let guard = current.read().map_err(|_| "Lock poisoned")?;
//         let children = match guard.c.as_ref() {
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
//     Ok(guard.v.clone())
// }

// #[inline]
// pub fn delete(_root: &Arc<RwLock<Node>>, _key: &str) -> Result<bool, String> {
//     Ok(true)
// }

use std::collections::{HashMap, HashSet};

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
    pub c: Option<HashMap<String, Box<Node>>>,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_bytes: usize,
    pub node_count: usize,
    pub smallest_node: usize,
    pub largest_node: usize,
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

pub fn node_count(node: &std::sync::RwLock<Node>) -> usize {
    fn count(node: &Node) -> usize {
        let mut c = 1;
        if let Some(children) = node.c.as_ref() {
            for child in children.values() {
                c += count(child);
            }
        }
        c
    }
    let guard = node.read().unwrap();
    count(&*guard)
}

// Returns (total_bytes, node_count, smallest, largest)
pub fn node_memory_stats(node: &std::sync::RwLock<Node>) -> (usize, usize, usize, usize) {
    use std::mem::size_of_val;
    fn stats(node: &Node) -> (usize, usize, usize, usize) {
        let mut size = size_of_val(node);
        if let Some(ref v) = node.v {
            size += value_size(v);
        }
        if let Some(ref children) = node.c {
            size += size_of_val(children);
            size += children.capacity()
                * (std::mem::size_of::<String>() + std::mem::size_of::<Box<Node>>());
        }
        let mut smallest = size;
        let mut largest = size;
        let mut total = size;
        let mut count = 1;
        if let Some(ref children) = node.c {
            for (_k, child) in children.iter() {
                let (child_total, child_count, child_smallest, child_largest) = stats(child);
                total += child_total;
                count += child_count;
                if child_smallest < smallest {
                    smallest = child_smallest;
                }
                if child_largest > largest {
                    largest = child_largest;
                }
            }
        }
        (total, count, smallest, largest)
    }
    let guard = node.read().unwrap();
    stats(&*guard)
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

pub fn set(root: &std::sync::RwLock<Node>, key: &str, value: Value) -> Result<(), String> {
    let path: Vec<&str> = if key.is_empty() {
        Vec::new()
    } else {
        key.split(':').collect()
    };
    let mut guard = root.write().map_err(|_| "Lock poisoned")?;
    let mut current = &mut *guard;
    if path.is_empty() {
        current.v = Some(value);
        return Ok(());
    }
    let mut val_opt = Some(value);
    for (i, part) in path.iter().enumerate() {
        if current.c.is_none() {
            current.c = Some(HashMap::new());
        }
        let children = current.c.as_mut().unwrap();
        current = children.entry(part.to_string())
            .or_insert_with(|| Box::new(Node::new()));
        if i == path.len() - 1 {
            current.v = val_opt.take();  // Moves the value only once
        }
    }
    Ok(())
}

pub fn get(root: &std::sync::RwLock<Node>, key: &str) -> Result<Option<Value>, String> {
    let path: Vec<&str> = if key.is_empty() {
        Vec::new()
    } else {
        key.split(':').collect()
    };
    let guard = root.read().map_err(|_| "Lock poisoned")?;
    let mut current = &*guard;
    for part in path {
        let children = match current.c.as_ref() {
            Some(children) => children,
            None => return Ok(None),
        };
        match children.get(part) {
            Some(child) => {
                current = child;
            }
            None => return Ok(None),
        }
    }
    Ok(current.v.clone())
}

#[inline]
pub fn delete(root: &std::sync::RwLock<Node>, key: &str) -> Result<bool, String> {
    let path: Vec<&str> = if key.is_empty() {
        Vec::new()
    } else {
        key.split(':').collect()
    };
    let mut guard = root.write().map_err(|_| "Lock poisoned")?;
    if path.is_empty() {
        guard.v = None;
        guard.t = None;
        guard.c = None;
        return Ok(true);
    }
    let mut current = &mut *guard;
    let last = path.last().copied();
    for part in &path[..path.len().saturating_sub(1)] {
        let children = match current.c.as_mut() {
            Some(c) => c,
            None => return Ok(false),
        };
        current = match children.get_mut(*part) {
            Some(child) => child,
            None => return Ok(false),
        };
    }
    if let Some(last_part) = last {
        if let Some(children) = current.c.as_mut() {
            children.remove(last_part);
            return Ok(true);
        }
    }
    Ok(false)
}
