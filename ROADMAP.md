# Architecture Alignment Roadmap

This roadmap outlines the changes required to align the codebase with the proper 4-layer architecture (Core → Systems → Plugins → Apps) where all system functionality is exposed through the ECS World via command processors.

## Phase 1: Core Layer Restructuring ✅ COMPLETED (Session 47)

### 1.1 Create core/console Package ✅
- ✅ Defined GENERIC console/logging contracts (not terminal-specific)
- ✅ `ConsoleContract` trait for any console implementation
- ✅ `LoggingContract` trait for any logging backend
- ✅ `InputContract` for console input support
- ✅ Defined `LogLevel`, `LogEntry`, `OutputStyle`, `Progress` types
- ✅ Command processor with `ConsoleCommand` and `ConsoleCommandHandler`
- ✅ NO implementation code, only traits and types
- ✅ Supports terminal, file, network, GUI backends

### 1.2 Update core/server Package ✅
- ✅ Removed dashboard functionality (moved to core/console)
- ✅ Defined GENERIC server contracts (not HTTP/WebSocket-specific)
- ✅ `ServerContract` for ANY server type (TCP, UDP, IPC, etc.)
- ✅ `ConnectionContract` for ANY connection type
- ✅ `MessageContract` and `MessageHandler` for ANY message protocol
- ✅ `ChannelContract` for ANY channel management
- ✅ Defined `ServerCommand` enum and `ServerCommandHandler` trait
- ✅ NO protocol-specific assumptions

### 1.3 Create core/client Package ✅
- ✅ Defined GENERIC contracts for ANY client implementation
- ✅ `ClientContract` for client lifecycle (connect, disconnect, update)
- ✅ `RenderingClientContract`, `InputClientContract`, `AudioClientContract`
- ✅ Generic input system (keyboard, mouse, touch, gamepad)
- ✅ Generic types: `ClientState`, `ClientCapabilities`, `RenderTarget`
- ✅ Defined `ClientCommand` enum and `ClientCommandHandler` trait
- ✅ NO browser-specific types or concepts

### 1.4 Update core/ecs Package (PENDING)
- Add generic command processor registration to `WorldContract`
- Define `SystemCommandProcessor` trait for all systems
- Extend command processor architecture beyond just World commands

## Phase 2: Systems Layer Implementation ✅ COMPLETED (Session 48-49)

### 2.1 Create systems/console Package ✅
- ✅ Implemented `ConsoleContract` from core/console for TERMINAL output
- ✅ Moved terminal dashboard implementation from systems/networking
- ✅ Implemented ANSI terminal UI rendering (specific implementation)
- ✅ Created `ConsoleCommand` processor for ECS integration
- ✅ Register with World on initialization
- ✅ NO direct exports - only accessible through ECS
- ✅ This is ONE possible console implementation (terminal)

### 2.2 Refactor systems/networking Package ✅
- ✅ Removed dashboard functionality (moved to systems/console)
- ✅ Implemented server contracts from core/server for WebSocket/HTTP:
  - ✅ WebSocket protocol handling (specific implementation)
  - ✅ HTTP/SSE for MCP server (specific implementation)
  - ✅ TCP/IP networking (specific implementation)
  - ✅ Binary packet protocol (specific implementation)
- ✅ Implemented `ServerCommandHandler` for command processing
- ✅ Register command processor with World
- ✅ Removed direct method exports
- ✅ This is ONE possible server implementation (WebSocket/HTTP)

### 2.3 Update systems/webgl Package ✅
- ✅ Implemented `ClientContract` from core/client for browser/WebGL
- ✅ Handle browser-specific functionality (WASM, WebGL, DOM)
- ✅ Implemented `ClientCommandHandler` for browser operations
- ✅ Register with World command processor
- ✅ This is ONE possible client implementation (browser)

### 2.4 Future: systems/vulkan Package (Example)
- Would implement `ClientContract` for native Vulkan client
- Handle native window management
- Different implementation, same contracts

### 2.4 Update systems/ecs Package ✅
- ✅ Extended World to support multiple command processors
- ✅ Added registry for system command processors
- ✅ Implemented routing of commands to appropriate systems
- ✅ Maintained existing World command processor functionality

### 2.5 Update systems/ui Package ✅
- ✅ Added UI command processor for external functionality
- ✅ No direct imports from other systems
- ✅ UI command processor ready for registration

### 2.6 Update systems/webgl Package ✅
- ✅ Renderer exposes functionality through command processor
- ✅ No direct system dependencies
- ✅ Rendering command processor ready for registration

## Phase 3: Systems/Logic API Gateway ⚠️ PARTIAL (Session 48-49)

### 3.1 Create Networking API Module ✅
- ✅ Added `systems/logic/src/networking_api.rs`
- Provide clean functions for server operations:
  - `start_server(port: u16)`
  - `stop_server()`
  - `send_packet(channel: u16, data: Bytes)`
  - `register_channel(name: String)`
  - `register_mcp_tool(tool: McpTool)`
- Internally use command processors via core/server functions

### 3.2 Create Console API Module ✅
- ✅ Added `systems/logic/src/console_api.rs`
- Provide logging and monitoring functions:
  - `log(level: LogLevel, message: String)`
  - `log_component(component: &str, level: LogLevel, message: String)`
  - `get_dashboard_state()`
