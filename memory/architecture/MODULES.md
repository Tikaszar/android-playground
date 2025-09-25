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
- **Dependencies**: `core/*` only (use module calls for systems access)

### 4. App Modules
- **Purpose**: Complete applications
- **Location**: `apps/*`
- **Examples**: `apps/editor`, `apps/game`
- **Contains**: Application logic, UI
- **Dependencies**: Everything they need

## Module Interface (Pure Rust, NO C ABI)

```rust
// Pure Rust interface - no extern "C" or repr(C)
#[no_mangle]
pub static PLAYGROUND_MODULE: Module = Module {
    metadata: &METADATA,
    vtable: &VTABLE,
};

pub struct Module {
    pub metadata: &'static ModuleMetadata,
    pub vtable: &'static ModuleVTable,
}

pub struct ModuleVTable {
    // Pure Rust function pointers - no extern "C"!
    pub create: fn() -> *mut u8,
    pub destroy: fn(*mut u8),
    pub initialize: fn(*mut u8, config: &[u8]) -> Result<(), String>,
    pub shutdown: fn(*mut u8) -> Result<(), String>,
    pub call: fn(*mut u8, method: &str, args: &[u8]) -> Result<Vec<u8>, String>,
    pub save_state: fn(*const u8) -> Vec<u8>,
    pub restore_state: fn(*mut u8, state: &[u8]) -> Result<(), String>,
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

### 1. Fast Path - Direct Function Calls via VTable
```rust
// For hot operations (1-5ns overhead)
let result = (module.vtable.call)(
    module.state,
    "spawn_entity",
    &[]
)?;
```

### 2. Module Loader Interface
```rust
// Safe wrapper for module calls
MODULE_LOADER.call_module(module_id, "method", args).await
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

### Single Unsafe Exception
```rust
// THE ONLY UNSAFE IN THE ENTIRE SYSTEM - DOCUMENTED EXCEPTION
let library = unsafe {
    Library::new(path)
        .map_err(|e| CoreError::ModuleLoadFailed(e.to_string()))?
};
```

### Module Loader (systems/module-loader)
- Load modules with single unsafe Library::new()
- Validate schemas before loading
- Track dependencies
- Handle hot-reload
- Manage module lifecycle

## State Preservation

Modules preserve state across reloads:

```rust
// Pure Rust - no extern "C"
fn save_state(state: *const u8) -> Vec<u8> {
    let module = unsafe { &*(state as *const EcsModule) };
    bincode::serialize(&module.world).unwrap()
}

fn restore_state(state: *mut u8, saved: &[u8]) -> Result<(), String> {
    let module = unsafe { &mut *(state as *mut EcsModule) };
    module.world = bincode::deserialize(saved)
        .map_err(|e| e.to_string())?;
    Ok(())
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
✅ **Type safe** - Schema validation before loading
✅ **Minimal unsafe** - Single unsafe Library::new() only
✅ **Pure Rust** - No C ABI or extern "C"
✅ **Fine-grained control** - Feature flags for minimal loading
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
1. Create `api/` crate with pure Rust interfaces
2. Build `systems/module-loader` with single unsafe
3. Remove VTable from core packages
4. Add module interfaces to all packages
5. Implement hot-reload with libloading
6. Test hot-reload functionality