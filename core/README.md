# Core Infrastructure

Foundation crates providing essential functionality for the Android Playground engine.

## Overview

The core layer provides the fundamental infrastructure that all other components build upon. These crates have minimal dependencies and define the core abstractions used throughout the engine.

## Crates

### [core/server](./server/README.md)
WebSocket multiplexer with channel management and integrated MCP server for universal LLM support.
- Binary packet protocol with priority queuing
- Frame-based batching at 60fps
- Dynamic channel registration
- MCP (Model Context Protocol) integration
- Static file serving for browser clients

### [core/client](./client/README.md)
WASM browser client with WebSocket connectivity and automatic reconnection.
- WebSocket connection management
- Exponential backoff reconnection
- Channel registration and discovery
- JavaScript/TypeScript bindings
- Binary size optimized (431KB)

### [core/types](./types/README.md)
Shared type definitions used across all crates.
- Network protocol types (Priority, Packet, ChannelId)
- Plugin system types (PluginId, PluginMetadata, Version)
- Context and state management
- Zero dependencies for fast compilation

### [core/plugin](./plugin/README.md)
Dynamic plugin system with hot-reload support.
- Load plugins from .so files
- State preservation across reloads
- Async lifecycle management
- Safe FFI boundaries
- Inter-plugin communication via Context

### [core/ecs](./ecs/README.md)
Minimal Entity Component System for Systems' internal state management.
- Generational entity IDs
- Async/concurrent operations
- Component versioning and migration
- Memory pool management
- Thread-safe with parking_lot

### [core/math](./math/README.md)
Mathematical primitives and utilities.
- Built on nalgebra
- GPU buffer compatibility (optional)
- Math library interoperability (optional)
- 🚧 Under construction

### [core/android](./android/README.md)
Android platform integration.
- JNI bindings for Android APIs
- Logcat integration
- Asset management
- Activity lifecycle handling
- Touch input processing

## Architecture Principles

### Layering Rules
- Core crates cannot depend on Systems
- Core crates cannot depend on Plugins
- Core crates cannot depend on Apps
- Core crates can only depend on other Core crates

### Dependency Guidelines
- Minimize external dependencies
- Prefer async operations
- Use Arc<RwLock<>> for thread safety
- All operations return Result<T, Error>
- Batch operations for performance

### Common Patterns

#### Thread Safety
```rust
use std::sync::Arc;
use tokio::sync::RwLock;

let shared_state = Arc::new(RwLock::new(State::new()));
```

#### Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Serialization failed: {0}")]
    Serialization(String),
}
```

#### Async Operations
```rust
use async_trait::async_trait;

#[async_trait]
pub trait AsyncComponent {
    async fn initialize(&mut self) -> Result<(), Error>;
    async fn update(&mut self, delta: f32) -> Result<(), Error>;
}
```

## Building

### All Core Crates
```bash
# Build all core crates
cargo build -p playground-core-server \
           -p playground-core-client \
           -p playground-core-types \
           -p playground-core-plugin \
           -p playground-core-ecs \
           -p playground-core-math \
           -p playground-core-android
```

### Individual Crates
```bash
# Server (native binary)
cargo build -p playground-core-server --release

# Client (WASM)
cargo build -p playground-core-client --target wasm32-unknown-unknown

# Plugin support
cargo build -p playground-core-plugin --release
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

## Channel Allocation

The core infrastructure defines standard channel ranges:

- **0**: Control channel (system messages)
- **1-999**: Systems (UI=10, Networking=20, etc.)
- **1000-1999**: Plugins (dynamic allocation)
- **2000-2999**: LLM sessions via MCP
- **3000+**: Reserved for future use

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

- `tokio`: Async runtime
- `async-trait`: Async traits
- `serde`: Serialization
- `bytes`: Efficient byte handling
- `thiserror`: Error definitions
- `parking_lot`: Fast synchronization
- `nalgebra`: Math operations

## See Also

- [Systems Layer](../systems/README.md) - Higher-level functionality
- [Plugins](../plugins/README.md) - Game logic plugins
- [Apps](../apps/README.md) - Complete applications
- [Architecture Docs](../ARCHITECTURE.md) - Overall design