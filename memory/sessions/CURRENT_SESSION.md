# Current Session - Session 83: Core/ECS Version Integration

## Session Goal
Integrate generated version constants into core/ecs to enable API version checking for hot-reload safety.

## Previous Session Summary (Session 82)
Session 82 fixed the build system to correctly generate version constants:
- Fixed API_VERSION to hash BOTH view AND model directories
- Fixed System modules to hash core's model for STATE_FORMAT_VERSION
- Fixed TOML parsing in get_core_path()
- Verified version generation works correctly

## Work Completed in Session 82 ✅

### 1. Fixed modules/build-utils version generation
- ✅ Updated `generate_api_version()` to hash both `src/view/` AND `src/model/` directories
- ✅ Core modules: API_VERSION = hash(view + model)
- ✅ System modules: API_VERSION = hash(core's view + core's model), STATE_FORMAT_VERSION = hash(core's model)
- ✅ Fixed TOML parsing to use `toml::from_str()` instead of `.parse()`
- ✅ Fixed rerun triggers to watch both view and model directories

### 2. Integrated core/ecs with build system
- ✅ Created `core/ecs/src/version.rs` to include generated versions
- ✅ Updated `core/ecs/src/lib.rs` to declare version module
- ✅ Implemented `api_version()` method in EcsView
- ✅ Verified: core/ecs compiles with API_VERSION = 3428876969

### 3. Verified systems/ecs version generation
- ✅ Confirmed build.rs generates both constants correctly
- ✅ API_VERSION = 3428876969 (matches core/ecs)
- ✅ STATE_FORMAT_VERSION = 935823075 (hash of core/ecs model)

## Work Remaining for Future Sessions

### 1. Rewrite systems/ecs to be Trait-Compliant
- **Clean up**: Delete the obsolete `systems/ecs/src/module_exports.rs` file.
- **Integrate Build System**: Create `build.rs` and update `Cargo.toml` to link with `playground-build-utils`.
- **Implement ViewModel**: In `systems/ecs/src/lib.rs`, define the `EcsViewModel` struct and implement the `ViewModelTrait`, including `api_version()`, `save_state()`, and `restore_state()` using the auto-generated version constants.
- **Rewrite Implementations**: Rewrite the existing function bodies in `systems/ecs/src/viewmodel/` to be compliant with the new `async fn` trait definitions. This involves removing all serialization logic and adapting the core logic to the new direct-call signatures.
- **Finalize**: Add the `impl EcsViewTrait for EcsViewModel` compile-time check and the `#[no_mangle]` static export for the `PLAYGROUND_VIEWMODEL`.

### 2. Test Complete Compilation and Hot-Reload
- Verify `systems/ecs` compiles successfully after the rewrite.
- Perform integration tests to ensure `core/ecs` and `systems/ecs` load and function correctly together.
- Test the stateful hot-reload functionality.

## Key Design Decisions Finalized

### Correct Versioning Scheme
- **Core modules**: API_VERSION = hash(src/view/ + src/model/)
- **System modules**:
  - API_VERSION = hash(core's src/view/ + core's src/model/) - must match Core
  - STATE_FORMAT_VERSION = hash(core's src/model/) - for state serialization validation
- Version matching is for compatibility checking, not enforcement
- Migration support is planned for future (not this session)

### Build System Pattern
- Central logic in `modules/build-utils/src/lib.rs`
- One-line boilerplate `build.rs` in each module
- TOML metadata in `Cargo.toml` for System modules
- Generated constants in `OUT_DIR/versions.rs`

## Success Criteria
- ✅ modules/build-utils generates correct version constants
- ✅ core/ecs integrated with version system and compiles
- ✅ systems/ecs generates correct version constants
- ⏳ systems/ecs integrated with version system (next session)
- ⏳ Full system compiles and versions match (next session)
