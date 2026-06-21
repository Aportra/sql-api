use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use axum::{Router, routing::{get,post}}
use sqlx::PgPool;

#[derive(Clone)]
struct AppState{
    db:PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on 127.0.0.1:8080");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Accepted connection from {addr}");

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            match socket.read(&mut buf).await {
                Ok(0) => return, // connection closed
                Ok(n) => {
                    println!("Read {n} bytes: {:?}", String::from_utf8_lossy(&buf[..n]));
                    if let Err(e) = socket.write_all(b"Hello, world!\n").await {
                        eprintln!("write error: {e}");
                    }
                }
                Err(e) => eprintln!("read error: {e}"),
            }
        });
    }
}
