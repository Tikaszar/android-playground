# CONTEXT.md - Current Session Context

## Active Session - 2025-08-21 (Session 2)

### Current Status
**UI system restructured, ECS fixed for mutable component access** - Compilation issues remain

### What Was Done This Session (2025-08-21 - Session 2)
- **Fixed UI directory structure violation** ✅
  - Split monolithic files into proper directory modules
  - components/ with element, layout, style, input, text modules
  - input/ with event, manager, gestures modules  
  - layout/ with engine, flexbox, absolute, docking modules
  - rendering/ with converter, element_renderer modules
  - terminal/ with manager, emulator modules
  - mobile/ with features, floating_toolbar modules
  - theme/ with types, colors, manager modules
  - All files now under 1000 lines as required

- **Fixed core/ecs for mutable component access** ✅
  - Changed storage to use Shared<ComponentBox> instead of ComponentBox
  - Added get_component_mut method to World
  - Added get_raw_mut to ComponentStorage trait
  - Updated SparseStorage and DenseStorage implementations
  - Added ComponentInUse error for removal conflicts
  - core/ecs now compiles successfully

- **Updated Component implementations** ✅
  - Fixed all UI components to use async trait methods
  - Proper serialize/deserialize with Bytes
  - Using TypeId for component IDs

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

### Remaining Issues
1. **Type mismatch with get_component_mut** - Returns Shared<ComponentBox> but need typed access
2. **Missing methods in UI system** - despawn_batch, send_to_channel
3. **Cannot access component fields directly** - Need proper casting from ComponentBox

### Next Immediate Steps (Next Session)
1. Fix get_component_mut to return typed components
2. Update UI system to properly use mutable component access
3. Fix remaining compilation errors in systems/ui
4. Test full compilation of playground-editor
5. Implement WebGL rendering in browser (update app.js)
6. Fix UI rendering (black screen issue)

### Build Command
```bash
cargo run -p playground-apps-editor  # Currently builds successfully
```

### Key Architecture Points
- Apps → Plugins → Systems → Core (strict layering)
- Browser is pure view, server has all logic
- Render commands sent via WebSocket channel 10
- 60fps frame batching for efficiency