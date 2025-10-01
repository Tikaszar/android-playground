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
    Component, ComponentId, ComponentRef,
    // Event types
    Event, EventId, EventRef, Priority, Subscription, SubscriptionId,
    // Query types
    Query, QueryId, QueryRef, QueryFilter,
    // Storage types
    Storage, StorageId, StorageRef,
    // System types
    System, SystemId, SystemRef, SystemStats,
    // World
    World, WorldRef, WorldStats, WorldMetadata,
};

