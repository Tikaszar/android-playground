# Roadmap - MVVM Implementation Path (Sessions 67-76)

## Phase 1: Create modules/* Infrastructure ✅ COMPLETE (Sessions 68-70)

### 1.1 modules/types ✅
- Defined Model, View, ViewModel base types (concrete classes, NO traits)
- Created module metadata structures
- Pure Rust interfaces (no C ABI)
- Proper Rust module organization (subdirectories with mod.rs)
- ViewAPI and ViewModelImpl are Copy+Clone (Session 70)

### 1.2 modules/loader ✅
- Single unsafe block for ALL operations
- Load .so/.dll files
- No runtime type checking
- Compiles successfully (Session 70)
- **NEEDS**: save_state/restore_state implementation (Session 76)

### 1.3 modules/binding ✅
- Connect View to ViewModel
- Direct function pointer binding
- Runtime binding (not compile-time)
- Compiles successfully (Session 70)

### 1.4 modules/resolver ✅
- Read Cargo.toml metadata
- Resolve dependencies
- Feature validation
- **NEEDS**: build.rs validation system (Session 76)

## Phase 2: Convert Core Modules to MVVM ✅ PARTIAL

### 2.1 core/ecs ✅ COMPLETE (Sessions 69-73)
- Split into model/ and view/
- Model: World, Entity, Component data (Session 71)
- View: spawn_entity, query, etc APIs (Session 72-73)
- Event System replaces Messaging

### 2.2 core/types 🔄 IN PROGRESS (Session 76)
- Thread-safe primitives: Handle, Shared, Atomic, Once
- Clean wrapper APIs instead of raw Arc/RwLock
- Base trait ThreadSafe for common functionality

### 2.3 core/console ⏳
- Model: Terminal state, Dashboard data
- View: write, read, clear APIs

### 2.4 core/rendering ⏳
- Model: Window, RenderTarget data
- View: create_window, render_frame APIs

## Phase 3: Convert System Modules to ViewModel 🔄 PARTIAL

### 3.1 systems/ecs 🔄 IN PROGRESS (Sessions 74-75)
- Component module: 14/14 functions ✅
- Entity module: 11/11 functions ✅
- Event module: 18/18 functions ✅
- Query module: 14/14 stubs ⚠️
- Storage module: 17/17 stubs ⚠️
- System module: 13/13 stubs ⚠️
- World module: 6/17 partial 🔄

### 3.2 systems/console ⏳
- Implement core/console View APIs
- Terminal handling logic

### 3.3 systems/webgl ⏳
- Implement core/rendering View APIs
- WebGL-specific logic

## Phase 4: Performance Optimization 🔄 IN PROGRESS (Session 76)

### 4.1 Component Pool Architecture 🔄
- Replace Bytes storage with generic ComponentPool<T>
- Native storage, zero serialization (2-5ns vs 100-500ns)
- On-demand pool creation per component type
- **Status**: Design complete, implementation pending

### 4.2 Thread-Safe Primitives 🔄
- Handle<T>: Immutable reference wrapper
- Shared<T>: Mutable with RwLock
- Atomic<T>: Lock-free for Copy types
- Once<T>: Initialize once pattern
- **Status**: Design complete, implementation pending

### 4.3 Component-Level Threading 🔄
- Components manage own thread-safety
- Field-level atomics for hot paths
- Component-level locking for complex data
- **Status**: Pattern established, implementation pending

### 4.4 World Parameter Passing 🔄
- Remove global World instance
- Pass World as parameter through ViewModel
- Module stores World reference in state
- **Status**: Design complete, implementation pending

### 4.5 64-bit ComponentIds 🔄
- Upgrade from 32-bit to 64-bit IDs
- Deterministic generation for networking/saves
- Collision-free design
- **Status**: Design complete, implementation pending

## Phase 5: Module System Completion ⏳

### 5.1 save_state/restore_state ⏳
- Implement in modules/loader
- Serialize World before hot-reload
- Restore after reload
- Required for testing

### 5.2 Build Validation ⏳
- Create build.rs for compile-time checks
- Validate Systems provide required features
- Module compatibility verification

### 5.3 Module Testing ⏳
- Test hot-reload with state preservation
- Verify module swapping
- Performance benchmarks

## Success Criteria

### Phase 1-3 (MVVM Implementation)
- ✅ NO VTable indirection
- ✅ Compile-time validation design
- ✅ Direct function calls (~1-5ns)
- 🔄 Hot-reload working (needs save_state)
- ✅ MVVM separation enforced

### Phase 4 (Performance)
- 🔄 Component access: 20-100x faster (pending implementation)
- 🔄 Memory usage: 50% reduction (pending implementation)
- 🔄 Lock contention: N-fold improvement (pending implementation)
- 🔄 Native storage instead of serialization (pending implementation)

### Phase 5 (Completion)
- ⏳ Hot-reload with state preservation
- ⏳ Build-time validation
- ⏳ Complete test coverage

## Timeline

### Completed
- Session 67: Design complete ✅
- Session 68: modules/* infrastructure ✅
- Session 69: core/ecs MVVM conversion ✅
- Session 70: Fixed modules/loader and modules/binding compilation ✅
- Session 71: Complete core/ecs Model layer ✅
- Session 72-73: Complete core/ecs View layer ✅
- Session 74: Component and Event ViewModel implementation ✅
- Session 75: Entity ViewModel implementation ✅

### Current
- Session 76: Performance optimization design 🔄

### Next Steps
1. Implement ThreadSafe primitives in core/types
2. Implement ComponentPool<T> system
3. Complete remaining ViewModel stubs (Query, Storage, System, World)
4. Implement save_state/restore_state
5. Create build.rs validation
6. Test hot-reload functionality

## Risk Mitigation

### Performance Risks
- **Risk**: ComponentPool type erasure complexity
- **Mitigation**: Use trait without dyn, or registry pattern

### Implementation Risks
- **Risk**: Breaking existing code during refactor
- **Mitigation**: Incremental changes, maintain compilation

### Testing Risks
- **Risk**: Hot-reload bugs only appear at runtime
- **Mitigation**: Comprehensive test suite, state validation