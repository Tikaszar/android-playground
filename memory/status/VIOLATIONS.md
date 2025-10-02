# Violations - Current Architecture Violations (Session 78)

## Resolved in Sessions 69-71 ✅

### 1. core/ecs MVVM Implementation Complete (Session 69)
**Status**: ✅ IMPLEMENTED
- Rewrote core/ecs from scratch with MVVM pattern
- Model layer: Pure data structures
- View layer: API contracts only
- Event System replaces Messaging (Pre/Post events)
- Compiles successfully as dylib

### 2. modules/loader and modules/binding Compilation Fixed (Session 70)
**Status**: ✅ IMPLEMENTED
- Added Copy+Clone to ViewAPI and ViewModelImpl
- Fixed symbol extraction with .clone()
- Fixed function pointer dereferencing
- Both packages compile successfully

## Pending - Core/Systems MVVM Conversion 🟡

### 3. Remove ALL VTable Code (Next: Session 71)
**Location**: All core/* packages
**Status**: PENDING
**Fix Required**:
- Delete vtable.rs files
- Remove VTable fields from structs
- Replace with MVVM View APIs

### 4. Split Core Modules into Model/View (Next: Session 71)
**Location**: core/ecs, core/console, core/rendering
**Status**: PENDING
**Fix Required**:
```
core/ecs/
├── model/
│   ├── world.rs
│   └── entity.rs
└── view/
    ├── spawn_entity.rs
    └── query.rs
```

### 5. Convert Systems to ViewModel (Next: Session 71)
**Location**: systems/ecs, systems/console, systems/webgl
**Status**: PENDING
**Fix Required**:
```
systems/ecs/
└── viewmodel/
    ├── spawn_entity.rs
    └── query.rs
```

## Build System Changes 🟡

### 6. Add Cargo.toml Metadata
**Location**: All apps/*, plugins/*
**Fix Required**:
```toml
[[package.metadata.modules.core]]
name = "playground-core-rendering"
features = ["shaders"]
systems = ["playground-systems-webgl"]
```

### 7. Add build.rs Validation
**Location**: All apps/*
**Fix Required**: Compile-time feature checking

### 8. Set Module Compilation
**Location**: All core/*, systems/*, plugins/*, apps/*
**Fix Required**:
```toml
[lib]
crate-type = ["cdylib"]
```

## Implementation Order

1. ✅ **First**: Create modules/* infrastructure (Session 68)
2. ✅ **Second**: Fix modules compilation (Session 70)
3. ✅ **Third**: Convert core/ecs to Model+View (Session 69)
4. **Fourth**: Convert systems/ecs to ViewModel (Session 71 - NEXT)
5. **Fifth**: Test basic loading and binding
6. **Sixth**: Convert remaining modules

## Critical Violations Discovered (Session 78) 🔴

### 1. ViewModelFunction uses dyn
**Location**: modules/types/src/viewmodel/function.rs
**Violation**: `Box<dyn Future<...>>` violates NO dyn rule
**Fix Required**: Direct function signatures without trait objects

### 2. All ViewModel implementations use serialization
**Location**: systems/ecs/src/viewmodel/*
**Violation**: All 74+ functions use `args: &[u8]` and serialization
**Fix Required**: Direct parameters like `world: &Handle<World>`

### 3. World as global state
**Location**: systems/ecs/src/state.rs
**Violation**: Uses global OnceCell for World
**Fix Required**: Pass World as parameter through all functions

## Success Criteria

- ❌ NO dyn compliance (ViewModelFunction violates)
- 🟡 All modules follow MVVM pattern (structure correct, signatures wrong)
- 🟡 Compile-time validation working (design complete)
- ❌ Direct function calls (still using serialization)
- 🟡 Hot-reload functional (needs new module system)