use futures::future::BoxFuture;
use tokio::sync::mpsc;

pub struct AnyTask {
  pub fut: BoxFuture<'static, ()>,
}

#[derive(Debug)]
pub struct GenericAsyncWorker {
  pub sender: mpsc::Sender<AnyTask>,
}

impl GenericAsyncWorker {
  pub fn new(buffer: usize) -> (Self, mpsc::Receiver<AnyTask>) {
    let (tx, rx) = mpsc::channel(buffer);
    (
      Self { sender: tx },
      rx,
    )
  }

  pub async fn spawn<F, Fut, T>(&self, f: F)
  where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
  {
    tracing::info!("Spawning task in worker");
    let fut = async move {
      f().await;
    };

    let task = AnyTask {
      fut: Box::pin(fut),
    };

    if let Err(e) = self.sender.send(task).await {
      tracing::warn!("Failed to send task to worker: {}", e);
    }
  } 
}
