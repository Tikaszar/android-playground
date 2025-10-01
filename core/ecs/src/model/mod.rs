//! Model - Data structures EXPORTS ONLY

pub mod entity;
pub mod component;
pub mod event;
pub mod query;
pub mod storage;
pub mod system;
pub mod world;

// Re-exports for convenience
pub use entity::{EntityId, Generation, Entity, EntityRef};
pub use component::{Component, ComponentId, ComponentRef};
pub use event::{Event, EventId, EventRef, Priority, Subscription, SubscriptionId};
pub use query::{Query, QueryId, QueryRef, QueryFilter};
pub use storage::{Storage, StorageId, StorageRef};
pub use system::{System, SystemId, SystemRef};
pub use world::{World, WorldRef};