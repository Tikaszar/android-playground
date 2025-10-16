# Patterns - Code Patterns and Examples

## Thread-Safe Primitives (Session 76)

### Core Wrapper Types
```rust
use core::types::{Handle, Shared, Atomic, Once};

// Immutable reference (Arc<T>)
let config = Handle::new(Config { ... });
let cfg = config.get();  // &Config

// Mutable with RwLock (Arc<RwLock<T>>)
let inventory = Shared::new(vec![]);
let items = inventory.read().await;  // Read access
let mut items = inventory.write().await;  // Write access

// Lock-free for Copy types (Arc<AtomicCell<T>>)
let position = Atomic::new(5.0);
position.store(10.0);  // Lock-free write
let x = position.load();  // Lock-free read

// Initialize once (Arc<OnceCell<T>>)
let cache = Once::new();
cache.set(expensive_computation());  // Only first call succeeds
let value = cache.get();  // Always returns same value
```

### Component Threading Patterns
```rust
// TIER 1: Ultra-Hot Path (> 10,000 accesses/frame)
// Use field-level atomics (2-5ns per field)
pub struct Position {
    pub x: Atomic<f32>,
    pub y: Atomic<f32>,
    pub z: Atomic<f32>,
}

// TIER 2: Hot Path (1,000-10,000 accesses/frame)
// Use atomics for frequently changed fields
pub struct Health {
    pub current: Atomic<u32>,  // Changes often
    pub max: Atomic<u32>,      // Read with current
}

// TIER 3: Warm Path (100-1,000 accesses/frame)
// Use Shared for component-level locking (20ns)
pub struct CharacterStats {
    pub data: Shared<StatsData>,
}

// TIER 4: Cold Path (< 100 accesses/frame)
// Can use coarser locking
pub struct QuestLog {
    pub entries: Shared<Vec<Quest>>,
}

// TIER 5: Read-Heavy
// Use Handle for copy-on-write (2ns read)
pub struct MeshData {
    pub vertices: Handle<Vec<Vertex>>,
}
```

## Component Pool Pattern (Session 76)

### Generic Pool Implementation
```rust
pub struct ComponentPool<T> {
    components: HashMap<EntityId, T>,  // Native storage, no serialization
}

impl<T> ComponentPool<T> {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: EntityId, component: T) {
        self.components.insert(entity, component);
    }

    pub fn get(&self, entity: EntityId) -> Option<&T> {
        self.components.get(&entity)  // Zero-overhead access
    }

    pub fn get_mut(&mut self, entity: EntityId) -> Option<&mut T> {
        self.components.get_mut(&entity)
    }

    pub fn remove(&mut self, entity: EntityId) -> Option<T> {
        self.components.remove(&entity)
    }
}
```

### World with Component Pools
```rust
pub struct World {
    // OLD: Serialized components with global lock
    // components: Shared<HashMap<EntityId, HashMap<ComponentId, Bytes>>>,

    // NEW: Native component pools, no global lock
    component_pools: HashMap<ComponentId, Box<dyn Any>>,  // Type-erased pools

    // Or better: Use a trait without dyn
    component_pools: HashMap<ComponentId, ComponentPoolHandle>,
}

// Access pattern
let pool = world.get_pool::<Position>()?;  // Get specific pool
let pos = pool.get(entity_id)?;  // Direct access, no serialization
pos.x.store(10.0);  // Lock-free update
```

## ViewModel Implementation Pattern (Session 78 - NEW)

### Direct Signature Pattern (NO dyn, NO serialization)
```rust
use playground_modules_types::ModuleResult;
use playground_core_types::Handle;
use playground_core_ecs::World;

// GOOD: Direct function signature
pub async fn function_name(
    world: &Handle<World>,
    param1: Type1,
    param2: Type2
) -> ModuleResult<ReturnType> {
    // Direct implementation - no deserialization needed

    // Access component pools directly
    let positions = world.get_pool::<Position>()?;
    let pos = positions.get(entity_id)?;

    // Direct atomic operations
    pos.x.store(pos.x.load() + delta_x);
    pos.y.store(pos.y.load() + delta_y);

    // Return concrete type
    Ok(result)
}

// BAD: Old serialization pattern (Session 74-77) - DEPRECATED
pub fn old_function(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    // This violates NO dyn rule!
}
```

