# Violations - MVVM Implementation Requirements (Session 69)

## Resolved in Session 69 ✅

### 1. core/ecs MVVM Implementation Complete
**Status**: ✅ IMPLEMENTED
- Rewrote core/ecs from scratch with MVVM pattern
- Model layer: Pure data structures
- View layer: API contracts only
- Event System replaces Messaging (Pre/Post events)
- Compiles successfully as dylib

## Pending - Core/Systems MVVM Conversion 🟡

### 2. Remove ALL VTable Code (Next: Session 69)
**Location**: All core/* packages
**Status**: PENDING
**Fix Required**:
- Delete vtable.rs files
- Remove VTable fields from structs
- Replace with MVVM View APIs

### 3. Split Core Modules into Model/View (Next: Session 69)
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

### 4. Convert Systems to ViewModel (Next: Session 69)
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

### 5. Add Cargo.toml Metadata
**Location**: All apps/*, plugins/*
**Fix Required**:
```toml
[[package.metadata.modules.core]]
name = "playground-core-rendering"
features = ["shaders"]
systems = ["playground-systems-webgl"]
```

### 6. Add build.rs Validation
**Location**: All apps/*
**Fix Required**: Compile-time feature checking

### 7. Set Module Compilation
**Location**: All core/*, systems/*, plugins/*, apps/*
**Fix Required**:
```toml
[lib]
crate-type = ["cdylib"]
```

## Implementation Order

1. **First**: Revert uncommitted changes
2. **Second**: Create modules/* infrastructure
3. **Third**: Convert core/ecs to Model+View
4. **Fourth**: Convert systems/ecs to ViewModel
5. **Fifth**: Test basic loading and binding
6. **Sixth**: Convert remaining modules

## Success Criteria

- ✅ Zero VTable code remaining
- ✅ All modules follow MVVM pattern
- ✅ Compile-time validation working
- ✅ Direct function calls (~1-5ns)
- ✅ Hot-reload functional