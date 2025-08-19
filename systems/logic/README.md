# playground-systems-logic

Full-featured game ECS with system initialization, parallel execution, and networking support.

## Overview

Systems/Logic provides a complete ECS framework for game development and **initializes ALL other systems**. This is the primary ECS that Plugins and Apps use for game logic.

### Key Features
- Hybrid archetype storage (fast iteration + efficient insertion)
- Parallel system execution with dependency graphs
- NetworkedComponent trait for automatic replication
- Component-based event system
- Query caching for performance
- Hot-reload support with migrations
- Batch-only API for mobile efficiency

## SPECIAL ROLE: System Initialization

**This system is unique** - it initializes ALL other systems in the engine:

```rust
use playground_systems_logic::{ECS, SystemsManager};

// Apps create ECS
let mut ecs = ECS::new();

// ECS initializes ALL systems internally
let systems = ecs.initialize_systems().await?;

// Systems include:
// - NetworkingSystem (manages core/server)
// - UiSystem 
// - RenderingSystem
// - PhysicsSystem (future)

// App passes systems to plugins via Context
let context = Context::new(systems);
plugin.on_load(&context).await?;
```

## Architecture

### Hybrid Storage
Combines archetype (fast iteration) with sparse (fast insertion):

```rust
// Dense components in archetypes
struct Position { x: f32, y: f32 }

// Sparse components in hashmaps  
struct PowerUp { type: PowerUpType }
```

### System Execution
```rust
// Define system with dependencies
fn physics_system(world: &mut World) -> Result<()> {
    // System logic
}

// Register with dependencies
ecs.add_system(physics_system)
    .depends_on::<InputSystem>()
    .runs_before::<RenderSystem>()
    .in_stage(Stage::Update);
```

## Usage

### Basic Setup
```rust
use playground_systems_logic::{ECS, Component, NetworkedComponent};

// Create ECS (Apps do this)
let mut ecs = ECS::new();

// Register components
ecs.register_component::<Position>()
    .networked()  // Auto-replicate
    .in_archetype(); // Store in archetype

ecs.register_component::<Health>()
    .networked()
    .sparse();  // Store sparsely

// Initialize systems (CRITICAL!)
let systems = ecs.initialize_systems().await?;
```

### Component Definition
```rust
#[derive(Component, Clone, Debug)]
struct Position {
    x: f32,
    y: f32,
}

// For networked components
impl NetworkedComponent for Position {
    fn is_dirty(&self) -> bool {
        // Return true if changed
    }
    
    fn clear_dirty(&mut self) {
        // Clear dirty flag
    }
}
```

### Query API (NO TURBOFISH!)
```rust
// Build query with caching
let query = ecs.query()
    .with_component(Position::component_id())
    .with_component(Velocity::component_id())
    .without_component(Dead::component_id())
    .cached()  // Cache for reuse
    .build();

// Iterate results
for entity in query.iter() {
    let pos = ecs.get::<Position>(entity)?;
    let vel = ecs.get::<Velocity>(entity)?;
    // Update logic
}
```

### Batch Operations
```rust
// Spawn entities (batch only!)
let entities = ecs.spawn_batch([
    bundle!(Position::new(0, 0), Velocity::new(1, 0)),
    bundle!(Position::new(10, 10), Velocity::new(-1, 0)),
]).await?;

// Add components batch
ecs.add_components_batch([
    (entity1, Health::new(100)),
    (entity2, Health::new(50)),
]).await?;

// Remove batch
ecs.remove_components_batch::<Health>(&entities).await?;
```

## System Definition

### Basic System
```rust
use playground_systems_logic::{System, World, Result};

pub struct PhysicsSystem;

#[async_trait]
impl System for PhysicsSystem {
    async fn update(&mut self, world: &mut World) -> Result<()> {
        let query = world.query()
            .with_component(Position::component_id())
            .with_component(Velocity::component_id())
            .build();
            
        for entity in query.iter() {
            // Physics update
        }
        Ok(())
    }
}
```

### System Dependencies
```rust
// Register with dependencies
ecs.add_system(PhysicsSystem)
    .depends_on::<InputSystem>()  // Must run after
    .runs_before::<RenderSystem>() // Must run before
    .in_stage(Stage::Update);      // Execution stage
```

### Execution Stages
```rust
pub enum Stage {
    PreUpdate,   // Input processing
    Update,      // Game logic
    PostUpdate,  // Physics
    PreRender,   // Prepare rendering
    Render,      // Render frame
}
```

