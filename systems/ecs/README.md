# playground-systems-ecs

The unified ECS (Entity Component System) implementation for the Android Playground engine.

## Overview

Systems/ECS is the single, authoritative ECS implementation for the entire engine. It implements all contracts defined in `core/ecs` and provides:

- **World**: Central container for all entities and components
- **Entity Management**: Generational entity IDs with safe recycling
- **Component Storage**: Sparse and Dense storage strategies (NO dyn)
- **Query System**: Efficient entity queries without turbofish
- **System Scheduling**: Staged execution pipeline
- **Messaging**: Core ECS functionality for inter-system communication
- **Memory Management**: Garbage collection and pooling

**Important**: This is the ONLY ECS implementation in the engine. There is no separate "game ECS" - this unified system serves all needs.

## Architecture

### The World

The World is the central container for all ECS data:

```rust
pub struct World {
    // Entity management
    entities: Shared<HashMap<EntityId, Vec<ComponentId>>>,
    allocator: Shared<EntityAllocator>,
    
    // Component storage
    storages: Shared<HashMap<ComponentId, Handle<ComponentStorage>>>,
    registry: Handle<ComponentRegistry>,
    
    // System scheduling
    scheduler: Handle<SystemScheduler>,
    
    // Messaging (core ECS functionality)
    message_bus: Handle<MessageBus>,
    
    // Memory management
    gc: Handle<GarbageCollector>,
    memory_stats: Shared<MemoryStats>,
}
```

### Storage System (NO dyn)

Component storage uses an enum pattern to avoid dynamic dispatch:

```rust
pub enum ComponentStorage {
    Sparse(SparseStorage),
    Dense(DenseStorage),
}
```

- **SparseStorage**: HashMap-based for components with few instances
- **DenseStorage**: Vec-based for components with many instances

### Staged Execution Pipeline

Systems execute in three stages:

```rust
pub enum ExecutionStage {
    Update,   // Game logic, input handling, state changes
    Layout,   // UI layout calculations, spatial organization
    Render,   // Generate render commands for browser
}
```

### Messaging as Core ECS

Messaging is NOT a separate system but fundamental ECS functionality:

```rust
impl World {
    // Messaging is built into the World
    pub async fn publish(&self, channel: ChannelId, message: Bytes) -> EcsResult<()>;
    pub async fn subscribe(&self, channel: ChannelId, handler_id: String) -> EcsResult<()>;
    pub fn message_bus(&self) -> Handle<MessageBus>;
}
```

## Usage

### Basic World Operations

```rust
use playground_systems_ecs::{World, ComponentData, EntityId};

// Create the unified world
let world = World::new();

// Register component types
world.register_component::<Position>().await?;
world.register_component::<Velocity>().await?;

// Spawn entities (batch operations)
let entities = world.spawn_batch(vec![
    vec![Position::new(0, 0), Velocity::new(1, 0)],
    vec![Position::new(10, 10), Velocity::new(-1, 0)],
]).await?;

// Query entities (NO TURBOFISH!)
let query = world.query()
    .with_component(Position::component_id())
    .with_component(Velocity::component_id());
    
let matching = world.execute_query(&query).await?;
```

### Component Definition

```rust
use playground_systems_ecs::{ComponentData, ComponentId, EcsResult};
use bytes::Bytes;
use async_trait::async_trait;

#[derive(Clone, Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[async_trait]
impl ComponentData for Position {
    fn component_id() -> ComponentId where Self: Sized {
        "Position".to_string()
    }
    
    fn component_name() -> &'static str where Self: Sized {
        "Position"
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        Ok(bincode::serialize(self)?.into())
    }
    
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> {
        Ok(bincode::deserialize(bytes)?)
    }
}
```

### System Implementation

```rust
use playground_systems_ecs::{System, ExecutionStage, EcsResult};

pub struct PhysicsSystem {
    // System state
}

#[async_trait]
impl System for PhysicsSystem {
    fn name(&self) -> &str {
        "PhysicsSystem"
    }
    
    fn stage(&self) -> ExecutionStage {
        ExecutionStage::Update
    }
    
    async fn update(&mut self, delta_time: f32) -> EcsResult<()> {
        // Update physics
        Ok(())
    }
}
```

### Messaging

```rust
// Publish a message
world.publish(channel_id, message_bytes).await?;

// Subscribe to messages
world.subscribe(channel_id, "my_handler".to_string()).await?;

// Get direct access to message bus
let bus = world.message_bus();
bus.subscribe_with_handler(channel_id, handler).await?;
```

## Key Features

### Generational Entity IDs

Prevents use-after-free bugs:

```rust
pub struct EntityId {
    index: u32,           // Entity slot
    generation: Generation, // Safety counter
}
```

### Query System (NO TURBOFISH!)

```rust
// CORRECT - Using ComponentId
let query = world.query()
    .with_component(Position::component_id())
    .without_component(Dead::component_id());

// WRONG - Never use turbofish!
// let query = world.query::<(&Position, &Velocity)>(); // DON'T DO THIS!
```

### Batch Operations

All operations are batch-optimized:

```rust
// Spawn multiple entities at once
let entities = world.spawn_batch(vec![
    components1,
    components2,
    components3,
]).await?;

// Despawn multiple entities
world.despawn_batch(entities).await?;
```

### Garbage Collection

Incremental GC with frame budget:

```rust
// GC runs automatically during update
world.update(delta_time).await?;

// Or manually trigger
let collected = world.run_gc().await?;
```

## Architectural Rules

- **NO dyn**: Enum patterns instead of trait objects
- **NO unsafe**: Everything is safe Rust
- **NO turbofish**: ComponentId-based queries
- **Handle<T> for external**: External references to objects
- **Shared<T> for internal**: Internal mutable state
- **Async everything**: All I/O is async
- **Batch operations**: Better performance on mobile
- **Result everywhere**: Explicit error handling

## Implementation Status

✅ **Complete**:
- World implementation with all core functionality
- Entity allocator with generational IDs
- Component registry and storage (Sparse/Dense)
- Query system without turbofish
- System scheduler with staged execution
- Messaging as core ECS functionality
- Garbage collection with frame budget
- Full implementation of all core/ecs contracts

## Relationship to Other Packages

- **core/ecs**: Defines the contracts we implement
- **systems/logic**: Provides public API using our World
- **Other systems**: Use our ECS for their state
- **Plugins**: Access through systems/logic (not directly)

## Performance Considerations

- **Generational IDs**: O(1) validation
- **Archetype Storage**: Cache-friendly iteration
- **Batch Operations**: Reduced allocator pressure
- **Incremental GC**: 2ms frame budget
- **Async Operations**: Non-blocking I/O

## Dependencies

- `playground-core-ecs`: Contracts we implement
- `playground-core-types`: Handle, Shared types
- `tokio`: Async runtime
- `async-trait`: Async trait support
- `bytes`: Efficient byte buffers
- `bincode`: Serialization
- `indexmap`: Ordered maps

## See Also

- [core/ecs](../../core/ecs/README.md) - The contracts we implement
- [systems/logic](../../systems/logic/README.md) - Public API for plugins/apps
- [DESIGN_DECISIONS.md](../../DESIGN_DECISIONS.md) - Architectural decisions
- [DESIGN_CLARIFICATION.md](../../DESIGN_CLARIFICATION.md) - Unified ECS design