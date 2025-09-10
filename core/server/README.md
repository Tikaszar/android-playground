# playground-core-server

Server infrastructure contracts and types for the Android Playground engine. This package defines ONLY contracts (traits) and types - all implementations live in systems/networking.

## Overview

**IMPORTANT**: This is a contracts-only package following the stateless core design principle. It provides:
- Server infrastructure contracts (traits)
- Network protocol types (Packet, Priority, etc.)
- NO implementation code
- NO state management
- NO actual server functionality

The actual server implementation is in `systems/networking` which implements these contracts.

## Contracts (Traits)

### 1. ServerContract

The main server contract that implementations must fulfill:

```rust
use async_trait::async_trait;
use playground_core_ecs::MessageBusContract;

#[async_trait]
pub trait ServerContract: Send + Sync {
    // Get server components
    fn dashboard(&self) -> Arc<dyn DashboardContract>;
    fn websocket(&self) -> Arc<dyn WebSocketContract>;
    fn channel_manager(&self) -> Arc<dyn ChannelManagerContract>;
    fn batcher(&self) -> Arc<dyn BatcherContract>;
    fn mcp(&self) -> Arc<dyn McpServerContract>;
    
    // Server lifecycle
    async fn start(&self, port: u16) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    
    // Connect to unified messaging system
    async fn connect_to_message_bus(&self, bus: Arc<dyn MessageBusContract>) -> Result<()>;
}
```

### 2. DashboardContract

Contract for server monitoring and logging:

```rust
#[async_trait]
pub trait DashboardContract: Send + Sync {
    async fn log(&self, level: LogLevel, message: String, details: Option<Value>);
    async fn register_channel(&self, id: u16, name: String, channel_type: ChannelType);
    async fn update_client(&self, id: usize, info: ClientInfo);
    async fn init_log_file(&self) -> Result<(), std::io::Error>;
    async fn start_render_loop(self: Arc<Self>);
}
```

### 3. WebSocketContract

Contract for WebSocket handling that integrates with ECS messaging:

```rust
use playground_core_ecs::MessageHandlerData;

#[async_trait]
pub trait WebSocketContract: Send + Sync + MessageHandlerData {
    // WebSocket IS a message handler in the unified system
    async fn add_connection(&self, conn: ConnectionHandle) -> Result<()>;
    async fn remove_connection(&self, id: usize) -> Result<()>;
    async fn connection_count(&self) -> usize;
}
```

### 4. ChannelManagerContract

Contract for channel registration and discovery:

```rust
#[async_trait]
pub trait ChannelManagerContract: Send + Sync {
    async fn register(&mut self, channel: u16, name: String) -> Result<()>;
    async fn unregister(&mut self, channel: u16) -> Result<()>;
    async fn get_manifest(&self) -> ChannelManifest;
    async fn get_channel_by_name(&self, name: &str) -> Option<u16>;
}
```

### 5. BatcherContract

Contract for frame-based packet batching:

```rust
#[async_trait]
pub trait BatcherContract: Send + Sync {
    async fn queue_packet(&self, packet: Packet);
    async fn get_batch(&self) -> Vec<Packet>;
    fn frame_duration(&self) -> Duration;
    fn set_frame_rate(&mut self, fps: u32);
}
```

### 6. McpServerContract

Contract for MCP (Model Context Protocol) server:

```rust
#[async_trait]
pub trait McpServerContract: Send + Sync {
    async fn register_tool(&mut self, tool: McpTool) -> Result<()>;
    async fn unregister_tool(&mut self, name: &str) -> Result<()>;
    async fn handle_request(&self, request: McpRequest) -> Result<McpResponse>;
    fn router(&self) -> Router;
}

```

## Types (Stateless Data Structures)

### Packet

The binary message format (pure data, no methods):

```rust
pub struct Packet {
    pub channel_id: u16,
    pub packet_type: u16,
    pub priority: Priority,
    pub payload: Vec<u8>,
}
```

### Priority

Priority levels for packet ordering:

```rust
pub enum Priority {
    Low,      // Background tasks
    Medium,   // Normal operations  
    High,     // User interactions
    Critical, // Important updates
    Blocker,  // Must send immediately
}
```

### LogLevel

Logging severity levels:

```rust
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}
```

### ChannelType

Channel categorization:

```rust
pub enum ChannelType {
    System,   // Core systems (1-999)
    Plugin,   // Plugin channels (1000+)
    Session,  // Dynamic sessions (2000+)
}
```

### ClientInfo

Client connection information:

```rust
pub struct ClientInfo {
    pub id: usize,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub ip_address: String,
    pub user_agent: Option<String>,
}
```

### McpTool

MCP tool definition:

```rust
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub handler_channel: u16,
}

```

## Key Architecture Principle: Unified Messaging

The most important change in this design is that **WebSocket is now a participant in the ECS messaging system**, not a separate system that needs bridging:

```rust
// WebSocketContract extends MessageHandlerData
pub trait WebSocketContract: Send + Sync + MessageHandlerData {
    // When messages are published to the MessageBus,
    // WebSocket receives them directly and forwards to clients
}
```

This eliminates the need for the MessageBridge - WebSocket handlers subscribe directly to the MessageBus channels they care about.

## Implementation Location

All implementations of these contracts are in `systems/networking`:

- `systems/networking/src/server_impl/` - Server implementation
- `systems/networking/src/dashboard.rs` - Dashboard implementation
- `systems/networking/src/websocket.rs` - WebSocket implementation
- `systems/networking/src/channel_manager.rs` - Channel manager implementation
- `systems/networking/src/batcher.rs` - Frame batcher implementation
- `systems/networking/src/mcp/` - MCP server implementation

## Usage Pattern

Since this is contracts-only, you cannot instantiate anything from this package directly:

```rust
// WRONG - This package has no implementations
use playground_core_server::Dashboard;
let dashboard = Dashboard::new(); // ERROR: Dashboard is a trait

// CORRECT - Use through systems/logic API
use playground_systems_logic::SystemsManager;
let systems = SystemsManager::new();
systems.initialize_all().await?;
// Server is started internally by systems/networking
```


## Architectural Rules

- **This is a Core package** - Contains ONLY contracts and types
- **NO implementation** - All implementation is in systems/networking
- **NO state** - Only stateless traits and data structures
- **Can use other core/* packages** - Like core/ecs for messaging contracts
- **Cannot use systems/** - Core cannot depend on systems layer
- **Thread-safety required** - All contracts must be Send + Sync


## Dependencies

Minimal dependencies for contracts:
- `async-trait`: For async trait definitions
- `bytes`: For byte buffer types in contracts
- `serde`: For serializable types
- `playground-core-ecs`: For messaging contracts (MessageHandlerData)
- `playground-core-types`: For Handle and Shared types

## See Also

- [core/client](../client/README.md) - Browser WASM client
- [core/types](../types/README.md) - Shared types (Priority, etc.)
- [systems/networking](../../systems/networking/README.md) - Higher-level networking
- [systems/logic](../../systems/logic/README.md) - Initializes all systems
- [MCP Specification](https://modelcontextprotocol.io/docs) - MCP protocol docs