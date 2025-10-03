//! Handle type for external references

use std::sync::Arc;

/// Handle for external references to objects with internal state
pub type Handle<T> = Arc<T>;

/// Create a new handle
pub fn handle<T>(value: T) -> Handle<T> {
    Arc::new(value)
}