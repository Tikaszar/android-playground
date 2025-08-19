# Systems Layer

Engine-level systems that provide core functionality to Plugins and Apps. Each system manages a specific domain and uses core/ecs internally for state management.

## Overview

The Systems layer provides the engine's core functionality. Apps and Plugins interact with these systems through `systems/logic`, which serves as both the game ECS and the access point for all other systems.

## System Architecture

```
Apps/Plugins
     ↓
systems/logic (Game ECS + System Access)
     ↓
Individual Systems (networking, ui, rendering, physics)
     ↓
Core Layer (ecs, server, types, etc.)
```

## Systems

### 🎮 [logic](./logic/README.md) - Game ECS & System Manager
**The central system that Apps/Plugins use directly**

- **Primary**: Full-featured game ECS for Apps/Plugins
- **Secondary**: Initializes and provides access to all other systems
- Hybrid archetype storage for performance
- Parallel system execution with dependencies
- NetworkedComponent trait for replication
- Query caching and hot-reload support

```rust
// Apps use this for game logic
let mut ecs = ECS::new();
let systems = ecs.initialize_systems().await?;

// Register game components
ecs.register_component::<Health>().await?;
ecs.register_component::<Position>().await?;

// Access other systems through SystemsManager
systems.networking.read().await.register_plugin("my-plugin").await?;
```

### 🌐 [networking](./networking/README.md) - WebSocket Communication
**Manages all network communication and channel routing**

