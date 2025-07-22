use crate::db::{Database, Value};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::OwnedWriteHalf;

pub struct CommandParts<'a> {
    parts: Vec<&'a [u8]>,
}

impl<'a> CommandParts<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        let mut parts = Vec::new();
        let mut start = 0;
        for (i, &b) in buf.iter().enumerate() {
            if b == b' ' {
                if start < i {
                    parts.push(&buf[start..i]);
                }
                start = i + 1;
            }
        }
        if start < buf.len() {
            parts.push(&buf[start..]);
        }
        CommandParts { parts }
    }
    pub fn get(&self, idx: usize) -> Option<&'a [u8]> {
        self.parts.get(idx).copied()
    }
    pub fn len(&self) -> usize {
        self.parts.len()
    }
}

pub async fn handle_set(
    parts: &CommandParts<'_>,
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    if parts.len() < 3 {
        return write_response(writer, b"Usage: SET key value\n").await;
    }
    let key = bytes_to_str(parts.get(1).unwrap());
    let value = Value::Text(bytes_to_str(parts.get(2).unwrap()).to_string());
    match database.set(key, value) {
        Ok(_) => write_response(writer, b"OK\n").await,
        Err(_) => write_response(writer, b"Error: SET failed\n").await,
    }
}

pub async fn handle_get(
    parts: &CommandParts<'_>,
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    if parts.len() < 2 {
        return write_response(writer, b"Usage: GET key\n").await;
    }
    let key = bytes_to_str(parts.get(1).unwrap());
    match database.get(key) {
        Ok(Some(Value::Text(val))) => write_str(writer, &val).await,
        Ok(Some(_)) => write_response(writer, b"(value)\n").await, // fallback for other types
        Ok(None) => write_response(writer, b"(nil)\n").await,
        Err(_) => write_response(writer, b"Error: GET failed\n").await,
    }
}

pub async fn handle_del(
    parts: &CommandParts<'_>,
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    if parts.len() < 2 {
        return write_response(writer, b"Usage: DEL key\n").await;
    }
    let key = bytes_to_str(parts.get(1).unwrap());
    match database.delete(key) {
        Ok(true) => write_response(writer, b"1\n").await,
        Ok(false) => write_response(writer, b"0\n").await,
        Err(_) => write_response(writer, b"Error: DEL failed\n").await,
    }
}

pub async fn handle_drop(
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    database.drop_all();
    write_response(writer, b"OK\n").await
}

pub async fn handle_dbsize(
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    let size = database.dbsize();
    let response = format!("{size}\n");
    write_response(writer, response.as_bytes()).await
}

//
// ─── Misc Helpers ──────────────────────────────────────────────────────────────
//

#[inline]
pub fn bytes_to_str(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap_or("")
}

#[inline(always)]
pub async fn write_response(
    writer: &mut BufWriter<OwnedWriteHalf>,
    data: &[u8],
) -> std::io::Result<()> {
    writer.write_all(data).await?;
    writer.flush().await
}

#[inline]
pub async fn write_str(writer: &mut BufWriter<OwnedWriteHalf>, value: &str) -> std::io::Result<()> {
    write_response(writer, format!("{value}\n").as_bytes()).await
}
