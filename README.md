# valo-member-bot

```rust
use tokio::sync::mpsc::{self, Sender, Receiver};
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use futures::FutureExt;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[derive(Clone)]
struct AsyncTaskDispatcher {
    tx: Arc<Sender<BoxFuture>>,
}

impl AsyncTaskDispatcher {
    fn new(buffer: usize) -> (Self, Receiver<BoxFuture>) {
        let (tx, rx) = mpsc::channel(buffer);
        (Self { tx: Arc::new(tx) }, rx)
    }

    async fn dispatch<F>(&self, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        if let Err(e) = self.tx.send(fut.boxed()).await {
            eprintln!("Failed to dispatch task: {}", e);
        }
    }
}

async fn do_something(dispatcher: AsyncTaskDispatcher) {
    dispatcher.dispatch(async {
        println!("🔧 do_something() is running");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        println!("✅ do_something() done");
    }).await;
}

async fn do_another(dispatcher: AsyncTaskDispatcher, value: i32) {
    dispatcher.dispatch(async move {
        println!("🔧 do_another({}) is running", value);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        println!("✅ do_another({}) done", value);
    }).await;
}


#[tokio::main]
async fn main() {
    let (dispatcher, mut rx) = AsyncTaskDispatcher::new(100);

    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            tokio::spawn(task);
        }
    });

    do_something(dispatcher.clone()).await;
    do_another(dispatcher.clone(), 42).await;
    do_another(dispatcher.clone(), 99).await;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
}
```
