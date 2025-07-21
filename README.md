# FlashTree

**FlashTree** is a high-performance, in-memory key-value store server written in Rust. Think of it as a lightweight, super-fast database you talk to using classic text commands—ideal for rapid prototyping, learning, small projects, or anywhere you want speed and simplicity.

---

## 🚀 What is FlashTree?

FlashTree is a small server you can run locally or on a server. It stores, retrieves, and deletes hierarchical, structured keys like `user:100:name` incredibly efficiently, making it especially fast and effective for keys with common prefixes.

---

## 🌟 Features

- **Blazing fast** — thanks to Rust and an efficient trie-based backend
- **Trie structure** — space-efficient for keys like `foo:bar:baz`
- **Concurrent and thread-safe** — handles many commands at once with ease
- **Multiple data types** — store integers, floats, booleans, or strings
- **Simple protocol** — connect via telnet, netcat, or your own client code
- **Zero configuration** — just run and go

---

## ⚡ Quick Start

### Requirements

- Rust (latest stable recommended)
- Cargo

### Build & Run

