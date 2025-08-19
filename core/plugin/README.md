# playground-core-plugin

Dynamic plugin system with hot-reload support for the Android Playground engine.

## Overview

The core plugin crate provides the infrastructure for dynamic plugin loading and management. It features:
- Dynamic loading from shared libraries (.so files)
- Hot-reload capability with state preservation
- Async plugin lifecycle management
- Safe FFI boundaries with proper error handling
- Plugin discovery and enumeration
- Memory-safe plugin unloading
- Inter-plugin communication via Context

## Features

### 1. Plugin Trait

The core plugin interface with async lifecycle methods:

**Features:**
- Async initialization and cleanup
- Frame update and rendering hooks
- Event handling with consumption
- Metadata access
- Context-based communication

**Code Example:**
```rust
use playground_core_plugin::Plugin;
use playground_core_types::{
    PluginMetadata, PluginId, Version,
    Context, Event, RenderContext, PluginError
};
use async_trait::async_trait;

struct InventoryPlugin {
    metadata: PluginMetadata,
    items: Vec<Item>,
    ui_visible: bool,
}

#[async_trait]
impl Plugin for InventoryPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn on_load(&mut self, ctx: &mut Context) -> Result<(), PluginError> {
        // Initialize plugin resources
        self.load_item_database().await?;
        
        // Register with other systems
        ctx.resources.insert(
            "inventory_api".to_string(),
            Box::new(self.create_api())
        );
        
        println!("Inventory plugin loaded");
        Ok(())
    }
    
    async fn on_unload(&mut self, ctx: &mut Context) {
        // Clean up resources
        self.save_inventory().await;
        ctx.resources.remove("inventory_api");
        println!("Inventory plugin unloaded");
    }
    
    async fn update(&mut self, ctx: &mut Context, delta_time: f32) {
        // Update inventory logic
        self.process_item_decay(delta_time);
        
        // Check for messages
        for message in ctx.messages.drain(..) {
            if message.recipient == self.metadata.id.to_string() {
                self.handle_message(message);
            }
        }
    }
    
    async fn render(&mut self, ctx: &mut RenderContext) {
        if self.ui_visible {
            // Render inventory UI
            self.render_inventory_grid(ctx);
            self.render_item_details(ctx);
        }
    }
    
    async fn on_event(&mut self, event: &Event) -> bool {
        match event.event_type.as_str() {
            "toggle_inventory" => {
                self.ui_visible = !self.ui_visible;
                true // Event consumed
            }
            "item_dropped" => {
                self.handle_item_drop(event);
                true // Event consumed
            }
            _ => false // Event not handled
        }
    }
}
```

### 2. Plugin Loader

Dynamic plugin loading and management:

**Features:**
- Load plugins from .so files
- Track loaded plugins by ID
- Safe plugin unloading
- Plugin enumeration
- Error handling for missing symbols

**Code Example:**
```rust
use playground_core_plugin::PluginLoader;
use playground_core_types::PluginId;
use std::path::Path;

// Create loader
let mut loader = PluginLoader::new();

// Load plugin from file
let plugin_id = loader.load_plugin(Path::new("plugins/inventory.so"))?;
println!("Loaded plugin: {}", plugin_id);

// Access loaded plugin
if let Some(plugin) = loader.get_plugin(&plugin_id) {
    println!("Plugin version: {}", plugin.metadata().version);
}

// Mutably access plugin
if let Some(plugin) = loader.get_plugin_mut(&plugin_id) {
    plugin.update(&mut context, 0.016).await;
}

// List all loaded plugins
let plugins = loader.list_plugins();
for id in plugins {
    println!("Loaded: {}", id);
}

// Unload plugin
loader.unload_plugin(&plugin_id)?;
println!("Plugin unloaded");
```

### 3. Plugin Entry Point

Required export for dynamic loading:

**Features:**
- C ABI compatibility
- No name mangling
- Raw pointer for FFI
- Type erasure via trait object

