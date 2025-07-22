use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};
use crate::db::Database;

pub async fn start(addr: &str, database: Database) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let max_connections = 5000;
    let semaphore = Arc::new(Semaphore::new(max_connections));
    let active_connections = Arc::new(AtomicUsize::new(0));
    
    println!("FlashTree server started on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        let semaphore = Arc::clone(&semaphore);
        let active_connections = Arc::clone(&active_connections);
        let database = database.clone(); // Clone the Database handle (cheap - just Arc clone)
        
        active_connections.fetch_add(1, Ordering::Relaxed);
        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            if let Err(e) = handle_client(stream, database).await {
                eprintln!("Connection error for {}: {}", addr, e);
            }
            active_connections.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

async fn handle_client(stream: TcpStream, database: Database) -> std::io::Result<()> {
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

        if crate::commands::handle_command(&line, &mut writer, &database).await? {
            break;
        }
    }
    Ok(())
}
