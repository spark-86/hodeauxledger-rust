use clap::Parser;
use futures::{SinkExt, StreamExt};
use hodeauxledger_io::disk;
use hodeauxledger_io::screen;
use hodeauxledger_proto::codec::RhexCodec;
use std::path::Path;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[derive(Parser, Debug)]
#[command(name = "usher", about = "HodeauxLedger Usher Tool")]
struct Cli {
    action: String,

    #[arg(short, long)]
    rhex: Option<String>,

    #[arg(short, long)]
    verbose: bool,

    #[arg(long)]
    host: Option<String>,

    #[arg(short, long)]
    port: Option<String>,
}

async fn submit_rhex(args: &Cli) -> anyhow::Result<(), anyhow::Error> {
    let rhex_path = args
        .rhex
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("rhex must be specified"))?;
    let host = args
        .host
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("host must be specified"))?;
    let port = args.port.as_deref();
    let verbose = args.verbose;

    if verbose {
        println!("rhex path: {}", rhex_path);
        println!("host: {}", host);
        println!("port: {}", port.unwrap_or("1984"));
    }

    let rhex = disk::load_rhex(&Path::new(rhex_path).to_path_buf())?;
    let addr = format!("{}:{}", host, port.unwrap_or("1984"));
    let stream = TcpStream::connect(addr).await?;
    let framed = Framed::new(stream, RhexCodec::new());
    let (mut sink, mut stream) = framed.split();

    if verbose {
        println!("Sending packet...")
    }
    sink.send(rhex).await?;
    sink.flush().await?;

    let mut replies = Vec::new();
    let idle = Duration::from_millis(3000);

    loop {
        match tokio::time::timeout(idle, stream.next()).await {
            Ok(Some(Ok(frame))) => replies.push(frame),
            Ok(Some(Err(e))) => return Err(e.into()),
            Ok(None) => break,
            Err(_elapsed) => break,
        }
    }

    for r in replies {
        if verbose {
            println!("Received reply: {:?}", r);
        }
        screen::pretty_print_rhex(&r);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let action = args.action.as_str();
    match action {
        "submit" => submit_rhex(&args).await?,
        _ => {
            anyhow::bail!("unknown operation");
        }
    };
    Ok(())
}
