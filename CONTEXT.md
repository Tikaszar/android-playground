# CONTEXT.md - Current Session Context

## Active Session - 2025-08-20 (Evening)

### Current Status
**RENDERING ARCHITECTURE IN PROGRESS** - Creating proper rendering pipeline for UI

### What's Being Done Right Now
- **Created core/rendering package** ✅
  - Base rendering traits (Renderer, RenderTarget, CommandEncoder)
  - RenderCommand enum with all drawing operations
  - RenderCommandBatch for efficient frame batching
  - Proper error handling with RenderError
  - Package added to workspace

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

### Issues Being Fixed
1. **Black Screen** - No render command generation or execution
2. **Missing core/rendering** - Now created with base traits
3. **Browser using Canvas2D** - Will switch to WebGL
4. **No Discord UI** - Will implement in UI Framework Plugin

### Next Immediate Steps (Next Session)
1. Update systems/rendering to use core/rendering traits
2. Split systems/ui/system.rs into smaller files
3. Add render command generation to UiSystem
4. Implement create_discord_ui in UI Framework Plugin
5. Update browser to use WebGL renderer
6. Fix client tracking in Dashboard

### Build Command
```bash
cargo run -p playground-apps-editor  # Currently builds successfully
```

### Key Architecture Points
- Apps → Plugins → Systems → Core (strict layering)
- Browser is pure view, server has all logic
- Render commands sent via WebSocket channel 10
- 60fps frame batching for efficiency