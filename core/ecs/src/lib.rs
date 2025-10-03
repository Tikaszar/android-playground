//! Core ECS Module - EXPORTS ONLY
//!
//! Provides the fundamental ECS infrastructure with Event System.
//! Model = data structures, View = API contracts

use playground_modules_types::ViewId;

// Module constants
pub const ECS_VIEW_ID: ViewId = 0x1000_0000_0000_0001;

// Public API
pub mod error;
pub mod model;
pub mod view;

// Re-export error types
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

// Re-export view traits and structs
pub use view::{
    EntityView, ComponentView, EventView, QueryView,
    StorageView, SystemView, WorldView, EcsViewTrait, EcsView,
};