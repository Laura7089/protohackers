#![deny(clippy::pedantic)]

use async_trait::async_trait;
pub mod problem0;
pub mod problem1;

pub const MAX_CHALLENGE: u8 = 1;

mod prelude {
    pub use super::Server;
    pub use std::{
        convert::Infallible,
        future::Future,
        io,
        pin::Pin,
        task::{Context, Poll},
    };

    pub use async_trait::async_trait;
    pub use thiserror::Error as ThisError;
    pub use tokio::io::{AsyncReadExt, AsyncWriteExt};
    pub use tower::Service;
    pub use tracing::{debug, error, info, instrument, trace, warn};
}
use prelude::*;

use tokio::net::TcpListener;
use tokio_stream::{wrappers::TcpListenerStream, StreamExt};

/// A trait for quick-and-dirty server execution
#[async_trait]
pub trait Server
where
    Self: Sized + std::fmt::Debug + Service<Vec<u8>, Response = Vec<u8>>,
    <Self as Service<Vec<u8>>>::Future: Send,
    <Self as Service<Vec<u8>>>::Error: std::fmt::Debug,
{
    #[instrument(skip(self))]
    async fn run(&mut self, port: u16) {
        let addr = format!("0.0.0.0:{port}");
        info!("starting TCP listener on {addr}");
        let mut listener = TcpListenerStream::new(
            TcpListener::bind(addr)
                .await
                .expect("failed to bind address"),
        );
        while let Some(mut stream) = listener.next().await.transpose().unwrap() {
            let mut buf = Vec::new();
            stream.read_buf(&mut buf).await.unwrap();
            debug!("handling request of len {}", buf.len());
            let resp = self.call(buf).await.unwrap();
            stream.write_all(&resp).await.unwrap();
        }
    }
}
