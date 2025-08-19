# playground-core-client

WASM browser client with WebSocket connectivity, automatic reconnection, and channel management for the Android Playground engine.

## Overview

The core client is a lightweight WASM module that provides browser-side connectivity to the playground server. It features:
- WebSocket connection management with binary protocol
- Automatic reconnection with exponential backoff and jitter
- Channel registration and discovery (systems 1-999, plugins 1000+)
- Minimal memory footprint (wee_alloc optional)
- Full TypeScript/JavaScript bindings via wasm-bindgen
- Optimized binary size (431KB compiled)

## Features

### 1. WebSocket Client

The `WebSocketClient` provides robust connection management:

**Features:**
- Binary message protocol (ArrayBuffer)
- Automatic reconnection on disconnect
- Configurable retry strategies
- Connection state tracking
- Message queuing via mpsc channels
- Event callbacks for lifecycle

**Code Example:**
```rust
use playground_core_client::{WebSocketClient, ReconnectConfig, ReconnectCallbacks};

// Create client with default config
let mut client = WebSocketClient::new("ws://localhost:8080/ws")?;

// Or with custom reconnection config
let config = ReconnectConfig {
    initial_delay_ms: 500,    // Start with 500ms delay
    max_delay_ms: 30000,       // Cap at 30 seconds
    multiplier: 2.0,           // Double each retry
    max_attempts: Some(10),    // Give up after 10 tries
    jitter: true,              // Add random jitter
};
let mut client = WebSocketClient::with_config("ws://localhost:8080/ws", config)?;

// Set reconnection callbacks
let callbacks = ReconnectCallbacks {
    on_reconnecting: Some(Box::new(|attempt| {
        web_sys::console::log_1(&format!("Reconnection attempt {}", attempt).into());
    })),
    on_reconnected: Some(Box::new(|| {
        web_sys::console::log_1(&"Reconnected successfully!".into());
    })),
    on_reconnect_failed: Some(Box::new(|reason| {
        web_sys::console::error_1(&format!("Reconnection failed: {}", reason).into());
    })),
};
client.set_reconnect_callbacks(callbacks);

// Connect to server
client.connect().await?;

// Check connection status
if client.is_connected() {
    println!("Connected!");
}

// Send control messages
client.send_control_register_system("ui-system", 10).await?;
client.send_control_register_plugin("my-plugin").await?;

// Disable auto-reconnect
client.set_auto_reconnect(false);
```

### 2. Packet System

Binary packet protocol matching the server:

**Features:**
- Compact binary serialization using bytes crate
- Priority levels (Low/Medium/High/Critical/Blocker)
- Channel-based routing
- Type-safe packet construction
- Control message types for channel management

**Code Example:**
```rust
use playground_core_client::{Packet, Priority, ControlMessageType};
use bytes::Bytes;

// Create a packet
let packet = Packet::new(
    10,                           // Channel ID
    100,                          // Packet type
    Priority::High,               // Priority
    Bytes::from(b"Hello".to_vec()) // Payload
);

// Serialize to binary
let binary = packet.serialize();

// Deserialize from binary  
let received = Packet::deserialize(binary)?;
assert_eq!(received.channel_id, 10);

// Control message types
let control_types = [
    ControlMessageType::RegisterSystem,   // Register system channel (1-999)
    ControlMessageType::RegisterPlugin,   // Register plugin channel (1000+)
    ControlMessageType::QueryChannel,     // Query channel by name
    ControlMessageType::ListChannels,     // List all channels
    ControlMessageType::RegisterResponse, // Registration response
    ControlMessageType::QueryResponse,    // Query response
    ControlMessageType::ListResponse,     // List response
    ControlMessageType::Error,           // Error message
];

// Send packet through client
client.send_packet(packet).await?;
```

### 3. Channel Management

The `ChannelManager` tracks channel registrations:

**Features:**
- System channel registration (1-999)
- Plugin channel registration (1000+)
- Name-based channel lookup
- ID-based channel lookup
- Control channel (0) pre-registered

**Code Example:**
```rust
use playground_core_client::ChannelManager;

let mut manager = ChannelManager::new();

// Register a system channel (1-999)
let ui_id = manager.register_system(
    "ui-system".to_string(),
    10  // Specific ID for systems
)?;
assert_eq!(ui_id, 10);

// Register a plugin channel (gets assigned by server)
let plugin_id = manager.register_plugin(
    "my-plugin".to_string(),
    1050  // Server will assign actual ID
)?;

// Look up by name
let channel = manager.get_channel_by_name("ui-system");
assert_eq!(channel.unwrap().id, 10);
assert_eq!(channel.unwrap().owner, "system");

// Look up by ID
let channel = manager.get_channel_by_id(10);
assert_eq!(channel.unwrap().name, "ui-system");

// Control channel is pre-registered
let control = manager.get_channel_by_id(0);
assert_eq!(control.unwrap().name, "control");
```

