use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use hodeauxledger_core::Rhex;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::codec::RhexCodec;
use futures::stream::SplitSink;
use futures::stream::SplitStream;

#[derive(Debug)]
pub struct Transport {
    pub rhex_sent: u64,
    pub rhex_received: u64,
    pub peer: String,
    sink: Option<SplitSink<Framed<TcpStream, RhexCodec>, Rhex>>,
    stream: Option<SplitStream<Framed<TcpStream, RhexCodec>>>,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            rhex_sent: 0,
            rhex_received: 0,
            peer: String::new(),
            sink: None,
            stream: None,
        }
    }

    /// Connect and prepare framed sink/stream for R⬢ transport.
    pub async fn connect(&mut self, host: &str, port: &str) -> Result<()> {
        self.peer = format!("{host}:{port}");
        let stream = TcpStream::connect(&self.peer)
            .await
            .with_context(|| format!("failed to connect to {}", self.peer))?;

        let framed = Framed::new(stream, RhexCodec::new());
        let (sink, stream) = framed.split();
        self.sink = Some(sink);
        self.stream = Some(stream);
        Ok(())
    }

    /// Send a single R⬢. Requires prior `connect()`.
    pub async fn send_rhex(&mut self, rhex: &Rhex) -> Result<()> {
        let sink = self
            .sink
            .as_mut()
            .context("not connected: call connect() before send_rhex")?;

        // Clone to move into the sink; Rhex is small (~1.5KB), cheap.
        sink.send(rhex.clone())
            .await
            .context("failed to send Rhex")?;
        self.rhex_sent += 1;
        Ok(())
    }

    /// Receive the next R⬢ from the peer (awaits one frame). Returns Ok(None) on clean EOF.
    pub async fn recv_next(&mut self) -> Result<Option<Rhex>> {
        let stream = self
            .stream
            .as_mut()
            .context("not connected: call connect() before recv_next")?;

        match stream.next().await {
            Some(Ok(r)) => {
                self.rhex_received += 1;
                Ok(Some(r))
            }
            Some(Err(e)) => Err(e).context("failed to decode incoming Rhex"),
            None => Ok(None), // peer closed
        }
    }

    /// For manual accounting if you process an R⬢ elsewhere.
    pub fn recv_rhex(&mut self, _rhex: &Rhex) {
        self.rhex_received += 1;
    }

    pub fn print_stats(&self) {
        println!(
            "Peer: {}",
            if self.peer.is_empty() {
                "<disconnected>"
            } else {
                &self.peer
            }
        );
        println!("Sent: {}", self.rhex_sent);
        println!("Received: {}", self.rhex_received);
    }
}
