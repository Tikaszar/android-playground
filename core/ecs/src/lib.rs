//! Core ECS contracts and traits
//! 
//! This module defines ONLY the contracts (traits and types) for the ECS.
//! All implementations live in systems/ecs.

pub mod component;
pub mod entity;
pub mod storage;
pub mod error;
pub mod messaging;
pub mod world;
pub mod system;
pub mod query;

// Re-export all public types
pub use component::*;
pub use entity::*;
pub use storage::*;
pub use error::*;
pub use messaging::*;
pub use world::*;
pub use system::*;
pub use query::*;