### 4. Reconnection Manager

Smart reconnection with exponential backoff:

**Features:**
- Exponential backoff with configurable multiplier
- Optional jitter to prevent thundering herd
- Configurable retry limits
- State tracking (Connected/Disconnected/Reconnecting/Failed)
- Automatic delay calculation
- Event callbacks for UI updates

**Code Example:**
```rust
use playground_core_client::{ReconnectManager, ReconnectConfig, ReconnectState};

let config = ReconnectConfig {
    initial_delay_ms: 1000,
    max_delay_ms: 60000,
    multiplier: 1.5,
    max_attempts: None,  // Retry forever
    jitter: true,        // Add randomization
};

let mut manager = ReconnectManager::new(config);

// Check state
match manager.state() {
    ReconnectState::Connected => println!("Online"),
    ReconnectState::Disconnected => println!("Offline"),
    ReconnectState::Reconnecting { attempt, next_delay_ms } => {
        println!("Reconnecting... attempt {} in {}ms", attempt, next_delay_ms);
    }
    ReconnectState::Failed { reason } => {
        println!("Failed: {}", reason);
    }
}

// Trigger reconnection
manager.on_disconnected();
if manager.should_reconnect() {
    manager.wait_before_reconnect().await?;
    // Attempt connection...
    
    // On success
    manager.on_connected();
    
    // On failure
    manager.on_reconnect_failed("Connection refused".to_string());
}
```

### 5. Client and ClientBuilder

Main client interface with builder pattern:

**Features:**
- Fluent API for configuration
- WASM bindings for JavaScript
- Async/await support
- Type-safe channel registration
- Automatic memory management

**Code Example:**
```rust
use playground_core_client::{Client, ClientBuilder, Priority};
use wasm_bindgen::prelude::*;

// Build client with configuration
let client = ClientBuilder::new("ws://localhost:8080/ws".to_string())
    .with_reconnect_config(
        1000,   // initial_delay_ms
        60000,  // max_delay_ms
        1.5,    // multiplier
        Some(10), // max_attempts
        true    // jitter
    )
    .build()?;

// Or create directly
let mut client = Client::new("ws://localhost:8080/ws")?;

// Connect
client.connect().await?;

// Register system channel
let channel_id = client.register_system("ui-system".to_string(), 10).await?;

// Register plugin (server assigns ID)
let plugin_id = client.register_plugin("my-plugin".to_string()).await?;

// Send packet
client.send_packet(
    channel_id,
    100,  // packet_type
    Priority::High as u8,
    b"Hello".to_vec()
).await?;

// Configure auto-reconnect
client.set_auto_reconnect(false);
```

### 6. WASM Bindings

Full JavaScript/TypeScript API:

**Features:**
- Constructor pattern with builder
- Async/await support
- Type-safe enums via wasm-bindgen
- Automatic memory management
- Small binary size with wee_alloc

**JavaScript Example:**
```javascript
import init, { Client, ClientBuilder } from './playground_core_client.js';

// Initialize WASM module
await init();

// Create client with builder
const client = new ClientBuilder("ws://localhost:8080/ws")
    .with_reconnect_config(
        1000,   // initial_delay_ms
        60000,  // max_delay_ms
        1.5,    // multiplier
        10,     // max_attempts (null for unlimited)
        true    // jitter
    )
    .build();

// Connect
await client.connect();

// Register system
const channelId = await client.register_system("ui-system", 10);

// Send packet (priority: 0=Low, 1=Medium, 2=High, 3=Critical, 4=Blocker)
await client.send_packet(
    channelId,        // channel
    100,              // packet_type
    2,                // priority (High)
    new Uint8Array([1, 2, 3])  // payload
);

// Check connection
if (client.is_connected()) {
    console.log("Connected!");
}

// Configure auto-reconnect
client.set_auto_reconnect(false);
```

### 7. Control Messages

Special packets for channel management:

**Features:**
- System registration (channels 1-999)
- Plugin registration (channels 1000+)
- Channel queries
- Channel listing
- Error responses

