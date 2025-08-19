# playground-systems-networking

WebSocket-based networking system with channel management, packet batching, and MCP tool registration.

## Overview

The Networking System manages all network communication between server and clients. It **internally starts and manages core/server**, providing a clean API for Plugins and Apps to send/receive data without knowing about the underlying WebSocket implementation.

### Key Features
- WebSocket multiplexing with binary protocol
- Channel-based routing (1-999 for Systems, 1000+ for Plugins)
- Packet batching at 60fps for efficiency
- Priority-based packet queuing
- MCP tool registration for LLM integration
- Connection state tracking via ECS
- Network statistics and monitoring
- Automatic reconnection handling

## Architecture

### Internal Server Management
**IMPORTANT**: NetworkingSystem starts core/server internally - Apps should NOT start it directly!

```rust
use playground_systems_networking::NetworkingSystem;

// NetworkingSystem handles server startup internally
let mut networking = NetworkingSystem::new().await?;
networking.initialize(None).await?; // Starts server on ws://localhost:8080/ws

// Apps should NEVER do this:
// ❌ let server = Server::new(); // WRONG - violates architecture
```

### Channel Allocation
```
0        Control channel (system messages)
1-999    System channels (reserved)
  10     UI System
  100    Networking System
  200    Rendering System
  300    Physics System
1000-1999 Plugin channels (dynamic)
  1000-1079 IDE plugins
  1100-1199 Game plugins  
  1200-1209 UI Framework Plugin
2000-2999 LLM sessions via MCP
```

### Packet Structure
```rust
pub struct Packet {
    channel_id: u16,      // Target channel
    packet_type: u16,     // Message type within channel
    priority: Priority,   // Critical/High/Normal/Low
    payload: Bytes,       // Binary data
}
```

## Usage

### Basic Setup
```rust
use playground_systems_networking::{NetworkingSystem, Priority};

// Create and initialize networking
let mut networking = NetworkingSystem::new().await?;
networking.initialize(None).await?; // Starts internal server

// Register a plugin for a channel
let channel_id = networking.register_plugin("my-plugin").await?;
println!("Plugin registered on channel {}", channel_id);
```

### Sending Packets
```rust
// Send with priority
networking.send_packet(
    channel_id,
    packet_type: 1, // Define your own packet types
    data: vec![1, 2, 3, 4],
    Priority::Normal,
).await?;

// Send reliable (with retries)
networking.send_reliable(
    channel_id,
    packet_type: 2,
    data: serde_json::to_vec(&message)?,
).await?;
```

### Receiving Packets
```rust
// Poll for incoming packets
let packets = networking.receive_packets(channel_id).await?;

for packet in packets {
    match packet.packet_type {
        1 => handle_type_1(packet.data),
        2 => handle_type_2(packet.data),
        _ => println!("Unknown packet type"),
    }
}
```

### MCP Tool Registration
Register tools that LLMs can call through MCP:

```rust
// Register a tool for code execution
networking.register_mcp_tool(
    "execute_code".to_string(),
    "Execute code in the playground".to_string(),
    serde_json::json!({
        "type": "object",
        "properties": {
            "code": {
                "type": "string",
                "description": "Code to execute"
            },
            "language": {
                "type": "string",
                "enum": ["rust", "javascript", "python"]
            }
        },
        "required": ["code", "language"]
    }),
    handler_channel: 1050, // Your plugin's channel
).await?;

// Unregister when done
networking.unregister_mcp_tool("execute_code").await?;
```

### Connection Management
Track peer connections using ECS components:

```rust
// Create a connection entity
let connection_id = networking.create_connection("peer_123").await?;

// Query all connections
let connections = networking.query_connections().await?;
for conn in connections {
    println!("Peer {} - Connected: {}, Latency: {}ms",
        conn.peer_id, conn.connected, conn.latency_ms);
}

// Get network statistics
let stats = networking.get_stats().await?;
println!("Active connections: {}", stats.connections_active);
println!("Average latency: {}ms", stats.average_latency_ms);
```

## Components (Internal)

The system uses core/ecs internally for state management:

### ConnectionComponent
```rust
pub struct ConnectionComponent {
    pub peer_id: String,
    pub connected: bool,
    pub latency_ms: u32,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub last_activity: u64,
}
```

### ChannelComponent
```rust
pub struct ChannelComponent {
    pub channel_id: u16,
    pub channel_name: String,
    pub is_system: bool, // true for 1-999
}
```

### NetworkStatsComponent
```rust
pub struct NetworkStatsComponent {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub average_latency_ms: u32,
}
```

## Packet Batching

Packets are batched at 60fps for efficiency:

```rust
// These packets will be sent in the same frame
networking.send_packet(ch, 1, data1, Priority::Normal).await?;
networking.send_packet(ch, 2, data2, Priority::Normal).await?;
networking.send_packet(ch, 3, data3, Priority::Normal).await?;
// All sent together at next frame boundary
```

