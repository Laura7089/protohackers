use problem0::TcpEcho;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower_service::Service;
use tracing::debug;

const LISTEN_ADDR: &str = "0.0.0.0:5000";

#[tokio::main]
#[tracing::instrument]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    debug!("listening on TCP socket {LISTEN_ADDR}");

    loop {
        let (mut stream, addr) = listener.accept().await?;
        debug!("new connection from {addr}");

        // TODO: this spawns a new task for every single incoming stream.
        // Can this cause resource starving?
        tokio::spawn(async move {
            let mut service = TcpEcho;
            let mut buf = vec![0u8; 1024];

            loop {
                let n = stream.read(&mut buf).await.unwrap();
                debug!("received {n} bytes from {addr}");
                if n == 0 {
                    // socket closed
                    debug!("socket closed");
                    return;
                }

                let resp = service.call(buf[0..n].to_owned()).await.unwrap();
                stream.write(&resp).await.unwrap();
                debug!("send {} bytes to {addr}", resp.len());
            }
        });
    }
}