- WebSocket multiplexing with binary protocol
- Channel-based routing (1-999 systems, 1000+ plugins)
- **Starts core/server internally** (Apps don't manage server)
- Packet batching at 60fps
- MCP tool registration for LLMs
- Connection state tracking via ECS

```rust
// NetworkingSystem starts server internally
let mut networking = NetworkingSystem::new().await?;
networking.initialize(None).await?; // Starts ws://localhost:8080/ws

// Register plugin for a channel
let channel = networking.register_plugin("inventory").await?;

// Send/receive packets
networking.send_packet(channel, type_id, data, Priority::Normal).await?;
let packets = networking.receive_packets(channel).await?;
```

### 🎨 [ui](./ui/README.md) - User Interface Framework
**Mobile-first UI with flexbox, gestures, and theming**

- Flexbox and absolute layout systems
- Docking panels for IDE interfaces
- Full touch gesture support
- Theme system with hot-reload
- Terminal integration for Termux
- Uses core/ecs for element state

```rust
let mut ui = UiSystem::new();
ui.initialize(renderer).await?;

// Create UI elements
let button = ui.create_element(Element::Button {
    text: "Click Me",
    on_click: Some(Box::new(|| println!("Clicked!"))),
}).await?;

// Register gestures
ui.register_gesture(element, GestureType::Swipe, |e| {
    println!("Swiped {}", e.direction);
}).await?;
```

### 🖼️ [rendering](./rendering/README.md) - GPU Graphics Pipeline
**High-performance rendering with WebGL/Vulkan backends**

- BaseRenderer trait for backend abstraction
- WebGL 2.0 implementation (Vulkan planned)
- Render graph for pass organization
- **Single draw call batching** for mobile
- Texture streaming with LOD
- Shader hot-reload
- Uses core/ecs for resource tracking

```rust
let mut rendering = RenderingSystem::<WebGLRenderer>::new().await?;
rendering.set_renderer(WebGLRenderer::new()?);
rendering.initialize().await?;

// Create GPU resources
let texture = rendering.create_texture(&TextureDesc { 
    width: 1024, height: 1024, format: RGBA8 
})?;

let pipeline = rendering.create_pipeline(&PipelineDesc {
    vertex_shader, fragment_shader, blend_state
})?;

// Render frame
rendering.begin_frame()?;
rendering.render(&render_graph)?;
rendering.present()?;
```

### ⚛️ [physics](./physics/README.md) - Physics Simulation (Minimal)
**Basic 2D physics - placeholder for future expansion**

- Simple 2D vector math
- Basic gravity and forces
- Static and dynamic bodies
- Euler integration
- **Not yet fully implemented** - will expand for games

```rust
let mut physics = PhysicsSystem::new();
physics.initialize()?;

// Add bodies
physics.add_body(PhysicsBody::new("player", Vector2::new(0.0, 10.0)))?;

// Simulation loop
physics.apply_force("player", Vector2::new(100.0, 0.0))?;
physics.step(delta_time)?;
```

## How Systems Work Together

### Initialization Flow
1. App creates `systems/logic` ECS
2. Logic's `initialize_systems()` creates SystemsManager
3. SystemsManager initializes each system:
   - NetworkingSystem starts core/server
   - UiSystem sets up with core/ecs
   - RenderingSystem (browser-side only)
   - PhysicsSystem (when implemented)
4. App gets SystemsManager for system access
5. Plugins receive Context with ECS + SystemsManager

### Communication Pattern
```
Browser ←→ WebSocket ←→ NetworkingSystem ←→ Channel Router
                              ↓
                        Other Systems
                              ↓
                     Plugins (via channels)
```

### State Management
Each system uses **core/ecs** for internal state:
- NetworkingSystem: Connection entities, channel entities
- UiSystem: Element entities with layout/style components  
- RenderingSystem: Resource entities (textures, buffers, etc.)
- PhysicsSystem: Will use ECS when expanded

Game logic uses **systems/logic** ECS:
- Game entities (players, enemies, items)
- Game components (Health, Position, Inventory)
- Game systems (CombatSystem, MovementSystem)

## Key Design Principles

### 1. Layer Separation
- Systems use Core APIs only
- Systems don't know about Plugins/Apps
- Plugins/Apps access systems through logic

### 2. Thread Safety
- All systems use `Arc<RwLock<>>` for sharing
- Async operations throughout
- No unsafe code

### 3. Mobile Optimization
- Batch operations preferred
- Single draw call rendering
- 60fps packet batching
- Touch-first UI

### 4. Error Handling
- All operations return `Result<T, Error>`
- Specific error types per system
- Graceful degradation

## Common Patterns

### Accessing Systems from Plugins
```rust
#[async_trait]
impl Plugin for MyPlugin {
    async fn on_load(&mut self, ctx: &Context) -> Result<()> {
        // Get channel from networking
        let channel = ctx.systems.networking.read().await
            .register_plugin("my-plugin").await?;
        
        // Create UI elements
        ctx.systems.ui.write().await
            .create_element(/* ... */).await?;
        
        // Use game ECS
        ctx.ecs.register_component::<MyComponent>().await?;
        
        Ok(())
    }
}
```

### System Updates
```rust
// Each system can be updated independently
loop {
    // Game ECS update (runs registered systems)
    ecs.update(delta_time).await?;
    
    // UI system update (process input, layout)
    ui.update(delta_time).await?;
    
    // Physics step
    physics.step(delta_time).await?;
    
    // Rendering frame
    rendering.render_frame().await?;
}
```

## Dependencies

All systems depend on:
- `playground-core-ecs`: Internal state management
- `playground-core-types`: Shared types
- `tokio`: Async runtime
- `async-trait`: Async traits

System-specific dependencies are listed in each system's README.

## Future Enhancements

### Physics System
- Full collision detection
- Rigid body dynamics
- 3D support
- Deterministic simulation

### Rendering System  
- Vulkan backend
- Compute shaders
- Ray tracing (far future)

### Audio System (Not Started)
- 3D spatial audio
- Music streaming
- Effect processing

### Animation System (Not Started)
- Skeletal animation
- Blend trees
- IK solving

## See Also

- [Core Layer](../core/README.md) - Foundation modules
- [Plugins Layer](../plugins/README.md) - Feature modules
- [Apps Layer](../apps/README.md) - Complete applications