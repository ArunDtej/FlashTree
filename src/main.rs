mod db;
mod commands;
mod server;
use crate::db::Database;


#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let addr = "127.0.0.1:2002";
    let database = Database::new();
    
    println!("Starting FlashTree server...");
    server::start(addr, database).await
}
