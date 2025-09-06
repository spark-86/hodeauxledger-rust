use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use hodeauxledger_core::Rhex;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
use tokio_util::codec::Framed;

use futures::stream::{SplitSink, SplitStream};
use hodeauxledger_proto::codec::RhexCodec;

/// Framed R⬢ transport over TCP with simple accounting and timeouts.
#[derive(Debug, Default)]
pub struct Transport {
    pub rhex_sent: u64,
    pub rhex_received: u64,
    pub peer: String,
    sink: Option<SplitSink<Framed<TcpStream, RhexCodec>, Rhex>>, // outbound frames
    stream: Option<SplitStream<Framed<TcpStream, RhexCodec>>>,   // inbound frames
}

impl Transport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if both sink & stream are present.
    pub fn is_connected(&self) -> bool {
        self.sink.is_some() && self.stream.is_some()
    }

    /// Connect and prepare framed sink/stream for R⬢ transport.
    pub async fn connect(&mut self, host: &str, port: &str) -> Result<()> {
        self.peer = format!("{host}:{port}");
        let stream = TcpStream::connect(&self.peer)
            .await
            .with_context(|| format!("failed to connect to {}", self.peer))?;

        // Nagle off helps for small framed messages (R⬢ ~1.5KB max typically).
        stream.set_nodelay(true).ok();

        let framed = Framed::new(stream, RhexCodec::new());
        let (sink, stream) = framed.split();
        self.sink = Some(sink);
        self.stream = Some(stream);
        Ok(())
    }

    /// Connect with a timeout window.
    pub async fn connect_with_timeout(
        &mut self,
        host: &str,
        port: &str,
        dur: Duration,
    ) -> Result<()> {
        timeout(dur, self.connect(host, port))
            .await
            .context("connect timeout")??;
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
        sink.flush().await.ok();

        self.rhex_sent += 1;
        Ok(())
    }

    /// Send many R⬢ records in-order.
    pub async fn send_many<I>(&mut self, iter: I) -> Result<()>
    where
        I: IntoIterator<Item = Rhex>,
    {
        let sink = self
            .sink
            .as_mut()
            .context("not connected: call connect() before send_many")?;

        for r in iter {
            sink.send(r).await.context("failed to send Rhex in batch")?;
            self.rhex_sent += 1;
        }
        sink.flush().await.ok();
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

    /// Receive with a timeout. Ok(None) if peer closed or timer elapsed without a frame.
    pub async fn recv_next_with_timeout(&mut self, dur: Duration) -> Result<Option<Rhex>> {
        match timeout(dur, self.recv_next()).await {
            Ok(res) => res,
            Err(_) => Ok(None), // timed out waiting for a frame
        }
    }

    /// For manual accounting if you process an R⬢ elsewhere.
    pub fn account_recv(&mut self) {
        self.rhex_received += 1;
    }

    /// Close the transport (drop halves). Any error is ignored by design.
    pub async fn close(&mut self) {
        if let Some(sink) = self.sink.as_mut() {
            let _ = sink.flush().await; // best-effort
            let _ = sink.close().await; // best-effort
        }
        self.sink.take();
        self.stream.take();
        self.peer.clear();
    }

    /// Print simple stats to stdout.
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
