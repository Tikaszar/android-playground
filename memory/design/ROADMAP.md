# Roadmap - MVVM Implementation Path (Session 67)

## Immediate Priority: Remove Uncommitted Changes

### Current State
- Partial api/ implementation (should be modules/)
- Partial systems/module-loader (wrong location)
- core/ecs changes started but incomplete

### Step 1: Clean Working Directory
1. Revert all uncommitted changes
2. Start fresh with MVVM design
3. No half-implementations

## Phase 1: Create modules/* Infrastructure

### 1.1 modules/types
- Define Model, View, ViewModel base types
- Create module metadata structures
- Pure Rust interfaces (no C ABI)

### 1.2 modules/loader
- Single unsafe for Library::new()
- Load .so/.dll files
- No runtime type checking

### 1.3 modules/binding
- Connect View to ViewModel
- Compile-time validation
- Direct function pointers

### 1.4 modules/resolver
- Read Cargo.toml metadata
- Resolve dependencies
- Validate features at compile time

## Phase 2: Convert Core Modules to MVVM

### 2.1 core/ecs
- Split into model/ and view/
- Model: World, Entity, Component data
- View: spawn_entity, query, etc APIs

### 2.2 core/console
- Model: Terminal state, Dashboard data
- View: write, read, clear APIs

### 2.3 core/rendering
- Model: Window, RenderTarget data
- View: create_window, render_frame APIs

## Phase 3: Convert System Modules to ViewModel

### 3.1 systems/ecs
- Implement core/ecs View APIs
- No data storage
- Pure logic implementation

### 3.2 systems/console
- Implement core/console View APIs
- Terminal handling logic

### 3.3 systems/webgl
- Implement core/rendering View APIs
- WebGL-specific logic

## Phase 4: Update Build System

### 4.1 Cargo.toml Metadata
- Add package.metadata.modules sections
- Declare Core, Systems, features

### 4.2 build.rs Validation
- Check Systems provide required features
- Compile-time errors for mismatches

### 4.3 Module Compilation
- Set crate-type = ["cdylib"] for modules
- Configure proper exports

## Phase 5: Test Infrastructure

### 5.1 Basic Loading
- Load core/ecs + systems/ecs
- Verify binding works
- Test API calls

### 5.2 Hot-Reload
- Change systems/ecs
- Reload without restart
- Verify state preserved

### 5.3 System Swapping
- Load systems/webgl
- Swap to systems/vulkan
- Verify seamless transition

## Success Criteria

- ✅ NO VTable indirection
- ✅ Compile-time validation
- ✅ Direct function calls (~1-5ns)
- ✅ Hot-reload working
- ✅ MVVM separation enforced

## Timeline

- Session 67: Design complete ✅
- Session 68: modules/* infrastructure
- Session 69: Core modules conversion
- Session 70: System modules conversion
- Session 71: Build system and testing
- Session 72: Plugin conversion