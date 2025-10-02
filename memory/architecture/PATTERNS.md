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

## ViewModel Implementation Pattern (Session 74-76)

### Updated with World Parameter
```rust
use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn function_name(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();  // CRITICAL: Copy args before async
    Box::pin(async move {
        // Deserialize arguments
        let args: ArgsStruct = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World from module state (NOT global)
        let world = crate::state::get_world()  // Will be updated to non-global
            .map_err(|e| ModuleError::Generic(e))?;

        // Access component pools directly
        let positions = world.get_pool::<Position>()?;
        let pos = positions.get(args.entity_id)?;

        // Direct atomic operations (no serialization!)
        pos.x.store(pos.x.load() + args.delta_x);
        pos.y.store(pos.y.load() + args.delta_y);

        // Return result
        Ok(vec![])  // Or serialize result if needed
    })
}
```

## Module Pattern (IMPLEMENTED Sessions 68-70 - Replaces VTable)

### Pure Rust Module Interface
```rust
// NO extern "C", NO repr(C) - Pure Rust!
#[no_mangle]
pub static PLAYGROUND_MODULE: Module = Module {
    metadata: &METADATA,
    vtable: &VTABLE,
};

static VTABLE: ModuleVTable = ModuleVTable {
    create: module_create,
    destroy: module_destroy,
    call: module_call,
    save_state: module_save_state,  // Session 76: Must implement
    restore_state: module_restore_state,  // Session 76: Must implement
};

// State management for hot-reload (Session 76)
fn module_save_state(state: *mut u8) -> Result<Vec<u8>, String> {
    // Serialize world and module state
    let module_state = unsafe { &*(state as *const ModuleState) };
    bincode::serialize(&module_state.world)
        .map_err(|e| e.to_string())
}

fn module_restore_state(state: *mut u8, data: &[u8]) -> Result<(), String> {
    // Deserialize and restore state
    let world: World = bincode::deserialize(data)
        .map_err(|e| e.to_string())?;
    let module_state = unsafe { &mut *(state as *mut ModuleState) };
    module_state.world = world;
    Ok(())
}
```

### Module Loader (Single Unsafe) - IMPLEMENTED
```rust
// THE ONLY UNSAFE BLOCK - in modules/loader/src/loader.rs
unsafe {
    // 1. Load the dynamic library
    let library = Library::new(&module_path)?;

    // 2. Get module symbol
    let module_symbol: Symbol<*const Module> = library.get(b"PLAYGROUND_MODULE\0")?;

    // 3. Get View/ViewModel symbols
    // 4. Initialize module
    // All in ONE unsafe block!
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