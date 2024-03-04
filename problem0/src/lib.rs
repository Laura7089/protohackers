use core::future::Future;
use std::convert::Infallible;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use tower_service::Service;

pub struct TcpEcho;

impl Service<Vec<u8>> for TcpEcho {
    type Response = Vec<u8>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Vec<u8>) -> Self::Future {
        Box::pin(async move {
            Ok(req)
        })
    }
}
