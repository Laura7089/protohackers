use async_trait::async_trait;
use std::io;
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
    pub use tokio::net::{TcpListener, TcpStream};
    pub use tower::Service;
    pub use tracing::{debug, error, info, instrument, trace, warn};
}
use prelude::*;

/// A trait for quick-and-dirty server execution
#[async_trait]
pub trait Server<E>
where
    Self: Sized + std::fmt::Debug + Service<Vec<u8>, Response = Vec<u8>, Error = E>,
    E: From<io::Error>,
    <Self as Service<Vec<u8>>>::Future: Send,
{
    #[instrument(skip_all)]
    async fn handle_socket(&mut self, mut sock: TcpStream) -> Result<(), E> {
        let mut buf = Vec::new();
        sock.read_buf(&mut buf).await?;
        debug!("handling request of len {}", buf.len());
        let resp = self.call(buf).await?;
        sock.write_all(&resp).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn run(&mut self, port: u16) -> Result<(), E> {
        info!("starting TCP listener on 0.0.0.0:{}", port);
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        loop {
            let (sock, _) = listener.accept().await?;
            self.handle_socket(sock).await?;
        }
    }
}