## Trait-Based MVVM Module Pattern (Sessions 79-80 - CURRENT)

### Core Module Exports (View + Models)
```rust
// core/ecs/src/lib.rs

use playground_modules_types::{ViewTrait, ViewId, ModelTypeInfo};

// Define a View struct that implements ViewTrait
pub struct EcsView;

impl ViewTrait for EcsView {
    fn view_id(&self) -> ViewId {
        // Unique ID for ECS View (could use hash of name)
        0x1234567890ABCDEF
    }
}

// Export View trait object for hot-loading
#[no_mangle]
pub static PLAYGROUND_VIEW: &dyn ViewTrait = &EcsView;

// Export Model type information for pool initialization
#[no_mangle]
pub static PLAYGROUND_MODELS: &[ModelTypeInfo] = &[
    ModelTypeInfo { model_type: 0x0001, type_name: "Entity" },
    ModelTypeInfo { model_type: 0x0002, type_name: "Component" },
    ModelTypeInfo { model_type: 0x0003, type_name: "Event" },
    ModelTypeInfo { model_type: 0x0004, type_name: "Query" },
    ModelTypeInfo { model_type: 0x0005, type_name: "Storage" },
    ModelTypeInfo { model_type: 0x0006, type_name: "System" },
    ModelTypeInfo { model_type: 0x0007, type_name: "World" },
];
```

### Core Module View Traits (Session 80: Fragment Pattern)
```rust
// core/ecs/src/view/entity.rs - Fragment trait definition

use async_trait::async_trait;
use playground_modules_types::{ViewFragmentTrait, ViewId, FragmentId};
use crate::model::*;
use crate::error::EcsResult;

pub const ENTITY_FRAGMENT_ID: FragmentId = 0x0001;

/// Entity operations View API Fragment
#[async_trait]
pub trait EntityView: ViewFragmentTrait {
    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity>;
    async fn despawn_entity(&self, world: &World, entity: Entity) -> EcsResult<()>;
    async fn exists(&self, world: &World, entity: Entity) -> EcsResult<bool>;
    async fn is_alive(&self, world: &World, entity: Entity) -> EcsResult<bool>;
    async fn clone_entity(&self, world: &World, entity: Entity) -> EcsResult<Entity>;
    // ... other methods
}

// core/ecs/src/view/mod.rs - Composite trait for compile-time enforcement
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

// core/ecs/src/lib.rs - Concrete struct implementing all fragments
pub struct EcsView;

impl ViewTrait for EcsView {
    fn view_id(&self) -> ViewId { ECS_VIEW_ID }
}

impl EntityView for EcsView {
    fn view_id(&self) -> ViewId { ECS_VIEW_ID }
    fn fragment_id(&self) -> FragmentId { ENTITY_FRAGMENT_ID }
    // ... stub implementations
}

impl EcsViewTrait for EcsView {}  // Compile-time enforcement
```

### System Module Exports (ViewModel)
```rust
// systems/ecs/src/lib.rs

use playground_modules_types::{ViewModelTrait, ViewTrait, ViewId};

// Single ViewModel struct that implements all View traits
pub struct EcsViewModel;

// Implement base ViewTrait
impl ViewTrait for EcsViewModel {
    fn view_id(&self) -> ViewId {
        0x1234567890ABCDEF  // Same as EcsView
    }
}

// Implement ViewModelTrait
#[async_trait]
impl ViewModelTrait for EcsViewModel {
    fn view_id(&self) -> ViewId {
        0x1234567890ABCDEF  // Same as EcsView
    }
}

// Export ViewModel trait object for hot-loading
#[no_mangle]
pub static PLAYGROUND_VIEWMODEL: &dyn ViewModelTrait = &EcsViewModel;
```

