# Current Session - Session 82: Build System Integration

## Session Goal
Integrate the automated build system into core/ecs, systems/ecs, and modules/build-utils to enable version generation for hot-reload safety.

## Context from Session 81
Session 81 completed the design and implementation of:
- Two-version safety scheme (API Version + State Format Version)
- Stateful hot-reload with save_state/restore_state methods
- BindingRegistry refactor with arc-swap for lock-free reads
- modules/build-utils crate for directory hashing

## Work Completed ✅

### 1. Updated modules/build-utils
- ✅ Added `toml = "0.9.8"` dependency for TOML parsing
- ✅ Updated `sha2 = "0.11.0-rc.2"` to latest version
- ✅ Updated `walkdir = "2.5.0"` to latest version
- ✅ Implemented `generate_versions()` unified function
- ✅ Implemented `check_if_system_module()` helper
- ✅ Implemented `get_core_path()` helper with TOML parsing
- ✅ Core modules generate API_VERSION only
- ✅ System modules generate API_VERSION + STATE_FORMAT_VERSION

### 2. Created core/ecs/build.rs
- ✅ One-line boilerplate: `playground_build_utils::generate_versions()`
- ✅ Generates `API_VERSION` from hashing `src/view/` directory
- ✅ Verified: Generated `API_VERSION = 1378979596`

### 3. Updated core/ecs/Cargo.toml
- ✅ Added `[build-dependencies]` section
- ✅ Added `playground-build-utils = { path = "../../modules/build-utils" }`

### 4. Created systems/ecs/build.rs
- ✅ One-line boilerplate: `playground_build_utils::generate_versions()`
- ✅ Detects System module via metadata
- ✅ Reads core_path from Cargo.toml metadata

### 5. Updated systems/ecs/Cargo.toml
- ✅ Added `[build-dependencies]` section
- ✅ Added `playground-build-utils = { path = "../../modules/build-utils" }`
- ✅ Added `[package.metadata.playground.implements]` section
- ✅ Added `core_path = "../../core/ecs"`

## Work Remaining

### 1. Update core/ecs to use generated API_VERSION
- Add `include!(concat!(env!("OUT_DIR"), "/versions.rs"));` to lib.rs
- Implement `api_version()` method in EcsView using API_VERSION constant

### 2. Update systems/ecs to use generated versions
- Add `include!(concat!(env!("OUT_DIR"), "/versions.rs"));` to lib.rs
- Implement `api_version()` method in EcsViewModel using API_VERSION constant
- Implement `save_state()` and `restore_state()` methods using STATE_FORMAT_VERSION

### 3. Test Full Build
- Verify core/ecs compiles completely
- Verify systems/ecs compiles completely
- Verify versions match between core and system

## Key Design Decisions

### Build System Pattern
The build system uses a "Boilerplate Hook" pattern:
- Central logic in `modules/build-utils/src/lib.rs`
- One-line `build.rs` in each module
- Metadata in `Cargo.toml` for System modules
- Generated constants in `OUT_DIR/versions.rs`

### Version Generation
- **Core modules**: Hash `src/view/` → API_VERSION
- **System modules**: Hash core's `src/view/` → API_VERSION, hash own `src/model/` → STATE_FORMAT_VERSION
- Deterministic via SHA256 hashing
- Automatic on every build

## Success Criteria
- ✅ modules/build-utils has unified generate_versions() function
- ✅ core/ecs has build.rs and generates API_VERSION
- ✅ systems/ecs has build.rs and metadata
- ⏳ core/ecs uses API_VERSION constant
- ⏳ systems/ecs uses both version constants
- ⏳ Full system compiles
