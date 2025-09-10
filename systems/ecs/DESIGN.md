# systems/ecs - Unified ECS Implementation

## Overview

`systems/ecs` is the **single, unified ECS implementation** for the entire Android Playground engine. This package provides the concrete World implementation that manages all entities, components, and systems across the engine.

## Architecture Position

```
┌─────────────────────────────────────────────────┐
│                    Apps                         │
│         (Use only systems/logic API)            │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│                  Plugins                        │
│      (High-level systems, hot-reloadable)       │
│         (Use only systems/logic API)            │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│              systems/logic                      │
│        (Stateless API Gateway)                  │
│    (Public types and functions only)            │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│              systems/ecs                        │
│         (Unified World Implementation)          │
│      (Manages all entities/components)          │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│                core/ecs                         │
│         (Contracts and Traits Only)             │
│              (No Implementation)                │
└──────────────────────────────────────────────────┘
```

## Key Design Principles

### 1. Single Source of Truth
- **ONE World** for the entire engine
- All entities and components live here
- No separate ECS systems causing confusion

### 2. Implements core/ecs Contracts
- Implements all traits defined in core/ecs
- Provides concrete types for abstract contracts
- Maintains compatibility with core interfaces

### 3. Staged Execution Pipeline
The World executes systems in three distinct stages:

```rust
pub enum ExecutionStage {
    Update,   // Game logic, input handling, state changes
    Layout,   // UI layout calculations, spatial organization
    Render,   // Generate render commands for browser
}
```

### 4. System Scheduling
- Engine systems (ui, networking, webgl) auto-register
- Plugins register through systems/logic API
- Deterministic execution order within stages
- Parallel execution where possible

## Core Components

### World
The central ECS container:
```rust
pub struct World {
    entities: Shared<EntityStore>,
    components: Shared<ComponentStore>,
    systems: Shared<SystemScheduler>,
    resources: Shared<ResourceMap>,
}
```

### EntityStore
Manages entity lifecycle:
- Entity creation/destruction
- Generation tracking for safety
- Efficient entity iteration

### ComponentStore
Stores component data:
- Type-erased storage (NO dyn, using concrete wrappers)
- Efficient archetype-based storage
- Fast component queries

### SystemScheduler
Orchestrates system execution:
- Stage-based execution
- Dependency resolution
- Parallel scheduling where safe

### Query System
Efficient component queries:
```rust
// NO TURBOFISH! Use ComponentId
query.with_component(Position::component_id())
     .with_component(Velocity::component_id())
     .execute(&world)
```

## Integration Points

### With core/ecs
- Implements all core/ecs traits
- Uses core/ecs types where defined
- Maintains contract compatibility

### With systems/logic
- systems/logic creates and owns the World
- Provides World access to registered systems
- Handles plugin registration requests

### With Engine Systems
- UI System queries UI components
- Networking System manages network entities
- WebGL System queries renderable entities

### With Plugins
- Plugins interact ONLY through systems/logic
- Cannot directly access World
- Use high-level API functions

## Strict Rules Compliance

### NO dyn
- Use concrete wrapper types
- Type erasure through enums or wrappers
- Static dispatch wherever possible

### NO unsafe
- All operations are safe Rust
- Use Arc/RwLock for shared state
- Generation IDs for entity safety

### Handle vs Shared
- World exposed as Handle<World> externally
- Internal fields use Shared<T> for mutation
- Clear ownership boundaries

### Async Operations
- All public methods are async
- Proper .await propagation
- tokio::sync::RwLock ONLY

## Performance Considerations

### Archetype Storage
- Components grouped by usage patterns
- Cache-friendly iteration
- Minimal memory overhead

### Parallel Execution
- Systems within a stage can run in parallel
- Automatic dependency detection
- Work-stealing scheduler

### Query Optimization
- Query results cached where possible
- Lazy evaluation of complex queries
- Batch operations preferred

## Example Usage

