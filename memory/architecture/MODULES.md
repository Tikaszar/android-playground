# Module Architecture - MVVM-Based Hot-Loading (Sessions 67-71)

## Overview

The entire engine uses MVVM pattern with hot-loadable modules where Core provides Model+View, Systems provide ViewModel, and everything can reload at runtime.

## Implementation Status (Sessions 68-71)

### modules/* Infrastructure ✅ COMPLETE
- **modules/types** - Base types with proper Rust module structure
  - Traits with generics allowed (NO dyn, NO Box)
  - One struct per file
  - Proper subdirectories with mod.rs
  - ViewAPI and ViewModelImpl are Copy+Clone (Session 70)
- **modules/loader** - Contains THE single unsafe block ✅ COMPILES
- **modules/binding** - Trait-based binding with generics ✅ COMPILES
- **modules/resolver** - Cargo.toml parsing
- **modules/registry** - Runtime orchestration

### core/ecs/model ✅ COMPLETE (Session 71)
- **entity/** - EntityId, Entity, EntityRef, Generation
- **component/** - ComponentId, Component, ComponentRef (NO ComponentData trait)
- **event/** - EventId, Event, EventRef, Priority, Subscription, SubscriptionId
- **query/** - QueryId, Query, QueryRef, QueryFilter
- **storage/** - StorageId, Storage, StorageRef
- **system/** - SystemId, System, SystemRef
- **world/** - World, WorldRef (contains all storage)

## MVVM Module Types

### 1. Core Modules (Model + View)
- **Purpose**: Define data structures AND API contracts
- **Location**: `core/*`
- **Structure**:
  - `model/` - Data structures only
  - `view/` - API function contracts
- **Examples**: `core/ecs`, `core/rendering`, `core/console`
- **NO**: Implementation logic

### 2. System Modules (ViewModel)
- **Purpose**: Implement Core API contracts
- **Location**: `systems/*`
- **Structure**:
  - `viewmodel/` - Implementation logic
- **Examples**: `systems/ecs`, `systems/webgl`, `systems/console`
- **NO**: Data storage (except internal state)

### 3. Plugin Modules
- **Purpose**: High-level features
- **Uses**: Core APIs only (never Systems directly)
- **Location**: `plugins/*`
- **Examples**: `plugins/editor-core`, `plugins/file-browser`

### 4. App Modules
- **Purpose**: Complete applications
- **Uses**: Plugin APIs primarily, Core APIs when needed
- **Declares**: Which Systems to load via Cargo.toml
- **Location**: `apps/*`
- **Examples**: `apps/editor`, `apps/game`

## Module Declaration in Cargo.toml

### App Declares Everything
```toml
# apps/editor/Cargo.toml
[[package.metadata.modules.core]]
name = "playground-core-rendering"
features = ["shaders", "textures", "multi-window"]
systems = [
    "playground-systems-vulkan",   # Primary choice
    "playground-systems-webgl"     # Fallback
]
```

### System Declares What It Provides
```toml
# systems/webgl/Cargo.toml
[package.metadata.provides]
core = "playground-core-rendering"
features = ["shaders", "textures", "2d", "basic-3d"]
```

## Module Infrastructure (modules/*)

```
modules/                  # NOT loadable - compiled into main binary
├── types/               # MVVM base types
├── loader/              # THE single unsafe (Library::new)
├── binding/             # Connect View to ViewModel
├── registry/            # Runtime module tracking
└── resolver/            # Read Cargo.toml, resolve dependencies
```

## Module Loading Process

1. **Read App Cargo.toml** - Find declared Core modules and Systems
2. **Validate Features** - build.rs checks Systems provide required features
3. **Load Core Modules** - Model + View (data + API)
4. **Load System Modules** - ViewModel (implementation)
5. **Bind View to ViewModel** - Connect API to implementation
6. **Load Plugins** - Use Core APIs
7. **Load App** - Ready to run

## Compile-Time Safety

### Feature Validation
```rust
// apps/editor/build.rs
fn main() {
    // Check System provides all features App needs
    // Check System provides all features Plugins need
    // Compile error if mismatch
}
```

### Benefits
- **Zero runtime checks** - All validation at compile time
- **Direct function calls** - ~1-5ns overhead
- **Type safety** - Rust compiler enforces signatures
- **Clear errors** - Know exactly what's missing

## Hot-Reload Process

1. **Detect Change** - File watcher sees .rs change
2. **Save State** - Module serializes current state
3. **Compile Module** - Incremental build (~500ms)
4. **Load New Version** - Using single unsafe
5. **Restore State** - Deserialize saved state
6. **Update Bindings** - View now calls new ViewModel

## No VTable, No Runtime Indirection

- View functions directly call ViewModel implementations
- Binding happens once at load time
- After binding, just direct function calls
- Compile-time checking ensures compatibility