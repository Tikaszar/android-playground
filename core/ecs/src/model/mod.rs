//! Model - Data structures EXPORTS ONLY

pub mod entity;
pub mod component;
pub mod event;
pub mod world;

// Re-exports for convenience
pub use entity::{EntityId, Generation, Entity, EntityRef};
pub use component::{Component, ComponentId, ComponentData};
pub use event::{Event, EventId, Priority, Subscription, SubscriptionId};
pub use world::World;