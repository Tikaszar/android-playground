# playground-core-ecs

ECS (Entity Component System) contracts and traits for the Android Playground engine.

## Overview

Core ECS provides ONLY the contracts (traits and types) that define the ECS architecture. All implementations live in `systems/ecs`. This package contains:

- **Contracts/Traits**: Interfaces that implementations must fulfill
- **Type Definitions**: Core types like EntityId, ComponentId, etc.
- **Error Types**: Standard error handling for ECS operations
- **NO Implementation Code**: This is purely contracts

**Important**: This package defines contracts only. For the actual ECS implementation, see `systems/ecs`.

## Architecture

### Core Contracts

#### WorldContract
```rust
#[async_trait]
pub trait WorldContract: Send + Sync {
    // Entity management
    async fn spawn_entity(&self) -> EcsResult<EntityId>;
    async fn despawn_batch(&self, entities: Vec<EntityId>) -> EcsResult<()>;
    
    // Component management
    async fn register_component<T: ComponentData>(&self) -> EcsResult<()>;
    async fn has_component(&self, entity: EntityId, component_id: ComponentId) -> bool;
    async fn get_component<T: ComponentData>(&self, entity: EntityId) -> EcsResult<T>;
    
    // Query system
    async fn query_entities(&self, required: Vec<ComponentId>, excluded: Vec<ComponentId>) -> EcsResult<Vec<EntityId>>;
    
    // System execution
    async fn update(&self, delta_time: f32) -> EcsResult<()>;
    
    // Messaging
    async fn publish(&self, channel: ChannelId, message: Bytes) -> EcsResult<()>;
    async fn subscribe(&self, channel: ChannelId, handler_id: String) -> EcsResult<()>;
}
```

#### ComponentData Trait
```rust
#[async_trait]
pub trait ComponentData: Send + Sync + 'static {
    fn component_id() -> ComponentId where Self: Sized;
    fn component_name() -> &'static str where Self: Sized;
    async fn serialize(&self) -> EcsResult<Bytes>;
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> where Self: Sized;
}
```

#### System Trait
```rust
#[async_trait]
pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn stage(&self) -> ExecutionStage;
    async fn initialize(&mut self) -> EcsResult<()>;
    async fn update(&mut self, delta_time: f32) -> EcsResult<()>;
    async fn cleanup(&mut self) -> EcsResult<()>;
}
```

#### Storage Trait
```rust
#[async_trait]
pub trait Storage: Send + Sync {
    fn storage_type(&self) -> StorageType;
    async fn contains(&self, entity: EntityId) -> bool;
    async fn clear(&self) -> EcsResult<()>;
    async fn len(&self) -> usize;
    async fn entities(&self) -> Vec<EntityId>;
    async fn mark_dirty(&self, entity: EntityId) -> EcsResult<()>;
    async fn get_dirty(&self) -> Vec<EntityId>;
}
```

### Core Types

#### Entity Types
```rust
pub struct EntityId {
    index: u32,
    generation: Generation,
}

pub struct Generation(u32);
```

#### Component Types
```rust
pub type ComponentId = String;  // String-based to avoid TypeId
```

#### Messaging Types
```rust
pub type ChannelId = u16;

#[async_trait]
pub trait MessageBusContract: Send + Sync {
    async fn publish(&self, channel: ChannelId, message: Bytes) -> EcsResult<()>;
    async fn subscribe(&self, channel: ChannelId, handler_id: String) -> EcsResult<()>;
    async fn unsubscribe(&self, channel: ChannelId, handler_id: &str) -> EcsResult<()>;
}
```

#### Execution Stages
```rust
pub enum ExecutionStage {
    Update,   // Game logic, input handling
    Layout,   // UI layout calculations
    Render,   // Generate render commands
}
```

## Usage

This package is used by:

1. **systems/ecs** - Implements all these contracts
2. **Other systems** - Use the traits to interact with the ECS
3. **Plugins** - Through systems/logic API (not directly)

### Example Implementation

```rust
// In systems/ecs
use playground_core_ecs::{WorldContract, ComponentData, EntityId};

pub struct World {
    // Implementation details
}

#[async_trait]
impl WorldContract for World {
    async fn spawn_entity(&self) -> EcsResult<EntityId> {
        // Actual implementation
    }
    // ... implement all trait methods
}
```

## Architectural Rules

- **NO Implementation Code**: This package contains ONLY contracts
- **NO dyn**: All traits avoid dynamic dispatch
- **NO unsafe**: No unsafe code anywhere
- **NO turbofish**: Use ComponentId instead of generics
- **Async Everything**: All I/O operations are async
- **Result Everywhere**: All fallible operations return Result

## Contract Design Principles

1. **Minimal Surface Area**: Only expose what's necessary
2. **Implementation Agnostic**: Don't assume implementation details
3. **Type Safe**: Strong typing without runtime type erasure
4. **Async First**: All potentially blocking operations are async
5. **Error Handling**: Explicit error types for all failures

## Relationship to systems/ecs

- **core/ecs**: Defines WHAT the ECS must do (contracts)
- **systems/ecs**: Defines HOW it does it (implementation)

This separation allows:
- Clean architectural boundaries
- Multiple implementations if needed
- Clear API contracts
- Better testing through mocking

## Dependencies

Minimal dependencies for contracts:
- `async-trait`: For async trait support
- `bytes`: For serialization contracts
- `serde`: For data types
- `thiserror`: For error types

## See Also

- [systems/ecs](../../systems/ecs/README.md) - The actual ECS implementation
- [systems/logic](../../systems/logic/README.md) - Public API for plugins/apps
- [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md) - Why this architecture