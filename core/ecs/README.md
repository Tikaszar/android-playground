# playground-core-ecs

Minimal ECS (Entity Component System) primitives for Systems' internal state management.

## Overview

Core ECS provides foundational ECS functionality that Systems use for their internal state management. It features:
- Generational entity IDs with safe recycling
- Async/concurrent operations with tokio
- Runtime component registration with versioning
- Binary serialization using bytes for networking
- Component migration support for version upgrades
- Memory pool management with configurable limits
- Thread-safe operations with parking_lot and DashMap

**Note**: This is NOT for game logic. Plugins and Apps should use `systems/logic` which provides a full-featured game ECS.

## Architecture

### Entity IDs
```rust
pub struct EntityId {
    index: u32,           // Entity slot index
    generation: Generation, // Generation counter for safety
}

pub struct Generation(u32);
```

Generational IDs prevent use-after-free bugs when entities are recycled. The `EntityAllocator` manages entity allocation with a free list for recycling.

### Component Storage
```rust
#[async_trait]
pub trait Component: Send + Sync + 'static {
    fn component_id() -> ComponentId where Self: Sized {
        TypeId::of::<Self>()
    }
    
    fn component_name() -> &'static str where Self: Sized {
        std::any::type_name::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes>;
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> where Self: Sized;
    
    fn size_hint(&self) -> usize {
        std::mem::size_of_val(self)
    }
}
```

Components implement async serialization for network replication with size hints for memory management.

## Usage

### Basic World Operations
```rust
use playground_core_ecs::{World, Component, EntityId};

// Create world
let mut world = World::new();

// Register component type
world.register_component::<Position>().await?;
world.register_component::<Velocity>().await?;

// Spawn entities (batch only!)
let entities = world.spawn_batch([
    vec![Position::new(0, 0), Velocity::new(1, 0)],
    vec![Position::new(10, 10), Velocity::new(-1, 0)],
]).await?;

// Query components (NO TURBOFISH!)
let query = world.query()
    .with_component(Position::component_id())
    .with_component(Velocity::component_id())
    .build();

for entity in query.iter().await? {
    let pos = world.get_component::<Position>(entity).await?;
    let vel = world.get_component::<Velocity>(entity).await?;
    // Update position based on velocity
}
```

### Component Definition
```rust
use playground_core_ecs::{Component, ComponentId, EcsResult};
use bytes::Bytes;
use async_trait::async_trait;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Position {
    x: f32,
    y: f32,
}

#[async_trait]
impl Component for Position {
    async fn serialize(&self) -> EcsResult<Bytes> {
        let data = bincode::serialize(self)
            .map_err(|e| EcsError::SerializationError(e.to_string()))?;
        Ok(Bytes::from(data))
    }
    
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| EcsError::DeserializationError(e.to_string()))
    }
}
```

### Query API (NO TURBOFISH!)
```rust
// CORRECT - Using ComponentId
let query = world.query()
    .with_component(Position::component_id())
    .with_component(Velocity::component_id())
    .without_component(Dead::component_id())
    .build();

// WRONG - Never use turbofish!
// let query = world.query::<(&Position, &Velocity)>(); // DON'T DO THIS!
```

## Component Registration

### Basic Registration
```rust
let registry = ComponentRegistry::new();
registry.register::<Position>().await?;
```

### Advanced Registration with Info
```rust
let info = ComponentInfo::new::<Position>()
    .with_version(2)
    .networked()
    .with_migration(|bytes, old_version| {
        // Migrate from old version to current
        Ok(bytes.clone())
    });

registry.register_with_info(info).await?;
```

### Storage Types
The `ComponentStorage` enum provides different storage strategies:
- `Vec<Option<T>>` for dense components
- `HashMap<EntityId, T>` for sparse components
- `SparseSet<T>` for cache-friendly iteration

## Memory Management

### Garbage Collection
```rust
// Configure GC settings
world.set_gc_config(GcConfig {
    enabled: true,
    frame_budget_ms: 2.0,  // Max 2ms per frame
    memory_threshold: 100 * 1024 * 1024, // 100MB
});

// GC runs automatically during update
world.update().await?;
```

### Entity Recycling
```rust
// Remove entity
world.despawn(entity_id).await?;

// Entity ID slot will be recycled with new generation
let new_entity = world.spawn_empty().await?;
// new_entity might reuse slot, but generation differs
```

## Batch Operations

ALL operations are batch-only for performance:

```rust
// Spawn multiple entities
let entities = world.spawn_batch([
    components1,
    components2,
    components3,
]).await?;

// Add components to multiple entities
world.add_components_batch([
    (entity1, Health::new(100)),
    (entity2, Health::new(50)),
]).await?;

// Remove components from multiple entities
world.remove_components_batch::<Health>(&[
    entity1,
    entity2,
]).await?;
```

## Thread Safety

All operations are thread-safe using Arc<RwLock<>>:

```rust
// World can be shared across threads
let world = Arc::new(RwLock::new(World::new()));

// Clone for another thread
let world_clone = world.clone();
tokio::spawn(async move {
    let mut world = world_clone.write().await;
    // Use world safely
});
```

## Testing

```rust
#[tokio::test]
async fn test_entity_spawn() {
    let mut world = World::new();
    world.register_component::<Position>().await.unwrap();
    
    let entities = world.spawn_batch([
        vec![Position::new(0, 0)],
    ]).await.unwrap();
    
    assert_eq!(entities.len(), 1);
    
    let pos = world.get_component::<Position>(entities[0])
        .await
        .unwrap();
    assert_eq!(pos.x, 0.0);
}
```

## Architectural Rules

- This is for Systems' INTERNAL state only
- Plugins/Apps should use `systems/logic` instead
- NO turbofish syntax - use ComponentId
- ALL operations must be batch-only
- NO unsafe code anywhere
- NO std::any::Any usage

## Common Patterns

### System Internal State
```rust
// How Systems should use core/ecs
pub struct UiSystem {
    world: World,  // Internal ECS state
}

impl UiSystem {
    pub async fn new() -> Result<Self> {
        let mut world = World::new();
        world.register_component::<UiElement>().await?;
        world.register_component::<UiLayout>().await?;
        Ok(Self { world })
    }
    
    pub async fn add_element(&mut self, element: UiElement) -> Result<EntityId> {
        let entities = self.world.spawn_batch([
            vec![element],
        ]).await?;
        Ok(entities[0])
    }
}
```

### Component Pools
```rust
// Prevent memory exhaustion
world.set_component_pool_size::<Bullet>(10000);

// Warn on large components
world.set_size_warning_threshold(1024); // Warn if component > 1KB
```

## Performance

- **Generational IDs**: O(1) validation, prevents use-after-free
- **Batch Operations**: Reduces allocator pressure on mobile
- **Async Operations**: Non-blocking I/O for serialization
- **Incremental GC**: 2ms frame budget prevents stutters
- **Memory Pools**: Reuses allocations

## Dependencies

- `tokio`: Async runtime
- `async-trait`: Async trait support
- `bytes`: Efficient byte buffers
- `bincode`: Default serialization format
- `dashmap`: Concurrent hash maps
- `parking_lot`: Fast synchronization primitives
- `serde`: Serialization framework

## See Also

- [systems/logic](../../systems/logic/README.md) - Full game ECS for Plugins/Apps
- [core/types](../types/README.md) - Shared types including ComponentId
- [systems/networking](../../systems/networking/README.md) - Network replication