### System Module ViewModel Implementations (Session 80: Fragment Pattern)
```rust
// systems/ecs/src/lib.rs

use async_trait::async_trait;
use playground_core_ecs::{EcsViewTrait, EntityView, World, Entity, Component, EcsResult, EntityId, Generation};
use playground_modules_types::{ViewModelTrait, ViewId, FragmentId};

pub struct EcsViewModel;

impl ViewModelTrait for EcsViewModel {
    fn view_id(&self) -> ViewId { ECS_VIEW_ID }
}

// Implement the EntityView fragment from core/ecs
#[async_trait]
impl EntityView for EcsViewModel {
    fn view_id(&self) -> ViewId { ECS_VIEW_ID }
    fn fragment_id(&self) -> FragmentId { ENTITY_FRAGMENT_ID }

    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity> {
        // ACTUAL implementation - accesses BindingRegistry pools
        let entity_id = EntityId(world.next_entity_id.fetch_add(1));
        let generation = Generation(1);

        let mut entities = world.entities.write().await;
        entities.insert(entity_id, generation);

        // Add components to entity
        for component in components {
            let mut comps = world.components.write().await;
            comps.entry(entity_id)
                .or_insert_with(HashMap::new)
                .insert(component.id, component);
        }

        Ok(Entity { id: entity_id, generation })
    }

    async fn despawn_entity(&self, world: &World, entity: Entity) -> EcsResult<()> {
        let mut entities = world.entities.write().await;
        entities.remove(&entity.id);

        let mut components = world.components.write().await;
        components.remove(&entity.id);

        Ok(())
    }

    // ... implement all other methods
}

// Compile-time enforcement: won't compile unless ALL fragments implemented
impl EcsViewTrait for EcsViewModel {}
```

### Stateful Hot-Reload Pattern (Session 81 Design)

To preserve state across hot-reloads, a `ViewModel` can optionally implement the `StatefulModule` trait from `modules/types`.

```rust
// In modules/types/src/lib.rs
use serde::{Serialize, de::DeserializeOwned};

#[async_trait::async_trait]
pub trait StatefulModule {
    type State: Serialize + DeserializeOwned + Send;

    async fn save_state(&self) -> Result<Vec<u8>, ModuleError>;
    async fn restore_state(&self, state: Vec<u8>) -> Result<(), ModuleError>;
}
```

An example implementation in `systems/ecs`:

```rust
// In systems/ecs/src/lib.rs
use playground_modules_types::StatefulModule;
use serde::Serialize;

// A serializable representation of the world's state
#[derive(Serialize, Deserialize)]
pub struct WorldState { /* ... */ }

// The EcsViewModel from before...
pub struct EcsViewModel {
    world: Handle<World>,
}

#[async_trait::async_trait]
impl StatefulModule for EcsViewModel {
    type State = WorldState;

    async fn save_state(&self) -> Result<Vec<u8>, ModuleError> {
        let state = self.world.capture_state().await; // Gathers state into WorldState
        bincode::serialize(&state).map_err(|e| ModuleError::StateSaveFailed(e.to_string()))
    }

    async fn restore_state(&self, state_bytes: Vec<u8>) -> Result<(), ModuleError> {
        let state: Self::State = bincode::deserialize(&state_bytes)?;
        self.world.apply_state(state).await; // Applies the state
        Ok(())
    }
}
```

### Module Loader (THE Single Unsafe Block)
```rust
// modules/loader/src/loader.rs - THE ONLY UNSAFE BLOCK

unsafe {
    // 1. Load the dynamic library
    let library = Library::new(&module_path)?;

    // 2. Get module metadata
    let module_symbol: Symbol<*const Module> = library.get(b"PLAYGROUND_MODULE\0")?;

    // 3. For Core modules: Extract View trait object
    let view: Symbol<&'static Handle<dyn ViewTrait>> =
        library.get(b"PLAYGROUND_VIEW\0")?;

    // 4. For Core modules: Extract Model type info
    let models: Symbol<*const &'static [ModelTypeInfo]> =
        library.get(b"PLAYGROUND_MODELS\0")?;

    // 5. For System modules: Extract ViewModel trait object
    let viewmodel: Symbol<&'static Handle<dyn ViewModelTrait>> =
        library.get(b"PLAYGROUND_VIEWMODEL\0")?;

    // 6. Initialize module
    (module.lifecycle.initialize)(&[])?;
}
```

### ECS Facade Pattern (Session 81 Design)

The concurrent `BindingRegistry` enables `systems/ecs` to act as a pure facade. The `World` object becomes a stateless pass-through that holds a reference to the registry.

