use std::sync::Arc;
use tokio::sync::RwLock;

/// Handle<T> - External reference to a type that manages its own internal state
/// Used when one class/struct needs to reference another
/// The referenced type manages its own internal locking via Shared<T> fields
pub type Handle<T> = Arc<T>;

/// Shared<T> - Internal mutable state within a class/struct
/// Used ONLY for private fields that need thread-safe mutation
/// Never expose Shared<T> directly to external code
pub type Shared<T> = Arc<RwLock<T>>;

/// Helper to create a new Handle<T> for external references
pub fn handle<T>(value: T) -> Handle<T> {
    Arc::new(value)
}

/// Helper to create a new Shared<T> for internal state
pub fn shared<T>(value: T) -> Shared<T> {
    Arc::new(RwLock::new(value))
}