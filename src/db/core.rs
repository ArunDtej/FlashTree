use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use crate::db::Database;

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
        println!("returning at root");
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
    println!("printing value{:?}", guard.value);
    Ok(guard.value.clone())
}