```rust
// In systems/ecs
pub struct World {
    registry: Handle<BindingRegistry>, // Handle is Arc
    // ... other stateless handles or config
}

impl World {
    pub fn new(registry: Handle<BindingRegistry>) -> Self {
        Self { registry }
    }

    // The World's public API methods become simple pass-through calls
    // to the underlying BindingRegistry.

    pub fn get_view(&self, view_id: ViewId) -> Option<Handle<dyn ViewTrait>> {
        self.registry.get_view(view_id)
    }

    pub async fn get_model(
        &self,
        view_id: ViewId,
        model_type: ModelType,
        model_id: ModelId,
    ) -> ModuleResult<Handle<dyn ModelTrait>> {
        self.registry.get_model(view_id, model_type, model_id).await
    }

    // Registration can also be passed through, as the registry supports
    // concurrent writes.
    pub fn register_view(&self, view: Handle<dyn ViewTrait>) {
        self.registry.register_view(view);
    }
}
```

### Consumer Pattern (Direct Trait Access)
```rust
// Get ViewModel once and store it. Note: this is a synchronous, lock-free call.
let entity_vm: Handle<dyn ViewModelTrait> = registry.get_viewmodel(view_id)
    .ok_or("ViewModel not found for view_id")?;

// Downcast to specific View trait (this is an accepted use of `Any`)
let entity_view = entity_vm.as_any().downcast_ref::<dyn EntityView>()
    .ok_or("ViewModel doesn't implement EntityView")?;

// Direct async trait method calls - NO registry lookup, NO serialization
let entity = entity_view.spawn_entity(&world, components).await?;
entity_view.despawn_entity(&world, entity).await?;
```

## Anti-Patterns to Avoid

### WRONG: Using global World
```rust
// ❌ NEVER DO THIS
static WORLD: Lazy<Handle<World>> = Lazy::new(World::new);

// ✅ DO THIS INSTEAD (Session 76)
pub struct ModuleState {
    world: Handle<World>,
}
// Pass World as parameter or store in module state
```

### WRONG: Serializing components
```rust
// ❌ NEVER DO THIS
let component_bytes = bincode::serialize(&position)?;
components.insert(entity_id, component_bytes);

// ✅ DO THIS INSTEAD (Session 76)
let pool = world.get_pool::<Position>()?;
pool.insert(entity_id, position);  // Store native type
```

### WRONG: Coarse locking
```rust
// ❌ NEVER DO THIS
pub struct World {
    everything: Shared<AllTheData>,  // Single lock for everything
}

// ✅ DO THIS INSTEAD (Session 76)
pub struct Position {
    x: Atomic<f32>,  // Each field can be accessed independently
    y: Atomic<f32>,
    z: Atomic<f32>,
}
```

### WRONG: Using DashMap
```rust
// ❌ NEVER DO THIS
use dashmap::DashMap;
let map = DashMap::new();

// ✅ DO THIS INSTEAD
use crate::types::Shared;
let map = Shared::new(HashMap::new());  // Explicit locking
```

## Performance Guidelines (Session 76)

### Access Pattern Optimization
```rust
// Sequential access - optimize for cache
for entity in entities.iter() {
    let pos = positions.get(entity);  // 2-5ns
    let vel = velocities.get(entity);  // 2-5ns
    // Process...
}

// Random access - minimize lock time
let positions = world.get_pool::<Position>();  // Get once
for entity in random_entities {
    if let Some(pos) = positions.get(entity) {  // Direct access
        pos.x.store(0.0);  // Lock-free
    }
}

// Batch operations - lock once
let mut inventory = inventory_data.write().await;  // Lock once
for item in items_to_add {
    inventory.push(item);  // Multiple operations
}
// Lock released
```

### Component Design by Access Frequency
| Frequency | Pattern | Example | Performance |
|-----------|---------|---------|-------------|
| > 10k/frame | Field atomics | Position.x | 2-5ns |
| 1k-10k/frame | Component atomics | Health.current | 3-5ns |
| 100-1k/frame | Shared component | Stats.data | 20ns |
| < 100/frame | Coarse lock | QuestLog | 20-50ns |
| Write-rare | Copy-on-write | MeshData | 2ns read |

## Compile-Time Safety Patterns (Session 76)

### Turn Runtime Bugs into Compile-Time Errors

