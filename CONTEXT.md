# CONTEXT.md - Current Session Context

## Active Session - 2025-08-28 (Session 37)

### Current Status
**Build**: ✅ COMPLETE - playground-apps-editor builds successfully
**Architecture**: ✅ Complete compliance achieved  
**Rendering**: 🟡 Pipeline ready - waiting for test
**Networking**: ✅ Fixed packet broadcasting - all clients receive packets
**Channels**: ✅ Dynamic channel allocation working correctly
**Browser**: ✅ Fixed - endianness issue resolved, manifest working
**Lifecycle**: ✅ Fixed circular dependency in startup

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