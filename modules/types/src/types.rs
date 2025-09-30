//! Type aliases for consistency across the codebase

use std::sync::Arc;
use tokio::sync::RwLock;

/// Handle for external references to objects with internal state
pub type Handle<T> = Arc<T>;

/// Shared mutable state (for internal fields only)
pub type Shared<T> = Arc<RwLock<T>>;