# playground-core-server

WebSocket multiplexer with channel management and integrated MCP (Model Context Protocol) server for universal LLM support.

## Overview

The core server is the central communication hub of the Android Playground engine. It provides:
- WebSocket multiplexing with binary protocol
- Channel-based message routing
- Frame-based packet batching (60fps)
- Priority queue system (5 levels)
- Integrated MCP server for AI agents
- Static file serving for browser clients
- Web-based development environment

## Features

### 1. Channel Management System

The `ChannelManager` provides dynamic channel registration and discovery:

**Features:**
- Channel ID allocation (1-999 for Systems, 1000+ for Plugins)
- Named channel registration for discovery
- Channel metadata storage
- Thread-safe operations with Shared<T> type alias

**Code Example:**
```rust
use playground_core_server::ChannelManager;

// Create channel manager
let manager = ChannelManager::new();

// Register a system channel (1-999)
let ui_channel = manager.register_channel(
    "ui-system".to_string(),
    10,  // Request specific ID
).await?;
assert_eq!(ui_channel, 10);

// Register a plugin channel (gets dynamic ID)
let plugin_channel = manager.register_channel(
    "my-plugin".to_string(),
    0,  // 0 means allocate dynamically
).await?;
assert!(plugin_channel >= 1000);

// Find channel by name
let channel_id = manager.get_channel_by_name("ui-system").await?;
assert_eq!(channel_id, Some(10));

// List all channels
let channels = manager.list_channels().await;
for (id, info) in channels {
    println!("Channel {}: {}", id, info.name);
}

// Unregister channel
manager.unregister_channel(plugin_channel).await?;
```

### 2. Binary Packet Protocol

The `Packet` struct defines the binary message format:

**Features:**
- Compact binary serialization
- Priority levels for QoS
- Channel-based routing
- Type safety with packet types

**Code Example:**
```rust
use playground_core_server::{Packet, Priority};
use bytes::{BytesMut, BufMut};

// Create a packet
let packet = Packet {
    channel_id: 10,
    packet_type: 100,  // Custom type
    priority: Priority::High,
    payload: b"Hello World".to_vec(),
};

// Serialize to bytes
let bytes = packet.to_bytes();

// Deserialize from bytes
let parsed = Packet::from_bytes(&bytes)?;
assert_eq!(parsed.channel_id, 10);

// Priority levels
let priorities = [
    Priority::Low,      // Background tasks
    Priority::Medium,   // Normal operations  
    Priority::High,     // User interactions
    Priority::Critical, // Important updates
    Priority::Blocker,  // Must send immediately
];

// Control packets for channel management
let control_packet = Packet::control_message(
    ControlMessageType::RegisterChannel,
    b"ui-system",
);
```

### 3. Frame-Based Batching

The `FrameBatcher` batches packets for efficient transmission:

**Features:**
- Configurable frame rate (default 60fps)
- Per-channel packet queues
- Priority-based packet ordering
- Automatic frame timing
- Memory-efficient circular buffers

**Code Example:**
```rust
use playground_core_server::{FrameBatcher, Packet, Priority};
use std::time::Duration;

// Create batcher for 60fps with 2000 channel capacity
let batcher = FrameBatcher::new(2000, 60);

// Queue packets (won't send immediately)
batcher.queue_packet(10, packet1, Priority::Low).await;
batcher.queue_packet(10, packet2, Priority::High).await;
batcher.queue_packet(20, packet3, Priority::Critical).await;

// Get frame duration for timing
let frame_duration = batcher.frame_duration();
assert_eq!(frame_duration, Duration::from_millis(16)); // ~60fps

// Manually flush a frame (normally automatic)
let packets = batcher.get_frame_packets().await;
// packets are ordered by priority within each channel

// Check if channel has pending packets
let has_packets = batcher.has_packets(10).await;

// Clear all packets for a channel
batcher.clear_channel(10).await;
```

### 4. WebSocket State Management

The `WebSocketState` manages all server-side WebSocket functionality:

**Features:**
- Connection tracking
- Integrated channel manager
- MCP tool registry
- Automatic batching coordination

**Code Example:**
```rust
use playground_core_server::{WebSocketState, McpTool};
use serde_json::json;

// Create WebSocket state
let state = WebSocketState::new();

// Register an MCP tool
let tool = McpTool {
    name: "show_file".to_string(),
    description: "Display file in editor".to_string(),
    input_schema: json!({
        "type": "object",
        "properties": {
            "path": {"type": "string"}
        }
    }),
    handler_channel: 1200,  // UI Framework Plugin channel
};
state.register_mcp_tool(tool).await;

// Get all MCP tools
let tools = state.get_mcp_tools().await;
for tool in tools {
    println!("Tool: {} -> channel {}", tool.name, tool.handler_channel);
}

// Unregister MCP tool
state.unregister_mcp_tool("show_file").await;

// Access channel manager
let channel_id = state.channel_manager
    .write().await
    .register_channel("test".to_string(), 0)
    .await?;

// Access batcher
let frame_duration = state.batcher.frame_duration();
```

