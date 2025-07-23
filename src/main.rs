mod db;
mod commands;
mod server;
use crate::db::Database;
use std::sync::Arc;



// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

// #[tokio::main]
// async fn main() -> std::io::Result<()> {

//     let addr = "127.0.0.1:2002";
//     let database = Database::new();
    
//     println!("Starting FlashTree server...");
//     server::start(addr, database).await
// }



#[tokio::main]
async fn main() -> std::io::Result<()> {
    let db = Arc::new(Database::new());
    server::start("0.0.0.0:2002", db).await
}