**Code Example:**
```rust
use playground_core_plugin::{Plugin, CreatePluginFn};

// Required export function
#[no_mangle]
pub unsafe extern "C" fn create_plugin() -> *mut dyn Plugin {
    let plugin = Box::new(MyPlugin::new());
    Box::into_raw(plugin)
}

// Alternative with error handling
#[no_mangle]
pub unsafe extern "C" fn create_plugin_safe() -> *mut dyn Plugin {
    match MyPlugin::try_new() {
        Ok(plugin) => Box::into_raw(Box::new(plugin)),
        Err(e) => {
            eprintln!("Failed to create plugin: {}", e);
            std::ptr::null_mut()
        }
    }
}
```

### 4. State Preservation

Save and restore plugin state across reloads:

**Features:**
- Serializable state types
- Version migration support
- Async save/load operations
- Hot-reload support

**Code Example:**
```rust
use playground_core_plugin::Stateful;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct InventoryState {
    items: Vec<Item>,
    gold: u32,
    capacity: usize,
}

#[async_trait]
impl Stateful for InventoryPlugin {
    async fn save_state(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let state = InventoryState {
            items: self.items.clone(),
            gold: self.gold,
            capacity: self.capacity,
        };
        Ok(bincode::serialize(&state)?)
    }
    
    async fn load_state(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let state: InventoryState = bincode::deserialize(data)?;
        self.items = state.items;
        self.gold = state.gold;
        self.capacity = state.capacity;
        Ok(())
    }
    
    fn state_version(&self) -> u32 {
        2 // Current state version
    }
    
    async fn migrate_state(&mut self, data: &[u8], from_version: u32) -> Result<(), Box<dyn std::error::Error>> {
        match from_version {
            1 => {
                // Migrate from v1 to v2
                #[derive(Deserialize)]
                struct OldState {
                    items: Vec<OldItem>,
                }
                let old: OldState = bincode::deserialize(data)?;
                self.items = old.items.into_iter().map(Item::from_old).collect();
                self.gold = 0; // New field, default value
                self.capacity = 20; // New field, default value
                Ok(())
            }
            _ => self.load_state(data).await
        }
    }
}
```

### 5. Hot Reload Implementation

Complete hot-reload workflow:

**Features:**
- State preservation across reloads
- Graceful error handling
- Plugin versioning
- Dependency management

**Code Example:**
```rust
use playground_core_plugin::{PluginLoader, Plugin, Stateful};
use std::path::Path;

struct PluginManager {
    loader: PluginLoader,
    states: HashMap<PluginId, Vec<u8>>,
}

impl PluginManager {
    async fn hot_reload(&mut self, plugin_id: &PluginId, new_path: &Path) -> Result<(), PluginError> {
        // Save current state
        if let Some(plugin) = self.loader.get_plugin(plugin_id) {
            if let Ok(state) = plugin.save_state().await {
                self.states.insert(plugin_id.clone(), state);
            }
            
            // Notify plugin of upcoming unload
            let mut ctx = Context::new();
            plugin.on_unload(&mut ctx).await;
        }
        
        // Unload old plugin
        self.loader.unload_plugin(plugin_id)?;
        
        // Load new plugin version
        let new_id = self.loader.load_plugin(new_path)?;
        
        // Restore state
        if let Some(state) = self.states.get(&new_id) {
            if let Some(plugin) = self.loader.get_plugin_mut(&new_id) {
                if let Err(e) = plugin.load_state(state).await {
                    eprintln!("Failed to restore state: {}", e);
                    // Try migration
                    plugin.migrate_state(state, 1).await?;
                }
            }
        }
        
        // Initialize new plugin
        if let Some(plugin) = self.loader.get_plugin_mut(&new_id) {
            let mut ctx = Context::new();
            plugin.on_load(&mut ctx).await?;
        }
        
        Ok(())
    }
}
```