## Events as Components

Events are just components with special handling:

```rust
#[derive(Component, Event)]
struct CollisionEvent {
    entity_a: EntityId,
    entity_b: EntityId,
    impact: f32,
}

// Send event
ecs.send_event(CollisionEvent {
    entity_a,
    entity_b,
    impact: 10.0,
});

// Query events
let events = ecs.query_events::<CollisionEvent>();
for event in events {
    // Handle collision
}
```

## Networking

### NetworkedComponent
```rust
#[derive(Component, NetworkedComponent)]
struct PlayerPosition {
    x: f32,
    y: f32,
    #[networked(dirty)]
    dirty: bool,
}

// Automatic replication
ecs.register_component::<PlayerPosition>()
    .networked()
    .replicate_to(ReplicationTarget::AllClients);
```

### Replication Modes
```rust
pub enum ReplicationTarget {
    AllClients,      // Replicate to everyone
    OtherClients,    // Everyone except owner  
    Owner,           // Only to owning client
    Server,          // Server authoritative
}
```

## Hot-Reload Support

### Component Migration
```rust
// Register with migration function
ecs.register_component::<Position>()
    .version(2)
    .migration(|old: &v1::Position| -> v2::Position {
        v2::Position {
            x: old.x,
            y: old.y,
            z: 0.0,  // New field
        }
    });
```

### Plugin Hot-Reload
```rust
// Save state before reload
let state = plugin.save_state()?;

// Reload plugin
plugin.reload()?;

// Restore state
plugin.load_state(state)?;
```

## Performance Features

### Query Caching
```rust
// Cache frequently used queries
let query = ecs.query()
    .with_component(Position::component_id())
    .cached()  // Reuse this query
    .build();

// Cached queries are ~10x faster
```

### Parallel Execution
```rust
// Systems run in parallel when possible
ecs.add_system(PhysicsSystem).parallel();
ecs.add_system(AiSystem).parallel();
ecs.add_system(AnimationSystem).parallel();

// Automatic dependency resolution
```

### Incremental GC
```rust
// Configure garbage collection
ecs.set_gc_config(GcConfig {
    enabled: true,
    frame_budget_ms: 2.0,
    incremental: true,
});
```

## Testing

```rust
#[tokio::test]
async fn test_system_execution() {
    let mut ecs = ECS::new();
    
    // Register components
    ecs.register_component::<Position>().await?;
    
    // Add system
    ecs.add_system(TestSystem);
    
    // Run one frame
    ecs.update(16.67).await?;
    
    // Verify results
    let query = ecs.query()
        .with_component(Position::component_id())
        .build();
    assert!(query.count() > 0);
}
```

## Architectural Rules

- **SPECIAL**: This system initializes ALL other systems
- Apps create this, then pass systems to Plugins
- This is the game ECS (not core/ecs)
- NO turbofish syntax allowed
- Batch operations only
- NO unsafe code
- Thread-safe with Arc<RwLock<>>

## Common Patterns

### Game Loop
```rust
// Main game loop
loop {
    let dt = frame_time();
    
    // Update ECS
    ecs.update(dt).await?;
    
    // Systems execute in order:
    // 1. PreUpdate (input)
    // 2. Update (logic)
    // 3. PostUpdate (physics)
    // 4. PreRender (prepare)
    // 5. Render (draw)
    
    // Present frame
    renderer.present()?;
}
```

### Resource Management
```rust
// Global resources
ecs.insert_resource(Time::default());
ecs.insert_resource(Input::default());

// Access in systems
let time = world.resource::<Time>()?;
let input = world.resource::<Input>()?;
```

## Performance

- **Hybrid Storage**: Best of both worlds
- **Parallel Systems**: Multi-core utilization
- **Query Caching**: 10x faster repeated queries
- **Batch Operations**: Reduced allocator pressure
- **Incremental GC**: No frame drops

## Dependencies

- `playground-core-ecs`: Foundation ECS primitives
- `playground-core-types`: Shared types
- `tokio`: Async runtime
- `rayon`: Parallel execution
- `async-trait`: Async traits

## See Also

- [core/ecs](../../core/ecs/README.md) - Minimal ECS for Systems
- [systems/networking](../networking/README.md) - Network replication
- [plugins/inventory](../../plugins/inventory/README.md) - Example game plugin