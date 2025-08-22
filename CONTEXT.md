# CONTEXT.md - Current Session Context

## Active Session - 2025-08-22 (Session 7)

### Current Status
**Major Architecture Refactor** - Replaced WebSocket-based internal communication with MessageBus system

### What Was Done This Session (2025-08-22 - Session 7)
- **Identified Client #0 Issue** ✅
  - Client #0 was NetworkingSystem connecting back to its own server via WebSocket
  - Systems were using WebSocket for internal communication (architectural violation)
  - Packets were being serialized/deserialized unnecessarily
  
- **Implemented MessageBus Architecture** ✅
  - Created MessageBus in core/ecs for internal system communication
  - Added GameMessageBus to systems/logic for plugin/app messaging
  - Created MessageBridge in core/server to bridge internal messages to WebSocket clients
  - Systems now communicate via direct memory instead of WebSocket
  
- **Updated Systems to Use MessageBus** ✅
  - UiSystem now publishes to MessageBus instead of NetworkingSystem.send_packet()
  - NetworkingSystem WebSocketClient removed (partially - needs completion)
  - Direct packet broadcast fix for WebSocket clients
  
- **Fixed WebSocket Broadcasting** ✅
  - Fixed issue where only first client received packets
  - Implemented direct broadcast to all WebSocket clients
  - Browser now connects as Client #1, #2, etc.
  
- **Remaining Tasks** ⚠️
  - Complete NetworkingSystem refactor (remove all WebSocketClient references)
  - Wire up MessageBridge in server initialization
  - Test that browser receives packets via new architecture

### Previous Session (2025-08-21 - Session 5)
- **Created complete WebGL renderer for browser** ✅
  - webgl/context.js - WebGL2 context management
  - webgl/shaders.js - Shader compilation and programs  
  - webgl/buffers.js - Vertex/index buffer batching
  - webgl/renderer.js - Command execution engine
  - webgl/textures.js - Texture management
  - webgl/text.js - Canvas-based text rendering
  
- **Completed render pipeline** ✅
  - Added render() method to UiSystem that generates RenderCommandBatch
  - Created batch_manager for frame-based command batching
  - Wired up 60fps render loop in SystemsManager
  - Updated app.js to use WebGL instead of Canvas2D
  
- **Fixed compilation issues** ✅
  - Removed duplicate render() methods
  - Fixed RenderCommandBatch frame_id parameter
  - Adjusted for missing networking APIs (send_packet)
  
- **Remaining issue** ⚠️
  - ECS spawn_batch doesn't work with trait objects (dyn Component)
  - Need to fix entity creation to avoid trait object type erasure
  - Created UiInterface in systems/logic for clean plugin access
  
- **Established proper render pipeline architecture** ✅
  - Created RenderingInterface in systems/logic 
  - Updated SystemsManager to expose UI and rendering interfaces
  - Plugins now use systems/logic World, not their own ECS
  - Clean separation between plugin state and UI system internals
  
- **Updated UI Framework Plugin** ✅
  - Removed playground-core-ecs dependency completely
  - Changed to use UiInterface instead of direct UiSystem access
  - Uses high-level create_discord_layout() method
  - Now compiles successfully with zero errors

### Architecture Rules Clarified
- **NO unsafe code** - Ever
- **NO std::any::Any** - Use enums and serialization
- **NO super** - Explicit trait implementations only
- **NO turbofish** - Use ComponentId instead
- **Files under 1000 lines** - Split large files into directories
- **lib.rs/mod.rs exports only** - No implementation code
- **Systems use core/ecs internally** for state management
- **Plugins ARE Systems** in systems/logic World
- **systems/logic manages ALL systems** including plugins

### Rendering Architecture Plan
```
core/rendering (DONE)
  ↓ defines base contracts
systems/rendering 
  ↓ implements WebGL/Vulkan renderers
systems/ui
  ↓ generates RenderCommands
UI Framework Plugin
  ↓ creates Discord-style UI
Browser (app.js)
  ↓ executes commands via WebGL
```

### Current Compilation Status
- ✅ **Core layer** - All packages compile successfully
- ✅ **Systems layer** - All packages compile successfully
- ✅ **Plugins layer** - UI Framework plugin compiles successfully
- ✅ **Apps layer** - playground-editor builds successfully

### Next Immediate Steps (Session 6)
1. **Fix ECS entity spawning** - Resolve trait object type erasure issue
   - Change spawn_batch to avoid dyn Component type names
   - Use add_component_raw with explicit ComponentIds
2. **Connect networking for render commands** - Send via channel 10
   - Fix WebSocket packet sending from UI to browser
   - Ensure channel manager is properly connected
3. **Test Discord UI rendering** - Verify visual output
   - Debug WebGL command execution
   - Check render command generation

### Build Command
```bash
cargo run -p playground-apps-editor  # Currently builds successfully
```

### Key Architecture Points
- Apps → Plugins → Systems → Core (strict layering)
- Browser is pure view, server has all logic
- Render commands sent via WebSocket channel 10
- 60fps frame batching for efficiency