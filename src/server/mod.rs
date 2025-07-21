use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};

pub async fn start(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let max_connections = 5000;
    let semaphore = Arc::new(Semaphore::new(max_connections));
    let active_connections = Arc::new(AtomicUsize::new(0));
    println!("Server started on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        let semaphore = Arc::clone(&semaphore);
        let active_connections = Arc::clone(&active_connections);
        active_connections.fetch_add(1, Ordering::Relaxed);
        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            if let Err(e) = handle_client(stream).await {
                eprintln!("Connection error for {}: {}", addr, e);
            }
            active_connections.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

async fn handle_client(stream: TcpStream) -> std::io::Result<()> {
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    let mut line = String::with_capacity(128);
    const IDLE_TIMEOUT: Duration = Duration::from_secs(300);

    loop {
        line.clear();
        let bytes = match timeout(IDLE_TIMEOUT, reader.read_line(&mut line)).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                writer.write_all(b"Timeout\n").await?;
                writer.flush().await?;
                break;
            }
        };
        if bytes == 0 {
            break;
        }

        if handle_command_bytes(&line, &mut writer).await? {
            break;
        }
    }
    Ok(())
}

fn normalize_command(input: &[u8]) -> &[u8] {
    // Remove trailing \r\n
    let mut end = input.len();
    while end > 0 && matches!(input[end - 1], b'\r' | b'\n') {
        end -= 1;
    }
    &input[..end]
}

/// Returns true if the command indicates the connection should close
async fn handle_command_bytes(command: &str, writer: &mut BufWriter<tokio::net::tcp::OwnedWriteHalf>) -> std::io::Result<bool> {
    let cmd_bytes = normalize_command(command.as_bytes());
    
    match cmd_bytes {
        b"ping" | b"PING" | b"Ping" => {
            writer.write_all(b"PONG\n").await?;
        }
        b"hello" | b"HELLO" | b"Hello" => {
            writer.write_all(b"Hi there!\n").await?;
        }
        b"exit" | b"EXIT" | b"Exit" => {
            writer.write_all(b"Bye!\n").await?;
            writer.flush().await?;
            return Ok(true);
        }
        _ => {
            let msg = format!("Unknown command: {}\n", std::str::from_utf8(cmd_bytes).unwrap_or("???"));
            writer.write_all(msg.as_bytes()).await?;
        }
    }
    writer.flush().await?;
    Ok(false)
}
