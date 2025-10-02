# Architecture - MVVM-Based Module System

## Core Architectural Pattern (Sessions 67-71)

### MVVM Architecture
```
Apps → Plugins → Core (Model+View) → [Module Binding] → Systems (ViewModel)
```

**Key Components:**
- **Model** = Data structures (core/*/model/)
- **View** = API contracts (core/*/view/)
- **ViewModel** = Implementation (systems/*/viewmodel/)
- **Binding** = Trait-based with generics (NO dyn, NO Box)

## Implementation (Sessions 68-71, 76)
- **modules/types** - MVVM base types (traits with generics, NO dyn)
- **modules/loader** - THE single unsafe block for Library::new() ✅ COMPILES
- **modules/binding** - Trait-based binding with concrete types ✅ COMPILES
- **modules/resolver** - Cargo.toml module declarations
- **modules/registry** - Runtime module orchestration
- **core/ecs/model** - Complete ECS Model layer with all data structures ✅ COMPILES
- **core/types** - Thread-safe wrappers (Handle, Shared, Atomic, Once) 🔄 Session 76

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
- **Core modules** = Model (data) + View (API contracts)
- **System modules** = ViewModel (implementation only)
- **Plugin modules** = High-level features (use Core APIs)
- **App modules** = Applications (use Plugin APIs, declare Systems)

### No Runtime Indirection
- Direct function calls via compile-time binding
- ~1-5ns overhead (no VTable, no serialization)
- Compile-time errors for missing implementations

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
    ├── spawn_entity.rs   # API contract (trait)
    └── query.rs          # API contract (trait)

systems/ecs/
└── viewmodel/
    ├── spawn_entity.rs   # Implementation (impl trait)
    └── query.rs          # Implementation (impl trait)
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

### Compile-Time Validation
- build.rs validates System provides required features
- Missing features = compile error
- Wrong System = compile error
- Plugin requirements checked against App declarations

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
- **Contains**: Structs with data fields, type definitions, thread-safe primitives
- **NO**: Implementation logic, business logic, I/O operations
- **Examples**: core/ecs, core/server, core/client, core/console

### Systems Layer
- **Purpose**: Implement all actual functionality
- **Contains**: ViewModel implementations, business logic, I/O operations
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

## World Instance Management (Session 76)

### OLD: Global Instances (Being Removed)
```rust
// DEPRECATED - prevents multiple worlds
static SERVER_INSTANCE: Lazy<Handle<Server>> = Lazy::new(|| Server::new());
```

### NEW: Parameter-Based World
```rust
// World passed as parameter through ViewModel
pub fn spawn_entity(world: &World, components: Vec<Component>) -> EntityId {
    // World is explicitly passed, not global
}

// Module stores World reference during initialization
pub struct ModuleState {
    world: Handle<World>,
}
```

## Component Storage Evolution (Session 76)

### Generation 1: Serialized Components (DEPRECATED)
```rust
// 100-500ns per access due to serialization
components: Shared<HashMap<EntityId, HashMap<ComponentId, Bytes>>>
```

### Generation 2: Generic Component Pools (CURRENT)
```rust
// 2-5ns per access with native storage
pub struct World {
    component_pools: HashMap<ComponentId, Box<dyn ComponentPoolTrait>>,
}

pub struct ComponentPool<T> {
    components: HashMap<EntityId, T>,
}
```

### ComponentId: 64-bit Deterministic (Session 76)
```rust
// OLD: 32-bit from type name (collision risk)
pub struct ComponentId(u32);

// NEW: 64-bit deterministic (collision-free)
pub struct ComponentId(u64);

impl ComponentId {
    pub fn from_module_and_name(module: &str, name: &str) -> Self {
        // Deterministic hash for networking/saves
        let hash = stable_hash_64(module, name);
        Self(hash)
    }
}
```

## System Isolation

### Strict Rules
- Systems can ONLY use core/* packages
- Systems CANNOT import other systems
- Cross-system communication through ECS only
- Each system implements specific core contracts

### Registration Pattern
```rust
// systems/ecs/src/registration.rs
pub async fn initialize(world: Handle<World>) -> CoreResult<()> {
    // Store world reference for module operations
    MODULE_STATE.set(ModuleState { world })?;

    // Register component pools
    register_component_pools().await?;

    Ok(())
}
```

## Architectural Invariants

1. **NO unsafe** - EXCEPTION: Single Library::new() in module loader only
2. **Traits allowed with generics** - Use `<T: Trait>` NOT `Box<dyn Trait>`
3. **NO Any** - Use serialization for type erasure when needed
4. **NO turbofish** - Use ComponentId not generics for components
5. **Model is pure data** - Only data fields with threading primitives, no logic
6. **NO global state** - World and instances passed as parameters (Session 76)
7. **Native storage** - Components stored as T, not Bytes (Session 76)
8. **Component-level locking** - Each component manages its concurrency (Session 76)

## Performance Targets (Session 76)

| Operation | Old (Serialized) | New (Native Pools) | Improvement |
|-----------|-----------------|-------------------|-------------|
| Component Read | 100-500ns | 2-5ns | **20-100x** |
| Component Write | 200-600ns | 5-10ns | **20-60x** |
| Memory Usage | 2x (ser+native) | 1x (native only) | **50% less** |
| Lock Contention | Global | Per-component | **N-fold** |