mod server;

#[tokio::main]
async fn main() {
    
    let addr = "127.0.0.1:2002";
    println!("⚡ Listening on {addr}");
    server::start(addr).await.unwrap();
}
