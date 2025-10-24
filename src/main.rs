mod db;
mod commands;
mod server;

use crate::db::Database;
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let db = Arc::new(Database::new());
    server::start("0.0.0.0:2002", db).await
}