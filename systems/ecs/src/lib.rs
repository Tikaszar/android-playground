//! ECS System Implementation
//! 
//! This provides the actual ECS logic and functionality.
//! Core/ecs provides the data structures, systems/ecs provides ALL the operations.
//! 
//! This is like implementing the methods of an abstract base class -
//! core/ecs has the structure, systems/ecs has the behavior.

mod world_impl;
mod storage_impl;
mod vtable_handlers;
mod registration;

pub use registration::register;
pub use vtable_handlers::register_handlers;

// Re-export implementation modules for internal use
pub(crate) use world_impl::WorldImpl;
pub(crate) use storage_impl::StorageImpl;