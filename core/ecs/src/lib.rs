//! Core ECS Module - EXPORTS ONLY
//!
//! This provides the fundamental ECS infrastructure with Event System.
//! All functionality is implemented by systems/ecs ViewModel.

// Public API exports
pub mod error;
pub mod model;
pub mod view;

// Module exports (symbols for loader)
mod module_exports;

// Re-exports for convenience
pub use error::{EcsError, EcsResult};

// Re-export all model types
pub use model::{
    // Entity types
    EntityId, Generation, Entity, EntityRef,
    // Component types
    Component, ComponentId, ComponentData,
    // Event types
    Event, EventId, Priority, Subscription, SubscriptionId,
    // World
    World,
};