#### Component Type Safety
```rust
// WRONG: Runtime type checking
pub fn get_component(&self, id: ComponentId) -> Result<Box<dyn Any>, Error> {
    self.components.get(&id)
        .ok_or(Error::NotFound)
        .and_then(|c| c.downcast().ok_or(Error::WrongType))  // Runtime failure!
}

// RIGHT: Compile-time type safety
pub fn get_component<T: Component>(&self) -> Option<&T> {
    self.pool::<T>().get(self.entity_id)  // Type known at compile time
}

// Usage:
let pos = world.get_component::<Position>(entity);  // Can't get wrong type
```

#### State Version Safety
```rust
// WRONG: Runtime version checking
fn load_state(data: &[u8]) -> Result<State, Error> {
    let state: State = deserialize(data)?;
    if state.version != CURRENT_VERSION {  // Runtime check!
        return Err(Error::VersionMismatch);
    }
    Ok(state)
}

// RIGHT: Compile-time version types
#[derive(Serialize, Deserialize)]
#[serde(version = 2)]  // Won't compile with V1 data
struct StateV2 {
    // ...
}

// Migration is explicit and compile-time checked
impl From<StateV1> for StateV2 {
    fn from(v1: StateV1) -> Self {
        // Explicit migration logic
    }
}
```

#### Module Interface Enforcement
```rust
// Every module MUST implement this or won't compile
trait RequiredInterface {
    fn init(&mut self) -> Result<(), Error>;
    fn update(&mut self, dt: f32) -> Result<(), Error>;
    fn save_state(&self) -> Result<Vec<u8>, Error>;
    fn restore_state(&mut self, data: &[u8]) -> Result<(), Error>;
}

// build.rs validates at compile time
const _: () = {
    fn assert_implements<T: RequiredInterface>() {}
    assert_implements::<MyModule>();  // Compile error if missing
};
```

#### Pool Access Safety
```rust
// WRONG: String-based pool lookup
pub fn get_pool(&self, name: &str) -> Option<&dyn Any> {
    self.pools.get(name)  // Could typo at runtime!
}

// RIGHT: Type-based pool access
pub fn get_pool<T: Component>(&self) -> &ComponentPool<T> {
    // Type parameter ensures correct pool
    &self.pools[TypeId::of::<T>()]
}

// Can't access wrong pool type at compile time
let positions = world.get_pool::<Position>();  // Always correct type
```

## Build Validation Pattern (Session 81 Design)

To guarantee dependencies at compile-time, `App` crates use a `build.rs` script to validate `System` capabilities against their own requirements, using metadata defined in `Cargo.toml`.

### `System` Crate: Declares Provisions

A `System`'s `Cargo.toml` declares which `Core` module it implements and the features it supports.

```toml
# In systems/webgl/Cargo.toml
[package.metadata.playground.provides]
core_module = "playground-core-rendering"
features = ["shaders", "textures", "2d"]
```

### `App` Crate: Declares Requirements

An `App`'s `Cargo.toml` declares its `Core` module dependencies, required features, and an ordered list of preferred `System` implementations.

```toml
# In apps/editor/Cargo.toml
[[package.metadata.playground.requires.core]]
name = "playground-core-rendering"
features = ["shaders", "textures"]
systems = ["playground-systems-vulkan", "playground-systems-webgl"]
```

### `App` Crate: `build.rs` Validation Logic

The `build.rs` script in the `App` crate performs the validation.

```rust
// In apps/editor/build.rs
fn main() {
    // 1. Parse own `requires` metadata from Cargo.toml.
    let required_cores = parse_app_requirements();

    // 2. For each requirement, parse the `provides` metadata from the
    //    corresponding System crates' Cargo.toml files.
    for core in required_cores {
        let mut found_compatible_system = false;
        for system_name in core.systems {
            let system_features = parse_system_provisions(&system_name);
            if system_features.is_superset_of(&core.features) {
                found_compatible_system = true;
                break; // Found a compatible system, move to next core requirement
            }
        }

        // 3. If no compatible system was found, panic to fail the build.
        if !found_compatible_system {
            panic!("Build failed: No compatible system found for core module '{}' with required features: {:?}", core.name, core.features);
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
}

```rust
// modules/loader/src/loader.rs - THE ONLY UNSAFE BLOCK

