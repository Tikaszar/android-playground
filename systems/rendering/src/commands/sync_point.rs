use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct SyncPoint {
    id: Arc<AtomicU64>,
}

impl SyncPoint {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self {
            id: Arc::new(AtomicU64::new(COUNTER.fetch_add(1, Ordering::Relaxed))),
        }
    }
    
    pub fn id(&self) -> u64 {
        self.id.load(Ordering::Relaxed)
    }
}