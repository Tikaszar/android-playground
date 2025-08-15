use tokio::sync::oneshot;

pub struct SyncPromise<T> {
    receiver: Option<oneshot::Receiver<T>>,
}

impl<T> SyncPromise<T> {
    pub fn new() -> (Self, oneshot::Sender<T>) {
        let (sender, receiver) = oneshot::channel();
        (Self { receiver: Some(receiver) }, sender)
    }
    
    pub async fn wait(mut self) -> Result<T, String> {
        match self.receiver.take() {
            Some(rx) => rx.await.map_err(|_| "Promise dropped".to_string()),
            None => Err("Promise already consumed".to_string()),
        }
    }
    
    pub fn try_get(&mut self) -> Option<T> {
        match &mut self.receiver {
            Some(rx) => rx.try_recv().ok(),
            None => None,
        }
    }
}

impl<T> Default for SyncPromise<T> 
where
    T: Default,
{
    fn default() -> Self {
        let (promise, sender) = Self::new();
        let _ = sender.send(T::default());
        promise
    }
}