### 6. Plugin Communication

Inter-plugin messaging via Context:

**Features:**
- Message passing
- Shared resource access
- Event broadcasting
- Type-erased communication

**Code Example:**
```rust
use playground_core_types::{Context, Message, Event};

impl InventoryPlugin {
    fn send_item_update(&mut self, ctx: &mut Context, item: &Item) {
        // Send message to UI system
        let message = Message {
            sender: self.metadata.id.to_string(),
            recipient: "ui_system".to_string(),
            payload: bincode::serialize(&ItemUpdate {
                item_id: item.id,
                quantity: item.quantity,
            }).unwrap(),
        };
        ctx.messages.push(message);
    }
    
    fn broadcast_inventory_full(&mut self, ctx: &mut Context) {
        // Create event
        let event = Event {
            event_type: "inventory_full".to_string(),
            source: self.metadata.id.to_string(),
            data: vec![],
        };
        
        // Broadcast to all plugins
        let message = Message {
            sender: self.metadata.id.to_string(),
            recipient: "*".to_string(), // Broadcast
            payload: bincode::serialize(&event).unwrap(),
        };
        ctx.messages.push(message);
    }
    
    fn register_api(&mut self, ctx: &mut Context) {
        // Share API with other plugins
        let api = InventoryAPI {
            add_item: Box::new(move |item| self.add_item(item)),
            remove_item: Box::new(move |id| self.remove_item(id)),
            get_items: Box::new(move || self.items.clone()),
        };
        
        ctx.resources.insert(
            "inventory_api".to_string(),
            Box::new(api) as Box<dyn Any + Send + Sync>
        );
    }
}
```

### 7. Error Handling

Comprehensive error management:

**Features:**
- Plugin-specific error types
- FFI-safe error propagation
- Graceful failure recovery
- Detailed error messages

**Code Example:**
```rust
use playground_core_types::PluginError;

impl InventoryPlugin {
    async fn validate_and_load(&mut self) -> Result<(), PluginError> {
        // Validate configuration
        if self.capacity == 0 {
            return Err(PluginError::InitializationFailed(
                "Invalid capacity: must be greater than 0".to_string()
            ));
        }
        
        // Load database
        self.load_database().await.map_err(|e| {
            PluginError::InitializationFailed(
                format!("Failed to load item database: {}", e)
            )
        })?;
        
        // Connect to network
        self.connect_to_server().await.map_err(|e| {
            PluginError::NetworkError(
                format!("Failed to connect to inventory server: {}", e)
            )
        })?;
        
        Ok(())
    }
}

// Safe plugin creation with validation
#[no_mangle]
pub unsafe extern "C" fn create_plugin() -> *mut dyn Plugin {
    let mut plugin = InventoryPlugin::new();
    
    // Validate synchronously
    if let Err(e) = plugin.validate_config() {
        eprintln!("Plugin validation failed: {}", e);
        return std::ptr::null_mut();
    }
    
    Box::into_raw(Box::new(plugin))
}
```

## Complete Plugin Example

