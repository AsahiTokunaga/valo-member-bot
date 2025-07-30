use futures::future::BoxFuture;
use tokio::sync::mpsc;

pub struct AnyTask {
  pub fut: BoxFuture<'static, ()>
}

pub struct Worker {
  pub tx: mpsc::Sender<AnyTask>,
}

impl Worker {
  pub fn new(buffer: usize) -> (Self, mpsc::Receiver<AnyTask>) {
    let (tx, rx) = mpsc::channel(buffer);
    (Self { tx }, rx)
  }

  pub async fn spawn<F, Fut>(&self, f: F)
  where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    let fut = async move {
      f().await;
    };

    let task = AnyTask {
      fut: Box::pin(fut), 
    };

    if let Err(e) = self.tx.send(task).await {
      tracing::error!("Failed to send task to worker: {}", e);
    }
  }
}
