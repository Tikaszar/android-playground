//! Shared type for internal mutable state

use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared mutable state (for internal fields only)
pub type Shared<T> = Arc<RwLock<T>>;

/// Create a new shared value
pub fn shared<T>(value: T) -> Shared<T> {
    Arc::new(RwLock::new(value))
}