# CONTEXT.md - Current Session Context

## Active Session - 2025-09-10 (Session 43)

### Current Status
**Build**: ✅ COMPLETE - playground-apps-editor builds successfully with 0 errors
**Architecture**: 🔄 REFACTORING - Implementing new unified ECS design  
**Unified ECS**: 🔴 NOT STARTED - Need to create systems/ecs package
**API Gateway**: 🔴 NOT STARTED - systems/logic needs refactoring to API-only
**Rendering**: 🟡 Pipeline ready - waiting for test
**Networking**: ✅ Fixed packet broadcasting - all clients receive packets
**Channels**: ✅ Dynamic channel allocation working correctly
**Browser**: ✅ Fixed - endianness issue resolved, manifest working
**Lifecycle**: ✅ Fixed circular dependency in startup
**Logging**: ✅ Component-specific logging fully implemented across codebase

### Session 43 Goals - New Architecture Implementation

#### 🔄 IN PROGRESS: Unified ECS Architecture Refactor

Based on DESIGN_CLARIFICATION.md, implementing major architectural changes:

- **Core Layer Changes**:
  1. core/ecs becomes contract-only (traits, no implementation)
  2. Remove all stateful code from core/*
  3. Define only interfaces and contracts
  
- **New Unified ECS System**:
  1. Create systems/ecs package with unified World implementation
  2. Single ECS for entire engine (replaces dual ECS design)
  3. Implements core/ecs contracts
  4. Manages all system scheduling and execution
  
- **Systems/Logic Refactor**:
  1. Convert to pure API gateway (stateless)
  2. Remove ECS implementation (moves to systems/ecs)
  3. Provide only public-facing types and functions
  4. Hide all core/* and systems/* from plugins/apps
  
- **System Registration Flow**:
  1. Engine systems auto-register with systems/ecs
  2. Apps explicitly register plugins via systems/logic API
  3. Compile-time manifest for engine system discovery
  4. Two-stage setup: engine systems then plugins

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

### Next Steps for Session 38
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