unsafe {
    // 1. Load the dynamic library
    let library = Library::new(&module_path)?;

    // 2. Get module metadata
    let module_symbol: Symbol<*const Module> = library.get(b"PLAYGROUND_MODULE\0")?;

    // 3. For Core modules: Extract View trait object
    let view: Symbol<&'static Handle<dyn ViewTrait>> =
        library.get(b"PLAYGROUND_VIEW\0")?;

    // 4. For Core modules: Extract Model type info
    let models: Symbol<*const &'static [ModelTypeInfo]> =
        library.get(b"PLAYGROUND_MODELS\0")?;

    // 5. For System modules: Extract ViewModel trait object
    let viewmodel: Symbol<&'static Handle<dyn ViewModelTrait>> =
        library.get(b"PLAYGROUND_VIEWMODEL\0")?;

    // 6. Initialize module
    (module.lifecycle.initialize)(&[])?;
}
```

### Consumer Pattern (Direct Trait Access)
```rust
// Get ViewModel once and store it. Note: this is a synchronous, lock-free call.
let entity_vm: Handle<dyn ViewModelTrait> = registry.get_viewmodel(view_id)
    .ok_or("ViewModel not found for view_id")?;

// Downcast to specific View trait (this is an accepted use of `Any`)
let entity_view = entity_vm.as_any().downcast_ref::<dyn EntityView>()
    .ok_or("ViewModel doesn't implement EntityView")?;

// Direct async trait method calls - NO registry lookup, NO serialization
let entity = entity_view.spawn_entity(&world, components).await?;
entity_view.despawn_entity(&world, entity).await?;
```

## ECS Facade Pattern (Session 81 Design)

The concurrent `BindingRegistry` enables `systems/ecs` to act as a pure facade. The `World` object becomes a stateless pass-through that holds a reference to the registry.

```rust
// In systems/ecs
pub struct World {
    registry: Handle<BindingRegistry>, // Handle is Arc
    // ... other stateless handles or config
}

impl World {
    pub fn new(registry: Handle<BindingRegistry>) -> Self {
        Self { registry }
    }

    // The World's public API methods become simple pass-through calls
    // to the underlying BindingRegistry.

    pub fn get_view(&self, view_id: ViewId) -> Option<Handle<dyn ViewTrait>> {
        self.registry.get_view(view_id)
    }

    pub async fn get_model(
        &self,
        view_id: ViewId,
        model_type: ModelType,
        model_id: ModelId,
    ) -> ModuleResult<Handle<dyn ModelTrait>> {
        self.registry.get_model(view_id, model_type, model_id).await
    }

    // Registration can also be passed through, as the registry supports
    // concurrent writes.
    pub fn register_view(&self, view: Handle<dyn ViewTrait>) {
        self.registry.register_view(view);
    }
}
```

## Anti-Patterns to Avoid

### WRONG: Using global World
```rust
// ❌ NEVER DO THIS
static WORLD: Lazy<Handle<World>> = Lazy::new(World::new);

// ✅ DO THIS INSTEAD (Session 76)
pub struct ModuleState {
    world: Handle<World>,
}
// Pass World as parameter or store in module state
```

### WRONG: Serializing components
```rust
// ❌ NEVER DO THIS
let component_bytes = bincode::serialize(&position)?;
components.insert(entity_id, component_bytes);

// ✅ DO THIS INSTEAD (Session 76)
let pool = world.get_pool::<Position>()?;
pool.insert(entity_id, position);  // Store native type
```

### WRONG: Coarse locking
```rust
// ❌ NEVER DO THIS
pub struct World {
    everything: Shared<AllTheData>,  // Single lock for everything
}

// ✅ DO THIS INSTEAD (Session 76)
pub struct Position {
    x: Atomic<f32>,  // Each field can be accessed independently
    y: Atomic<f32>,
    z: Atomic<f32>,
}
```

### WRONG: Using DashMap
```rust
// ❌ NEVER DO THIS
use dashmap::DashMap;
let map = DashMap::new();

// ✅ DO THIS INSTEAD
use crate::types::Shared;
let map = Shared::new(HashMap::new());  // Explicit locking
```

## Performance Guidelines (Session 76)

### Access Pattern Optimization
```rust
// Sequential access - optimize for cache
for entity in entities.iter() {
    let pos = positions.get(entity);  // 2-5ns
    let vel = velocities.get(entity);  // 2-5ns
    // Process...
}

