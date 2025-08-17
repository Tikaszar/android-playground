# WebSocket Integration for UI System

## Overview
The UI system has been fully integrated with the core/server WebSocket infrastructure, replacing direct WebSocket connections with the channel-based multiplexer system.

## Key Changes

### 1. Message System (`messages.rs`)
- Created comprehensive message definitions for UI-client communication
- Defined packet types for:
  - Element management (create, update, delete)
  - Input events
  - Terminal operations
  - Render batching
- Implemented serialization/deserialization helpers using serde_json
- Added binary protocol helpers for efficient encoding

### 2. UI System Integration (`system.rs`)
- Added WebSocket message handling to UiSystem
- Integrated with core/server's ChannelManager and FrameBatcher
- Registered UI system on channel 10
- Implemented message handlers for:
  - Element creation/updates
  - Input event routing
  - Terminal operations
- Added packet sending through the batched channel system

### 3. Terminal Migration (`terminal/`)
- Created new `connection.rs` module for channel-based terminal connections
- Replaced direct WebSocket usage with core/server channels
- Maintains terminal state through the UI system's messaging infrastructure
- Terminal now uses `TerminalConnection` that communicates via UI packets

## Architecture

```
Browser Client
    ↓
WebSocket (Binary Protocol)
    ↓
core/server (Channel Multiplexer)
    ↓
Channel 10 (UI System)
    ↓
systems/ui (Message Handlers)
    ↓
Terminal/Elements (via ECS)
```

## Message Flow Example

1. **Terminal Input**:
   - User types in terminal UI
   - Client sends `TerminalInput` packet on channel 10
   - UI system receives and routes to terminal connection
   - Terminal processes input and sends to Termux
   - Output comes back as `TerminalOutput` packet
   - UI system updates terminal display

2. **Element Creation**:
   - Client requests new UI element
   - `CreateElement` packet sent on channel 10
   - UI system creates ECS entity with components
   - Response sent as `ElementCreated` packet
   - Client updates its view

## Benefits

1. **Unified Communication**: All UI communication goes through the same WebSocket multiplexer
2. **Batched Messages**: Frame-based batching at 60fps reduces network overhead
3. **Priority Queues**: Important messages can be prioritized
4. **Binary Protocol**: Efficient serialization for better performance
5. **Channel Isolation**: UI system has its dedicated channel (10)
6. **No Direct WebSocket**: Removed all direct WebSocket dependencies from UI components

## Usage

```rust
// Initialize UI system with server
let mut ui_system = UiSystem::new();
ui_system.initialize(renderer).await?;
ui_system.register_with_server(channel_manager).await?;

// Handle incoming messages
let packet = // ... receive from channel
ui_system.handle_message(packet).await?;

// Send messages to clients
let batch = RenderBatchMessage { /* ... */ };
ui_system.send_render_batch(batch).await?;
```

## Next Steps

1. Implement remaining message handlers (update, delete, etc.)
2. Add UUID to EntityId mapping for element tracking
3. Connect to actual Termux terminal process
4. Implement render command batching from ECS queries
5. Add client-side reconnection logic with exponential backoff