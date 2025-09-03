use anyhow::Result;
use bytes::BytesMut;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use hodeauxledger_core::{Key, Rhex};
use hodeauxledger_io::cache::build::build_cache_db;
use hodeauxledger_io::disk::key as diskkey;
use hodeauxledger_proto::codec::RhexCodec;
use std::time::Instant;
use std::{net::SocketAddr, path::Path};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Encoder, Framed};

use crate::argv::Command;

mod argv;
mod bootstrap;
mod processor;

#[derive(Parser, Debug)]
#[command(name = "usher", about = "HodeauxLedger Usher Tool")]
struct Cli {
    #[arg(long)]
    host: Option<String>,

    #[arg(short, long)]
    port: Option<String>,

    #[arg(short, long)]
    ledger_path: Option<String>,

    #[arg(short, long)]
    keyfile: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Default, Debug, Clone)]
struct ConnStats {
    records_in: u64,
    bytes_in: u64,
    records_out: u64,
    bytes_out: u64,
}

impl ConnStats {
    fn add_in(&mut self, n: usize) {
        self.records_in += 1;
        self.bytes_in += n as u64;
    }
    fn add_out(&mut self, n: usize) {
        self.records_out += 1;
        self.bytes_out += n as u64;
    }
}

async fn accept_loop(listener: TcpListener, verbose: bool) {
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("🛰️🟢 connection from {addr}");
                tokio::spawn(async move {
                    if let Err(e) = handle_conn(stream, addr, verbose).await {
                        eprintln!("⚠️ {addr} error: {e}");
                    }
                    println!("🛰️🔴 {addr} closed");
                });
            }
            Err(e) => {
                eprintln!("accept error: {e}");
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }
}

async fn handle_conn(conn: TcpStream, addr: SocketAddr, verbose: bool) -> Result<()> {
    let framed = Framed::new(conn, RhexCodec::new());
    let (mut sink, mut stream) = framed.split();

    let mut stats = ConnStats::default();
    let mut codec = RhexCodec::new(); // used locally to measure encoded sizes

    // FIXME: This is so not the answer but I so don't feel like fucking
    // with it right now so we're gonna load from "./hot.key" and call
    // it good for the day.
    let hot_key32 = diskkey::load_key_hot(Path::new("./hot.key"))?;
    let hot_key = Key::from_bytes(&hot_key32);
    // Read frames until the peer closes or an error occurs.
    while let Some(in_msg) = stream.next().await {
        let started = Instant::now();
        let rhex_in: Rhex = in_msg?; // decode via RhexCodec

        // Measure inbound size by re-encoding with the same codec.
        let mut in_buf = BytesMut::new();
        codec.encode(rhex_in.clone(), &mut in_buf)?;
        let in_len = in_buf.len();
        stats.add_in(in_len);

        if verbose {
            // If your Rhex has getters, feel free to swap these placeholders.
            println!(
                "📥 {addr} in: {} bytes | did: verify+echo | record_type: {}",
                in_len,
                rhex_in.intent.record_type // adjust if your API differs
            );
        }

        // TODO: replace this with real server-side handling:
        //   - verify author's sig / linkage
        //   - maybe co-sign as usher
        //   - maybe emit quorum status / finalization
        // For now, echo the same record as a simple ACK.
        // (We also measure the encoded outbound size the same way.)
        let out_rhex = processor::process_rhex(&rhex_in, &hot_key, verbose)?;

        for rhex in out_rhex {
            let mut out_buf = BytesMut::new();
            codec.encode(rhex.clone(), &mut out_buf)?;
            let out_len = out_buf.len();
            sink.send(rhex).await?;
            sink.flush().await?;
            stats.add_out(out_len);
        }
        if verbose {
            let elapsed = started.elapsed();
            println!(
                "📤 {addr} out: {} bytes | action: echo | took: {} ms",
                stats.bytes_out,
                elapsed.as_millis()
            );
        }
    }

    // Per-connection summary
    println!(
        "📊 {addr} summary: in {} records / {} bytes | out {} records / {} bytes",
        stats.records_in, stats.bytes_in, stats.records_out, stats.bytes_out
    );

    Ok(())
}

async fn setup_listener(host: &str, port: &str) -> anyhow::Result<TcpListener> {
    let addr = format!("{host}:{port}");
    Ok(TcpListener::bind(addr).await?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = argv::Cli::parse();
    let action = args.cmd;

    match action {
        Command::Listen(listen_args) => listen(&listen_args).await?,
        Command::Rebuild(rebuild_args) => rebuild(&rebuild_args).await?,
    }

    //bootstrap::bootstrap(args.verbose)?;

    let listener = setup_listener("0.0.0.0", "1984").await?;
    println!("listening on {}", listener.local_addr()?);

    let verbose = args.verbose;

    // task that waits for Ctrl+C to trigger shutdown
    let shutdown = tokio::spawn(async {
        let _ = tokio::signal::ctrl_c().await;
        println!("\nshutdown signal received");
    });

    tokio::select! {
        _ = accept_loop(listener, verbose) => {}
        _ = shutdown => {}
    }

    println!("bye!");
    Ok(())
}

async fn listen(args: &argv::ListenArgs) -> anyhow::Result<()> {
    Ok(())
}

async fn rebuild(args: &argv::RebuildArgs) -> anyhow::Result<()> {
    let path = &args.db_path;
    let path = if path.is_none() {
        path.clone().unwrap().to_string()
    } else {
        "./data/cache.sqlite".to_string()
    };
    build_cache_db(&path);
    Ok(())
}
