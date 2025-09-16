# Core/ECS Architecture Design

## Overview

This document describes the feature-gated VTable architecture for the playground engine. The design eliminates runtime errors through compile-time feature checking and provides clear error boundaries between layers.

## Core Design Principles

1. **Compile-Time Safety**: Features control what's available at compile time
2. **Clear Error Boundaries**: Errors occur at well-defined layer boundaries
3. **Single Import Point**: Apps/Plugins only import `playground-core-ecs`
4. **No Runtime Type Erasure**: Everything is concrete types
5. **Explicit Registration**: Systems must register before use

## Architecture Layers

### Core Design Principle

Each `core/*` package:
1. Defines contracts (concrete structs, not traits)
2. Defines command/response types
3. **Provides API functions that use VTable dispatch**
4. Exports features to `core/ecs`

Each `systems/*` package:
1. Implements the actual functionality
2. Registers its command handler with the VTable
3. Never exposes APIs directly

`core/ecs`:
1. Provides World and VTable infrastructure
2. **Re-exports all core/* APIs and features**
3. Is the single import point for Apps/Plugins

## Complete Architecture: core/* → systems/* with Feature-Gated APIs

### 1. **core/ecs - The Hub**

```toml
# core/ecs/Cargo.toml
[package]
name = "playground-core-ecs"

[dependencies]
playground-core-types = { path = "../types" }

# Re-export features from other core packages
playground-core-server = { path = "../server", optional = true }
playground-core-ui = { path = "../ui", optional = true }
playground-core-client = { path = "../client", optional = true }
playground-core-console = { path = "../console", optional = true }
playground-core-rendering = { path = "../rendering", optional = true }

[features]
default = ["world"]  # Core ECS always available

# Core ECS feature
world = []

# Re-export features from other core packages
networking = ["playground-core-server/networking"]
ui = ["playground-core-ui/ui"]
rendering = ["playground-core-rendering/rendering"]
console = ["playground-core-console/console"]
client = ["playground-core-client/client"]
physics = ["playground-core-physics/physics"]
audio = ["playground-core-audio/audio"]
storage = ["playground-core-storage/storage"]
```

```rust
// core/ecs/src/lib.rs
// Core ECS functionality (always available)
pub mod world;
pub mod vtable;
pub mod entity;
pub mod component;

pub use world::*;
pub use vtable::*;
pub use entity::*;
pub use component::*;

// Re-export APIs from other core packages (feature-gated)
#[cfg(feature = "networking")]
pub use playground_core_server::api as networking;

#[cfg(feature = "ui")]
pub use playground_core_ui::api as ui;

#[cfg(feature = "rendering")]
pub use playground_core_rendering::api as rendering;

#[cfg(feature = "console")]
pub use playground_core_console::api as console;

#[cfg(feature = "client")]
pub use playground_core_client::api as client;
```

### 2. **VTable Architecture**

```rust
// core/ecs/src/vtable.rs
use tokio::sync::mpsc;
use playground_core_types::{Shared, shared};

/// Single concrete VTable containing all capability channels
pub struct VTable {
    #[cfg(feature = "networking")]
    pub networking: Shared<Option<mpsc::Sender<NetworkingCommand>>>,
    
    #[cfg(feature = "ui")]
    pub ui: Shared<Option<mpsc::Sender<UiCommand>>>,
    
    #[cfg(feature = "rendering")]
    pub rendering: Shared<Option<mpsc::Sender<RenderCommand>>>,
    
    #[cfg(feature = "console")]
    pub console: Shared<Option<mpsc::Sender<ConsoleCommand>>>,
    
    #[cfg(feature = "storage")]
    pub storage: Shared<Option<mpsc::Sender<StorageCommand>>>,
    
    #[cfg(feature = "physics")]
    pub physics: Shared<Option<mpsc::Sender<PhysicsCommand>>>,
    
    #[cfg(feature = "audio")]
    pub audio: Shared<Option<mpsc::Sender<AudioCommand>>>,
    
    #[cfg(feature = "input")]
    pub input: Shared<Option<mpsc::Sender<InputCommand>>>,
    
    #[cfg(feature = "client")]
    pub client: Shared<Option<mpsc::Sender<ClientCommand>>>,
}

impl VTable {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "networking")]
            networking: shared(None),
            #[cfg(feature = "ui")]
            ui: shared(None),
            #[cfg(feature = "rendering")]
            rendering: shared(None),
            #[cfg(feature = "console")]
            console: shared(None),
            #[cfg(feature = "storage")]
            storage: shared(None),
            #[cfg(feature = "physics")]
            physics: shared(None),
            #[cfg(feature = "audio")]
            audio: shared(None),
            #[cfg(feature = "input")]
            input: shared(None),
            #[cfg(feature = "client")]
            client: shared(None),
        }
    }
}
```

### 3. **Concrete World Type**

```rust
// core/ecs/src/world.rs
use playground_core_types::{Handle, handle, Shared, shared};

pub struct World {
    // Core ECS data
    entities: Shared<HashMap<EntityId, Generation>>,
    components: Shared<HashMap<EntityId, HashMap<ComponentId, Component>>>,
    
    // THE vtable - single instance, not a HashMap
    pub vtable: VTable,
    
    // Channels for core operations
    entity_commands: mpsc::Sender<EntityCommand>,
    component_commands: mpsc::Sender<ComponentCommand>,
}

impl World {
    pub fn new() -> Handle<Self> {
        handle(Self {
            entities: shared(HashMap::new()),
            components: shared(HashMap::new()),
            vtable: VTable::new(),
            // ... initialize channels
        })
    }
    
    // Core ECS operations
    pub async fn spawn_entity(&self) -> CoreResult<EntityId> {
        // Direct implementation, no vtable needed
    }
    
    pub async fn add_component(&self, entity: EntityId, component: Component) -> CoreResult<()> {
        // Direct implementation
    }
}
```

### 4. **Example: Networking API**

```rust
// core/server/src/commands.rs
use bytes::Bytes;

#[derive(Debug, Clone)]
pub enum NetworkingOp {
    StartServer { port: u16 },
    StopServer,
    SendPacket { channel: u16, data: Bytes },
    Broadcast { data: Bytes },
    RegisterChannel { name: String },
}

pub struct NetworkingCommand {
    pub op: NetworkingOp,
    pub response: oneshot::Sender<CoreResult<NetworkingResponse>>,
}

#[derive(Debug, Clone)]
pub enum NetworkingResponse {
    Started,
    Stopped,
    PacketSent,
    ChannelRegistered(u16),
    Error(String),
}
```

```rust
// core/server/src/api.rs - THE API LIVES HERE!
use playground_core_ecs::{get_world};
use crate::commands::*;

pub async fn start_server(port: u16) -> CoreResult<()> {
    let world = get_world().await?;
    
    let sender = {
        let guard = world.vtable.networking.read().await;
        guard.as_ref().ok_or(CoreError::NotRegistered("networking"))?.clone()
    };
    
    let (tx, rx) = oneshot::channel();
    sender.send(NetworkingCommand {
        op: NetworkingOp::StartServer { port },
        response: tx,
    }).await.map_err(|_| CoreError::SendError)?;
    
    match rx.await.map_err(|_| CoreError::ReceiveError)?? {
        NetworkingResponse::Started => Ok(()),
        NetworkingResponse::Error(e) => Err(CoreError::Generic(e)),
        _ => Err(CoreError::UnexpectedResponse),
    }
}
```

### 5. **System Implementation**

```rust
// systems/networking/src/lib.rs
use playground_core_server::{NetworkingCommand, NetworkingOp, NetworkingResponse};
use playground_core_ecs::{get_world};

pub async fn register() -> CoreResult<()> {
    let world = get_world().await?;
    
    // Create command channel
    let (tx, mut rx) = mpsc::channel::<NetworkingCommand>(100);
    
    // Register with vtable
    {
        let mut guard = world.vtable.networking.write().await;
        *guard = Some(tx);
    }
    
    // Spawn handler task
    tokio::spawn(async move {
        let mut server = NetworkingSystem::new();
        
        while let Some(cmd) = rx.recv().await {
            let result = match cmd.op {
                NetworkingOp::StartServer { port } => {
                    server.start(port).await
                }
                NetworkingOp::SendPacket { channel, data } => {
                    server.send_packet(channel, data).await
                }
                // ... handle other ops
            };
            
            let _ = cmd.response.send(result);
        }
    });
    
    Ok(())
}
```

## Error Handling Strategy

### Compile-Time Errors

1. **Missing Features**: If a plugin tries to use `networking::start_server` without the `networking` feature, it won't compile
2. **Type Mismatches**: All types are concrete, so mismatches are caught at compile time
3. **Missing Dependencies**: Cargo enforces dependency requirements

### Runtime Errors (Explicit and Traceable)

1. **Not Registered**: `CoreError::NotRegistered("networking")` - System not registered
2. **Channel Errors**: `CoreError::SendError` / `CoreError::ReceiveError` - Communication failures
3. **Operation Errors**: Wrapped in response enums with clear error messages

```rust
pub enum CoreError {
    NotInitialized,           // World not initialized
    AlreadyInitialized,       // World already exists
    NotRegistered(String),    // System not registered
    SendError,                // Channel send failed
    ReceiveError,            // Channel receive failed
    UnexpectedResponse,       // Wrong response type
    Generic(String),          // System-specific error
}
```

## Usage Example

### Plugin Usage - Single Import!

```rust
// plugins/editor/src/lib.rs
use playground_core_ecs::{
    // Core ECS (always available)
    World, Entity, Component,
    initialize_world, get_world,
    
    // Networking API (from core/server)
    #[cfg(feature = "networking")]
    networking::{start_server, send_packet, register_channel},
    
    // UI API (from core/ui)
    #[cfg(feature = "ui")]
    ui::{create_element, update_element, UiElementKind},
    
    // Console API (from core/console)
    #[cfg(feature = "console")]
    console::{log, LogLevel},
};

pub struct EditorPlugin {
    name: String,
}

impl EditorPlugin {
    pub async fn initialize(&mut self) -> CoreResult<()> {
        // Everything through playground-core-ecs!
        
        #[cfg(feature = "networking")]
        {
            start_server(8080).await?;
            let channel = register_channel("editor".into()).await?;
        }
        
        #[cfg(feature = "ui")]
        {
            create_element("editor-panel", UiElementKind::Panel).await?;
            create_element("file-tree", UiElementKind::List).await?;
        }
        
        #[cfg(feature = "console")]
        {
            log(LogLevel::Info, "Editor", "Plugin initialized").await?;
        }
        
        Ok(())
    }
}
```

### App Usage

```rust
// apps/playground-editor/src/main.rs
use playground_core_ecs::{
    // Initialize world
    initialize_world,
    
    // All features available
    networking, ui, console,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the world once
    let world = initialize_world().await?;
    
    // Register all system implementations
    playground_systems_networking::register().await?;
    playground_systems_ui::register().await?;
    playground_systems_console::register().await?;
    playground_systems_webgl::register().await?;
    
    // Now use the APIs
    networking::start_server(8080).await?;
    console::log(LogLevel::Info, "App", "Started").await?;
    
    // Load plugins
    let mut editor = EditorPlugin::new();
    editor.initialize().await?;
    
    // Run main loop
    loop {
        world.update(0.016).await?;
    }
}
```

## Benefits

1. **Single Import Point**: Apps/Plugins only need `playground-core-ecs`
2. **Compile-Time Feature Control**: Missing features caught at compile time
3. **No systems/logic Layer**: APIs come directly from core/* packages
4. **Clear Error Boundaries**: Registration errors are explicit
5. **Type Safety**: No dynamic dispatch, everything is concrete
6. **Runtime Performance**: No vtable indirection, just channel communication
7. **Extensibility**: New capabilities just add new VTable fields

## Migration Path

1. Refactor `core/ecs` to contain only World and VTable infrastructure
2. Move API functions from `systems/logic` to respective `core/*` packages
3. Update `core/ecs` to re-export all APIs
4. Convert `systems/*` to registration-only pattern
5. Update all plugins/apps to use single import
6. Remove `systems/logic` entirely

## Architecture Rules Compliance

- ✅ **NO dyn**: Everything is concrete types
- ✅ **NO unsafe**: Pure safe Rust
- ✅ **NO traits for contracts**: Concrete structs only
- ✅ **NO enums for type erasure**: VTable is a single struct
- ✅ **Handle/Shared pattern**: Used consistently
- ✅ **Async everywhere**: All operations are async
- ✅ **Result everywhere**: All fallible operations return Result