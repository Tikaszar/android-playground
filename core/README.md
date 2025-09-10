# Core Infrastructure

Foundation layer providing contracts, traits, and essential infrastructure for the Android Playground engine.

## Overview

The core layer is **entirely stateless**, providing only contracts (traits), type definitions, and essential infrastructure. Core packages define WHAT components must do, not HOW they do it. All stateful implementations live in the systems layer.

## Architecture Principle: Stateless Contracts Only

**CRITICAL**: Core packages contain NO implementation of business logic. They provide:
- **Traits and Contracts**: Interfaces that define behavior
- **Type Definitions**: Shared types used across layers
- **Infrastructure**: Essential services like networking and client support
- **NO State Management**: All state lives in systems layer
- **NO Business Logic**: All logic lives in systems or plugins

## Crates

### [core/ecs](./ecs/README.md) - ECS Contracts
**Pure contracts and traits** for the Entity Component System.
- `WorldContract` trait - defines what a World must do
- `ComponentData` trait - defines component behavior
- `Storage` trait - defines storage interfaces
- `System` trait - defines system interfaces
- Type definitions (EntityId, ComponentId, ChannelId)
- **NO implementation** - see systems/ecs for the actual ECS

### [core/ui](./ui/README.md) - UI Contracts
**Pure contracts and traits** for UI systems.
- `UiRenderer` trait - defines rendering interface
- `UiElement` trait - defines element behavior
- `LayoutEngine` trait - defines layout calculations
- Common UI types and enums
- **NO implementation** - see systems/ui for actual UI

### [core/types](./types/README.md) - Shared Types
Fundamental type definitions used across all layers.
- `Handle<T>` and `Shared<T>` concurrency types
- Network protocol types (Priority, Packet, ChannelId)
- Error types and results
- Common enums and constants
- Zero dependencies for fast compilation

### [core/server](./server/README.md) - Infrastructure Service
WebSocket server infrastructure (NOT business logic).
- Binary packet protocol with priority queuing
- Frame-based batching at 60fps
- Channel multiplexing
- MCP (Model Context Protocol) server
- Dashboard and logging infrastructure
- **Note**: This is infrastructure, not business logic

### [core/client](./client/README.md) - Browser Infrastructure
WASM browser client infrastructure.
- WebSocket connection management
- Exponential backoff reconnection
- Binary protocol handling
- JavaScript/TypeScript bindings
- Binary size optimized (431KB)
- **Note**: This is infrastructure, not business logic

### Deprecated/To Be Removed

The following packages violate the stateless principle and should be removed or refactored:

- **core/plugin** - Plugin loading belongs in systems layer
- **core/math** - Should be in systems or a separate utility crate
- **core/android** - Platform integration belongs in systems layer

## Architecture Principles

### Stateless Design Rules
- **NO implementation** of business logic in core
- **NO state management** - only type definitions
- **ONLY contracts** - traits that define behavior
- **ONLY infrastructure** - essential services like networking
- Core defines WHAT, Systems define HOW

### Layering Rules
- Core crates cannot depend on Systems
- Core crates cannot depend on Plugins
- Core crates cannot depend on Apps
- Core crates can only depend on other Core crates

### Dependency Guidelines
- Minimize external dependencies
- All traits must use `async_trait`
- Use `Handle<T>` for external references
- Use `Shared<T>` for internal state (in systems layer)
- All operations return `Result<T, Error>`
- Batch operations for performance

### Common Patterns

#### Contract Definition (core layer)
```rust
use async_trait::async_trait;
use playground_core_types::{Handle, EcsResult};

#[async_trait]
pub trait WorldContract: Send + Sync {
    async fn spawn_entity(&self) -> EcsResult<EntityId>;
    async fn despawn_batch(&self, entities: Vec<EntityId>) -> EcsResult<()>;
    // Define WHAT, not HOW
}
```

#### Implementation (systems layer)
```rust
// This goes in systems/ecs, NOT in core!
pub struct World {
    // Actual implementation with state
}

#[async_trait]
impl WorldContract for World {
    async fn spawn_entity(&self) -> EcsResult<EntityId> {
        // HOW it's done
    }
}
```

#### Handle/Shared Pattern
```rust
use playground_core_types::{Handle, handle, Shared, shared};

// External reference (no locking needed)
let world: Handle<World> = handle(World::new());
world.some_method().await;

// Internal state (requires locking) - ONLY in systems layer
struct MySystem {
    data: Shared<HashMap<String, Value>>, // Private field
}
```

## Building

### Active Core Crates
```bash
# Build all active core crates
cargo build -p playground-core-ecs \
           -p playground-core-ui \
           -p playground-core-types \
           -p playground-core-server \
           -p playground-core-client
```

### Individual Crates
```bash
# ECS contracts
cargo build -p playground-core-ecs

# UI contracts
cargo build -p playground-core-ui

# Types
cargo build -p playground-core-types

# Server infrastructure
cargo build -p playground-core-server --release

# Client (WASM)
cargo build -p playground-core-client --target wasm32-unknown-unknown
```

## Testing

```bash
# Test all core crates
cargo test --workspace --all-features

# Test specific crate
cargo test -p playground-core-ecs

# Integration tests
cargo test --test '*' --workspace
```

## Performance Characteristics

| Crate | Binary Size | Memory Usage | Startup Time |
|-------|------------|--------------|--------------|
| server | ~5MB | ~50MB | <100ms |
| client | 431KB WASM | ~2MB | <50ms |
| types | N/A (library) | Minimal | N/A |
| plugin | ~1MB | Per-plugin | <10ms |
| ecs | N/A (library) | Configurable | <5ms |
| math | N/A (library) | Minimal | N/A |
| android | ~2MB | ~20MB | <200ms |

## Channel System

The engine uses a dynamic channel allocation system:

- **Channel 0**: Control channel (ONLY hardcoded channel)
  - Used for discovery and channel manifest
  - Browser connects here first to learn about other channels
- **All other channels**: Dynamically allocated
  - Systems and plugins register with SystemsManager
  - Channels assigned sequentially as needed
  - Browser discovers mappings via channel 0 manifest

## Common Issues

### WebSocket Connection Failed
- Ensure server is running on correct port
- Check firewall settings
- Verify protocol (ws:// vs wss://)

### Plugin Loading Failed
- Verify .so file exists and has correct permissions
- Check for missing dependencies
- Ensure create_plugin export exists

### WASM Module Not Loading
- Set correct MIME type for .wasm files
- Verify file path is correct
- Check browser console for errors

## Dependencies

Core crates use minimal, well-maintained dependencies:

**For Contracts (core/ecs, core/ui)**:
- `async-trait`: Async trait support
- `bytes`: Byte buffer types
- `serde`: Serialization traits
- `thiserror`: Error definitions

**For Infrastructure (core/server, core/client)**:
- `tokio`: Async runtime
- `axum`: Web server framework (server only)
- `wasm-bindgen`: WASM bindings (client only)

**Note**: NO `parking_lot` - use `tokio::sync::RwLock` only

## See Also

- [Systems Layer](../systems/README.md) - Higher-level functionality
- [Plugins](../plugins/README.md) - Game logic plugins
- [Apps](../apps/README.md) - Complete applications
- [Architecture Docs](../ARCHITECTURE.md) - Overall design