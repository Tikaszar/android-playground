# Module Architecture - Hot-Loadable Module System

## Overview

The entire Playground engine uses a hot-loadable module architecture where EVERYTHING (Core, Systems, Plugins, and Apps) can be reloaded at runtime without restart. This enables true live development and self-modifying software.

## Module Types

### 1. Core Modules
- **Purpose**: Define data structures and contracts
- **Location**: `core/*`
- **Examples**: `core/ecs`, `core/rendering`, `core/types`
- **Contains**: Data structures, type definitions, API stubs
- **NO**: Implementation logic

### 2. System Modules
- **Purpose**: Implement core contracts
- **Location**: `systems/*`
- **Examples**: `systems/ecs`, `systems/webgl`, `systems/networking`
- **Contains**: Actual implementation logic
- **Dependencies**: Only `core/*` modules

### 3. Plugin Modules
- **Purpose**: Add features and gameplay
- **Location**: `plugins/*`
- **Examples**: `plugins/movement`, `plugins/combat`
- **Contains**: Game features, tools
- **Dependencies**: `core/*` and `systems/*` via message bus

### 4. App Modules
- **Purpose**: Complete applications
- **Location**: `apps/*`
- **Examples**: `apps/editor`, `apps/game`
- **Contains**: Application logic, UI
- **Dependencies**: Everything they need

## Module Interface (ABI-Stable)

```rust
// All modules implement this base interface
#[repr(C)]
#[derive(StableAbi)]
pub struct BaseModule {
    pub get_metadata: extern "C" fn() -> RString,
    pub start_with_context: extern "C" fn(RRef<'_, ModuleContext>) -> RResult<(), RString>,
    pub stop: extern "C" fn() -> RResult<(), RString>,
    pub get_dependencies: extern "C" fn() -> RVec<ModuleDependency>,
    pub save_state: extern "C" fn() -> RResult<RVec<u8>, RString>,
    pub restore_state: extern "C" fn(RRef<'_, RVec<u8>>) -> RResult<(), RString>,
}

// Module context passed during initialization
#[repr(C)]
#[derive(StableAbi)]
pub struct ModuleContext {
    pub features: RVec<RString>,      // Active features
    pub version: Version,              // Module version
    pub module_id: u32,                // Unique ID
}
```

## Dependency System

### Feature-Based Dependencies

```toml
# Module declares its dependencies with features
[dependencies]
"core/rendering" = {
    version = "^1.0",  # Semantic versioning
    features = ["shaders", "textures", "buffers"]
}
"core/ecs" = {
    version = ">=1.0, <2.0",  # Version range
    features = ["components", "queries", "resources"]
}
```

### Dependency Resolution
1. Module declares what it needs (not who needs it)
2. Hot-loader builds dependency graph
3. Topological sort determines load order
4. Circular dependencies detected and prevented
5. Feature compatibility checked

## Communication Methods

### 1. Fast Path - Direct Function Pointers
```rust
// For hot operations (1-5ns overhead)
pub struct WorldOps {
    spawn_entity: extern "C" fn() -> EntityId,
    add_component: extern "C" fn(EntityId, ComponentId, *const u8) -> bool,
}
```

### 2. Slow Path - Message Bus
```rust
// For cold operations (1000ns overhead)
MESSAGE_BUS.call("systems.ecs", "complex_operation", params).await
```

## Hot-Reload Process

1. **File Change Detection** - Watcher detects .rs file change
2. **Build Module** - Incremental cargo build (~100-500ms)
3. **Save State** - Module serializes current state
4. **Find Dependents** - Compute who depends on this module
5. **Unload Chain** - Unload dependents then module
6. **Load New Version** - Load new .module file
7. **Reload Dependents** - Reload in dependency order
8. **Restore State** - Deserialize and restore state

## Module Loader

### Minimal Launcher
```rust
// launcher/src/main.rs - Knows nothing about modules
fn main() {
    let config = load_config("launch.toml");
    for module_path in config.modules {
        let module = BaseModule_Ref::load_from_file(module_path)?;
        module.start()?;
    }
    // Keep running and watch for reloads
}
```

### Module Management (core/modules)
- Create new modules from templates
- Build modules via cargo
- Load/unload modules at runtime
- Track dependencies
- Handle hot-reload

## State Preservation

Modules preserve state across reloads:

```rust
extern "C" fn save_state() -> RResult<RVec<u8>, RString> {
    let snapshot = ModuleSnapshot {
        entities: self.entities.clone(),
        components: self.components.clone(),
        // Don't save transient data (caches, etc)
    };
    ROk(bincode::serialize(&snapshot)?.into())
}

extern "C" fn restore_state(data: RRef<'_, RVec<u8>>) -> RResult<(), RString> {
    let snapshot: ModuleSnapshot = bincode::deserialize(&data)?;
    self.entities = snapshot.entities;
    self.rebuild_caches();
    ROk(())
}
```

## MCP Integration

Modules expose tools for debugging:

```rust
MCP_SERVER.register_tool("module.create", create_module_tool);
MCP_SERVER.register_tool("module.reload", reload_module_tool);
MCP_SERVER.register_tool("debug.inspect", inspect_module_tool);
```

This allows Claude to:
- Create new modules
- Modify and reload existing modules
- Inspect module state
- Debug the running system

## Benefits

✅ **Zero downtime development** - Change anything without restart
✅ **Fast iteration** - See changes in ~500ms
✅ **Type safe** - abi_stable ensures compatibility
✅ **No unsafe code** - abi_stable handles FFI
✅ **Fine-grained control** - Feature flags for minimal loading
✅ **Backwards compatible** - Semantic versioning support
✅ **Self-modifying** - IDE can reload itself

## Migration from VTable

### Current (VTable-based)
- VTable in core packages
- Serialization overhead (~1000ns per call)
- Complex message routing
- Systems register handlers

### New (Module-based)
- Direct function calls (~1-5ns per call)
- Modules are self-contained
- Simple dependency management
- Everything hot-loadable

### Migration Steps
1. Add `abi_stable` dependency
2. Create `api/` crate with interfaces
3. Remove VTable from core packages
4. Add module interfaces to all packages
5. Create module loader in launcher
6. Test hot-reload functionality