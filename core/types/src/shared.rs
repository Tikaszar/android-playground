use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared mutable state using Arc<RwLock<T>>
/// This is the ONLY approved concurrent access pattern in the codebase
pub type Shared<T> = Arc<RwLock<T>>;

/// Helper to create a new Shared<T>
pub fn shared<T>(value: T) -> Shared<T> {
    Arc::new(RwLock::new(value))
}