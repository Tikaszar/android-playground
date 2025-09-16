# Core/ECS Architecture Design

## Overview

This document describes the VTable architecture for the playground engine as implemented in Sessions 53-54. The design provides a foundation ECS layer with **data structures only** and generic string-based VTable dispatch. ALL logic is implemented in systems/ecs.

**Key Principle**: core/ecs is like abstract base classes in OOP - it defines the structure (data fields) but ALL behavior (methods) is implemented in systems/ecs.

## Core Design Principles

1. **Foundation Layer**: core/ecs is the base that everything builds on
2. **Zero Dependencies**: core/ecs imports NO other core/* packages
3. **String-Based VTable**: Generic capability registration by name
4. **Multiple Imports**: Apps import core/ecs + specific core/* packages needed
5. **API Ownership**: Each core/* package provides its own API
6. **Explicit Registration**: Systems must register handlers before use
7. **DATA ONLY in core/ecs**: NO implementation logic, just data structures
8. **ALL LOGIC in systems/ecs**: Every operation is implemented in systems/ecs

## Architecture Layers

### Core Design Principle - Data vs Logic Separation

`core/ecs` (Data Only):
1. Provides World and VTable data structures
2. Contains ONLY fields - entities, components, vtable, etc.
3. Public API methods are thin wrappers that delegate through VTable
4. NO implementation logic - like abstract base classes
5. Example: World struct has `entities: Shared<HashMap<EntityId, Generation>>` field

`systems/ecs` (All Logic):
1. Implements ALL actual ECS operations
2. WorldImpl::spawn_entity() contains the actual spawning logic
3. StorageImpl contains all component storage operations
4. Registers handlers with VTable for all operations
5. Operates on core/ecs data structures

Each `core/*` package:
1. Defines contracts (concrete structs, not traits)
2. Defines command/response types
3. **Provides API functions that use VTable dispatch**
4. Imported separately by apps/plugins as needed

Each `systems/*` package:
1. Implements the actual functionality
2. Registers its command handler with the VTable
3. Never exposes APIs directly

## Complete Architecture: core/* → systems/* with String-Based VTable

### 1. **core/ecs - The Foundation**

```toml
# core/ecs/Cargo.toml
[package]
name = "playground-core-ecs"

[dependencies]
playground-core-types = { path = "../types" }
# NO other core/* dependencies - this is the foundation!

bytes = "1.5"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

```rust
// core/ecs/src/lib.rs
// Core ECS functionality (always available)
pub mod world;
pub mod vtable;
pub mod entity;
pub mod component;
pub mod messaging;
pub mod query;
pub mod storage;
pub mod system;
pub mod registry;
pub mod error;

pub use world::*;
pub use vtable::*;
pub use entity::*;
pub use component::*;
pub use messaging::*;
pub use query::*;
pub use storage::*;
pub use system::*;
pub use registry::*;
pub use error::*;

// NO re-exports from other packages!
// Apps import core/ecs + whatever core/* packages they need
```

### 2. **VTable Architecture (Actual Implementation)**

```rust
// core/ecs/src/vtable.rs - Generic string-based dispatch
use std::collections::HashMap;
use bytes::Bytes;
use tokio::sync::mpsc;
use playground_core_types::{Shared, shared};

/// Generic command that can be sent through the VTable
pub struct VTableCommand {
    pub capability: String,
    pub operation: String,
    pub payload: Bytes,
    pub response: mpsc::Sender<VTableResponse>,
}

/// Generic response from a VTable command
pub struct VTableResponse {
    pub success: bool,
    pub payload: Option<Bytes>,
    pub error: Option<String>,
}

/// Generic VTable that stores channels by capability name
pub struct VTable {
    /// Registered capability channels
    channels: Shared<HashMap<String, mpsc::Sender<VTableCommand>>>,
}

impl VTable {
    pub fn new() -> Self {
        Self {
            channels: shared(HashMap::new()),
        }
    }
    
    /// Register a capability channel
    pub async fn register(
        &self,
        capability: String,
        sender: mpsc::Sender<VTableCommand>
    ) -> CoreResult<()> {
        let mut channels = self.channels.write().await;
        channels.insert(capability, sender);
        Ok(())
    }
    
    /// Send a command to a capability
    pub async fn send_command(
        &self,
        capability: &str,
        operation: String,
        payload: Bytes
    ) -> CoreResult<VTableResponse> {
        // Dispatch to registered handler
    }
}
```

### 3. **Concrete World Type**

```rust
// core/ecs/src/world.rs - DATA STRUCTURE ONLY
use playground_core_types::{Handle, handle, Shared, shared};

pub struct World {
    // Core ECS data fields (public for systems/ecs to access)
    pub entities: Shared<HashMap<EntityId, Generation>>,
    pub components: Shared<HashMap<EntityId, HashMap<ComponentId, Component>>>,
    pub vtable: VTable,
    pub next_entity_id: AtomicU32,
    // NO command channels, NO implementation!
}

impl World {
    pub fn new() -> Handle<Self> {
        // Just data initialization, no logic
        handle(Self {
            entities: shared(HashMap::new()),
            components: shared(HashMap::new()),
            vtable: VTable::new(),
            next_entity_id: AtomicU32::new(1),
        })
    }
    
    // Public API - just delegates to VTable, NO LOGIC HERE
    pub async fn spawn_entity(&self) -> CoreResult<EntityId> {
        let response = self.vtable.send_command(
            "ecs.entity", "spawn", Bytes::new()
        ).await?;
        // Deserialize and return
    }
}

// systems/ecs/src/world_impl.rs - ALL THE LOGIC
pub struct WorldImpl;

impl WorldImpl {
    pub async fn spawn_entity(world: &Handle<World>) -> CoreResult<EntityId> {
        // ACTUAL implementation logic here
        let id = world.next_entity_id.fetch_add(1, Ordering::SeqCst);
        let entity_id = EntityId::new(id, Generation::new(0));
        
        let mut entities = world.entities.write().await;
        entities.insert(entity_id, Generation::new(0));
        // etc...
    }
}
```

### 4. **Example: How Core Packages Provide APIs**

```rust
// core/server/src/api.rs - API lives in core/server, NOT in core/ecs!
use bytes::Bytes;
use serde::{Serialize, Deserialize};
use playground_core_ecs::{get_world, CoreResult, CoreError};

#[derive(Serialize, Deserialize)]
pub enum NetworkingOp {
    StartServer { port: u16 },
    StopServer,
    SendPacket { channel: u16, data: Bytes },
    RegisterChannel { name: String },
}

#[derive(Serialize, Deserialize)]
pub enum NetworkingResponse {
    Started,
    Stopped,
    PacketSent,
    ChannelRegistered(u16),
}

/// Start the networking server (API function in core/server)
pub async fn start_server(port: u16) -> CoreResult<()> {
    let world = get_world().await?;
    
    // Serialize the operation
    let op = NetworkingOp::StartServer { port };
    let payload = bincode::serialize(&op)
        .map_err(|e| CoreError::Generic(e.to_string()))?;
    
    // Send through VTable using generic dispatch
    let response = world.vtable.send_command(
        "networking",  // capability name
        "execute".to_string(),  // operation
        Bytes::from(payload)
    ).await?;
    
    if !response.success {
        return Err(CoreError::Generic(
            response.error.unwrap_or_else(|| "Unknown error".to_string())
        ));
    }
    
    Ok(())
}

/// Send a packet (API function in core/server)
pub async fn send_packet(channel: u16, data: Bytes) -> CoreResult<()> {
    // Similar implementation using VTable dispatch
}
```

### 5. **System Implementation**

```rust
// systems/networking/src/lib.rs
use bytes::Bytes;
use tokio::sync::mpsc;
use playground_core_ecs::{get_world, VTableCommand, VTableResponse};
use playground_core_server::{NetworkingOp, NetworkingResponse};

pub async fn register() -> CoreResult<()> {
    let world = get_world().await?;
    
    // Create command channel for VTable
    let (tx, mut rx) = mpsc::channel::<VTableCommand>(100);
    
    // Register with VTable using string-based capability name
    world.vtable.register("networking".to_string(), tx).await?;
    
    // Spawn handler task
    tokio::spawn(async move {
        let mut server = NetworkingSystem::new();
        
        while let Some(cmd) = rx.recv().await {
            // Deserialize the operation
            let op: NetworkingOp = match bincode::deserialize(&cmd.payload) {
                Ok(op) => op,
                Err(e) => {
                    let _ = cmd.response.send(VTableResponse {
                        success: false,
                        payload: None,
                        error: Some(e.to_string()),
                    }).await;
                    continue;
                }
            };
            
            // Handle the operation
            let result = match op {
                NetworkingOp::StartServer { port } => {
                    server.start(port).await
                }
                NetworkingOp::SendPacket { channel, data } => {
                    server.send_packet(channel, data).await
                }
                // ... handle other ops
            };
            
            // Send response
            let response = VTableResponse {
                success: result.is_ok(),
                payload: result.ok().and_then(|r| bincode::serialize(&r).ok().map(Bytes::from)),
                error: result.err().map(|e| e.to_string()),
            };
            
            let _ = cmd.response.send(response).await;
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

### Plugin Usage - Multiple Imports!

```rust
// plugins/editor/src/lib.rs
// Import core/ecs for the foundation
use playground_core_ecs::{
    World, Entity, Component,
    initialize_world, get_world,
    CoreResult, CoreError,
};

// Import specific core packages for their APIs
use playground_core_server::api as networking;
use playground_core_ui::api as ui;
use playground_core_console::api as console;

pub struct EditorPlugin {
    name: String,
}

impl EditorPlugin {
    pub async fn initialize(&mut self) -> CoreResult<()> {
        // Use APIs from each core package
        
        // Networking API from core/server
        networking::start_server(8080).await?;
        let channel = networking::register_channel("editor".into()).await?;
        
        // UI API from core/ui
        ui::create_element("editor-panel", ui::ElementKind::Panel).await?;
        ui::create_element("file-tree", ui::ElementKind::List).await?;
        
        // Console API from core/console
        console::log(console::LogLevel::Info, "Editor", "Plugin initialized").await?;
        
        Ok(())
    }
}
```

### App Usage

```rust
// apps/playground-editor/src/main.rs
use playground_core_ecs::{initialize_world, get_world};
use playground_core_server::api as networking;
use playground_core_ui::api as ui;
use playground_core_console::api as console;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the world once
    let world = initialize_world().await?;
    
    // Register all system implementations with VTable
    playground_systems_networking::register().await?;
    playground_systems_ui::register().await?;
    playground_systems_console::register().await?;
    playground_systems_webgl::register().await?;
    
    // Now use the APIs from each core package
    networking::start_server(8080).await?;
    console::log(console::LogLevel::Info, "App", "Started").await?;
    
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

1. **Foundation Layer**: core/ecs is the base everything builds on
2. **Zero Coupling**: core/ecs knows nothing about other packages
3. **Clear Separation**: Each core/* package owns its API
4. **String-Based Dispatch**: Generic VTable works with any capability
5. **Type Safety**: Concrete types everywhere, no dyn
6. **Runtime Registration**: Systems register handlers at startup
7. **Extensibility**: New capabilities just register with VTable

## Architecture Summary

```
Apps/Plugins Import:
  ├── playground-core-ecs       (foundation: World, VTable, Entity, etc.)
  ├── playground-core-server     (networking API)
  ├── playground-core-ui         (UI API)
  ├── playground-core-console    (logging API)
  └── playground-core-client     (client API)

core/ecs contains:
  - World struct with data fields ONLY
  - ComponentStorage struct with data fields ONLY
  - VTable for dispatching to systems/ecs
  - Public API methods that just delegate through VTable
  - NO implementation logic whatsoever

systems/ecs contains:
  - WorldImpl with ALL entity/component operations
  - StorageImpl with ALL storage operations
  - VTable handlers that receive commands from core/ecs
  - ALL the actual ECS logic

Each core/* package:
  - Defines its own API functions
  - Uses VTable dispatch through core/ecs
  - Knows nothing about systems/*

Systems register handlers:
  - systems/ecs → registers "ecs.entity", "ecs.component", "ecs.query" capabilities
  - systems/networking → registers "networking" capability
  - systems/ui → registers "ui" capability
  - systems/console → registers "console" capability
  - etc.
```

## Architecture Rules Compliance

- ✅ **NO dyn**: Everything is concrete types
- ✅ **NO unsafe**: Pure safe Rust
- ✅ **NO traits for contracts**: Concrete structs only
- ✅ **String-based VTable**: Generic dispatch mechanism
- ✅ **Handle/Shared pattern**: Used consistently
- ✅ **Async everywhere**: All operations are async
- ✅ **Result everywhere**: All fallible operations return Result
- ✅ **Clean separation**: core/ecs is truly foundational
- ✅ **Data vs Logic**: core/ecs has ONLY data, systems/ecs has ALL logic
- ✅ **Abstract Base Class Pattern**: core/ecs defines structure, systems/ecs provides behavior