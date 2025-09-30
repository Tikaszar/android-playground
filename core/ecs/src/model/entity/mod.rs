//! Entity module - EXPORTS ONLY

pub mod entity_id;
pub mod generation;
pub mod entity;
pub mod entity_ref;

// Re-exports
pub use entity_id::EntityId;
pub use generation::Generation;
pub use entity::Entity;
pub use entity_ref::EntityRef;