**Code Example:**
```rust
use playground_core_client::{Packet, ControlMessageType, Priority};
use bytes::Bytes;

// Register system with specific channel ID
let register_system = Packet::new(
    0,  // Control channel
    ControlMessageType::RegisterSystem as u16,
    Priority::High,
    Bytes::from(format!("ui-system:10").into_bytes())
);

// Register plugin (server assigns ID)
let register_plugin = Packet::new(
    0,  // Control channel
    ControlMessageType::RegisterPlugin as u16,
    Priority::High,
    Bytes::from(b"my-plugin".to_vec())
);

// Query channel by name
let query = Packet::new(
    0,  // Control channel
    ControlMessageType::QueryChannel as u16,
    Priority::Medium,
    Bytes::from(b"ui-system".to_vec())
);

// List all channels
let list = Packet::new(
    0,  // Control channel
    ControlMessageType::ListChannels as u16,
    Priority::Low,
    Bytes::from(vec![])
);
```

## Complete Client Example

```rust
use playground_core_client::{
    Client, ClientBuilder, Packet, Priority, 
    ReconnectCallbacks, ControlMessageType
};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub async fn run_client() -> Result<(), JsValue> {
    // Initialize WASM
    console::log_1(&"Starting playground client...".into());
    
    // Build client with configuration
    let mut client = ClientBuilder::new("ws://localhost:8080/ws".to_string())
        .with_reconnect_config(1000, 60000, 1.5, None, true)
        .build()?;
    
    // Connect to server
    client.connect().await?;
    console::log_1(&"Connected to server".into());
    
    // Register as UI system
    let ui_channel = client.register_system("ui-system".to_string(), 10).await?;
    console::log_1(&format!("Registered UI system on channel {}", ui_channel).into());
    
    // Send a test packet
    client.send_packet(
        ui_channel,
        100,  // Custom packet type
        Priority::Medium as u8,
        b"Hello from WASM!".to_vec()
    ).await?;
    
    // Query available channels
    client.send_packet(
        0,  // Control channel
        ControlMessageType::ListChannels as u16,
        Priority::Low as u8,
        vec![]
    ).await?;
    
    Ok(())
}

#[wasm_bindgen]
pub struct GameClient {
    client: Client,
    player_channel: u16,
}

#[wasm_bindgen]
impl GameClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<GameClient, JsValue> {
        let client = ClientBuilder::new("ws://localhost:8080/ws".to_string())
            .with_reconnect_config(500, 30000, 2.0, Some(10), true)
            .build()?;
        
        Ok(GameClient {
            client,
            player_channel: 0,
        })
    }
    
    #[wasm_bindgen]
    pub async fn connect(&mut self) -> Result<(), JsValue> {
        self.client.connect().await?;
        
        // Register as game plugin
        self.player_channel = self.client.register_plugin(
            "game-client".to_string()
        ).await?;
        
        console::log_1(&format!("Game client channel: {}", self.player_channel).into());
        Ok(())
    }
    
    #[wasm_bindgen]
    pub async fn send_input(&self, action: &str) -> Result<(), JsValue> {
        self.client.send_packet(
            self.player_channel,
            1,  // Input packet type
            Priority::High as u8,
            action.as_bytes().to_vec()
        ).await
    }
}
```

## Building for WASM

### Prerequisites
```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-pack (optional, for npm packaging)
cargo install wasm-pack
```

### Build Commands
```bash
# Build WASM module
cargo build -p playground-core-client --target wasm32-unknown-unknown --release

# Or use wasm-pack for npm
wasm-pack build core/client --target web --out-dir pkg

# Optimize size with wee_alloc
cargo build -p playground-core-client \
    --target wasm32-unknown-unknown \
    --release \
    --features wee_alloc
```

### HTML Integration
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Playground Client</title>
</head>
<body>
    <script type="module">
        import init, { Client } from './playground_client.js';
        
        async function run() {
            // Initialize WASM
            await init();
            
            // Create and connect client
            const client = new Client("ws://localhost:8080/ws");
            await client.connect();
            
            // Use client
            console.log("Connected:", client.is_connected());
        }
        
        run();
    </script>
