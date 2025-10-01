//! Component module - EXPORTS ONLY

pub mod component_id;
pub mod component;
pub mod component_ref;

// Re-exports
pub use component_id::ComponentId;
pub use component::Component;
pub use component_ref::ComponentRef;