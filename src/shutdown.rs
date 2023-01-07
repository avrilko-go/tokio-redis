use tokio::sync::broadcast;
use tracing::info;

#[derive(Debug)]
pub struct Shutdown {
    inner:broadcast::Receiver<()>,
    is_shutdown :bool
}

impl Shutdown {
    pub fn new(recv :broadcast::Receiver<()>) -> Self {
        Self {
            inner: recv,
            is_shutdown: false,
        }
    }

    pub async fn recv(&mut self) {
        if self.is_shutdown {
            return;
        }
        let _ = self.inner.recv().await;
        self.is_shutdown = true;
    }

    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown
    }
}