# playground-core-types

Shared type definitions and traits used across all Android Playground crates.

## Overview

The core types crate provides fundamental type definitions that are shared between server, client, and all other components. It features:
- Zero-dependency design for fast compilation
- Network protocol types (Priority, Packet, ChannelId)
- Plugin system types (PluginId, PluginMetadata, Version)
- Context and state management types
- Error handling definitions
- Event system foundations
- Thread-safe resource management

## Features

### 1. Networking Types

Core types for the binary packet protocol:

**Features:**
- Channel ID type definitions
- Priority levels for QoS
- Packet structure definitions
- Control message enumerations

**Code Example:**
```rust
use playground_core_types::{ChannelId, Priority, Packet, ControlMessageType};

// Channel IDs
let control_channel: ChannelId = 0;      // Control channel
let ui_channel: ChannelId = 10;          // System channel (1-999)
let plugin_channel: ChannelId = 1050;    // Plugin channel (1000+)

// Priority levels (ordered by importance)
let priorities = [
    Priority::Low,      // Background tasks (0)
    Priority::Medium,   // Normal operations (1)
    Priority::High,     // User interactions (2)
    Priority::Critical, // Important updates (3)
    Priority::Blocker,  // Must send immediately (4)
];

// Convert from u8
let priority = Priority::try_from(2u8)?; // Returns Priority::High

// Packet structure
let packet = Packet {
    channel_id: 10,
    packet_type: 100,
    priority: Priority::High as u8,
    payload_size: 5,
    payload: vec![1, 2, 3, 4, 5],
};

// Control message types
let control_types = [
    ControlMessageType::RegisterSystem,   // Register system channel
    ControlMessageType::RegisterPlugin,   // Register plugin channel
    ControlMessageType::QueryChannel,     // Query channel by name
    ControlMessageType::ListChannels,     // List all channels
    ControlMessageType::RegisterResponse, // Registration response
    ControlMessageType::QueryResponse,    // Query response
    ControlMessageType::ListResponse,     // List response
    ControlMessageType::Error,           // Error message (255)
];
```

### 2. Plugin System Types

Types for the plugin architecture:

**Features:**
- Plugin identification
- Metadata management
- Semantic versioning
- Display formatting

**Code Example:**
```rust
use playground_core_types::{PluginId, PluginMetadata, Version};

// Create plugin ID
let plugin_id = PluginId("inventory-system".to_string());
println!("Plugin: {}", plugin_id); // Display trait implemented

// Version management
let version = Version {
    major: 1,
    minor: 2,
    patch: 3,
};
println!("Version: {}", version); // Prints "1.2.3"

// Complete plugin metadata
let metadata = PluginMetadata {
    id: plugin_id,
    name: "Inventory System".to_string(),
    version,
};

// Version comparison
let v1 = Version { major: 1, minor: 0, patch: 0 };
let v2 = Version { major: 1, minor: 0, patch: 0 };
assert_eq!(v1, v2); // Versions are comparable

// Plugin ID is hashable for collections
use std::collections::HashMap;
let mut plugins = HashMap::new();
plugins.insert(metadata.id.clone(), metadata);
```

### 3. Context Management

Runtime context for resource and message sharing:

**Features:**
- Type-erased resource storage
- Message queue management
- Thread-safe resource access
- Dynamic type casting

**Code Example:**
```rust
use playground_core_types::{Context, Message};
use std::any::Any;

// Create context
let mut context = Context::new();

// Store resources (type-erased)
struct GameState {
    score: u32,
    level: u8,
}

let state = Box::new(GameState { score: 100, level: 5 });
context.resources.insert(
    "game_state".to_string(),
    state as Box<dyn Any + Send + Sync>
);

// Retrieve resources with downcast
if let Some(resource) = context.resources.get("game_state") {
    if let Some(state) = resource.downcast_ref::<GameState>() {
        println!("Score: {}, Level: {}", state.score, state.level);
    }
}

// Queue messages
context.messages.push(Message {
    sender: "player".to_string(),
    recipient: "ui".to_string(),
    payload: vec![1, 2, 3],
});

// Process messages
for message in context.messages.drain(..) {
    println!("Message from {} to {}", message.sender, message.recipient);
}
```

### 4. Error Handling

Unified error types for the plugin system:

**Features:**
- Plugin-specific error variants
- String-based error messages
- Result type aliases
- Error propagation support

