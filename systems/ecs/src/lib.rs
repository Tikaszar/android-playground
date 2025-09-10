//! Unified ECS Implementation for Android Playground Engine
//! 
//! This is the single, authoritative ECS implementation for the entire engine.
//! It provides the concrete World that manages all entities, components, and systems.

pub mod world;
pub mod storage;
pub mod entity;
pub mod component;
pub mod query;
pub mod scheduler;
pub mod messaging;

// Re-export main types
pub use world::World;
pub use scheduler::SystemScheduler;
pub use messaging::{MessageBus, MessageHandler, BroadcasterWrapper};

// Re-export from core/ecs (contracts only)
pub use playground_core_ecs::{
    ComponentData, ComponentId, EntityId, Generation,
    EcsError, EcsResult,
    ChannelId, MessageHandlerData, BroadcasterData, MessageBusContract,
    Storage, StorageType,
    WorldContract, System, ExecutionStage,
    Query
};