// Random access - minimize lock time
let positions = world.get_pool::<Position>();  // Get once
for entity in random_entities {
    if let Some(pos) = positions.get(entity) {  // Direct access
        pos.x.store(0.0);  // Lock-free
    }
}

// Batch operations - lock once
let mut inventory = inventory_data.write().await;  // Lock once
for item in items_to_add {
    inventory.push(item);  // Multiple operations
}
// Lock released
```

### Component Design by Access Frequency
| Frequency | Pattern | Example | Performance |
|-----------|---------|---------|-------------|
| > 10k/frame | Field atomics | Position.x | 2-5ns |
| 1k-10k/frame | Component atomics | Health.current | 3-5ns |
| 100-1k/frame | Shared component | Stats.data | 20ns |
| < 100/frame | Coarse lock | QuestLog | 20-50ns |
| Write-rare | Copy-on-write | MeshData | 2ns read |

## Compile-Time Safety Patterns (Session 76)

### Turn Runtime Bugs into Compile-Time Errors

#### Component Type Safety
```rust
// WRONG: Runtime type checking
pub fn get_component(&self, id: ComponentId) -> Result<Box<dyn Any>, Error> {
    self.components.get(&id)
        .ok_or(Error::NotFound)
        .and_then(|c| c.downcast().ok_or(Error::WrongType))  // Runtime failure!
}

// RIGHT: Compile-time type safety
pub fn get_component<T: Component>(&self) -> Option<&T> {
    self.pool::<T>().get(self.entity_id)  // Type known at compile time
}

// Usage:
let pos = world.get_component::<Position>(entity);  // Can't get wrong type
```

#### State Version Safety
```rust
// WRONG: Runtime version checking
fn load_state(data: &[u8]) -> Result<State, Error> {
    let state: State = deserialize(data)?;
    if state.version != CURRENT_VERSION {  // Runtime check!
        return Err(Error::VersionMismatch);
    }
    Ok(state)
}

// RIGHT: Compile-time version types
#[derive(Serialize, Deserialize)]
#[serde(version = 2)]  // Won't compile with V1 data
struct StateV2 {
    // ...
}

// Migration is explicit and compile-time checked
impl From<StateV1> for StateV2 {
    fn from(v1: StateV1) -> Self {
        // Explicit migration logic
    }
}
```

#### Module Interface Enforcement
```rust
// Every module MUST implement this or won't compile
trait RequiredInterface {
    fn init(&mut self) -> Result<(), Error>;
    fn update(&mut self, dt: f32) -> Result<(), Error>;
    fn save_state(&self) -> Result<Vec<u8>, Error>;
    fn restore_state(&mut self, data: &[u8]) -> Result<(), Error>;
}

// build.rs validates at compile time
const _: () = {
    fn assert_implements<T: RequiredInterface>() {}
    assert_implements::<MyModule>();  // Compile error if missing
};
```

#### Pool Access Safety
```rust
// WRONG: String-based pool lookup
pub fn get_pool(&self, name: &str) -> Option<&dyn Any> {
    self.pools.get(name)  // Could typo at runtime!
}

// RIGHT: Type-based pool access
pub fn get_pool<T: Component>(&self) -> &ComponentPool<T> {
    // Type parameter ensures correct pool
    &self.pools[TypeId::of::<T>()]
}

// Can't access wrong pool type at compile time
let positions = world.get_pool::<Position>();  // Always correct type
```

## Build Validation Pattern (Session 76)

### Cargo.toml Module Declaration
```toml
# apps/game/Cargo.toml
[[package.metadata.modules.core]]
name = "playground-core-ecs"
features = ["queries", "events"]
systems = ["playground-systems-ecs"]
```

### build.rs Validation
```rust
// apps/game/build.rs
fn main() {
    // Parse Cargo.toml metadata
    let metadata = parse_cargo_metadata();

    // Validate each system provides required features
    for module in metadata.modules {
        for system in module.systems {
            validate_system_features(&system, &module.features)?;
        }
    }

    // Generate compile error if mismatch
    if !valid {
        panic!("System {} doesn't provide feature {}", system, feature);
    }
}
```