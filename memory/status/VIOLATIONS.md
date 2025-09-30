# Violations - MVVM Implementation Requirements (Session 67)

## Critical - Uncommitted Changes 🔴

### 1. Session 66 Partial Implementation
**Location**: Working directory
**Issue**: Half-implemented module system
**Files**:
- api/ directory (should be modules/)
- systems/module-loader/ (wrong location)
- core/ecs partially modified
**Fix**: Revert all changes, start fresh with MVVM

## Major - Architecture Changes Needed 🟠

### 2. Remove ALL VTable Code
**Location**: All core/* packages
**Fix Required**:
- Delete vtable.rs files
- Remove VTable fields from structs
- Replace with MVVM View APIs

### 3. Split Core Modules into Model/View
**Location**: core/ecs, core/console, core/rendering
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

### 4. Convert Systems to ViewModel
**Location**: systems/ecs, systems/console, systems/webgl
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