### 5. WebSocket Handler

The main WebSocket connection handler:

**Features:**
- Automatic message routing
- Binary and text message support
- Connection lifecycle management
- Control message processing
- Concurrent send/receive tasks

**Code Example:**
```rust
use playground_core_server::{websocket_handler, WebSocketState};
use playground_core_types::{Handle, handle};
use axum::{Router, routing::get};

// Create shared state
let state = handle(WebSocketState::new());

// Create router with WebSocket endpoint
let app = Router::new()
    .route("/ws", get(websocket_handler))
    .with_state(state);

// The handler automatically:
// - Accepts WebSocket upgrades
// - Splits connection into send/receive tasks
// - Processes control messages
// - Routes packets to channels
// - Handles disconnections gracefully
```

### 6. MCP Server Integration

Complete Model Context Protocol implementation:

**Features:**
- SSE (Server-Sent Events) transport
- JSON-RPC 2.0 protocol
- Built-in test tools
- Dynamic tool registration
- Session management
- Tool forwarding to channels

**Code Example:**
```rust
use playground_core_server::mcp::{McpServer, MpcSession};

// MCP server is integrated into WebSocketState
// Tools are registered dynamically

// Built-in test tools
let test_tools = [
    "ping",          // Test connection
    "echo",          // Echo input
    "get_status",    // Server status
    "list_channels", // List WebSocket channels
];

// UI forwarding tools (to channel 1200)
let ui_tools = [
    "show_file",           // Display file
    "update_editor",       // Update editor
    "show_terminal_output", // Show terminal
];

// Session management
// Sessions are created automatically when LLMs connect
// Each session gets a channel ID (2000+)
```

### 7. Control Messages

Special packets for channel management:

**Features:**
- Channel registration/unregistration
- Channel discovery
- Plugin lifecycle events
- System status queries

**Code Example:**
```rust
use playground_core_server::{Packet, ControlMessageType};

// Register channel
let register = Packet::control_message(
    ControlMessageType::RegisterChannel,
    b"my-plugin",
);

// Unregister channel  
let unregister = Packet::control_message(
    ControlMessageType::UnregisterChannel,
    &channel_id.to_le_bytes(),
);

// Query channel by name
let query = Packet::control_message(
    ControlMessageType::QueryChannel,
    b"ui-system",
);

// List all channels
let list = Packet::control_message(
    ControlMessageType::ListChannels,
    b"",
);

// MCP tool registration (packet types 100/101)
let register_tool = Packet {
    channel_id: 0,  // Control channel
    packet_type: 100,  // RegisterMcpTool
    priority: Priority::High,
    payload: tool_json.as_bytes().to_vec(),
};
```

### 8. Static File Serving

Serves browser UI and assets:

**Features:**
- Multiple root directories
- Automatic index.html serving
- Path sanitization
- Cache headers
- Compression support

**Code Example:**
```rust
use axum::{Router, routing::get};
use tower_http::services::ServeDir;

// Serve playground-editor UI
let app = Router::new()
    .nest_service(
        "/playground-editor",
        ServeDir::new("apps/playground-editor/static")
            .append_index_html_on_directories(true)
    )
    .route("/", get(redirect_to_editor));

// Files are served from:
// /playground-editor/* -> apps/playground-editor/static/*
// / -> redirects to /playground-editor/
```

### 9. Handler Functions

HTTP endpoint handlers:

**Features:**
- Plugin listing
- Plugin hot-reload
- Health checks
- Root redirect

**Code Example:**
```rust
use playground_core_server::handlers::{list_plugins, reload_plugin, root};

let app = Router::new()
    .route("/", get(root))  // Redirects to editor
    .route("/api/plugins", get(list_plugins))
    .route("/api/plugins/:name/reload", post(reload_plugin));
```

## Complete Server Setup Example

