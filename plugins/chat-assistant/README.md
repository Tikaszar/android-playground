# Chat Assistant Plugin

MCP-enabled chat interface for connecting to LLMs (Claude Code, GPT, etc.) with remote IDE capabilities.

## Overview

The Chat Assistant Plugin provides a conversational interface that connects to any LLM through the Model Context Protocol (MCP). It enables remote IDE operations, allowing LLMs to browse files, execute commands, and modify code through a chat interface.

## Architecture

```
chat-assistant/
├── plugin.rs      # Main plugin implementation
├── chat_view.rs   # Chat UI component with message rendering
├── mcp_client.rs  # MCP client for LLM communication
└── lib.rs         # Plugin exports
```

## Core Features

### 1. MCP Client Integration

Connect to any MCP-compatible LLM server:

```rust
use playground_plugins_chat_assistant::McpClient;

let mut mcp_client = McpClient::new("http://localhost:3001".to_string());

// Connect and establish session
mcp_client.connect().await?;

// Send prompts with context
let response = mcp_client.send_prompt(
    "Help me refactor this function".to_string(),
    vec!["/src/main.rs".to_string()]  // Context files
).await?;

// Execute tools remotely
let result = mcp_client.execute_tool(
    "edit_file",
    serde_json::json!({
        "path": "/src/lib.rs",
        "content": updated_content
    })
).await?;
```

### 2. Chat View Component

Interactive chat interface with message history:

```rust
use playground_plugins_chat_assistant::{ChatView, MessageSender};

let mut chat_view = ChatView::new();

// Add messages
chat_view.add_message(
    MessageSender::User,
    "How do I implement a binary search?".to_string()
);

chat_view.add_message(
    MessageSender::Assistant,
    "Here's an implementation of binary search...".to_string()
);

chat_view.add_message(
    MessageSender::System,
    "Connected to Claude Code via MCP".to_string()
);

// Message types
pub enum MessageSender {
    User,       // User messages
    Assistant,  // LLM responses
    System,     // System notifications
}
```

### 3. Remote IDE Operations

Execute IDE operations through MCP tools:

```rust
// Available MCP tools for remote IDEs
- list_files      // Browse directory structure
- read_file       // Read file contents
- edit_file       // Modify files
- create_file     // Create new files
- delete_file     // Remove files
- run_command     // Execute shell commands
- search_code     // Search across codebase
- get_diagnostics // Get compilation errors
```

### 4. Streaming Support

Real-time streaming of LLM responses:

```rust
// Stream response tokens as they arrive
let mut stream = mcp_client.send_prompt_streaming(prompt).await?;

while let Some(token) = stream.next().await {
    chat_view.append_to_last_message(token);
    // Update UI in real-time
}
```

## MCP Protocol Implementation

### Session Management
```rust
// Initialize session
let init_message = json!({
    "type": "initialize",
    "client_info": {
        "name": "android-playground",
        "version": "0.1.0",
        "capabilities": ["tools", "streaming"]
    },
    "protocol_version": "1.0"
});

// Keep alive
mcp_client.send_ping().await?;
```

### Tool Execution
```rust
// Tool call format
let tool_call = json!({
    "type": "tool_call",
    "tool": "edit_file",
    "arguments": {
        "path": "/src/main.rs",
        "old_content": "fn old()",
        "new_content": "fn new()"
    }
});

let result = mcp_client.send_message(tool_call).await?;
```

## Usage Examples

### Complete Example: LLM Integration

```rust
use playground_plugins_chat_assistant::ChatAssistantPlugin;

// Initialize plugin
let mut plugin = ChatAssistantPlugin::new();
plugin.on_load(&mut context).await?;

// Process user message
plugin.process_user_message(
    "Create a REST API endpoint for user authentication".to_string()
);

// The plugin will:
// 1. Send request to connected LLM via MCP
// 2. LLM can request file reads, edits, etc.
// 3. Display responses in chat view
// 4. Execute any requested tool calls
```

### Connecting External LLMs

```bash
# Start MCP server
cargo run -p playground-apps-editor

# Connect Claude Code
claude --mcp http://localhost:3001

# Or connect GPT
gpt --mcp-endpoint http://localhost:3001
```

## Channel Communication

Uses channels 1040-1049 for IDE plugin communication:

```rust
// Channel 1040: Main chat channel
// Channel 1041: MCP responses
// Channel 1042: Tool execution results
```

## Configuration

```rust
pub struct ChatAssistantConfig {
    pub mcp_server_url: String,      // Default: "http://localhost:3001"
    pub auto_connect: bool,           // Auto-connect on startup
    pub streaming_enabled: bool,      // Enable response streaming
    pub max_message_history: usize,   // Message history limit
    pub tool_timeout_ms: u64,        // Tool execution timeout
}
```

## Dependencies

- `playground-core-plugin`: Plugin trait
- `playground-core-types`: Core types
- `playground-systems-ui`: UI rendering
- `playground-systems-networking`: Channel communication
- `reqwest`: HTTP client for MCP
- `serde`/`serde_json`: Message serialization
- `async-trait`: Async plugin support
- `tokio`: Async runtime
- `tracing`: Logging

## Testing

```bash
# Run unit tests
cargo test -p playground-plugins-chat-assistant

# Test MCP connection
cargo test -p playground-plugins-chat-assistant mcp_

# Test chat view
cargo test -p playground-plugins-chat-assistant chat_
```

## License

See the main project LICENSE file for details.