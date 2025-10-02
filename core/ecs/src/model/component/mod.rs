//! Component module - EXPORTS ONLY

pub mod component_id;
pub mod component;
pub mod component_ref;
pub mod component_pool;

// Re-exports
pub use component_id::ComponentId;
pub use component::Component;
pub use component_ref::ComponentRef;
pub use component_pool::ComponentPool;