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
pub mod world_commands;
pub mod world_access;
pub mod system_commands;

// Re-export all public types
pub use component::*;
pub use entity::*;
pub use storage::*;
pub use error::*;
pub use messaging::*;
pub use world::*;
pub use system::*;
pub use query::*;
pub use world_commands::*;
pub use world_access::*;
pub use system_commands::*;