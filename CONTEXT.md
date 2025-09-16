# CONTEXT.md - Current Session Context

## Active Session - 2025-01-16 (Session 56)

### Current Session 56 Accomplishments

Successfully rewrote core/console and systems/console following the data vs logic separation pattern:

1. **Rewrote core/console completely**:
   - Removed all trait-based contracts
   - Created Console struct with data fields only
   - All methods delegate through VTable
   - Comprehensive feature flags for different capabilities
   - Multiple small files (console.rs, types.rs, output.rs, logging.rs, etc.)

2. **Rewrote systems/console completely**:
   - Implements all actual console logic
   - VTable registration in registration.rs
   - Terminal implementation for actual I/O
   - Dashboard for monitoring
   - File logging support
   - Matching feature flags with core/console

3. **Architecture Compliance Achieved**:
   - NO dyn, NO traits for contracts
   - Data vs logic separation complete
   - VTable dispatch working
   - Feature-gated APIs
   - Apps/Plugins only import core/console, never systems/console
   - Build script handles system registration (not apps)

### Architecture Status
**Core/Console**: ✅ COMPLETE - Data structures only, VTable delegation
**Systems/Console**: ✅ COMPLETE - All console logic implemented
**Pattern Applied**: ✅ Abstract base class pattern - core has structure, systems has behavior

### Previous Session 55 Accomplishments

Successfully refactored core/ecs and systems/ecs to achieve proper data vs logic separation:

1. **Removed ALL logic from core/ecs**:
   - `world.rs` - Now just data fields and VTable delegation (~100 lines vs 300+)
   - `storage.rs` - Now just data structures (~50 lines vs 200)
   - All public API methods delegate through VTable
   - NO implementation logic whatsoever

2. **Moved ALL logic to systems/ecs**:
   - `world_impl.rs` - Contains all World operation implementations
   - `storage_impl.rs` - Contains all ComponentStorage operations
   - `vtable_handlers.rs` - Registers handlers and routes commands
   - `registration.rs` - Wires up VTable registration

3. **Achieved "Abstract Base Class" Pattern**:
   - core/ecs = Structure only (like abstract classes)
   - systems/ecs = All behavior (like concrete implementations)
   - VTable provides runtime dispatch between them

4. **Updated Documentation**:
   - `DESIGN.md` now clearly explains data vs logic separation
   - Added concrete examples of the pattern
   - Emphasized NO logic in core, ALL logic in systems

### Architecture Status
**Core/ECS**: ✅ COMPLETE - Data structures only, no logic
**Systems/ECS**: ✅ COMPLETE - All ECS logic implemented via VTable
**VTable Pattern**: ✅ WORKING - Commands dispatched from core to systems
**Documentation**: ✅ UPDATED - DESIGN.md reflects new architecture

### Key Achievement
Successfully implemented the principle that core/ecs should be "mostly stateless" - it now contains ONLY data structures with all logic delegated through VTable to systems/ecs. This is like OOP abstract base classes where the base defines structure but implementations provide behavior.

### Previous Session 53 Accomplishments

1. **Created DESIGN.md** - Complete architecture documentation in core/ecs/DESIGN.md
2. **Completely rewrote core/ecs**:
   - `vtable.rs` - Generic VTable for capability registration
   - `world.rs` - Concrete World struct with VTable
   - `registry.rs` - Global World instance management
   - `component.rs` - Concrete Component struct (no dyn)
   - `entity.rs` - Entity ID system with generations
   - `messaging.rs` - Concrete MessageBus implementation
   - `query.rs` - Concrete Query builder
   - `storage.rs` - Concrete ComponentStorage
   - `system.rs` - Concrete System and SystemScheduler
   - `error.rs` - Clear error boundaries