**Code Example:**
```rust
use playground_core_types::PluginError;

// Create errors
let error = PluginError::InitializationFailed("Missing config".to_string());
let network_error = PluginError::NetworkError("Connection refused".to_string());
let update_error = PluginError::UpdateFailed("Invalid state".to_string());

// Use in functions
fn initialize_plugin() -> Result<(), PluginError> {
    // Some initialization logic
    if missing_config {
        return Err(PluginError::InitializationFailed(
            "Config file not found".to_string()
        ));
    }
    Ok(())
}

// Error propagation
fn setup() -> Result<(), PluginError> {
    initialize_plugin()?;
    connect_network()?;
    Ok(())
}
```

### 5. Event System

Foundation types for the event system:

**Features:**
- Event type definitions
- Event data payloads
- Event routing support
- Serializable events

**Code Example:**
```rust
use playground_core_types::Event;

// Create events
let event = Event {
    event_type: "player_moved".to_string(),
    source: "player_controller".to_string(),
    data: vec![/* position data */],
};

// Event handling pattern
match event.event_type.as_str() {
    "player_moved" => handle_movement(&event.data),
    "item_collected" => handle_collection(&event.data),
    "enemy_spawned" => handle_spawn(&event.data),
    _ => {} // Unknown event
}
```

### 6. Message System

Inter-component messaging:

**Features:**
- Sender/recipient identification
- Binary payload support
- Queue-based delivery
- Async-friendly design

**Code Example:**
```rust
use playground_core_types::Message;

// Create message
let message = Message {
    sender: "physics_system".to_string(),
    recipient: "render_system".to_string(),
    payload: bincode::serialize(&collision_data)?,
};

// Message routing
fn route_message(msg: Message, systems: &mut HashMap<String, System>) {
    if let Some(system) = systems.get_mut(&msg.recipient) {
        system.handle_message(msg);
    }
}

// Batch message processing
let mut message_queue = Vec::new();
message_queue.push(message);

// Process all messages in one frame
for msg in message_queue.drain(..) {
    route_message(msg, &mut systems);
}
```

### 7. Render Context

Rendering context for UI operations:

**Features:**
- Render state management
- Resource references
- Frame timing information
- GPU resource handles

**Code Example:**
```rust
use playground_core_types::RenderContext;

// Create render context
let render_ctx = RenderContext {
    frame_number: 1234,
    delta_time: 0.016, // 60 FPS
    viewport: (1920, 1080),
    resources: HashMap::new(),
};

// Use in rendering
fn render_frame(ctx: &RenderContext) {
    // Access frame timing
    let fps = 1.0 / ctx.delta_time;
    
    // Check viewport
    let (width, height) = ctx.viewport;
    
    // Access render resources
    if let Some(texture) = ctx.resources.get("main_texture") {
        // Use texture for rendering
    }
}
```

### 8. Stateful Trait

Trait for components with persistent state:

**Features:**
- State serialization interface
- State restoration support
- Version migration hooks
- Async state operations

**Code Example:**
```rust
use playground_core_types::Stateful;
use async_trait::async_trait;

struct PlayerInventory {
    items: Vec<Item>,
    capacity: usize,
}

#[async_trait]
impl Stateful for PlayerInventory {
    async fn save_state(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(&self.items)?)
    }
    
    async fn load_state(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.items = bincode::deserialize(data)?;
        Ok(())
    }
    
    fn state_version(&self) -> u32 {
        1
    }
    
    async fn migrate_state(&mut self, data: &[u8], from_version: u32) -> Result<(), Box<dyn std::error::Error>> {
        if from_version < 1 {
            // Migrate from version 0 to 1
            // Add default values for new fields
        }
        self.load_state(data).await
    }
}
```

## Complete Example

