# Architecture - MVVM-Based Module System

## Core Architectural Pattern (Sessions 67-71, 79)

### MVVM Architecture
```
Apps → Plugins → Core (Model+View) → [Module Binding] → Systems (ViewModel)
```

**Key Components:**
- **Model** = Data structures (core/*/model/), with `World` acting as a facade to these structures.
- **View** = API contracts (core/*/view/) - Trait definitions.
- **ViewModel** = Implementation (systems/*/viewmodel/) - Trait implementations that interact with the `BindingRegistry` via the `World` facade.
- **Binding** = Trait-based with Arc<dyn Trait> (Session 79)

## Implementation (Sessions 68-71, 76-79, 80)
- **modules/types** - Trait-based MVVM with fragments (Session 80) ✅ COMPILES
- **modules/loader** - THE single unsafe block for Library::new() ✅ COMPILES
- **modules/binding** - Triple-nested sharding with ModelPools ✅ Session 79
- **modules/resolver** - Cargo.toml module declarations ✅ COMPILES
- **modules/registry** - Runtime module orchestration ✅ COMPILES
- **core/ecs/model** - Complete ECS Model layer with all data structures ✅ COMPILES
- **core/types** - Thread-safe wrappers (Handle, Shared, Atomic, Once) ✅ Session 77

## Module System Architecture (Sessions 79-80)

### Trait-Based MVVM Infrastructure (Session 80, Refined Session 81)
```rust
// 64-bit unique identifiers
pub type ViewId = u64;
pub type FragmentId = u64;  // Session 80: Fragment identification
pub type ModelId = u64;
pub type ModelType = u64;

// Base traits
#[async_trait::async_trait]
pub trait ViewTrait: Send + Sync {
    fn view_id(&self) -> ViewId;
    fn api_version(&self) -> u32; // For API compatibility
}

// Session 80: Fragment support for logical grouping
#[async_trait::async_trait]
pub trait ViewFragmentTrait: Send + Sync {
    fn view_id(&self) -> ViewId;
    fn fragment_id(&self) -> FragmentId;
}

#[async_trait::async_trait]
pub trait ViewModelTrait: Send + Sync {
    fn view_id(&self) -> ViewId;  // Which View this implements
    fn api_version(&self) -> u32; // For API compatibility

    // Optional: For stateful hot-reloading. Default is stateless.
    async fn save_state(&self) -> Option<Result<Vec<u8>, ModuleError>> {
        None
    }

    async fn restore_state(&self, _state: Vec<u8>) -> Option<Result<(), ModuleError>> {
        None
    }
}

// Session 80: Fragment support for ViewModels
#[async_trait::async_trait]
pub trait ViewModelFragmentTrait: Send + Sync {
    fn view_id(&self) -> ViewId;
    fn fragment_id(&self) -> FragmentId;
}

#[async_trait::async_trait]
pub trait ModelTrait: Send + Sync {
    fn model_id(&self) -> ModelId;
    fn model_type(&self) -> ModelType;  // For pool routing
}
```

### Runtime Type Generation (Session 80)
```rust
// Generate unique ModelType from Rust types at runtime
pub fn model_type_of<T: 'static>() -> ModelType {
    let type_id = TypeId::of::<T>();
    let mut hasher = DefaultHasher::new();
    type_id.hash(&mut hasher);
    hasher.finish()
}

// All ECS models implement ModelTrait
impl ModelTrait for Entity {
    fn model_id(&self) -> ModelId {
        self.id.0 as u64
    }
    fn model_type(&self) -> ModelType {
        model_type_of::<Entity>()  // Runtime-generated, deterministic
    }
}
```

### Concurrent Binding Registry (Session 79, Refined in Session 81)
```rust
use arc_swap::ArcSwap;

pub struct BindingRegistry {
    // Concurrently updatable with lock-free reads
    views: ArcSwap<HashMap<ViewId, Handle<dyn ViewTrait>>>,
    viewmodels: ArcSwap<HashMap<ViewId, Handle<dyn ViewModelTrait>>>,

    // Flattened map for models: (ViewId, ModelType) -> Pool
    models: ArcSwap<HashMap<(ViewId, ModelType), ModelPool>>,
}

pub struct ModelPool {
    active: Shared<HashMap<ModelId, Handle<dyn ModelTrait>>>,
    recycled: Shared<Vec<Handle<dyn ModelTrait>>>,  // Object pooling
}
```

### Concurrency and Locking Model
- **Registry Access**: All registry read operations (getting views, viewmodels, or pools) are completely lock-free and fast (~5ns) via `arc-swap`.
- **Registry Updates**: Updates (registrations) use a Read-Copy-Update (RCU) strategy, which is non-blocking for readers and enables concurrent writes.
- **Pool Operations**: The `ModelPool` itself uses an `RwLock`, so contention is isolated to operations on the *same pool* (i.e., same ViewId and ModelType).

### Object Recycling (Session 79)
```rust
// Reduce allocations by reusing deleted models
pool.get_or_recycle(model_id, || create_new_model()).await
```

## Component Pool Architecture (Session 76)

### Performance-Critical Design
```rust
// OLD: Serialization overhead (100-500ns per access)
components: Shared<HashMap<EntityId, HashMap<ComponentId, Bytes>>>

// NEW: Native storage with pools (2-5ns per access)
component_pools: HashMap<ComponentId, ComponentPool<T>>

pub struct ComponentPool<T> {
    components: HashMap<EntityId, T>,  // Native T, no serialization
}
```

### Component Threading Strategies
```rust
// Hot path: Field-level atomics (2-5ns)
pub struct Position {
    pub x: Atomic<f32>,
    pub y: Atomic<f32>,
    pub z: Atomic<f32>,
}

// Complex data: Component-level locking (20ns)
pub struct Inventory {
    pub items: Shared<Vec<Item>>,
}

// Read-heavy: Copy-on-write (2ns read)
pub struct Mesh {
    pub data: Handle<MeshData>,
}
```

## Thread-Safe Primitives (Session 76)

### Core Wrappers
```rust
// Four fundamental primitives for all thread-safe data
Handle<T>   // Arc<T> - Immutable reference
Shared<T>   // Arc<RwLock<T>> - Mutable with RwLock
Atomic<T>   // Arc<AtomicCell<T>> - Lock-free for Copy types
Once<T>     // Arc<OnceCell<T>> - Initialize once

// Clean API instead of verbose Arc/RwLock
let pos = Shared::new(position);  // Not Arc::new(RwLock::new(position))
```

### Performance Characteristics
| Primitive | Read | Write | Use Case |
|-----------|------|-------|----------|
| Handle | 2ns | N/A | Immutable data |
| Shared | 20ns | 25ns | Complex mutable |
| Atomic | 3ns | 5ns | Simple values |
| Once | 2ns | One-time | Lazy init |

## MVVM Separation Pattern

### Strict Module Types
- **Core modules** = Model (data) + View (API contracts as traits)
- **System modules** = ViewModel (trait implementations only)
- **Plugin modules** = High-level features (use Core APIs)
- **App modules** = Applications (use Plugin APIs, declare Systems)

### Direct Trait Access (Session 79)
```rust
// Consumers get Handle<dyn Trait> once, store it
let entity_vm = registry.get_viewmodel(view_id).await?;
self.entity_vm = entity_vm;

// Direct calls forever after - NO registry lookup
self.entity_vm.spawn_entity(&world, components).await?;
```

### Module Structure Example
```
core/ecs/
├── model/                # Pure data structures
│   ├── entity/          # EntityId, Entity, EntityRef, Generation
│   ├── component/       # ComponentId, Component, ComponentRef
│   ├── event/           # EventId, Event, EventRef, Priority, Subscription
│   ├── query/           # QueryId, Query, QueryRef, QueryFilter
│   ├── storage/         # StorageId, Storage, StorageRef
│   ├── system/          # SystemId, System, SystemRef
│   └── world/           # World, WorldRef
└── view/
    ├── entity.rs        # EntityView trait
    └── query.rs         # QueryView trait

systems/ecs/
└── viewmodel/
    ├── entity.rs        # impl EntityView for EntityViewModel
    └── query.rs         # impl QueryView for QueryViewModel
```

## Module Declaration System

### Apps Declare Everything
```toml
# apps/editor/Cargo.toml
[[package.metadata.modules.core]]
name = "playground-core-rendering"
features = ["shaders", "textures"]
systems = [
    "playground-systems-vulkan",
    "playground-systems-webgl"  # Fallback
]
```

### Automated Build System (Session 81 Design)

To automate versioning and validation, the workspace uses a centralized build-logic pattern. This ensures consistency and avoids duplicated code, while integrating perfectly with standard `cargo build` commands.

1.  **Central Logic Crate (`modules/build-utils`)**: A dedicated library crate that contains all the logic for version generation. It provides a single function, `generate_versions()`.

2.  **Boilerplate Hook (`build.rs`)**: Every `Core` and `System` module that requires versioning has an identical, one-line `build.rs` script. This file is pure boilerplate that calls the central logic:
    ```rust
    // In core/ecs/build.rs, systems/ecs/build.rs, etc.
    fn main() { playground_build_utils::generate_versions(); }
    ```

3.  **Configuration (`Cargo.toml`)**: Each of these modules adds `playground-build-utils` to its `[build-dependencies]`. `System` modules also declare which `Core` module they implement:
    ```toml
    # In systems/ecs/Cargo.toml
    [package.metadata.playground.implements]
    core_path = "../core/ecs"
    ```

4.  **Process**: During `cargo build`, the hook script for each crate is triggered. It calls the central `generate_versions()` function, which then inspects that crate's `Cargo.toml` and generates the appropriate `API_VERSION` and/or `STATE_FORMAT_VERSION` constants for it.

## Module Loading from Cargo.toml

### App-Driven Loading
- Apps/Plugins declare Core modules in Cargo.toml
- Apps declare which Systems implement Core modules
- Features specified at Core level, apply to any System
- Only declared modules load (unused Core/Systems don't load)

### System Feature Validation
```toml
# systems/webgl/Cargo.toml
[package.metadata.provides]
core = "playground-core-rendering"
features = ["shaders", "textures", "2d", "basic-3d"]
```

## Package Layers

### Core Layer
- **Purpose**: Define contracts and data structures
- **Contains**: Structs with data fields, type definitions, thread-safe primitives, trait definitions
- **NO**: Implementation logic, business logic, I/O operations
- **Examples**: core/ecs, core/server, core/client, core/console

### Systems Layer
- **Purpose**: Implement all actual functionality via trait implementations
- **Contains**: ViewModel trait implementations, business logic, I/O operations
- **Dependencies**: Can ONLY use core/* packages
- **Examples**: systems/ecs, systems/networking, systems/webgl, systems/console

### Apps Layer
- **Purpose**: Complete applications that orchestrate
- **Dependencies**: Use core/* packages ONLY (with features)
- **Examples**: playground-apps-editor, playground-apps-game

### Plugins Layer
- **Purpose**: High-level features
- **Dependencies**: Use core/* packages ONLY (with features)
- **Run by**: systems/ecs scheduler
- **Examples**: All IDE plugins, game features

## Symbol Extraction (Session 79)

### Core Module Exports
```rust
// Core modules export View + Models
#[no_mangle]
pub static PLAYGROUND_VIEW: &dyn ViewTrait = &EcsView;

#[no_mangle]
pub static PLAYGROUND_MODELS: &[ModelTypeInfo] = &[
    ModelTypeInfo { model_type: 0x0001, type_name: "Entity" },
    ModelTypeInfo { model_type: 0x0002, type_name: "Component" },
];
```

### System Module Exports
```rust
// System modules export ViewModel
#[no_mangle]
pub static PLAYGROUND_VIEWMODEL: &dyn ViewModelTrait = &EcsViewModel;
```

### Automated Versioning and Safety (Session 81 Design)

To ensure safe hot-reloading, the architecture uses a two-version scheme that is fully automated via `build.rs` scripts and content hashing. This prevents both API contract mismatches and data corruption from incompatible state.

**1. API Version (for API Compatibility)**

This version ensures that a `ViewModel` implementation from a `System` is compatible with the `View` trait contract from `Core`.

- **Generation**: A `build.rs` script in both the `Core` and `System` crates generates a hash of the `core/.../view` directory source files. This hash is compiled as an `API_VERSION` constant into both modules.
- **Enforcement**: The `api_version()` method on `ViewTrait` and `ViewModelTrait` exposes this version. The `BindingRegistry` asserts that these versions are equal when binding a `ViewModel` to a `View`. A mismatch aborts the binding, preventing a crash.

**2. State Format Version (for Data Compatibility)**

This version tracks the physical structure of the data being saved for stateful hot-reloading.

- **Generation**: The `build.rs` script in a stateful `System` (e.g., `systems/ecs`) generates a hash of the `core/.../model` directory source files. This hash is compiled as a `STATE_FORMAT_VERSION` constant.
- **Enforcement**: The `save_state` method embeds this version inside the serialized data. The `restore_state` method reads the version from the data and compares it to its own compiled version. If they do not match, the state is rejected, preventing data corruption.

This two-version scheme makes hot-reloading safe and reliable while remaining 100% automated.

### Loader Extraction (THE unsafe block)
```rust
unsafe {
    // Extract View trait object
    let view: Symbol<&'static Handle<dyn ViewTrait>> =
        library.get(b"PLAYGROUND_VIEW\0")?;

    // Extract Model type info
    let models: Symbol<*const &'static [ModelTypeInfo]> =
        library.get(b"PLAYGROUND_MODELS\0")?;

    // Extract ViewModel trait object
    let viewmodel: Symbol<&'static Handle<dyn ViewModelTrait>> =
        library.get(b"PLAYGROUND_VIEWMODEL\0")?;
}
```

## Architectural Invariants

1. **NO unsafe** - EXCEPTION: Single Library::new() in module loader only
2. **NO dyn (except modules/*)** - modules/* uses Arc<dyn Trait> for hot-loading (Session 79)
3. **NO Any** - Use concrete types or generics
4. **NO turbofish** - Use ComponentId not generics for components
5. **Model is pure data** - Only data fields with threading primitives, no logic
6. **NO global state** - World passed as Handle<World> parameter
7. **Native storage** - Components stored as T, not Bytes (Session 77)
8. **Component-level locking** - Each component manages its concurrency (Session 76)
9. **Compile-time safety** - Turn ALL runtime bugs into compile-time errors (Session 76)
10. **Direct trait access** - No HashMap lookups after initial binding (Session 79)

## Compile-Time Safety Principles (Session 76)

### Core Philosophy
**Every potential runtime failure must be a compile-time error**

### Enforced at Compile Time
- **Type Safety**: Generic pools prevent type mismatches
- **Interface Contracts**: Traits ensure all required functions exist
- **State Compatibility**: Versioned types prevent incompatible states
- **Module Dependencies**: build.rs validates at compile time
- **Component Access**: Type parameters ensure correct pool access

### How We Achieve This
1. **Generics over Any**: `get_pool<T>()` not `get_pool(string)`
2. **Traits over Runtime Checks**: `impl RequiredInterface` enforced
3. **Versioned Types**: `StateV2` won't accept `StateV1` data
4. **Build Validation**: Missing dependencies won't compile
5. **Type-Safe APIs**: Wrong types won't compile

### What Remains Runtime (Acceptable)
- **I/O Failures**: Disk full, network down (Result<T, Error>)
- **Resource Exhaustion**: Out of memory (graceful degradation)
- **External Data**: User input, file corruption (validation)
- **Concurrency**: Race conditions (proper synchronization)

## Performance Targets (Sessions 76, 79)

| Operation | Old (Serialized) | New (Native Pools) | Improvement |
|-----------|-----------------|-------------------|-------------|
| View/ViewModel/Pool Lookup | N/A | Lock-free (~5ns) | **Baseline** |
| Model Access (Same Pool) | 100-500ns | 20-30ns (RwLock) | **3-15x** |
| Model Create/Recycle | 200-600ns | 30-40ns (pooled) | **5-15x** |
| Lock Contention | Global | Per-pool | **N-fold** |
