use clap::Parser;
use futures::{SinkExt, StreamExt};
use hodeauxledger_io::disk::key as diskkey;
use hodeauxledger_io::disk::rhex as diskrhex;
use hodeauxledger_io::screen;
use hodeauxledger_proto::codec::RhexCodec;
use std::path::Path;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::argv::Command;
use crate::argv::SubmitArgs;
use crate::bstrapnet::bootstrap_network;

mod argv;
mod bstrapnet;
mod head;

async fn submit_rhex(args: &SubmitArgs, verbose: bool) -> anyhow::Result<(), anyhow::Error> {
    let rhex_path = &args.input;
    let host = &args.host;
    let port = &args.port;

    if verbose {
        println!("rhex path: {}", rhex_path);
        println!("host: {}", host);
        println!("port: {}", port);
    }

    let rhex = diskrhex::load_rhex(&Path::new(&rhex_path).to_path_buf())?;
    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(addr).await?;
    let framed = Framed::new(stream, RhexCodec::new());
    let (mut sink, mut stream) = framed.split();

    if verbose {
        println!("Sending packet...")
    }
    sink.send(rhex).await?;
    sink.flush().await?;
    if verbose {
        println!("Packet sent!")
    }

    let mut replies = Vec::new();
    let idle = Duration::from_millis(500);

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
    let args = argv::Cli::parse();
    match args.cmd {
        Command::Submit(submit_args) => submit_rhex(&submit_args, args.verbose).await?,
        Command::Auth(auth_args) => get_authorities(&auth_args, args.verbose).await?,
        _ => {
            anyhow::bail!("unknown operation");
        }
    };
    //bootstrap_network(verbose, &author_sk).await?;
    Ok(())
}

async fn get_authorities(args: &argv::AuthArgs, verbose: bool) -> anyhow::Result<()> {
    Ok(())
}
