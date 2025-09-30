# Current Session - Session 67: MVVM Architecture Design

## Session Goal
Design and document MVVM-based module system that eliminates VTable while maintaining hot-loading and compile-time safety.

## Work Completed This Session

### 1. Designed MVVM Architecture
- **Model**: Data structures (core/*/model/)
- **View**: API contracts (core/*/view/)
- **ViewModel**: Implementation (systems/*/viewmodel/)
- Core modules = Model + View
- System modules = ViewModel only

### 2. App-Driven Module Loading
- Apps declare which Core modules they need
- Apps declare which Systems implement them
- Features declared at Core level
- Unused modules don't load

### 3. Compile-Time Safety
- build.rs validates Systems provide required features
- Plugin requirements checked against App declarations
- Missing features = compile error
- Type mismatches = compile error

### 4. Cargo.toml Module Declaration
```toml
[[package.metadata.modules.core]]
name = "playground-core-rendering"
features = ["shaders", "textures"]
systems = ["playground-systems-vulkan", "playground-systems-webgl"]
```

### 5. No Runtime Indirection
- Direct function calls via binding
- ~1-5ns overhead (no VTable)
- Binding happens once at load time
- After binding, just function pointers

## Key Design Decisions

1. **MVVM instead of VTable** - Clean separation with compile-time safety
2. **Apps control everything** - Declare Core, Systems, and features
3. **Cargo.toml driven** - No separate config files
4. **Compile-time validation** - All errors before runtime
5. **modules/* infrastructure** - Not loadable, enables loading

## Next Session (68)

1. Revert uncommitted changes from Session 66
2. Create modules/* infrastructure
3. Start with modules/types for MVVM base
4. Implement modules/loader with single unsafe