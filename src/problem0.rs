use crate::prelude::*;

#[derive(Debug)]
pub struct SmokeTest;

impl Service<Vec<u8>> for SmokeTest {
    type Response = Vec<u8>;
    type Error = io::Error;
    type Future = Pin<Box<dyn Send + Future<Output = io::Result<Vec<u8>>>>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        // Always ready
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Vec<u8>) -> Self::Future {
        Box::pin(async { Ok(req) })
    }
}

impl Server for SmokeTest {}