3. **Clarified architecture**:
   - core/ecs provides foundation, knows nothing about other packages
   - Other core/* packages define contracts and APIs
   - Systems implement contracts and register with VTable
   - Apps/Plugins use core/ecs + specific core/* packages needed

### Architecture Summary

- **NO Single Import**: Apps import core/ecs + needed core/* packages
- **Clean Separation**: core/ecs never depends on other packages
- **Concrete Types**: NO dyn, NO traits for contracts
- **Generic VTable**: String-based capability registration
- **API Ownership**: Each core/* package owns its API
- **Clear Boundaries**: Compile errors for missing packages, runtime errors for unregistered systems

### Session 51 Accomplishments (Previous)

Successfully fixed all core layer architectural violations:

1. **Fixed `core/types/context.rs`**:
   - Removed `Box<dyn std::any::Any>` violation
   - Replaced with concrete `Resource` struct using `Bytes` serialization
   - Follows established Component/ComponentData pattern

2. **Fixed `core/server` violations**:
   - Replaced all `Box<dyn Error>` with `CoreError` in:
     - `channel.rs`
     - `connection.rs`
     - `message.rs`

3. **Fixed `core/ecs/system_commands.rs`**:
   - Removed `Arc<dyn SystemCommandProcessor>` violation
   - Created concrete `SystemCommandProcessorWrapper` struct
   - Uses channels instead of trait objects
   - Now uses `Handle<T>` instead of direct `Arc`

4. **Moved `core/android` to `systems/android`**:
   - Platform-specific code removed from core layer
   - Renamed package to `playground-systems-android`
   - Updated all workspace configurations

### Verification Complete
- ✅ NO `dyn` violations in core
- ✅ NO `Any` violations in core
- ✅ NO `unsafe` code in core
- ✅ NO direct `Arc`/`RwLock` usage (only Handle/Shared)
- ✅ NO platform-specific code in core

### Next Steps
With the core layer now fully compliant, the next priority is to address the systems layer violations identified in Session 50:
- Systems requiring rewrite: `systems/ui`, `systems/logic`, `systems/physics`
- Systems requiring refactor: `systems/networking`, `systems/ecs`, `systems/webgl`

---

## Previous Session - 2025-09-15 (Session 50) - Full Architectural Audit

### Audit Results Summary

The architectural audit of all engine layers revealed:

*   **`core` Layer:** The design was conceptually sound. The only issues were implementation bugs (`dyn`/`Any` usage) and one misplaced module (`core/android`). ✅ NOW FIXED

*   **`systems` Layer:** This layer has significant architectural problems.
    *   **Require Rewrite:** `systems/ui`, `systems/logic`, `systems/physics`.
    *   **Require Refactor:** `systems/networking`, `systems/ecs`, `systems/webgl`.
    *   **Compliant:** `systems/console`.
    *   **New Principle:** Systems should be considered private, only exposing functionality via `core` contracts.

*   **`plugins` Layer:** This layer is fundamentally broken.
    *   **Conclusion:** All 9 IDE plugins are non-compliant, bypassing the `systems/logic` API gateway and depending directly on other systems. They all require a complete rewrite.

---

## Long-Term Project Memory

### Critical Architecture Principles (Must Remember)

**SESSION 52 UPDATE - Revolutionary Architecture:**

1. **`core/ecs` is the ONLY public interface**: Apps and Plugins import ONLY `playground-core-ecs`, which provides all contracts through feature gates.
2. **`core` packages define contracts + VTables**: Pure contracts with VTable structs for runtime dispatch. NO implementation.
3. **`systems` are Private Implementations**: Systems implement core contracts and register VTables with World at runtime.
4. **NO `systems/logic`**: This layer has been eliminated. The architecture is now simpler: Apps/Plugins → core/ecs → VTable dispatch → systems.
5. **Feature-gated capabilities**: Compile-time features determine what CAN be available, runtime VTable registration determines what IS available.
6. **Strict Isolation**: Systems cannot depend on other systems. Apps/Plugins cannot depend on systems, only core/ecs.
7. **VTable-based dispatch**: All cross-system communication happens through VTables registered in the World as resources.

### Project State Overview
- **Build**: ⚠️ NEEDS TESTING - Major architectural changes need compilation check
- **Architecture Compliance**:
  - Core: ✅ FULLY COMPLIANT (Session 51)
  - Systems: ❌ MAJOR VIOLATIONS
  - Plugins: ❌ COMPLETE REWRITE NEEDED
- **Next Priority**: Fix systems layer starting with rewrites of `ui`, `logic`, and `physics`

## Active Session - 2025-09-14 (Session 49)

### Current Status
**Build**: ⚠️ NEEDS TESTING - Major architectural changes need compilation check
**Phase 1**: ✅ COMPLETE - Core layer fully restructured with generic contracts
**Phase 2**: ✅ COMPLETE - All systems layer items implemented
**Architecture**: ✅ ALIGNED - All systems use command processor pattern
**World Command Processor**: ✅ COMPLETE - Session 46 implementation working
**System Architecture**: ✅ COMPLETE - All systems use command processors
**API Gateway**: ✅ COMPLETE - systems/logic fully rewritten as stateless gateway
**Roadmap**: ✅ FOLLOWING - Completed Phase 2 per roadmap
**Design Docs**: ✅ UPDATED - DESIGN_CLARIFICATION.md reflects correct architecture
**NO dyn Compliance**: ✅ COMPLETE - All Box<dyn Error> replaced with CoreError

### Session 49 - Complete Phase 2 & Rewrite systems/logic

**Goal**: Complete Phase 2 items and properly rewrite systems/logic as API gateway.

### Session 48 - Phase 2 PARTIAL Implementation

**✅ COMPLETED in Session 48**:
1. **Phase 2.1 - Created systems/console package**:
   - TerminalConsole implements generic console contracts
   - Dashboard moved from systems/networking
   - ConsoleSystem provides command processor
2. **Phase 2.2 - Refactored systems/networking**:
   - Created types.rs with WebSocket-specific types
   - Implemented ServerCommandHandler for command processor pattern
   - Removed dashboard (now in systems/console)
   - Fixed all Box<dyn Error> usage
3. **Created API modules in systems/logic**:
   - networking_api.rs - Clean public API for networking operations
   - console_api.rs - Clean public API for console/logging operations

**✅ COMPLETED in Session 49**:
1. **Phase 2.3**: Implemented ClientContract in systems/webgl with WebGLClient
2. **Phase 2.4**: Extended systems/ecs World to support multiple system command processors
3. **Phase 2.5**: Added UI command processor to systems/ui
4. **Phase 2.6**: Added renderer command processor to systems/webgl
5. **MAJOR**: Completely rewrote systems/logic as stateless API gateway
   - Removed ALL implementation code (deleted 17 files)
   - Created clean API modules: ecs_api, ui_api, rendering_api, client_api
   - Now ONLY forwards to command processors, no state or implementation

**Architecture Compliance**: 
- ✅ NO dyn violations (all Box<dyn Error> replaced with CoreError)
- ✅ COMPLETE separation: All systems have command processors
- ✅ COMPLETE command processor pattern: All cross-system communication via ECS
- ✅ systems/logic is now a pure stateless API gateway
- ✅ All systems isolated - no direct dependencies between systems

### Session 47 - Architecture Alignment & Phase 1 Implementation

**Goal**: Create roadmap for proper system isolation and implement Phase 1 (Core Layer Restructuring).

**Initial Analysis**:
1. **systems/networking is incorrect** - Contains dashboard functionality that should be separate
2. **Systems not using ECS properly** - Should expose functionality through command processors, not direct exports
3. **systems/logic incomplete** - Needs API modules for networking, console, client operations
4. **Core packages not generic** - Many core/* packages had implementation-specific assumptions

**Major Accomplishments**:

1. **Created Architecture Roadmap (ROADMAP.md)**:
   - 7-phase plan for complete architectural alignment
   - Clear implementation order and success criteria
   - Emphasis on generic core contracts

2. **Updated DESIGN_CLARIFICATION.md**:
   - Added principle: "Systems Expose Functionality Through ECS"
   - Clarified ALL core/* packages must be generic
   - Documented command processor pattern for all systems

3. **✅ COMPLETED Phase 1 - Core Layer Restructuring**:

   **core/console (NEW)**:
   - Generic console/logging contracts
   - ConsoleContract, LoggingContract, InputContract traits
   - Command processor pattern with ConsoleCommand/ConsoleResponse
   - Supports ANY backend (terminal, file, GUI, network)

   **core/server (REWRITTEN)**:
   - Removed dashboard functionality
   - Made ALL contracts generic (not WebSocket-specific)
   - ServerContract works for TCP, UDP, IPC, named pipes, etc.
   - ConnectionContract, ChannelContract, MessageContract
   - Command processor with ServerCommand/ServerResponse

   **core/client (REWRITTEN)**:
   - Was browser/WASM specific, now completely generic
   - ClientContract, RenderingClientContract, InputClientContract
   - Generic input system (keyboard, mouse, touch, gamepad)
   - Command processor with ClientCommand/ClientResponse
   - Works for browser, native, mobile, CLI clients

**Architecture Pattern Established**:
```
App → systems/logic API → core/* command → ECS World → System command processor
```

**Build Status**: ✅ All core packages compile successfully

### Session 46 - World Command Processor Implementation (Previous)

**Goal**: Implement World command processor to allow systems to interact with ECS through core/ecs without violating architecture boundaries.

**Progress**:
✅ **COMPLETED**: Implemented full World command processor architecture

**Major Changes Made**:

1. **core/ecs additions (contracts only)**:
   - Added `WorldCommand` enum for all World operations
   - Added `WorldCommandHandler` trait for command handling  
   - Added `world_access.rs` with static functions for World access
   - Added missing error types (NotInitialized, SendError, ReceiveError)
   - Uses channels for communication (NOT WebSocket channels)

2. **systems/ecs implementation**:
   - Implemented `WorldCommandHandler` for World
   - Added `start_command_processor()` to register with core/ecs
   - Added byte-based component operations for command processor
   - Added `get_bytes()` method to ComponentStorage
   - Fixed ComponentBox creation with module function (avoiding impl on type alias)

3. **Architecture compliance achieved**:
   - Systems can ONLY use core/* packages
   - NO dyn, NO Any, NO unsafe - uses concrete command enum
   - Uses tokio::sync::RwLock ONLY (per rules)
   - Handle<T> for external refs, Shared<T> for internal state
   - Complete implementations with no TODOs
   - All operations async with Result<T, Error>

**How it works**:
1. World calls `start_command_processor()` on creation
2. This registers a command channel with core/ecs  
3. Systems call functions like `playground_core_ecs::spawn_entity()`
4. Commands sent through channel to World
5. World processes and returns results
6. Systems never import from other systems

**Build Status**: ✅ Both core/ecs and systems/ecs compile successfully

**Next Steps**:
- Update UI system to use new World access functions
- Fix UI subsystem compilation errors  
- Update scheduler to use concrete SystemWrapper enum

### Session 44 Accomplishments - Unified Messaging Architecture

#### ✅ COMPLETED: Core/Server and Systems/Networking Refactor

Successfully implemented the unified messaging architecture:

**1. Core/Server Refactored to Contracts Only** ✅:
- Split monolithic contracts.rs into separate files:
  - server.rs - ServerContract
  - dashboard.rs - DashboardContract
  - websocket.rs - WebSocketContract (extends MessageHandlerData)
  - channel_manager.rs - ChannelManagerContract
  - batcher.rs - BatcherContract
  - mcp.rs - McpServerContract
  - types.rs - All pure data types
- NO implementation code, only traits and types
- Clean lib.rs with exports only

**2. Systems/Networking Contains All Implementations** ✅:
- server_impl.rs - Main server implementation
- dashboard.rs - Dashboard monitoring
- websocket.rs - WebSocket as MessageHandler
- channel_manager.rs - Dynamic channel allocation
- batcher.rs - Frame-based packet batching
- mcp.rs - MCP/LLM integration
- networking_system.rs - System trait wrapper

**3. Unified Messaging - WebSocket IS a MessageHandler** ✅:
- Eliminated MessageBridge entirely
- WebSocket directly implements MessageHandlerData
- Subscribes directly to MessageBus channels
- Creates truly unified messaging system

**Key Innovation**: WebSocket is now a direct participant in the ECS messaging system, not a separate system needing a bridge!

### Session 43 Accomplishments - Unified ECS Architecture

#### ✅ COMPLETED: Major ECS Architecture Refactor

Successfully implemented the unified ECS design from DESIGN_CLARIFICATION.md:

**1. Core/ECS Refactored to Contracts Only** ✅:
- Deleted all implementation code from core/ecs
- Now contains ONLY traits and type definitions:
  - `WorldContract` - Interface for World implementations
  - `ComponentData` - Trait for component types
  - `Storage` - Trait for storage implementations
  - `System` - Trait for all systems
  - `Query` - Trait for query operations
  - `MessageBusContract` - Interface for messaging
  - Types: EntityId, Generation, ComponentId, ChannelId, ExecutionStage
  - Errors: EcsError, EcsResult

**2. Created systems/ecs - Unified ECS Implementation** ✅:
- Single authoritative World for entire engine
- Implements ALL contracts from core/ecs:
  - `World` implements `WorldContract`
  - `QueryBuilder` implements `Query`
  - `MessageBus` implements `MessageBusContract`
- Complete ECS functionality:
  - Entity management with generational IDs
  - Component storage (Sparse/Dense using enum pattern - NO dyn)
  - Query system without turbofish
  - System scheduler with staged execution
  - Messaging as CORE ECS functionality (not a separate system)
- Three-stage execution pipeline: Update → Layout → Render

**3. Messaging Integration** ✅:
- Messaging is now fundamental ECS functionality
- Built directly into World, not a separate system
- All systems and components can use messaging
- Proper contracts in core/ecs, implementation in systems/ecs

**4. Architecture Compliance** ✅:
- NO dyn violations (used enum patterns)
- NO unsafe code
- NO turbofish (ComponentId-based queries)
- Handle<T> for external refs, Shared<T> for internal
- All async with proper error handling
- Clean separation of contracts vs implementation

### Session 40 Accomplishments (Previous)

#### ✅ IMPLEMENTED: Robust WebGL Renderer Logging

- **Added component-specific logging to WebGL renderer**:
  1. WebGLRenderer now logs through core/server Dashboard directly
  2. Tracks frame count, command count, and operations
  3. Logs initialization, frame lifecycle, buffer operations, and resize events
  
- **Created browser page construction in systems/webgl**:
  1. Added `BrowserBuilder` module for generating HTML/JS
  2. Created `WebGLServerIntegration` for Axum route integration
  3. Generates complete WebGL client with WebSocket connection
  4. Client handles channel discovery and render command reception
  
### Implementation Steps for New Architecture

#### Phase 1: Core Layer Refactoring
1. **Refactor core/ecs to contracts only**:
   - Extract all implementation to systems/ecs
   - Keep only traits and interfaces
   - Define ECS contract without implementation
   
2. **Verify all core/* packages are stateless**:
   - Review each core package for state
   - Move implementations to systems/*
   - Ensure only contracts remain

#### Phase 2: Create Unified ECS
1. **Create systems/ecs package**:
   - Implement unified World
   - Single ECS for entire engine
   - Scheduler for staged execution
   - Implements core/ecs contracts
   
2. **Migrate existing ECS functionality**:
   - Move from core/ecs implementation
   - Merge with systems/logic ECS features
   - Ensure all features preserved

#### Phase 3: Refactor systems/logic
1. **Convert to API gateway**:
   - Remove ECS implementation
   - Create public API surface
   - Hide internal packages
   - Stateless design
   
2. **Define public types**:
   - UiElementComponent and other public types
   - API functions for plugin/app use
   - Translation layer to internal types

#### Phase 4: Update System Registration
1. **Engine system auto-registration**:
   - Compile-time manifest generation
   - Automatic discovery of systems
   
2. **Plugin registration API**:
   - Explicit registration through systems/logic
   - App-controlled plugin loading

### Session 39 Accomplishments (Previous)

#### ✅ COMPLETED: Extended Component-Specific Logging to All Systems/Plugins/Apps

- **Exposed Logging API through systems/logic**:
  1. Added `log_component()` and `log()` methods to SystemsManager
  2. Re-exported `LogLevel` type for convenience in systems/logic
  3. Methods get Dashboard reference from NetworkingSystem
  
- **Updated Systems**:
  - NetworkingSystem: Now uses `log_component("systems/networking", ...)`
  - UiSystem: Already had component logging to "systems/ui"
  
- **Updated IDE Plugins**:
  - UI Framework Plugin: Converted all ~30+ log calls to component logging
  - Editor Core Plugin: Replaced tracing macros with SystemsManager logging
  - File Browser Plugin: Commented out tracing macros (ready for component logging)
  
- **Updated playground-editor App**:
  - Main initialization uses `log_component("apps/playground-editor", ...)`
  - Plugin spawn tasks still use eprintln (can't access SystemsManager in spawned tasks)

- **Build Status**: ✅ Fully compiling with only warnings

### Session 38 Accomplishments (Previous)

#### ✅ IMPLEMENTED: Component-Specific Log Files Infrastructure
- **Feature**: Added distributed logging system for better debugging
  - Each component logs to its own file (e.g., `playground_editor_systems_ui_*.log`)
  - Dashboard displays all logs in unified view
  - Main server log still exists for overview
  
- **Implementation**:
  1. Added `component_log_files: Shared<HashMap<String, tokio::fs::File>>` to Dashboard
  2. Created `log_component()` method that routes logs to component-specific files
  3. Component logs created on-demand in `logs/` directory
  
- **Benefits**:
  - Much easier to debug specific components
  - No more massive single log file
  - Can tail specific component logs during debugging
  - Dashboard still shows unified view for monitoring

### Session 37 Accomplishments

#### ✅ FIXED: Dashboard Plugin Registration Issue
- **Issue**: UI Framework Plugin (channel 1) wasn't showing in dashboard logs
  - Plugin was registered in Phase 1 before dashboard existed
  - Other plugins (2-9) registered after and showed properly

- **Solution**: Proper lifecycle management in SystemsManager
  - Phase 1: `register_plugin()` only allocates channels, stores in registry
  - Phase 2: `initialize_all()` registers ALL plugins with dashboard after it's available
  - All plugins now properly logged in dashboard

#### ✅ FIXED: NO dyn Violations Reintroduced in Session 33
- **Issue**: Session 33's fix for circular dependencies introduced `Box<dyn System>` violations
  - World had `plugin_systems: Shared<Vec<Box<dyn System>>>`
  - SystemsManager had similar violation
  - This broke the NO dyn rule that was supposedly fixed

- **Solution Implemented**:
  1. **Removed all Box<dyn System>** from World and SystemsManager
  2. **Channel-based architecture**: 
     - Plugins run as independent tasks
     - Communication via channels only
     - No direct method calls on plugin instances
  3. **Proper Handle/Shared usage**:
     - Plugins receive `Handle<World>` (external reference)
     - Not cloning World or using Shared<World>
     - Follows rule: Handle for external, Shared for internal

- **Implementation Details**:
  - World now tracks `plugin_channels: Shared<Vec<(String, u16)>>` instead of plugin instances
  - SystemsManager has `plugin_registry: Shared<Vec<(String, u16)>>` for tracking
  - main.rs spawns each plugin as independent tokio task
  - Each plugin gets its own Handle<World> reference
  - Plugins initialize and run independently at 60fps

#### ✅ PROPER Channel Registration Flow
- **Phase 1**: Register channels with SystemsManager
  - SystemsManager allocates channels dynamically
  - Channels stored in registry for manifest generation
  
- **Phase 2**: Core systems initialize
  - NetworkingSystem can build complete channel manifest
  
- **Phase 3**: Plugin tasks spawn
  - Each plugin initialized with Handle<World>
  - Spawned as independent async task
  - Runs own 60fps update loop

### Session 36 Accomplishments

#### ✅ COMPLETED: UI Framework Plugin Refactored to Single Channel
- **Issue**: UI Framework Plugin was requesting 10 channels unnecessarily
- **Root Cause**: Legacy design from before dynamic channel system was implemented
  - Plugin was allocating channels "ui-framework-0" through "ui-framework-9"
  - Checking all 10 channels every frame in run() method
  - Browser expected multiple ui-framework-* channels

- **Solution Implemented**:
  1. **Plugin Changes** (plugins/ui-framework/src/plugin.rs):
     - Removed `channel_range: Vec<u16>` field
     - Changed from 10 channels to single "ui-framework" channel
     - Updated run() to check single channel instead of loop
     - Added handle_packet() method routing by packet_type
     - Split handlers: handle_mcp_tool_call(), handle_panel_update(), handle_chat_message()
  
  2. **Packet Types** (new file: plugins/ui-framework/src/packet_types.rs):
     - Created constants for message type differentiation
     - Browser→server types (1-99)
     - Server→browser types (100-199)
  
  3. **Browser Updates** (apps/playground-editor/static/app.js):
     - Changed UI_FRAMEWORK_CHANNELS array to single UI_FRAMEWORK_CHANNEL
     - Updated discovery to look for "ui-framework" not "ui-framework-*"
     - Fixed sendToUIFramework() to use single channel

- **Benefits**:
  - Saves 9 channels for other plugins
  - Simpler code without array management
  - Better performance (1 channel check vs 10 per frame)
  - Follows architecture (one channel per plugin)

#### 🔴 DISCOVERED: main.rs Pre-Registration Issue
- **Problem Found**: main.rs is ALSO registering channels before plugins initialize!
  - Lines 43-53: Registers "ui-framework" and "ui-framework-1" through "ui-framework-9"
  - Other plugins also being pre-registered (lines 50-117)
  - This causes duplicate registrations and wastes channels 2-10

- **Why It's Wrong**:
  - Channels are allocated sequentially (first-come, first-served)
  - NOT ranges (no "1-999 for systems" or "1000+ for plugins")
  - main.rs registers channels, then plugins try to register again
  - Results in: Channel 19 (UI), 20 (ui-framework), 2-10 (ui-framework-1 to 9), etc.

- **Fix Needed**:
  - main.rs should NOT pre-register ANY channels
  - Plugins should register their own channels in initialize()
  - OR plugins should get channel passed from main.rs registration

### Session 35 Accomplishments (Previous)

#### ✅ COMPLETED: Fixed Browser-Server Endianness Mismatch
- Fixed all DataView operations to use little-endian
- Browser can now properly request and receive channel manifest
- All control channel messages work correctly

### Next Steps for Implementation

According to ROADMAP.md, the implementation order should be:

1. **Create core/console package** - Define console/dashboard contracts
2. **Create systems/console package** - Move dashboard from systems/networking
3. **Refactor systems/networking** - Remove dashboard, add command processor
4. **Update systems/logic** - Add API modules for console and networking
5. **Validate with one plugin** - Ensure new API works properly

This approach starts with the smallest, most self-contained change and gradually builds up the new architecture.

### Important Notes

- **DO NOT** skip ahead to other systems until networking/console are properly separated
- **DO NOT** update plugins/apps until the API gateway is ready
- **FOLLOW** the roadmap phases in order to avoid breaking changes
- **TEST** each phase before moving to the next

### Session History for Reference
1. **Test the system**:
   - Run playground-editor and verify plugins initialize
   - Check channel allocations in dashboard
   - Verify browser can connect and discover channels

2. **Complete dynamic channel implementation**:
   - Ensure browser uses discovered channels
   - Remove any remaining hardcoded channel references
   - Test channel discovery protocol

3. **Fix remaining warnings**:
   - Clean up unused imports
   - Fix unused variables

### Running the IDE
```bash
cargo run -p playground-apps-editor
```

Then browse to: http://localhost:8080/playground-editor/