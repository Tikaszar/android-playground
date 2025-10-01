# Architecture - MVVM-Based Module System

## Core Architectural Pattern (Sessions 67-71)

### MVVM Architecture
```
Apps → Plugins → Core (Model+View) → [Module Binding] → Systems (ViewModel)
```

**Key Components:**
- **Model** = Data structures (core/*/model/)
- **View** = API contracts (core/*/view/)
- **ViewModel** = Implementation (systems/*/viewmodel/)
- **Binding** = Trait-based with generics (NO dyn, NO Box)

## Implementation (Sessions 68-71)
- **modules/types** - MVVM base types (traits with generics, NO dyn)
- **modules/loader** - THE single unsafe block for Library::new() ✅ COMPILES
- **modules/binding** - Trait-based binding with concrete types ✅ COMPILES
- **modules/resolver** - Cargo.toml module declarations
- **modules/registry** - Runtime module orchestration
- **core/ecs/model** - Complete ECS Model layer with all data structures ✅ COMPILES

## MVVM Separation Pattern

### Strict Module Types
- **Core modules** = Model (data) + View (API contracts)
- **System modules** = ViewModel (implementation only)
- **Plugin modules** = High-level features (use Core APIs)
- **App modules** = Applications (use Plugin APIs, declare Systems)

### No Runtime Indirection
- Direct function calls via compile-time binding
- ~1-5ns overhead (no VTable, no serialization)
- Compile-time errors for missing implementations

### Module Structure Example
```
core/ecs/
├── model/                # Pure data structures
│   ├── entity/          # EntityId, Entity, EntityRef, Generation
│   ├── component/       # ComponentId, Component, ComponentRef
│   ├── event/           # EventId, Event, EventRef, Priority, Subscription
│   ├── query/           # QueryId, Query, QueryRef, QueryFilter
│   ├── storage/         # StorageId, Storage, StorageRef
│   ├── system/          # SystemId, System, SystemRef
│   └── world/           # World, WorldRef
└── view/
    ├── spawn_entity.rs   # API contract (trait)
    └── query.rs          # API contract (trait)

systems/ecs/
└── viewmodel/
    ├── spawn_entity.rs   # Implementation (impl trait)
    └── query.rs          # Implementation (impl trait)
```

## Module Declaration System

### Apps Declare Everything
```toml
# apps/editor/Cargo.toml
[[package.metadata.modules.core]]
name = "playground-core-rendering"
features = ["shaders", "textures"]
systems = [
    "playground-systems-vulkan",
    "playground-systems-webgl"  # Fallback
]
```

### Compile-Time Validation
- build.rs validates System provides required features
- Missing features = compile error
- Wrong System = compile error
- Plugin requirements checked against App declarations

## Module Loading from Cargo.toml

### App-Driven Loading
- Apps/Plugins declare Core modules in Cargo.toml
- Apps declare which Systems implement Core modules
- Features specified at Core level, apply to any System
- Only declared modules load (unused Core/Systems don't load)

### System Feature Validation
```toml
# systems/webgl/Cargo.toml
[package.metadata.provides]
core = "playground-core-rendering"
features = ["shaders", "textures", "2d", "basic-3d"]
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

1. **NO unsafe** - EXCEPTION: Single Library::new() in module loader only
2. **Traits allowed with generics** - Use `<T: Trait>` NOT `Box<dyn Trait>`
3. **NO Any** - Use serialization for type erasure
4. **NO turbofish** - Use ComponentId not generics for components
5. **Model is pure data** - Only data fields, no logic
6. **NO Option<Handle>** - Use Handle/Shared or Weak directly
7. **Consistent pattern** - Every module has Id, Data, Ref types
8. **World contains all** - Central storage for all ECS data