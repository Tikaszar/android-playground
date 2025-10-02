# Roadmap - MVVM Implementation Path (Sessions 67-76)

## Phase 1: Create modules/* Infrastructure ✅ COMPLETE (Sessions 68-70, 79)

### 1.1 modules/types ✅ COMPLETE (Session 79)
- Trait-based MVVM (ModelTrait, ViewTrait, ViewModelTrait)
- 64-bit IDs (ViewId, ModelId, ModelType)
- async-trait for async trait methods
- ModelTypeInfo for pool initialization
- Pure Rust interfaces (no C ABI)
- Proper Rust module organization (subdirectories with mod.rs)

### 1.2 modules/loader ✅ COMPLETE (Session 79)
- THE single unsafe block for ALL operations
- Load .so/.dll files
- Extract trait objects from symbols
- No runtime type checking
- Compiles successfully
- **NEEDS**: save_state/restore_state implementation (future)

### 1.3 modules/binding ✅ COMPLETE (Session 79)
- Triple-nested sharding (ViewId → ModelType → ModelPool)
- Lock-free Views/ViewModels via Handle<HashMap>
- Object recycling in ModelPools
- Runtime binding (not compile-time)
- Compiles successfully

### 1.4 modules/resolver ✅ COMPLETE
- Read Cargo.toml metadata
- Resolve dependencies
- Feature validation
- **NEEDS**: build.rs validation system (future)

### 1.5 modules/registry ✅ COMPLETE (Session 79)
- Runtime module orchestration
- Hot-reload infrastructure
- Module lifecycle management
- Compiles successfully

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

## Phase 3: Convert System Modules to ViewModel 🔄 IN PROGRESS

### 3.1 systems/ecs ⚠️ NEEDS TRAIT CONVERSION (Session 80 - IN PROGRESS)
- Has old serialization-based implementations (Sessions 74-78)
- Component module: 14/14 functions (needs trait impl)
- Entity module: 11/11 functions (needs trait impl)
- Event module: 18/18 functions (needs trait impl)
- Query module: 14/14 functions (needs trait impl)
- World module: 17/17 functions (needs trait impl)
- Storage module: 17/17 stubs (needs trait impl)
- System module: 13/13 stubs (needs trait impl)
**STATUS**: Session 80 converting to trait implementations

### 3.2 systems/console ⏳
- Implement core/console View APIs
- Terminal handling logic

### 3.3 systems/webgl ⏳
- Implement core/rendering View APIs
- WebGL-specific logic

## Phase 4: Performance Optimization ✅ PARTIAL (Sessions 76-77)

### 4.1 Component Pool Architecture ✅
- Replace Bytes storage with generic ComponentPool<T>
- Native storage, zero serialization (2-5ns vs 100-500ns)
- On-demand pool creation per component type
- **Status**: Implemented in Session 77

### 4.2 Thread-Safe Primitives ✅
- Handle<T>: Immutable reference wrapper
- Shared<T>: Mutable with RwLock
- Atomic<T>: Lock-free for Copy types
- Once<T>: Initialize once pattern
- **Status**: Implemented in Session 77

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
- Session 72-73: Complete core/ecs View layer (stubs) ✅
- Session 74: Component and Event ViewModel implementation (old pattern) ✅
- Session 75: Entity ViewModel implementation (old pattern) ✅
- Session 76: Performance optimization design ✅
- Session 77: ThreadSafe primitives and ComponentPool ✅
- Session 78: Query/World ViewModel implementation (old pattern) ✅
- Session 79: Trait-based MVVM modules/* infrastructure ✅

### Current
- Session 80: Convert core/ecs and systems/ecs to trait-based MVVM 🔄

### Next Steps (Session 81+)
1. Test module loading with trait-based system
2. Implement save_state/restore_state for hot-reload
3. Create build.rs validation
4. Convert remaining core modules (console, server, client, rendering, ui)
5. Convert remaining system modules (webgl, ui, console)
6. Test hot-reload functionality with state preservation
7. Performance benchmarking

## Risk Mitigation

### Compile-Time Safety (Session 76 Update)

All potential runtime failures must be compile-time errors:

#### Module Interface Safety
- **Risk**: Module missing required functions
- **Mitigation**: Trait-based contracts - won't compile without implementation
- **Enforcement**: `impl ModuleInterface` required for all modules

#### State Compatibility
- **Risk**: State version mismatch during hot-reload
- **Mitigation**: Versioned state types with serde
- **Enforcement**: `#[serde(version = N)]` - incompatible versions won't compile

#### Type Safety
- **Risk**: Component type mismatches
- **Mitigation**: Generic pools with compile-time type checking
- **Enforcement**: `get_pool<T>()` - wrong type won't compile

#### Module Dependencies
- **Risk**: Missing or incompatible system modules
- **Mitigation**: build.rs compile-time validation
- **Enforcement**: Build fails if dependencies not met

### Performance Risks
- **Risk**: ComponentPool type erasure complexity
- **Mitigation**: Use generics and traits without dyn
- **Enforcement**: Compile-time monomorphization

### Implementation Risks
- **Risk**: Breaking existing code during refactor
- **Mitigation**: Incremental changes, maintain compilation
- **Enforcement**: CI/CD requires all code to compile

### Acceptable Runtime Risks (Unavoidable)
- **Disk I/O failures**: Handled with Result<T, Error>
- **Network failures**: Graceful degradation
- **Resource exhaustion**: Monitoring and limits
- **File corruption**: Checksums and validation

### Core Principle
**Turn runtime bugs into compile-time errors wherever possible**