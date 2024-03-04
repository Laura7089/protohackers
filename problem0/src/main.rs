use problem0::TcpEcho;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower_service::Service;
use tracing::{debug, debug_span, trace, error};

const LISTEN_ADDR: &str = "0.0.0.0:5000";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    debug!("listening on TCP socket {LISTEN_ADDR}");

    loop {
        let (mut stream, addr) = listener.accept().await?;
        // TODO: this spawns a new task for every single incoming stream.
        // Can this cause resource starving?
        tokio::spawn(async move {
            // TODO: don't run this string allocation unless we're in debug level
            let addr = format!("{addr}");
            let span = debug_span!("conn", addr);
            debug!(parent: &span, "new connection");

            let mut service = TcpEcho;

            let mut buf = Vec::new();
            let n = match stream.read_to_end(&mut buf).await {
                Ok(n) => n,
                Err(e) => {
                    error!(parent: &span, "io error receiving data: {e}");
                    return;
                }
            };
            trace!(parent: &span, "received {n} bytes");

            // unwrap because TcpEcho is infallible
            let resp = service.call(buf).await.unwrap();

            let n = match stream.write(&resp).await {
                Ok(n) => n,
                Err(e) => {
                    error!(parent: &span, "io error sending response: {e}");
                    return;
                }
            };
            trace!(parent: &span, "sent {n} bytes, closing");
            if let Err(e) = stream.shutdown().await {
                error!(parent: &span, "io error sending response: {e}");
            }
        });
    }
}