</body>
</html>
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_packet_serialization() {
        let packet = Packet::new(
            10,
            100,
            Priority::High,
            bytes::Bytes::from(vec![1, 2, 3])
        );
        
        let serialized = packet.serialize();
        let deserialized = Packet::deserialize(serialized).unwrap();
        
        assert_eq!(packet.channel_id, deserialized.channel_id);
        assert_eq!(packet.packet_type, deserialized.packet_type);
        assert_eq!(packet.priority as u8, deserialized.priority as u8);
        assert_eq!(packet.payload, deserialized.payload);
    }
    
    #[test]
    fn test_channel_registration() {
        let mut manager = ChannelManager::new();
        
        // System channel
        let id = manager.register_system("test".to_string(), 10).unwrap();
        assert_eq!(id, 10);
        
        // Duplicate should fail
        assert!(manager.register_system("test2".to_string(), 10).is_err());
        
        // Plugin channel
        let id = manager.register_plugin("plugin".to_string(), 1050).unwrap();
        assert_eq!(id, 1050);
    }
    
    #[test]
    fn test_exponential_backoff() {
        let config = ReconnectConfig {
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            multiplier: 2.0,
            max_attempts: None,
            jitter: false,
        };
        
        let mut manager = ReconnectManager::new(config);
        
        assert_eq!(manager.current_delay_ms, 1000);
        manager.current_delay_ms = (manager.current_delay_ms as f32 * 2.0) as u32;
        assert_eq!(manager.current_delay_ms, 2000);
        manager.current_delay_ms = (manager.current_delay_ms as f32 * 2.0) as u32;
        assert_eq!(manager.current_delay_ms, 4000);
    }
}
```

### Browser Testing
```bash
# Run WASM tests in browser
wasm-pack test --headless --chrome

# Or with Firefox
wasm-pack test --headless --firefox
```

## Configuration

### Compile-Time Features
```toml
[features]
default = ["console_error_panic_hook"]
wee_alloc = ["dep:wee_alloc"]  # Smaller allocator

# Enable in Cargo.toml:
playground-core-client = { 
    path = "../client", 
    features = ["wee_alloc"] 
}
```

### Runtime Configuration
```javascript
// Configure reconnection
const client = new ClientBuilder("ws://localhost:8080/ws")
    .with_reconnect_config(
        100,    // Very aggressive retry
        5000,   // Low max delay
        1.2,    // Slow growth
        20,     // Many attempts
        false   // No jitter
    )
    .build();

// Disable reconnection
client.set_auto_reconnect(false);
```

### Memory Optimization
```rust
// Use wee_alloc for smaller WASM size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

## Performance Optimizations

- **wee_alloc**: Reduces WASM size by ~100KB
- **Binary Protocol**: 5-10x smaller than JSON
- **Message Queuing**: Uses mpsc channels for efficient buffering
- **Lazy Deserialization**: Only parse needed fields
- **Zero-Copy Bytes**: Avoid unnecessary allocations
- **Compiled Size**: 431KB with all optimizations

## Architectural Rules

- This is a Core crate - uses NO Systems
- Can only depend on core/types
- Must compile to both WASM and native
- All async operations use wasm-bindgen-futures
- Memory safety with no unsafe code

## Common Issues and Solutions

### WASM Not Loading
**Problem**: Module fails to initialize
**Solution**:
```javascript
// Ensure proper MIME type
<script type="module">
// Correct path to WASM file
await init('./pkg/playground_core_client_bg.wasm');
</script>
```

### Connection Refused
**Problem**: Can't connect to server
**Solution**:
```rust
// Check server is running
// Verify URL protocol (ws:// vs wss://)
// Ensure CORS headers if cross-origin
```

### Reconnection Storm
**Problem**: Too many reconnection attempts
**Solution**:
```rust
// Increase delays
let config = ReconnectConfig {
    initial_delay_ms: 5000,  // Start slower
    multiplier: 2.0,         // Grow faster
    max_attempts: Some(5),   // Limit attempts
    ..Default::default()
};
```

## Dependencies

- `wasm-bindgen`: WASM bindings generator
- `web-sys`: Browser API bindings
- `js-sys`: JavaScript API bindings
- `bytes`: Efficient byte buffers
- `futures`: Async primitives
- `gloo-timers`: WASM-compatible timers
- `wee_alloc`: Small allocator (optional)
- `console_error_panic_hook`: Better panic messages
- `playground-core-types`: Shared types (Priority, etc.)

## See Also

- [core/server](../server/README.md) - Server-side WebSocket handling
- [core/types](../types/README.md) - Shared types (Priority, etc.)
- [core/message](../message/README.md) - Message protocol definitions
- [wasm-bindgen book](https://rustwasm.github.io/docs/wasm-bindgen/) - WASM development guide
- [WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket) - Browser WebSocket docs