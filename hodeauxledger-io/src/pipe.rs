use anyhow::Result;
use futures::{Sink, Stream};
use hodeauxledger_core::Rhex;
use hodeauxledger_proto::codec::RhexCodec;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct UsherPipe {
    framed: Framed<TcpStream, RhexCodec>,
}

impl UsherPipe {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            framed: Framed::new(stream, RhexCodec::new()),
        }
    }
}

impl Stream for UsherPipe {
    type Item = Result<Rhex>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // delegate directly
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.framed) };
        inner
            .poll_next(cx)
            .map(|opt| opt.map(|res| res.map_err(Into::into)))
    }
}

impl Sink<Rhex> for UsherPipe {
    type Error = anyhow::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.framed) };
        inner.poll_ready(cx).map_err(Into::into)
    }

    fn start_send(self: Pin<&mut Self>, item: Rhex) -> Result<(), Self::Error> {
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.framed) };
        inner.start_send(item).map_err(Into::into)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.framed) };
        inner.poll_flush(cx).map_err(Into::into)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.framed) };
        inner.poll_close(cx).map_err(Into::into)
    }
}
