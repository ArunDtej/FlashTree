use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::OwnedWriteHalf;
use crate::db::{Database, Value};

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

#[inline]
fn normalize_command(input: &[u8]) -> &[u8] {
    let mut end = input.len();
    while end > 0 && matches!(input[end - 1], b'\r' | b'\n') {
        end -= 1;
    }
    &input[..end]
}

#[inline]
fn ascii_lower(b: u8) -> u8 {
    if b >= b'A' && b <= b'Z' {
        b + 32
    } else {
        b
    }
}

#[inline]
fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && 
    a.iter().zip(b.iter()).all(|(x, y)| ascii_lower(*x) == ascii_lower(*y))
}

struct CommandParts<'a> {
    parts: [&'a [u8]; 8],
    count: usize,
}

impl<'a> CommandParts<'a> {
    fn new(buf: &'a [u8]) -> Self {
        let mut parser = CommandParts {
            parts: [&[]; 8],
            count: 0,
        };
        
        let mut start = 0;
        for (i, &byte) in buf.iter().enumerate() {
            if byte == b' ' && start < i && parser.count < 8 {
                parser.parts[parser.count] = &buf[start..i];
                parser.count += 1;
                start = i + 1;
            }
        }
        
        // Add final part if exists
        if start < buf.len() && parser.count < 8 {
            parser.parts[parser.count] = &buf[start..];
            parser.count += 1;
        }
        
        parser
    }
    
    fn get(&self, index: usize) -> Option<&'a [u8]> {
        if index < self.count { Some(self.parts[index]) } else { None }
    }
    
    fn len(&self) -> usize { self.count }
}

// ============================================================================
// COMMAND DISPATCHING
// ============================================================================

pub async fn handle_command(
    line: &str,
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<bool> {
    let raw = normalize_command(line.as_bytes());
    let parts = CommandParts::new(raw);

    if parts.len() == 0 {
        write_response(writer, b"Empty command\n").await?;
        return Ok(false);
    }

    let cmd = parts.get(0).unwrap();
    
    match dispatch_command(cmd) {
        Command::Ping => {
            write_response(writer, b"PONG\n").await?;
        }
        Command::Hello => {
            write_response(writer, b"Hi there! FlashTree v0.1\n").await?;
        }
        Command::Exit => {
            write_response(writer, b"Bye!\n").await?;
            return Ok(true);
        }
        Command::Set => {
            handle_set(&parts, writer, database).await?;
        }
        Command::Get => {
            handle_get(&parts, writer, database).await?;
        }
        Command::Del => {
            handle_del(&parts, writer, database).await?;
        }
        Command::Unknown => {
            write_response(writer, b"Unknown command\n").await?;
        }
    }

    Ok(false)
}

// Command enum for cleaner dispatch
#[derive(Debug)]
enum Command {
    Ping,
    Hello, 
    Exit,
    Set,
    Get,
    Del,
    Unknown,
}

// Fast command matching using first byte + length
#[inline]
fn dispatch_command(cmd: &[u8]) -> Command {
    if cmd.is_empty() { return Command::Unknown; }
    
    match (ascii_lower(cmd[0]), cmd.len()) {
        (b'p', 4) if eq_ignore_ascii_case(cmd, b"ping") => Command::Ping,
        (b'h', 5) if eq_ignore_ascii_case(cmd, b"hello") => Command::Hello,
        (b'e', 4) if eq_ignore_ascii_case(cmd, b"exit") => Command::Exit,
        (b'q', 4) if eq_ignore_ascii_case(cmd, b"quit") => Command::Exit,
        (b's', 3) if eq_ignore_ascii_case(cmd, b"set") => Command::Set,
        (b'g', 3) if eq_ignore_ascii_case(cmd, b"get") => Command::Get,
        (b'd', 3) if eq_ignore_ascii_case(cmd, b"del") => Command::Del,
        _ => Command::Unknown,
    }
}

// ============================================================================
// COMMAND HANDLERS
// ============================================================================

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
        Ok(Some(value)) => write_value(writer, &value).await,
        Ok(None) => write_response(writer, b"(nil)\n").await,
        Err(_) => write_response(writer, b"Error: GET failed\n").await,
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
        Ok(true) => write_response(writer, b"1\n").await,
        Ok(false) => write_response(writer, b"0\n").await,
        Err(_) => write_response(writer, b"Error: DEL failed\n").await,
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

#[inline]
fn bytes_to_str(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap_or("")
}

fn parse_value(s: &str) -> Value {
    // Try integer first (most common)
    if let Ok(int_val) = s.parse::<i64>() {
        return Value::Int(int_val);
    }
    
    // Try boolean
    match s {
        "true" | "TRUE" => return Value::Bool(true),
        "false" | "FALSE" => return Value::Bool(false),
        _ => {}
    }
    
    // Try float
    if s.contains('.') {
        if let Ok(float_val) = s.parse::<f64>() {
            return Value::Float(float_val);
        }
    }
    
    // Default to text
    Value::Text(s.to_string())
}

async fn write_response(
    writer: &mut BufWriter<OwnedWriteHalf>,
    data: &[u8]
) -> std::io::Result<()> {
    writer.write_all(data).await?;
    writer.flush().await
}

async fn write_value(
    writer: &mut BufWriter<OwnedWriteHalf>,
    value: &Value
) -> std::io::Result<()> {
    match value {
        Value::Int(i) => {
            let s = i.to_string();
            writer.write_all(s.as_bytes()).await?;
        }
        Value::Float(f) => {
            let s = f.to_string();
            writer.write_all(s.as_bytes()).await?;
        }
        Value::Bool(b) => {
            let s = if *b { "true" } else { "false" };
            writer.write_all(s.as_bytes()).await?;
        }
        Value::Text(s) => {
            writer.write_all(s.as_bytes()).await?;
        }
        _ => {
            writer.write_all(b"(value)").await?;
        }
    }
    writer.write_all(b"\n").await?;
    writer.flush().await
}
