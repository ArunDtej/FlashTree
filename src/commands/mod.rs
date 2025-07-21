use crate::db::{Database, Value};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::OwnedWriteHalf;

//
// ─── Utility Functions ───────────────────────────────────────────────────────────
//

/// Removes trailing `\r\n` or `\n` for robust network command parsing.
#[inline]
fn normalize_command(input: &[u8]) -> &[u8] {
    let mut end = input.len();
    while end > 0 && matches!(input[end - 1], b'\r' | b'\n') {
        end -= 1;
    }
    &input[..end]
}

/// Convert a single ASCII letter to lowercase for case-insensitive comparison.
#[inline]
fn ascii_lower(b: u8) -> u8 {
    if b >= b'A' && b <= b'Z' {
        b + 32
    } else {
        b
    }
}

/// Compare two byte slices case-insensitively (ASCII only, no allocs).
#[inline]
fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|(x, y)| ascii_lower(*x) == ascii_lower(*y))
}

/// Non-allocating, fixed-size command argument splitter.
/// E.g., b"SET foo 1" => ["SET", "foo", "1"]
struct CommandParts<'a> {
    parts: [&'a [u8]; 8],
    count: usize,
}

impl<'a> CommandParts<'a> {
    fn new(buf: &'a [u8]) -> Self {
        let mut parts: [&[u8]; 8] = [&[]; 8];
        let mut count = 0;
        let mut start = 0;
        for (i, &b) in buf.iter().enumerate() {
            if b == b' ' && start < i && count < 8 {
                parts[count] = &buf[start..i];
                count += 1;
                start = i + 1;
            }
        }
        if start < buf.len() && count < 8 {
            parts[count] = &buf[start..];
            count += 1;
        }
        CommandParts { parts, count }
    }

    fn get(&self, idx: usize) -> Option<&'a [u8]> {
        if idx < self.count { Some(self.parts[idx]) } else { None }
    }
    fn len(&self) -> usize { self.count }
}

//
// ─── Command Matching ────────────────────────────────────────────────────────────
//

#[derive(Debug)]
enum Command {
    Ping,
    Hello,
    Exit,
    Set,
    Get,
    Del,
    Drop,
    Unknown,
}

#[inline]
fn dispatch_command(cmd: &[u8]) -> Command {
    if cmd.is_empty() {
        return Command::Unknown;
    }
    match (ascii_lower(cmd[0]), cmd.len()) {
        (b'p', 4) if eq_ignore_ascii_case(cmd, b"ping") => Command::Ping,
        (b'h', 5) if eq_ignore_ascii_case(cmd, b"hello") => Command::Hello,
        (b'e', 4) if eq_ignore_ascii_case(cmd, b"exit") => Command::Exit,
        (b'q', 4) if eq_ignore_ascii_case(cmd, b"quit") => Command::Exit,
        (b's', 3) if eq_ignore_ascii_case(cmd, b"set") => Command::Set,
        (b'g', 3) if eq_ignore_ascii_case(cmd, b"get") => Command::Get,
        (b'd', 3) if eq_ignore_ascii_case(cmd, b"del") => Command::Del,
        (b'd', 4) if eq_ignore_ascii_case(cmd, b"drop") => Command::Drop,
        _ => Command::Unknown,
    }
}

//
// ─── Main Entry Point: Command Handler ──────────────────────────────────────────
//

pub async fn handle_command(
    line: &str,
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<bool> {
    let raw = normalize_command(line.as_bytes());
    let parts = CommandParts::new(raw);

    if parts.len() == 0 {
        return write_response(writer, b"Empty command\n").await.map(|_| false);
    }
    let cmd = parts.get(0).unwrap();
    match dispatch_command(cmd) {
        Command::Ping   => write_response(writer, b"PONG\n").await?,
        Command::Hello  => write_response(writer, b"Hi there! FlashTree v0.1\n").await?,
        Command::Exit   => {
            write_response(writer, b"Bye!\n").await?;
            return Ok(true);
        }
        Command::Set    => handle_set(&parts, writer, database).await?,
        Command::Get    => handle_get(&parts, writer, database).await?,
        Command::Del    => handle_del(&parts, writer, database).await?,
        Command::Drop   => handle_drop(writer, database).await?,
        Command::Unknown=> write_response(writer, b"Unknown command\n").await?,
    }
    Ok(false)
}

//
// ─── Command Implementations ────────────────────────────────────────────────────
//

async fn handle_set(
    parts: &CommandParts<'_>,
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    if parts.len() < 3 {
        return write_response(writer, b"Usage: SET key value\n").await;
    }
    let key = bytes_to_str(parts.get(1).unwrap());
    let value_str = bytes_to_str(parts.get(2).unwrap());
    let value = parse_value(value_str);

    match database.set(key, value) {
        Ok(_) => write_response(writer, b"OK\n").await,
        Err(_) => write_response(writer, b"Error: SET failed\n").await,
    }
}

async fn handle_get(
    parts: &CommandParts<'_>,
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    if parts.len() < 2 {
        return write_response(writer, b"Usage: GET key\n").await;
    }
    let key = bytes_to_str(parts.get(1).unwrap());
    match database.get(key) {
        Ok(Some(val)) => write_value(writer, &val).await,
        Ok(None)      => write_response(writer, b"(nil)\n").await,
        Err(_)        => write_response(writer, b"Error: GET failed\n").await,
    }
}

async fn handle_del(
    parts: &CommandParts<'_>,
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    if parts.len() < 2 {
        return write_response(writer, b"Usage: DEL key\n").await;
    }
    let key = bytes_to_str(parts.get(1).unwrap());
    match database.delete(key) {
        Ok(true)  => write_response(writer, b"1\n").await,
        Ok(false) => write_response(writer, b"0\n").await,
        Err(_)    => write_response(writer, b"Error: DEL failed\n").await,
    }
}

async fn handle_drop(
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    database.drop_all();
    write_response(writer, b"OK\n").await
}

//
// ─── Helpers ────────────────────────────────────────────────────────────────────
//

#[inline]
fn bytes_to_str(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap_or("")
}

fn parse_value(s: &str) -> Value {
    if let Ok(int_val) = s.parse::<i64>() {
        return Value::Int(int_val);
    }
    match s {
        "true" | "TRUE"  => return Value::Bool(true),
        "false" | "FALSE"=> return Value::Bool(false),
        _ => {}
    }
    if s.contains('.') {
        if let Ok(float_val) = s.parse::<f64>() {
            return Value::Float(float_val);
        }
    }
    Value::Text(s.to_string())
}

async fn write_response(
    writer: &mut BufWriter<OwnedWriteHalf>,
    data: &[u8],
) -> std::io::Result<()> {
    writer.write_all(data).await?;
    writer.flush().await
}

async fn write_value(writer: &mut BufWriter<OwnedWriteHalf>, value: &Value) -> std::io::Result<()> {
    match value {
        Value::Int(i) => write_response(writer, format!("{}\n", i).as_bytes()).await,
        Value::Float(f) => write_response(writer, format!("{}\n", f).as_bytes()).await,
        Value::Bool(b) => write_response(writer, if *b { b"true\n" } else { b"false\n" }).await,
        Value::Text(s) => write_response(writer, format!("{}\n", s).as_bytes()).await,
        _ => write_response(writer, b"(value)\n").await,
    }
}