```rust
use playground_core_plugin::{Plugin, Stateful};
use playground_core_types::*;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

pub struct PhysicsPlugin {
    metadata: PluginMetadata,
    bodies: Vec<RigidBody>,
    gravity: f32,
    time_step: f32,
}

impl PhysicsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: PluginId("physics-engine".to_string()),
                name: "Physics Engine".to_string(),
                version: Version { major: 1, minor: 0, patch: 0 },
            },
            bodies: Vec::new(),
            gravity: -9.81,
            time_step: 1.0 / 60.0,
        }
    }
    
    fn simulate_step(&mut self, delta_time: f32) {
        let steps = (delta_time / self.time_step) as usize;
        for _ in 0..steps {
            self.integrate_forces();
            self.detect_collisions();
            self.resolve_collisions();
        }
    }
    
    fn integrate_forces(&mut self) {
        for body in &mut self.bodies {
            body.velocity.y += self.gravity * self.time_step;
            body.position += body.velocity * self.time_step;
        }
    }
    
    fn detect_collisions(&mut self) {
        // Collision detection logic
    }
    
    fn resolve_collisions(&mut self) {
        // Collision resolution logic
    }
}

#[async_trait]
impl Plugin for PhysicsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn on_load(&mut self, ctx: &mut Context) -> Result<(), PluginError> {
        println!("Physics engine v{} loaded", self.metadata.version);
        
        // Register physics API
        ctx.resources.insert(
            "physics_api".to_string(),
            Box::new(PhysicsAPI::new(self))
        );
        
        Ok(())
    }
    
    async fn on_unload(&mut self, ctx: &mut Context) {
        ctx.resources.remove("physics_api");
        println!("Physics engine unloaded");
    }
    
    async fn update(&mut self, ctx: &mut Context, delta_time: f32) {
        self.simulate_step(delta_time);
        
        // Send collision events
        for collision in self.get_collisions() {
            let event = Event {
                event_type: "collision".to_string(),
                source: self.metadata.id.to_string(),
                data: bincode::serialize(&collision).unwrap(),
            };
            
            ctx.messages.push(Message {
                sender: self.metadata.id.to_string(),
                recipient: "game_logic".to_string(),
                payload: bincode::serialize(&event).unwrap(),
            });
        }
    }
    
    async fn render(&mut self, ctx: &mut RenderContext) {
        // Debug rendering of physics bodies
        if ctx.debug_mode {
            for body in &self.bodies {
                self.render_body_outline(ctx, body);
            }
        }
    }
    
    async fn on_event(&mut self, event: &Event) -> bool {
        match event.event_type.as_str() {
            "spawn_body" => {
                let body: RigidBody = bincode::deserialize(&event.data).unwrap();
                self.bodies.push(body);
                true
            }
            "remove_body" => {
                let id: u32 = bincode::deserialize(&event.data).unwrap();
                self.bodies.retain(|b| b.id != id);
                true
            }
            "set_gravity" => {
                self.gravity = bincode::deserialize(&event.data).unwrap();
                true
            }
            _ => false
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PhysicsState {
    bodies: Vec<RigidBody>,
    gravity: f32,
}

#[async_trait]
impl Stateful for PhysicsPlugin {
    async fn save_state(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let state = PhysicsState {
            bodies: self.bodies.clone(),
            gravity: self.gravity,
        };
        Ok(bincode::serialize(&state)?)
    }
    
    async fn load_state(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let state: PhysicsState = bincode::deserialize(data)?;
        self.bodies = state.bodies;
        self.gravity = state.gravity;
        Ok(())
    }
}

// Required export
#[no_mangle]
pub unsafe extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(PhysicsPlugin::new()))
}
```

## Building Plugins

### Cargo.toml Configuration
```toml
[package]
name = "my-plugin"
version = "1.0.0"

[lib]
crate-type = ["cdylib"]  # Required for .so output

[dependencies]
playground-core-plugin = { path = "../core/plugin" }
playground-core-types = { path = "../core/types" }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
```