Priority determines order within a batch:
1. **Critical**: Sent immediately (auth, errors)
2. **High**: First in batch (control messages)
3. **Normal**: Standard game data
4. **Low**: Background updates

## Binary Protocol

The system uses a binary protocol for efficiency:

```rust
// Packet format (12 byte header + payload)
// [channel_id: u16][packet_type: u16][priority: u8][reserved: 3 bytes][length: u32][payload: bytes]
```

### Serialization
```rust
// Use bincode for structured data
#[derive(Serialize, Deserialize)]
struct PlayerUpdate {
    position: (f32, f32, f32),
    rotation: f32,
    animation_id: u16,
}

let update = PlayerUpdate { /* ... */ };
let data = bincode::serialize(&update)?;
networking.send_packet(channel, 10, data, Priority::Normal).await?;
```

## Error Handling

All operations return `NetworkResult<T>`:

```rust
use playground_systems_networking::{NetworkError, NetworkResult};

match networking.send_packet(channel, type, data, priority).await {
    Ok(()) => println!("Sent successfully"),
    Err(NetworkError::NotConnected) => {
        // Handle disconnection
    },
    Err(NetworkError::ChannelNotFound(ch)) => {
        println!("Channel {} not registered", ch);
    },
    Err(e) => eprintln!("Network error: {}", e),
}
```

## WebSocket Integration

The system wraps core/server's WebSocket implementation:

```rust
// Internal: NetworkingSystem creates WebSocketClient
// which connects to core/server's WebSocket endpoint
// Plugins/Apps never interact with this directly

// The flow:
// Plugin → NetworkingSystem → WebSocketClient → core/server → Browser
```

## Thread Safety

All operations are thread-safe using `Arc<RwLock<>>`:

```rust
// NetworkingSystem can be shared across threads
let networking = Arc::new(networking);

// Clone for another thread
let net_clone = networking.clone();
tokio::spawn(async move {
    net_clone.send_packet(/* ... */).await;
});
```

## Performance

- **Batching**: Reduces syscalls by 60x
- **Binary protocol**: 5-10x smaller than JSON
- **Channel routing**: O(1) lookup
- **Priority queues**: Important messages first
- **Connection pooling**: Reuses WebSocket connections
- **Async everything**: Non-blocking I/O

## Testing

```rust
#[tokio::test]
async fn test_networking_system() {
    // Create networking system
    let mut networking = NetworkingSystem::new().await.unwrap();
    
    // Initialize (starts server internally)
    networking.initialize(None).await.unwrap();
    
    // Register a test plugin
    let channel = networking.register_plugin("test").await.unwrap();
    assert!(channel >= 1000);
    
    // Send and receive
    let data = vec![1, 2, 3];
    networking.send_packet(channel, 1, data.clone(), Priority::Normal).await.unwrap();
    
    // Allow batching
    tokio::time::sleep(Duration::from_millis(20)).await;
    
    let packets = networking.receive_packets(channel).await.unwrap();
    assert!(!packets.is_empty());
}
```

## Common Patterns

### Request-Response
```rust
// Send request
let request_id = generate_id();
let request = Request { id: request_id, /* ... */ };
networking.send_packet(channel, REQ_TYPE, serialize(&request), Priority::High).await?;

// Wait for response
loop {
    let packets = networking.receive_packets(channel).await?;
    for packet in packets {
        if packet.packet_type == RESP_TYPE {
            let response: Response = deserialize(&packet.data)?;
            if response.request_id == request_id {
                return Ok(response);
            }
        }
    }
    tokio::time::sleep(Duration::from_millis(10)).await;
}
```

### Broadcast to All
```rust
// Use channel 0 (control) with broadcast packet type
let broadcast = BroadcastMessage { /* ... */ };
networking.send_packet(0, BROADCAST_TYPE, serialize(&broadcast), Priority::High).await?;
```

### Connection Monitoring
```rust
// Periodic connection health check
tokio::spawn(async move {
    loop {
        let stats = networking.get_stats().await.unwrap();
        if stats.average_latency_ms > 100 {
            warn!("High latency detected: {}ms", stats.average_latency_ms);
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
});
```

## Architecture Rules

- **Manages core/server internally** - Apps must NOT start server
- Uses core/ecs for internal state management
- Thread-safe with Arc<RwLock<>> throughout
- All operations are async
- Batch operations preferred
- NO unsafe code
- Result<T, NetworkError> for all fallible operations

## Dependencies

- `playground-core-ecs`: Internal state management
- `playground-core-types`: Shared types (ChannelId, Priority)
- `playground-core-server`: WebSocket server (managed internally)
- `tokio`: Async runtime
- `bytes`: Efficient byte handling
- `serde`/`bincode`: Serialization
- `async-trait`: Async trait implementations

## See Also

- [core/server](../../core/server/README.md) - WebSocket server (internal)
- [systems/logic](../logic/README.md) - Creates and manages this system
- [plugins/ui-framework](../../plugins/ui-framework/README.md) - Example channel usage