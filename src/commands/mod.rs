use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::tcp::OwnedWriteHalf;
use crate::db::{Database, Value};

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
    if (b'A'..=b'Z').contains(&b) {
        b + 32
    } else {
        b
    }
}

// Compare case-insensitive, match only ASCII chars
#[inline]
fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| ascii_lower(*x) == ascii_lower(*y))
}

#[inline]
fn split_command<'a>(buf: &'a [u8]) -> Vec<&'a [u8]> {
    buf.split(|b| *b == b' ').filter(|s| !s.is_empty()).collect()
}

// Returns true if connection should close
pub async fn handle_command(
    line: &str,
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<bool> {
    let raw = normalize_command(line.as_bytes());
    let parts = split_command(raw);

    if parts.is_empty() {
        writer.write_all(b"Empty command\n").await?;
        writer.flush().await?;
        return Ok(false);
    }

    let cmd = parts[0];

    // Fast-path byte compare without allocation
    // Commands: PING, HELLO, EXIT/QUIT, SET, GET, DEL
    if eq_ignore_ascii_case(cmd, b"ping") {
        writer.write_all(b"PONG\n").await?;
    } else if eq_ignore_ascii_case(cmd, b"hello") {
        writer.write_all(b"Hi there! FlashTree v0.1\n").await?;
    } else if eq_ignore_ascii_case(cmd, b"exit") || eq_ignore_ascii_case(cmd, b"quit") {
        writer.write_all(b"Bye!\n").await?;
        writer.flush().await?;
        return Ok(true);
    } else if eq_ignore_ascii_case(cmd, b"set") {
        handle_set_command(&parts, writer, database).await?;
    } else if eq_ignore_ascii_case(cmd, b"get") {
        handle_get_command(&parts, writer, database).await?;
    } else if eq_ignore_ascii_case(cmd, b"del") {
        handle_del_command(&parts, writer, database).await?;
    } else {
        writer.write_all(b"Unknown command\n").await?;
    }

    writer.flush().await?;
    Ok(false)
}

async fn handle_set_command(
    parts: &[&[u8]],
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    if parts.len() < 3 {
        writer.write_all(b"Usage: SET key value\n").await?;
        return Ok(());
    }
    let key = std::str::from_utf8(parts[1]).unwrap_or("");
    let value_str = std::str::from_utf8(parts[2]).unwrap_or("");

    // Parse value types
    let value = if let Ok(int_val) = value_str.parse::<i64>() {
        Value::Int(int_val)
    } else if let Ok(float_val) = value_str.parse::<f64>() {
        Value::Float(float_val)
    } else if let Ok(bool_val) = value_str.parse::<bool>() {
        Value::Bool(bool_val)
    } else {
        Value::Text(value_str.to_string())
    };

    match database.set(key, value) {
        Ok(_) => writer.write_all(b"OK\n").await?,
        Err(e) => writer.write_all(format!("Error: {}\n", e).as_bytes()).await?,
    }

    Ok(())
}

async fn handle_get_command(
    parts: &[&[u8]],
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    if parts.len() < 2 {
        writer.write_all(b"Usage: GET key\n").await?;
        return Ok(());
    }

    let key = std::str::from_utf8(parts[1]).unwrap_or("");
    match database.get(key) {
        Ok(Some(value)) => {
            let response = format!("{:?}\n", value);
            writer.write_all(response.as_bytes()).await?;
        }
        Ok(None) => {
            writer.write_all(b"(nil)\n").await?;
        }
        Err(e) => {
            writer.write_all(format!("Error: {}\n", e).as_bytes()).await?;
        }
    }
    Ok(())
}

async fn handle_del_command(
    parts: &[&[u8]],
    writer: &mut BufWriter<OwnedWriteHalf>,
    database: &Database,
) -> std::io::Result<()> {
    if parts.len() < 2 {
        writer.write_all(b"Usage: DEL key\n").await?;
        return Ok(());
    }
    let key = std::str::from_utf8(parts[1]).unwrap_or("");
    match database.delete(key) {
        Ok(true) => writer.write_all(b"1\n").await?,
        Ok(false) => writer.write_all(b"0\n").await?,
        Err(e) => writer.write_all(format!("Error: {}\n", e).as_bytes()).await?,
    }
    Ok(())
}
