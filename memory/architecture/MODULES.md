# Module Architecture - Trait-Based Hot-Loading (Sessions 67-71, 79)

## Overview

The entire engine uses MVVM pattern with hot-loadable modules where Core provides Model+View (as traits), Systems provide ViewModel (trait implementations), and everything can reload at runtime.

## Implementation Status (Sessions 68-71, 79)

### modules/* Infrastructure ✅ COMPLETE (Session 79)
- **modules/types** - Trait-based MVVM (ModelTrait, ViewTrait, ViewModelTrait)
  - 64-bit IDs (ViewId, ModelId, ModelType)
  - async-trait for async trait methods
  - ModelTypeInfo for pool initialization
- **modules/loader** - THE single unsafe block ✅ COMPILES
- **modules/binding** - Triple-nested sharding with ModelPools ✅ COMPILES
- **modules/registry** - Runtime orchestration ✅ COMPILES
- **modules/resolver** - Cargo.toml parsing ✅ COMPILES

### core/ecs/model ✅ COMPLETE (Session 71)
- **entity/** - EntityId, Entity, EntityRef, Generation
- **component/** - ComponentId, Component, ComponentRef
- **event/** - EventId, Event, EventRef, Priority, Subscription, SubscriptionId
- **query/** - QueryId, Query, QueryRef, QueryFilter
- **storage/** - StorageId, Storage, StorageRef
- **system/** - SystemId, System, SystemRef
- **world/** - World, WorldRef (contains all storage)

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

## Trait-Based Architecture (Session 79)

### Core Traits
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

#[async_trait::async_trait]
pub trait ViewTrait: Send + Sync {
    fn view_id(&self) -> ViewId;
}

// modules/types/src/viewmodel/base.rs
#[async_trait::async_trait]
pub trait ViewModelTrait: Send + Sync {
    fn view_id(&self) -> ViewId;  // Which View this implements
}
```

### Core Module Example
```rust
// core/ecs/src/view/entity.rs
#[async_trait::async_trait]
pub trait EntityView: ViewTrait {
    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity>;
    async fn despawn_entity(&self, world: &World, entity: Entity) -> EcsResult<()>;
    // ... 11 methods total
}

// Export symbol
#[no_mangle]
pub static PLAYGROUND_VIEW: &dyn ViewTrait = &EcsView;

#[no_mangle]
pub static PLAYGROUND_MODELS: &[ModelTypeInfo] = &[
    ModelTypeInfo { model_type: 0x0001, type_name: "Entity" },
    ModelTypeInfo { model_type: 0x0002, type_name: "Component" },
];
```

### System Module Example
```rust
// systems/ecs/src/viewmodel/entity.rs
pub struct EntityViewModel;

impl ViewTrait for EntityViewModel {
    fn view_id(&self) -> ViewId { 0x1234567890ABCDEF }
}

#[async_trait::async_trait]
impl ViewModelTrait for EntityViewModel {
    fn view_id(&self) -> ViewId { 0x1234567890ABCDEF }
}

#[async_trait::async_trait]
impl EntityView for EntityViewModel {
    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity> {
        // Direct implementation - no serialization
        let entity_id = EntityId(world.next_entity_id.fetch_add(1));
        let generation = Generation(1);

        let mut entities = world.entities.write().await;
        entities.insert(entity_id, generation);

        Ok(Entity { id: entity_id, generation })
    }

    // ... implement all 11 methods
}

// Export symbol
#[no_mangle]
pub static PLAYGROUND_VIEWMODEL: &dyn ViewModelTrait = &EntityViewModel;
```

## Module Declaration in Cargo.toml

### App Declares Everything
```toml
# apps/editor/Cargo.toml
[[package.metadata.modules.core]]
name = "playground-core-rendering"
features = ["shaders", "textures", "multi-window"]
systems = [
    "playground-systems-vulkan",   # Primary choice
    "playground-systems-webgl"     # Fallback
]
```

### System Declares What It Provides
```toml
# systems/webgl/Cargo.toml
[package.metadata.provides]
core = "playground-core-rendering"
features = ["shaders", "textures", "2d", "basic-3d"]
```

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

## Binding Registry Architecture (Session 79)

### Triple-Nested Sharding
```rust
pub struct BindingRegistry {
    // Lock-free singleton access
    views: Handle<HashMap<ViewId, Handle<dyn ViewTrait>>>,
    viewmodels: Handle<HashMap<ViewId, Handle<dyn ViewModelTrait>>>,

    // Triple-nested: ViewId → ModelType → Pool
    models: Handle<HashMap<ViewId, Handle<HashMap<ModelType, ModelPool>>>>,
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
- **View/ViewModel lookup**: Lock-free (~5ns)
- **Model pool lookup**: Lock-free (~10ns)
- **Model access**: RwLock at pool level (~20-30ns)
- **Object recycling**: Reuse deleted models
- **Lock contention**: Only same ViewId + same ModelType contend

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

## Hot-Reload Process

1. **Detect Change** - File watcher sees .rs change
2. **Save State** - Module serializes current state
3. **Compile Module** - Incremental build (~500ms)
4. **Load New Version** - Using single unsafe
5. **Extract New Trait** - Get new ViewModel trait object
6. **Update Registry** - Replace old `Handle<dyn ViewModelTrait>` with new one
7. **Consumers Refresh** - Next `get_viewmodel()` call returns new implementation

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