- Internally use command processors via core/console functions

### 3.3 Create Client API Module ✅
- ✅ Added `systems/logic/src/client_api.rs`
- Provide GENERIC client management:
  - `connect_client(id: ClientId)`
  - `disconnect_client(id: ClientId)`
  - `send_to_client(id: ClientId, data: Bytes)`
  - `handle_client_input(id: ClientId, input: InputEvent)`
- Internally use command processors via core/client functions
- NO browser-specific or WebGL-specific functions

### 3.4 Update Existing API Modules ✅ COMPLETE REWRITE
- ✅ DELETED all old implementation code (17 files removed)
- ✅ Created new API modules: ecs_api, ui_api, rendering_api
- ✅ All modules only use core/* contracts
- ✅ NO imports from other systems/*
- ✅ All use command processors for cross-system communication
- ✅ systems/logic is now a pure stateless API gateway

## Phase 4: Plugin Layer Updates

### 4.1 Audit All Plugins
- Check each plugin in plugins/* directory
- Ensure they ONLY import from systems/logic
- Remove any direct core/* imports
- Remove any direct systems/* imports

### 4.2 Update Plugin Initialization
- Plugins should use systems/logic API for all operations
- Update any networking calls to use networking_api
- Update any logging calls to use console_api

## Phase 5: App Layer Validation

### 5.1 Update playground-editor App
- Ensure it only imports systems/logic
- Remove any direct core/* dependencies
- Remove any direct systems/* dependencies
- Use systems/logic API for all engine operations

### 5.2 Create App Template
- Document proper app structure
- Show how to use systems/logic API
- Demonstrate plugin loading through API

## Phase 6: Static Registry Implementation

### 6.1 Create Build Script
- Add build.rs to discover all systems
- Generate static registration code
- Ensure only build script knows concrete types

### 6.2 Update System Registration
- Each system implements self-registration
- Systems register their command processors
- Remove manual system wiring

## Phase 7: Testing and Validation

### 7.1 Create Architecture Tests
- Test that plugins cannot import core/*
- Test that plugins cannot import systems/* (except systems/logic)
- Test that systems cannot import other systems
- Test that core/* contains no implementation

### 7.2 Create Integration Tests
- Test command processor routing
- Test API gateway functions
- Test complete data flow from App → Plugin → System → Core

### 7.3 Update Examples
- Create example showing proper architecture
- Demonstrate command processor pattern
- Show API gateway usage

## Implementation Order

1. **Start with core/console** - Smallest, self-contained change
2. **Create systems/console** - Move dashboard functionality
3. **Refactor systems/networking** - Remove dashboard, add command processor
4. **Update systems/logic** - Add API modules for console and networking
5. **Update one plugin** - Validate the new API works
6. **Continue with remaining systems** - Apply pattern everywhere
7. **Update all plugins** - Ensure compliance
8. **Validate apps** - Final layer verification

## Success Criteria

- [x] All core/* packages contain only contracts (no implementation) - Phase 1 ✅
- [x] All systems/* expose functionality through command processors - Phase 2 (partial) ✅
- [x] systems/logic provides complete API for all operations - Phase 2 (partial) ✅
- [x] No system imports another system (except systems/logic using core/*) - Phase 2 ✅
- [ ] All plugins import only from systems/logic
- [ ] All apps import only from systems/logic
- [x] Command processors handle all cross-system communication - Phase 2 ✅
- [ ] Static registry manages system discovery
- [x] Full compilation with zero architecture violations - Phase 2 (networking) ✅

## Architecture Invariants to Maintain

1. **Generic Core**: core/* defines ONLY generic contracts, NO implementation-specific assumptions
2. **Stateless Core**: core/* must never contain implementation
3. **System Isolation**: systems/* cannot depend on each other
4. **Single API Gateway**: systems/logic is the ONLY public API
5. **Command Processor Pattern**: All system functionality exposed through ECS
6. **No Direct Access**: Plugins/Apps never access core/* or systems/* directly
7. **Handle/Shared Pattern**: Handle<T> for external refs, Shared<T> for internal
8. **NO dyn**: Use concrete types with enum patterns
9. **NO unsafe**: 100% safe Rust
10. **Async Everything**: All I/O operations must be async
11. **Result Everywhere**: All fallible operations return Result<T, Error>

## Core Design Principle

**core/* packages are GENERIC CONTRACTS that make no assumptions about implementation:**
- core/server could be implemented as WebSocket, TCP, UDP, IPC, or any other protocol
- core/console could be implemented as terminal, file, GUI, or network logging
- core/client could be implemented as browser, native app, CLI, or mobile app
- core/rendering defines rendering contracts, not WebGL or Vulkan specifics
- core/ui defines UI contracts, not DOM or native widget specifics

**systems/* packages are SPECIFIC IMPLEMENTATIONS of those contracts:**
- systems/networking implements core/server for WebSocket/HTTP
- systems/console implements core/console for terminal/ANSI
- systems/webgl implements core/client and core/rendering for browsers
- systems/ui implements core/ui contracts (generic across renderers)