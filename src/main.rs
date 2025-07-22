mod db;
mod commands;
mod server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:2002";
    
    println!("Starting FlashTree server...");
    server::start(addr).await
}
