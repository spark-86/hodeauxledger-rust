use owo_colors::OwoColorize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener; // read/write helpers

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1984").await?;
    println!(
        "{}{}",
        "Listening on port".black().bold(),
        "1984".white().bold()
    );

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("Connection closed");
                        break;
                    }
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("read error: {e}");
                        break;
                    }
                };
                println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
                // optional echo:
                if let Err(e) = socket.write_all(&buf[..n]).await {
                    eprintln!("write error: {e}");
                    break;
                }
            }
        });
    }
}
