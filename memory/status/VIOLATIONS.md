# Violations - Current Architecture Violations (Session 80)

## Session 79 Complete ✅

### modules/* Infrastructure Trait-Based MVVM (Session 79)
**Status**: ✅ COMPLETE
- Replaced function pointers with trait-based MVVM
- ModelTrait, ViewTrait, ViewModelTrait with 64-bit IDs
- Concurrent, flattened binding map (refined from original triple-nested sharding).
- Object recycling for memory efficiency
- Lock-free Views/ViewModels access
- THE single unsafe block complete in loader
- All modules/* packages compile successfully

## Critical Issues Remaining 🔴

### 1. Core/Systems Modules NOT Updated to Use New Traits
**Location**: core/ecs/src/module_exports.rs, systems/ecs/src/module_exports.rs
**Status**: ❌ BLOCKING
**Issue**: Still using OLD Session 78 pattern with deleted types
**Details**:
- Reference `ViewAPI` struct (deleted from modules/types in Session 79)
- Use `Pin<Box<dyn Future<...>>>` signatures (violates NO dyn rule)
- Use serialization `_args: &[u8]` pattern (deprecated)
- Won't compile against new modules/types

**Fix Required**:
```rust
// DELETE: core/ecs/src/module_exports.rs (entire file)
// DELETE: systems/ecs/src/module_exports.rs (entire file)

// CREATE: core/ecs/src/view/*.rs - Trait definitions
#[async_trait]
pub trait EntityView: ViewTrait {
    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity>;
    // ... other methods
}

// CREATE: core/ecs exports
#[no_mangle]
pub static PLAYGROUND_VIEW: &dyn ViewTrait = &EcsView;

#[no_mangle]
pub static PLAYGROUND_MODELS: &[ModelTypeInfo] = &[
    ModelTypeInfo { model_type: 0x0001, type_name: "Entity" },
    ModelTypeInfo { model_type: 0x0002, type_name: "Component" },
];

// CREATE: systems/ecs/src/viewmodel/*.rs - Trait implementations
pub struct EntityViewModel;

#[async_trait]
impl EntityView for EntityViewModel {
    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity> {
        // Direct implementation
    }
}

// CREATE: systems/ecs exports
#[no_mangle]
pub static PLAYGROUND_VIEWMODEL: &dyn ViewModelTrait = &EntityViewModel;
```

### 2. Core/ECS View Layer Needs Trait Definitions
**Location**: core/ecs/src/view/
**Status**: ⚠️ NEEDS WORK
**Current**: Has stub async functions that return errors
**Required**: Convert to trait definitions per Session 79 architecture

**Files to Create**:
- `core/ecs/src/view/entity.rs` - EntityView trait
- `core/ecs/src/view/component.rs` - ComponentView trait
- `core/ecs/src/view/event.rs` - EventView trait
- `core/ecs/src/view/query.rs` - QueryView trait
- `core/ecs/src/view/storage.rs` - StorageView trait
- `core/ecs/src/view/system.rs` - SystemView trait
- `core/ecs/src/view/world.rs` - WorldView trait

### 3. Systems/ECS ViewModel Layer Needs Trait Implementations
**Location**: systems/ecs/src/viewmodel/
**Status**: ⚠️ PARTIAL
**Current**: Has old serialization-based implementations
**Required**: Convert to trait implementations per Session 79 architecture

**Files to Update**:
- All `systems/ecs/src/viewmodel/*/*.rs` files
- Remove serialization signatures
- Implement View traits directly
- Use async-trait for async methods

## Pending - Other Modules Need MVVM Conversion 🟡

### 4. Other Core Modules
**Location**: core/console, core/server, core/client, core/rendering, core/ui
**Status**: PENDING
**Fix Required**: Same MVVM conversion as core/ecs (Model+View split)

### 5. Other System Modules
**Location**: systems/console, systems/webgl, systems/ui
**Status**: PENDING
**Fix Required**: ViewModel implementations for their Core modules

## Build System Changes 🟡

### 6. Add `build.rs` Hooks and Metadata
**Location**: All `core/*` and `systems/*` crates.
**Fix Required**: Each module must have a `build.rs` file that calls the central `modules/build-utils` logic. `System` modules must also have the correct `implements` metadata in their `Cargo.toml`.

```toml
# In systems/ecs/Cargo.toml
[build-dependencies]
playground-build-utils = { path = "../../modules/build-utils" }

[package.metadata.playground.implements]
core_path = "../core/ecs"
```

### 7. Create `modules/build-utils` Crate
**Location**: `modules/`
**Fix Required**: The central `modules/build-utils` library crate, which contains all the versioning and validation logic, needs to be created.

### 8. Set Module Compilation
**Location**: All core/*, systems/*, plugins/*, apps/*
**Fix Required**:
```toml
[lib]
crate-type = ["cdylib"]
```

## Implementation Order

1. ✅ **Session 68**: Create modules/* infrastructure
2. ✅ **Session 70**: Fix modules compilation
3. ✅ **Session 69**: Convert core/ecs to Model layer
4. ✅ **Session 72-73**: Create core/ecs View stub layer
5. ✅ **Session 74-75**: Create systems/ecs ViewModel stubs
6. ✅ **Session 79**: Replace modules/* with trait-based MVVM
7. **Session 80 - NEXT**: Update core/ecs and systems/ecs to use new traits
8. **Future**: Convert remaining modules
9. **Future**: Test hot-loading with state preservation
10. **Future**: Add build.rs validation

## Success Criteria

- ✅ modules/* uses trait-based MVVM (Session 79)
- ❌ Core/Systems modules implement traits (BLOCKING)
- 🟡 All modules follow MVVM pattern (structure exists, needs trait conversion)
- 🟡 Compile-time validation working (design complete)
- ❌ Direct trait method calls (needs trait implementations)
- 🟡 Hot-reload functional (infrastructure ready, needs module updates)

## Next Immediate Actions (Session 80)

1. Delete `core/ecs/src/module_exports.rs` (obsolete)
2. Delete `systems/ecs/src/module_exports.rs` (obsolete)
3. Convert `core/ecs/src/view/*.rs` to trait definitions
4. Convert `systems/ecs/src/viewmodel/*.rs` to trait implementations
5. Add proper exports with `#[no_mangle]` symbols
6. Test compilation
7. Test module loading
