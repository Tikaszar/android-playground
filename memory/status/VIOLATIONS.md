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
- **DELETE** `core/ecs/src/module_exports.rs` and `systems/ecs/src/module_exports.rs` entirely.
- **CONVERT** `core/ecs/src/view/` files to be proper trait definitions.
- **REWRITE** `systems/ecs/src/viewmodel/` files to implement the new traits with direct `async fn` signatures, removing all serialization.

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
**Required**: Rewrite existing implementations to be compliant with the new trait-based architecture.

**Actions**:
- For each file in `systems/ecs/src/viewmodel/`:
  1. Change the function signature from the old `Pin<Box<dyn Future>>` and `&[u8]` pattern to the new direct-call `async fn` signature (e.g., `async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity>`).
  2. Remove all `bincode::deserialize` and `bincode::serialize` calls.
  3. Adapt the core function logic to work with the direct parameters and return types.
  4. Wrap the rewritten functions in an `impl TraitName for EcsViewModel` block.

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
7. ✅ **Session 80-83**: Update core/ecs and systems/ecs to use new traits (in progress)
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

## Next Immediate Actions (Session 83)

1. Complete systems/ecs World module conversion.
2. Create final lib.rs integration for systems/ecs.
3. Test compilation for systems/ecs.
4. Delete `core/ecs/src/module_exports.rs` (obsolete)
5. Delete `systems/ecs/src/module_exports.rs` (obsolete)
6. Convert `core/ecs/src/view/*.rs` to trait definitions
7. Convert `systems/ecs/src/viewmodel/*.rs` to trait implementations
8. Add proper exports with `#[no_mangle]` symbols
9. Test module loading