### From systems/logic (API Gateway)
```rust
// systems/logic provides this API to plugins
pub async fn create_entity_with_components(
    components: Vec<ComponentData>
) -> Result<EntityId> {
    // Internally calls systems/ecs World
    let world = self.world.read().await;
    world.create_entity(components).await
}
```

### From Engine System
```rust
// UiSystem queries UI components
async fn update(&mut self, world: &World) -> Result<()> {
    let query = world.query()
        .with_component(UiElement::component_id())
        .with_component(Transform::component_id());
    
    for entity in query.execute().await? {
        // Process UI elements
    }
    Ok(())
}
```

### From Plugin (through systems/logic)
```rust
// Plugin uses systems/logic API
async fn update(&mut self, api: &SystemsLogicApi) -> Result<()> {
    // Create UI element through API
    api.create_ui_element(UiElementData {
        id: "button",
        text: "Click me",
    }).await?;
    Ok(())
}
```

## Migration from core/ecs

### Current core/ecs Implementation (To Be Moved)
The following implementations currently in core/ecs will be moved to systems/ecs:

#### Complete Implementations to Move:
- **World struct** - Full ECS container with all functionality
- **ComponentStorage** - SparseStorage and DenseStorage implementations
- **ComponentRegistry** - Component registration with memory pooling
- **EntityAllocator** - Entity ID generation and lifecycle
- **GarbageCollector** - Incremental GC for entity cleanup
- **MessageBus** - Event system (needs refactor to remove dyn MessageHandler)
- **Query system** - QueryBuilder and execution logic
- **MemoryStats** - Memory tracking and pressure monitoring

#### What Stays in core/ecs (Contracts Only):
- **ComponentData trait** - Interface for component types
- **Storage trait** - Storage system interface
- **ComponentId type** (String alias)
- **EntityId struct** - Just the ID structure, no allocator
- **Error types** - EcsError, EcsResult

### Known Issues to Address:
1. **MessageHandler uses dyn** - Needs refactoring to concrete wrapper pattern
2. **DashMap in dependencies** - Replace with Shared<HashMap>
3. **Component wrapper pattern** - Already good, keep as-is

## Migration Path

### Phase 1: Create Package Structure
1. Copy World implementation from core/ecs
2. Copy storage implementations (Sparse, Dense)
3. Copy EntityAllocator and GarbageCollector
4. Fix imports to use core/ecs traits

### Phase 2: Refactor for Compliance
1. Remove MessageHandler dyn usage
2. Replace DashMap with Shared<HashMap>
3. Ensure all async propagation
4. Verify NO unsafe, NO dyn rules

### Phase 3: Add Staged Execution
1. Implement ExecutionStage enum
2. Create SystemScheduler for staged execution
3. Add system registration mechanism
4. Implement parallel execution within stages

### Phase 4: Integrate with systems/logic
1. systems/logic creates World instance
2. Expose through API functions
3. Remove old ECS code from systems/logic
4. Create translation layer for public types

### Phase 5: Update core/ecs to Contracts Only
1. Remove all implementation code
2. Keep only traits and type definitions
3. Update all imports across codebase
4. Ensure backward compatibility

### Phase 6: Migrate Engine Systems
1. Update systems to use new World location
2. Add auto-registration for engine systems
3. Test query-based access
4. Verify system isolation

### Phase 7: Enable Plugin Access
1. Complete API surface in systems/logic
2. Test with existing plugins
3. Ensure hot-reload still works
4. Verify plugins can't access World directly

## Testing Strategy

### Unit Tests
- Component storage and retrieval
- Entity lifecycle management
- Query correctness

### Integration Tests
- Multi-system coordination
- Stage execution order
- Plugin interaction through API

### Performance Tests
- Query performance benchmarks
- Parallel execution scaling
- Memory usage patterns

## Success Criteria

1. ✅ Single World instance for entire engine
2. ✅ All core/ecs contracts implemented
3. ✅ systems/logic successfully uses World
4. ✅ Engine systems query components correctly
5. ✅ Plugins work through API without direct access
6. ✅ Hot-reload continues to function
7. ✅ No dyn, no unsafe, no turbofish
8. ✅ All operations async with proper error handling