### Build Command
```bash
# Build plugin as shared library
cargo build --release

# Output will be at:
# target/release/libmy_plugin.so

# Rename for loading
mv target/release/libmy_plugin.so plugins/my_plugin.so
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let mut plugin = MyPlugin::new();
        let mut ctx = Context::new();
        
        // Test loading
        assert!(plugin.on_load(&mut ctx).await.is_ok());
        
        // Test update
        plugin.update(&mut ctx, 0.016).await;
        
        // Test event handling
        let event = Event {
            event_type: "test".to_string(),
            source: "test".to_string(),
            data: vec![],
        };
        assert!(!plugin.on_event(&event).await);
        
        // Test unload
        plugin.on_unload(&mut ctx).await;
    }
    
    #[tokio::test]
    async fn test_state_persistence() {
        let mut plugin = MyPlugin::new();
        plugin.set_value(42);
        
        // Save state
        let state = plugin.save_state().await.unwrap();
        
        // Create new plugin
        let mut new_plugin = MyPlugin::new();
        
        // Restore state
        new_plugin.load_state(&state).await.unwrap();
        assert_eq!(new_plugin.get_value(), 42);
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
use playground_core_plugin::PluginLoader;
use std::path::Path;

#[test]
fn test_plugin_loading() {
    let mut loader = PluginLoader::new();
    
    // Load test plugin
    let result = loader.load_plugin(Path::new("test_plugin.so"));
    assert!(result.is_ok());
    
    let plugin_id = result.unwrap();
    
    // Verify plugin loaded
    assert!(loader.get_plugin(&plugin_id).is_some());
    
    // Unload plugin
    assert!(loader.unload_plugin(&plugin_id).is_ok());
    
    // Verify plugin unloaded
    assert!(loader.get_plugin(&plugin_id).is_none());
}
```

## Safety Considerations

### FFI Safety
- All plugin functions use `unsafe extern "C"`
- Raw pointers are immediately converted to safe types
- No unwinding across FFI boundary
- Proper memory management with Box

### Thread Safety
- Plugins must be `Send + Sync`
- Shared state uses Arc<RwLock<>>
- No global mutable state
- Context provides safe communication

### Memory Safety
- Plugins own their memory
- Loader manages library lifetime
- No dangling pointers after unload
- State serialization avoids references

## Performance Optimizations

- **Async Operations**: Non-blocking plugin lifecycle
- **Batch Updates**: Process multiple plugins per frame
- **Event Filtering**: Early return for unhandled events
- **State Caching**: Avoid repeated serialization
- **Lazy Loading**: Load plugins on demand

## Architectural Rules

- This is a Core crate - uses NO Systems
- Depends only on core/types
- Plugins cannot directly call other plugins
- All communication through Context
- Hot-reload must preserve state

## Common Patterns

### Plugin Registry
```rust
struct PluginRegistry {
    loader: PluginLoader,
    metadata: HashMap<PluginId, PluginMetadata>,
}

impl PluginRegistry {
    fn discover_plugins(&mut self, dir: &Path) -> Result<Vec<PluginId>, PluginError> {
        let mut discovered = Vec::new();
        
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension() == Some(OsStr::new("so")) {
                match self.loader.load_plugin(&path) {
                    Ok(id) => discovered.push(id),
                    Err(e) => eprintln!("Failed to load {}: {}", path.display(), e),
                }
            }
        }
        
        Ok(discovered)
    }
}
```

### Plugin Dependencies
```rust
impl Plugin for MyPlugin {
    async fn on_load(&mut self, ctx: &mut Context) -> Result<(), PluginError> {
        // Check for required dependencies
        if !ctx.resources.contains_key("physics_api") {
            return Err(PluginError::InitializationFailed(
                "Required dependency 'physics_api' not found".to_string()
            ));
        }
        
        // Continue initialization
        Ok(())
    }
}
```

## Migration Guide

### From Version 0.x to 1.0
```rust
// Old synchronous API
impl Plugin for MyPlugin {
    fn update(&mut self, ctx: &mut Context, dt: f32) {
        // Synchronous update
    }
}

// New async API
#[async_trait]
impl Plugin for MyPlugin {
    async fn update(&mut self, ctx: &mut Context, dt: f32) {
        // Async update
    }
}
```

## Dependencies

- `async-trait`: Async trait support
- `libloading`: Dynamic library loading
- `playground-core-types`: Core type definitions
- No other dependencies

## See Also

- [core/types](../types/README.md) - Plugin types and traits
- [plugins/](../../plugins/README.md) - Example plugins
- [systems/logic](../../systems/logic/README.md) - Plugin integration
- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html) - FFI safety