```rust
use playground_core_server::{WebSocketState, websocket_handler};
use playground_core_types::{Handle, handle};
use axum::{Router, routing::get};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create shared state
    let state = handle(WebSocketState::new());
    
    // Pre-register system channels
    {
        let mut manager = state.channel_manager.write().await;
        manager.register_channel("ui-system".to_string(), 10).await;
        manager.register_channel("networking".to_string(), 20).await;
    }
    
    // Register MCP tools
    state.register_mcp_tool(McpTool {
        name: "custom_tool".to_string(),
        description: "My custom tool".to_string(),
        input_schema: json!({}),
        handler_channel: 1500,
    }).await;
    
    // Build complete application
    let app = Router::new()
        // WebSocket endpoint
        .route("/ws", get(websocket_handler))
        
        // MCP endpoints
        .route("/mcp", get(mcp_sse_handler).post(mcp_post_handler))
        
        // API endpoints
        .route("/api/plugins", get(list_plugins))
        
        // Static files
        .nest_service(
            "/playground-editor",
            ServeDir::new("apps/playground-editor/static")
        )
        
        // Root redirect
        .route("/", get(root))
        
        // Attach state
        .with_state(state);
    
    // Start server
    let addr = "0.0.0.0:8080".parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_channel_registration() {
        let manager = ChannelManager::new();
        
        // Test system channel
        let id = manager.register_channel("test".to_string(), 10).await.unwrap();
        assert_eq!(id, 10);
        
        // Test plugin channel
        let id = manager.register_channel("plugin".to_string(), 0).await.unwrap();
        assert!(id >= 1000);
    }
    
    #[test]
    fn test_packet_serialization() {
        let packet = Packet {
            channel_id: 10,
            packet_type: 100,
            priority: Priority::High,
            payload: vec![1, 2, 3],
        };
        
        let bytes = packet.to_bytes();
        let parsed = Packet::from_bytes(&bytes).unwrap();
        
        assert_eq!(parsed.channel_id, packet.channel_id);
        assert_eq!(parsed.payload, packet.payload);
    }
    
    #[tokio::test]
    async fn test_frame_batcher() {
        let batcher = FrameBatcher::new(100, 60);
        
        // Queue packets
        batcher.queue_packet(10, packet1, Priority::Low).await;
        batcher.queue_packet(10, packet2, Priority::High).await;
        
        // Get frame
        let packets = batcher.get_frame_packets().await;
        
        // High priority should come first
        assert_eq!(packets[0].priority, Priority::High);
        assert_eq!(packets[1].priority, Priority::Low);
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
use playground_core_server::*;
use tokio_tungstenite::connect_async;

#[tokio::test]
async fn test_websocket_connection() {
    // Start server in background
    let server = start_test_server().await;
    
    // Connect client
    let (ws_stream, _) = connect_async("ws://localhost:8080/ws")
        .await
        .unwrap();
    
    // Send control message
    let register = Packet::control_message(
        ControlMessageType::RegisterChannel,
        b"test-client",
    );
    ws_stream.send(Message::Binary(register.to_bytes())).await.unwrap();
    
    // Receive response
    let msg = ws_stream.next().await.unwrap().unwrap();
    // Verify response
}
```

## Configuration

### Environment Variables
```bash
PORT=8080              # Server port (default: 3000)
HOST=0.0.0.0          # Bind address (default: 0.0.0.0)
FRAME_RATE=60         # Batching fps (default: 60)
MAX_CONNECTIONS=1000  # Max WebSocket connections
MAX_CHANNELS=2000     # Max channel count
RUST_LOG=info         # Log level
```

### Runtime Configuration
```rust
// Configure at runtime
let mut batcher = FrameBatcher::new(2000, 120); // 120fps
batcher.set_frame_rate(30); // Change to 30fps

// Configure channel limits
let manager = ChannelManager::with_capacity(5000);
```

## Performance Optimizations

- **Packet Batching**: Reduces syscalls by 98% at 60fps
- **Binary Protocol**: 5-10x smaller than JSON
- **Channel Isolation**: O(1) routing with HashMap
- **Memory Pooling**: Reuses packet buffers
- **Lock-Free Reads**: RwLock allows concurrent reads
- **Lazy Serialization**: Only serialize when sending

## Architectural Rules

- This is a Core crate - uses NO Systems
- Can only depend on other Core crates
- Provides infrastructure for Systems to use
- MCP tools forward to channels, don't execute directly
- All operations must be thread-safe

## Common Issues and Solutions

### MCP Not Connecting
**Problem**: LLM can't connect to MCP endpoint
**Solution**:
```rust
// Ensure SSE content type
.header("Content-Type", "text/event-stream")
.header("Cache-Control", "no-cache")

// Send initial ready message
sender.send("event: endpoint-ready\ndata: {}\n\n").await?;
```

### Channel Registration Failed
**Problem**: Can't register channel
**Solution**:
```rust
// Check ID range
if requested_id > 0 && requested_id < 1000 {
    // System channel - must not exist
    if manager.channels.contains_key(&requested_id) {
        return Err("Channel already exists");
    }
} else {
    // Plugin channel - allocate dynamically
    requested_id = manager.next_plugin_id();
}
```

### Packets Not Sending
**Problem**: Packets queued but not received
**Solution**:
```rust
// Verify batching is running
tokio::spawn(async move {
    let mut interval = time::interval(Duration::from_millis(16));
    loop {
        interval.tick().await;
        let packets = batcher.get_frame_packets().await;
        // Send packets
    }
});
```

## Dependencies

- `axum`: Web framework and WebSocket support
- `tokio`: Async runtime
- `tokio-tungstenite`: WebSocket protocol
- `bytes`: Binary serialization
- `serde_json`: JSON for MCP and config
- `tower-http`: Static file serving
- `futures-util`: Stream utilities
- `tracing`: Structured logging

## See Also

- [core/client](../client/README.md) - Browser WASM client
- [core/types](../types/README.md) - Shared types (Priority, etc.)
- [systems/networking](../../systems/networking/README.md) - Higher-level networking
- [systems/logic](../../systems/logic/README.md) - Initializes all systems
- [MCP Specification](https://modelcontextprotocol.io/docs) - MCP protocol docs