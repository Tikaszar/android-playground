# Module Architecture - Trait-Based Hot-Loading (Sessions 67-71, 79)

## Overview

The entire engine uses MVVM pattern with hot-loadable modules where Core provides Model+View (as traits), Systems provide ViewModel (trait implementations), and everything can reload at runtime.

## Implementation Status (Sessions 68-71, 79)

### modules/* Infrastructure ✅ COMPLETE (Sessions 79-80)
- **modules/types** - Trait-based MVVM with Associated Types & Runtime Types
  - 64-bit IDs (ViewId, ModelId, ModelType)
  - Runtime type generation via `model_type_of<T>()`
  - async-trait for async trait methods
  - ModelTypeInfo for pool initialization
- **modules/loader** - THE single unsafe block ✅ COMPILES
- **modules/binding** - Triple-nested sharding with ModelPools ✅ COMPILES
- **modules/registry** - Runtime orchestration ✅ COMPILES
- **modules/resolver** - Cargo.toml parsing ✅ COMPILES

### core/ecs ✅ PARTIAL (Sessions 71, 80)
- **model/** - All models implement ModelTrait with runtime types ✅
  - **entity/** - EntityId, Entity, EntityRef, Generation
  - **component/** - ComponentId, Component, ComponentRef
  - **event/** - EventId, Event, EventRef, Priority, Subscription, SubscriptionId
  - **query/** - QueryId, Query, QueryRef, QueryFilter
  - **storage/** - StorageId, Storage, StorageRef
  - **system/** - SystemId, System, SystemRef
  - **world/** - World, WorldRef (needs update to be lightweight)
- **view/** - Associated Types pattern with fragments ✅
  - All fragments (EntityView, ComponentView, etc.) complete
  - EcsView composes all fragments via associated types

## MVVM Module Types

### 1. Core Modules (Model + View)
- **Purpose**: Define data structures AND API contract traits
- **Location**: `core/*`
- **Structure**:
  - `model/` - Data structures only
  - `view/` - Trait definitions (API contracts)
- **Examples**: `core/ecs`, `core/rendering`, `core/console`
- **NO**: Implementation logic

### 2. System Modules (ViewModel)
- **Purpose**: Implement Core View traits
- **Location**: `systems/*`
- **Structure**:
  - `viewmodel/` - Trait implementations
- **Examples**: `systems/ecs`, `systems/webgl`, `systems/console`
- **NO**: Data storage (except internal state)

### 3. Plugin Modules
- **Purpose**: High-level features
- **Uses**: Core APIs only (never Systems directly)
- **Location**: `plugins/*`
- **Examples**: `plugins/editor-core`, `plugins/file-browser`

### 4. App Modules
- **Purpose**: Complete applications
- **Uses**: Plugin APIs primarily, Core APIs when needed
- **Declares**: Which Systems to load via Cargo.toml
- **Location**: `apps/*`
- **Examples**: `apps/editor`, `apps/game`

## Trait-Based Architecture (Sessions 79-80)

### Core Traits (Session 80: Added Fragments)
```rust
// modules/types/src/model/base.rs
pub type ModelId = u64;
pub type ModelType = u64;

#[async_trait::async_trait]
pub trait ModelTrait: Send + Sync {
    fn model_id(&self) -> ModelId;
    fn model_type(&self) -> ModelType;  // For pool routing
}

// modules/types/src/view/base.rs
pub type ViewId = u64;
pub type FragmentId = u64;  // Session 80

#[async_trait::async_trait]
pub trait ViewTrait: Send + Sync {
    fn view_id(&self) -> ViewId;
}

// Session 80: Fragment traits for logical grouping
#[async_trait::async_trait]
pub trait ViewFragmentTrait: Send + Sync {
    fn view_id(&self) -> ViewId;
    fn fragment_id(&self) -> FragmentId;
}

// modules/types/src/viewmodel/base.rs
#[async_trait::async_trait]
pub trait ViewModelTrait: Send + Sync {
    fn view_id(&self) -> ViewId;  // Which View this implements
}

// Session 80: Fragment traits for ViewModels
#[async_trait::async_trait]
pub trait ViewModelFragmentTrait: Send + Sync {
    fn view_id(&self) -> ViewId;
    fn fragment_id(&self) -> FragmentId;
}
```

### Core Module Example (Session 80: Fragment Pattern)
```rust
// core/ecs/src/view/entity.rs - Fragment trait
#[async_trait::async_trait]
pub trait EntityView: ViewFragmentTrait {
    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity>;
    async fn despawn_entity(&self, world: &World, entity: Entity) -> EcsResult<()>;
    // ... 11 methods total
}

// core/ecs/src/view/mod.rs - Composite trait
pub trait EcsViewTrait:
    ViewTrait +
    EntityView +
    ComponentView +
    EventView +
    QueryView +
    StorageView +
    SystemView +
    WorldView
{}

// core/ecs/src/lib.rs - Concrete implementation
pub struct EcsView;

impl ViewTrait for EcsView {
    fn view_id(&self) -> ViewId { ECS_VIEW_ID }
}

impl EntityView for EcsView {
    fn view_id(&self) -> ViewId { ECS_VIEW_ID }
    fn fragment_id(&self) -> FragmentId { ENTITY_FRAGMENT_ID }
    // ... stub implementations
}

impl EcsViewTrait for EcsView {}

// Export symbol
#[no_mangle]
pub static PLAYGROUND_VIEW: &dyn ViewTrait = &EcsView;

#[no_mangle]
pub static PLAYGROUND_MODELS: &[ModelTypeInfo] = &[
    ModelTypeInfo { model_type: 0x0001, type_name: "Entity" },
    ModelTypeInfo { model_type: 0x0002, type_name: "Component" },
];
```

### System Module Example (Session 80: Fragment Pattern)
```rust
// systems/ecs/src/lib.rs
pub struct EcsViewModel;

impl ViewModelTrait for EcsViewModel {
    fn view_id(&self) -> ViewId { ECS_VIEW_ID }
}

#[async_trait::async_trait]
impl EntityView for EcsViewModel {
    fn view_id(&self) -> ViewId { ECS_VIEW_ID }
    fn fragment_id(&self) -> FragmentId { ENTITY_FRAGMENT_ID }

    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity> {
        // ACTUAL implementation - accesses BindingRegistry pools
        let entity_id = EntityId(world.next_entity_id.fetch_add(1));
        let generation = Generation(1);

        let mut entities = world.entities.write().await;
        entities.insert(entity_id, generation);

        Ok(Entity { id: entity_id, generation })
    }

    // ... implement all 11 methods
}

// Compile-time enforcement
impl EcsViewTrait for EcsViewModel {}

// Export symbol
#[no_mangle]
pub static PLAYGROUND_VIEWMODEL: &dyn ViewModelTrait = &EcsViewModel;
```

## Module Declaration in Cargo.toml (Session 81 Design)

The relationship between `Apps`, `Core` modules, and `Systems` is declared explicitly in `Cargo.toml` files to enable build-time validation.

### App Declares Requirements
An `App` declares which `Core` modules it uses, the `features` it needs from them, and an ordered list of preferred `Systems` to provide the implementation.
```toml
# in apps/editor/Cargo.toml
[[package.metadata.playground.requires.core]]
name = "playground-core-rendering"
features = ["shaders", "textures"]
systems = ["playground-systems-vulkan", "playground-systems-webgl"]
```

### System Declares Provisions
A `System` declares which `Core` module it implements and the set of `features` it supports.
```toml
# in systems/webgl/Cargo.toml
[package.metadata.playground.provides]
core_module = "playground-core-rendering"
features = ["shaders", "textures", "2d"]
```

This structure allows an `App`'s `build.rs` script to validate that a compatible `System` exists for all of its requirements before the engine is ever compiled, preventing runtime errors.

## Module Infrastructure (modules/*)

```
modules/                  # NOT loadable - compiled into main binary
├── types/               # Trait-based MVVM types
│   ├── model/          # ModelTrait, ModelId, ModelType, ModelTypeInfo
│   ├── view/           # ViewTrait, ViewId
│   └── viewmodel/      # ViewModelTrait
├── loader/              # THE single unsafe (Library::new)
├── binding/             # Triple-nested sharding with pools
│   ├── pool.rs         # ModelPool with object recycling
│   └── registry.rs     # BindingRegistry with sharded storage
├── registry/            # Runtime module tracking
└── resolver/            # Read Cargo.toml, resolve dependencies
```

## Module Loading Process (Session 79)

1. **Read App Cargo.toml** - Find declared Core modules and Systems
2. **Validate Features** - build.rs checks Systems provide required features
3. **Load Core Modules** - Extract View trait + ModelTypeInfo
4. **Initialize Pools** - Create ModelPool for each ModelType
5. **Load System Modules** - Extract ViewModel trait
6. **Bind ViewModel to View** - Store in BindingRegistry by ViewId
7. **Consumers Get Handle** - `registry.get_viewmodel(view_id)` returns `Handle<dyn ViewModelTrait>`
8. **Direct Calls** - Consumer stores Handle, calls trait methods directly

## Binding Registry Architecture (Session 79, Refined Session 81)

### Concurrent, Flattened Map
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

### Access Pattern
```rust
// Get ViewModel once, store it
let entity_vm: Handle<dyn ViewModelTrait> = registry.get_viewmodel(0x1234...)?;
app.entity_vm = entity_vm;

// Direct trait method calls - NO registry lookup
app.entity_vm.spawn_entity(&world, components).await?;
```

### Performance
- **Registry Read**: Lock-free (~5ns) for any lookup (View, ViewModel, or Model Pool).
- **Registry Write**: Non-blocking for readers, using `arc-swap`'s RCU mechanism.
- **Model access**: `RwLock` at the individual `ModelPool` level (~20-30ns).
- **Object recycling**: Reuses deleted models to reduce allocations.
- **Lock contention**: Highly minimized. Contention only occurs when multiple threads write to the *exact same model pool* simultaneously.

## Compile-Time Safety

### Feature Validation
```rust
// apps/editor/build.rs
fn main() {
    // Check System provides all features App needs
    // Check System provides all features Plugins need
    // Compile error if mismatch
}
```

### Benefits
- **Zero runtime checks** - All validation at compile time
- **Direct trait calls** - ~1-5ns overhead (vtable dispatch only)
- **Type safety** - Rust compiler enforces signatures
- **Clear errors** - Know exactly what's missing

## Hot-Reload Process (With State Preservation)

1.  **Detect Change**: A file watcher (or other trigger) detects a change in a module's source code.
2.  **Save State**: The module loader checks if the current (old) `ViewModel` implements the `StatefulModule` trait. If so, it calls `save_state()` and stores the resulting state bytes in memory.
3.  **Unload Old Module**: The old module (`.so`/`.dll`) is unloaded.
4.  **Compile Module**: The build system compiles the new version of the module.
5.  **Load New Module**: The loader loads the newly compiled `.so`/`.dll` file.
6.  **Extract Symbols**: The loader extracts the `PLAYGROUND_VIEWMODEL` symbol to get the new `ViewModel` trait object.
7.  **Update Registry**: The `BindingRegistry` is updated to point the `ViewId` to the new `ViewModel` implementation.
8.  **Restore State**: The loader checks if the new `ViewModel` implements `StatefulModule`. If so, it calls `restore_state()` with the bytes saved in step 2.
9.  **Resume**: The engine continues, with all new calls to the `View` now being directed to the new, state-restored `ViewModel`.

## Symbol Extraction (THE Unsafe Block)

```rust
unsafe {
    let library = Library::new(&module_path)?;

    // Core modules
    if module_type == ModuleType::Core {
        let view: Symbol<&'static Handle<dyn ViewTrait>> =
            library.get(b"PLAYGROUND_VIEW\0")?;

        let models: Symbol<*const &'static [ModelTypeInfo]> =
            library.get(b"PLAYGROUND_MODELS\0")?;

        registry.register_view((*view).clone());

        for model_info in **models {
            registry.register_pool(view_id, model_info.model_type, ModelPool::new());
        }
    }

    // System modules
    if module_type == ModuleType::System {
        let viewmodel: Symbol<&'static Handle<dyn ViewModelTrait>> =
            library.get(b"PLAYGROUND_VIEWMODEL\0")?;

        registry.bind_viewmodel((*viewmodel).clone())?;
    }
}
```

## No Serialization, No VTable Indirection

### Old Way (Sessions 74-78)
```rust
// BAD: Serialization overhead
fn spawn_entity(args: &[u8]) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>>>> {
    let components: Vec<Component> = bincode::deserialize(args)?;
    // ... implementation
    Ok(bincode::serialize(&result)?)
}
```

### New Way (Session 79)
```rust
// GOOD: Direct trait method
#[async_trait::async_trait]
impl EntityView for EntityViewModel {
    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity> {
        // Direct parameters, no serialization
        // Direct return type, no serialization
    }
}
```

### Benefits
- **No serialization overhead** - Direct parameters and return types
- **Type safety** - Compiler checks all arguments
- **Clean API** - Natural Rust async functions
- **Minimal overhead** - Just vtable dispatch (~2-5ns)
