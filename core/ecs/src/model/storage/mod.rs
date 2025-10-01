//! Storage module - EXPORTS ONLY

pub mod storage_id;
pub mod storage;
pub mod storage_ref;

// Re-exports
pub use storage_id::StorageId;
pub use storage::Storage;
pub use storage_ref::StorageRef;