```rust
use playground_core_types::{
    PluginId, PluginMetadata, Version,
    Context, Message, Event,
    Priority, Packet, ChannelId,
    PluginError, Stateful
};

// Define a plugin
struct CombatPlugin {
    metadata: PluginMetadata,
    channel_id: ChannelId,
    context: Context,
}

impl CombatPlugin {
    fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: PluginId("combat-system".to_string()),
                name: "Combat System".to_string(),
                version: Version { major: 1, minor: 0, patch: 0 },
            },
            channel_id: 1100, // Plugin channel
            context: Context::new(),
        }
    }
    
    fn handle_packet(&mut self, packet: Packet) -> Result<(), PluginError> {
        match packet.packet_type {
            1 => self.handle_attack(packet.payload),
            2 => self.handle_defend(packet.payload),
            _ => Err(PluginError::UpdateFailed(
                format!("Unknown packet type: {}", packet.packet_type)
            )),
        }
    }
    
    fn send_damage_event(&mut self, target: String, damage: u32) {
        let event = Event {
            event_type: "damage_dealt".to_string(),
            source: self.metadata.id.to_string(),
            data: bincode::serialize(&(target, damage)).unwrap(),
        };
        
        let message = Message {
            sender: self.metadata.id.to_string(),
            recipient: "game_logic".to_string(),
            payload: bincode::serialize(&event).unwrap(),
        };
        
        self.context.messages.push(message);
    }
    
    fn handle_attack(&mut self, data: Vec<u8>) -> Result<(), PluginError> {
        // Parse attack data
        // Calculate damage
        // Send events
        Ok(())
    }
    
    fn handle_defend(&mut self, data: Vec<u8>) -> Result<(), PluginError> {
        // Parse defense data
        // Reduce damage
        // Update state
        Ok(())
    }
}

#[async_trait::async_trait]
impl Stateful for CombatPlugin {
    async fn save_state(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(&self.context.resources)?)
    }
    
    async fn load_state(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Load saved state
        Ok(())
    }
}
```

## Design Principles

### Zero Dependencies
This crate has no external dependencies (except serde for serialization) to ensure:
- Fast compilation times
- Maximum compatibility
- Minimal binary size
- No version conflicts

### Thread Safety
All types that need to be shared across threads implement:
- `Send` for transfer between threads
- `Sync` for shared references
- `Clone` where appropriate

### Serialization
Most types derive or implement serialization for:
- Network transmission
- State persistence
- Message passing
- Configuration files

## Performance Considerations

- **Copy Types**: Small types like `Priority` and `ChannelId` are `Copy`
- **String Interning**: Consider caching frequently used strings
- **Type Erasure**: `Context` uses `Any` for flexibility at runtime cost
- **Zero Allocation**: Many operations avoid heap allocation

## Architectural Rules

- This is a Core crate - uses NO Systems
- Cannot depend on other playground crates
- Must maintain backward compatibility
- All breaking changes require major version bump

## Common Patterns

### Plugin Registration
```rust
// Standard plugin registration flow
let metadata = PluginMetadata {
    id: PluginId("my-plugin".to_string()),
    name: "My Plugin".to_string(),
    version: Version { major: 1, minor: 0, patch: 0 },
};

// Register with system
register_plugin(metadata)?;
```

### Message Broadcasting
```rust
// Broadcast to all systems
fn broadcast(msg: Message, systems: &[String]) {
    for system in systems {
        let mut broadcast_msg = msg.clone();
        broadcast_msg.recipient = system.clone();
        send_message(broadcast_msg);
    }
}
```

### Priority Queue Implementation
```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Eq, PartialEq)]
struct PriorityPacket {
    priority: Priority,
    packet: Packet,
}

impl Ord for PriorityPacket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for PriorityPacket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

let mut queue = BinaryHeap::new();
queue.push(PriorityPacket { priority: Priority::High, packet });
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Blocker > Priority::Critical);
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }
    
    #[test]
    fn test_version_display() {
        let version = Version { major: 2, minor: 1, patch: 3 };
        assert_eq!(version.to_string(), "2.1.3");
    }
    
    #[test]
    fn test_plugin_id_equality() {
        let id1 = PluginId("test".to_string());
        let id2 = PluginId("test".to_string());
        assert_eq!(id1, id2);
    }
    
    #[test]
    fn test_priority_conversion() {
        assert_eq!(Priority::try_from(0u8).unwrap(), Priority::Low);
        assert_eq!(Priority::try_from(4u8).unwrap(), Priority::Blocker);
        assert!(Priority::try_from(5u8).is_err());
    }
}
```

## Migration Guide

### From Version 0.x to 1.0
```rust
// Old API
let priority = 2; // Raw u8

// New API  
let priority = Priority::High; // Type-safe enum

// Migration helper
fn migrate_priority(old: u8) -> Priority {
    Priority::try_from(old).unwrap_or(Priority::Medium)
}
```

## Dependencies

- `serde`: Serialization framework (with derive)
- `async-trait`: Async trait support (for Stateful)
- No other external dependencies

## See Also

- [core/server](../server/README.md) - Uses these types for networking
- [core/client](../client/README.md) - Uses these types for communication
- [core/plugin](../plugin/README.md) - Uses plugin types for architecture
- [core/message](../message/README.md) - Extended message handling
- [systems/logic](../../systems/logic/README.md) - Uses Context for game state