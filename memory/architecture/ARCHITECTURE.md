# Architecture - Complete Engine Architecture

## Core Architectural Pattern (Sessions 52-57)

### The Current Architecture
```
Apps/Plugins → core/* packages (contracts only) → [VTable dispatch] → systems/* (implementations)
```

## Data vs Logic Separation Pattern

### Abstract Base Class Pattern
- **core/** packages = Abstract base classes (data fields ONLY, no logic)
- **systems/** packages = Concrete implementations (ALL the logic)
- **VTable** = Runtime method dispatch between them

This pattern allows polymorphic behavior without `dyn` trait objects, maintaining compile-time type safety while achieving runtime dispatch.

### Example Structure
```rust
// core/server/src/server.rs - Data ONLY
pub struct Server {
    pub vtable: VTable,
    pub config: Shared<ServerConfig>,
    pub stats: Shared<ServerStats>,
    pub connections: Shared<HashMap<ConnectionId, ConnectionInfo>>,
    // ... just data fields
}

// systems/networking/src/vtable_handlers.rs - Logic ONLY
pub async fn handle_server_operations(operation: String, payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "start" => handle_server_start(payload).await,
        "stop" => handle_server_stop().await,
        // ... actual implementation
    }
}
```

## VTable Dispatch System

### How VTable Works
1. Core packages create global instances with VTable
2. Systems register handlers during initialization
3. Core methods delegate through VTable to systems
4. Systems perform work and update core's data fields

### VTable Command Flow
```rust
// 1. App calls core API
playground_core_server::start_server(config).await

// 2. Core delegates through VTable
server.vtable.send_command("server", "start", payload).await

// 3. System handler receives and processes
handle_server_operations("start", payload).await

// 4. System updates core's data fields
let server = playground_core_server::get_server_instance()?;
*server.is_running.write().await = true;
```

## Feature Flag System

### Compile-Time Capabilities
Core packages use Cargo features to determine available capabilities:
- `websocket`, `tcp`, `udp`, `ipc` - Transport protocols
- `channels` - Channel-based messaging
- `batching` - Message batching
- `compression`, `encryption` - Data processing
- `rendering`, `input`, `audio` - Client capabilities

### Feature-Gated Code
```rust
#[cfg(feature = "channels")]
pub channels: Shared<HashMap<ChannelId, ChannelInfo>>,

#[cfg(feature = "rendering")]
pub render_targets: Shared<HashMap<u32, RenderTarget>>,
```

## Package Layers

### Core Layer
- **Purpose**: Define contracts and data structures
- **Contains**: Structs with data fields, VTable, type definitions
- **NO**: Implementation logic, business logic, I/O operations
- **Examples**: core/ecs, core/server, core/client, core/console

### Systems Layer
- **Purpose**: Implement all actual functionality
- **Contains**: VTable handlers, business logic, I/O operations
- **Dependencies**: Can ONLY use core/* packages
- **Examples**: systems/ecs, systems/networking, systems/webgl, systems/console

### Apps Layer
- **Purpose**: Complete applications that orchestrate
- **Dependencies**: Use core/* packages ONLY (with features)
- **Examples**: playground-apps-editor, playground-apps-game

### Plugins Layer
- **Purpose**: High-level features
- **Dependencies**: Use core/* packages ONLY (with features)
- **Run by**: systems/ecs scheduler
- **Examples**: All IDE plugins, game features

## Global Instances

Core packages maintain global instances using `once_cell::sync::Lazy`:

```rust
// core/server/src/api.rs
static SERVER_INSTANCE: Lazy<Handle<Server>> = Lazy::new(|| Server::new());

// core/client/src/api.rs
static CLIENT_INSTANCE: Lazy<Handle<Client>> = Lazy::new(|| Client::new());
```

Systems access these through API functions:
```rust
let server = playground_core_server::get_server_instance()?;
let client = playground_core_client::get_client_instance()?;
```

## Type Aliases

### Handle<T> vs Shared<T>
- **Handle<T>** = `Arc<T>` - For external references to objects with internal state
- **Shared<T>** = `Arc<RwLock<T>>` - For internal mutable state (private fields only)

### Usage Rules
```rust
// Objects with internal Shared fields use Handle
let server: Handle<Server> = handle(Server::new());
server.some_method().await;  // No .read().await needed!

// Simple data uses Shared
struct MyStruct {
    data: Shared<HashMap<String, Value>>,  // INTERNAL state
}
let guard = self.data.write().await;
```

## System Isolation

### Strict Rules
- Systems can ONLY use core/* packages
- Systems CANNOT import other systems
- Cross-system communication through VTable/ECS only
- Each system implements specific core contracts

### Registration Pattern
```rust
// systems/networking/src/registration.rs
pub async fn initialize() -> CoreResult<()> {
    // Get global instances from core
    if let Ok(server) = playground_core_server::get_server_instance() {
        register_server_handlers(server.clone()).await?;
    }
    if let Ok(client) = playground_core_client::get_client_instance() {
        register_client_handlers(client.clone()).await?;
    }
    Ok(())
}
```

## Architectural Invariants

1. **NO unsafe** - Use OnceCell, Lazy, not static mut
2. **NO dyn** - Use concrete types with VTable dispatch
3. **NO Any** - Use serialization for type erasure
4. **NO turbofish** - Use ComponentId not generics
5. **Core is stateless** - Only data fields, no logic
6. **Systems are replaceable** - Same contracts, different implementations
7. **Compile-time safety** - Missing features caught at compile time
8. **Runtime dispatch** - VTable provides polymorphism without dyn