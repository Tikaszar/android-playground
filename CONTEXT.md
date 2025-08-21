# CONTEXT.md - Current Session Context

## Active Session - 2025-08-21

### Current Status
**WebGL renderer created and compiling** - Ready for browser integration

### What Was Done This Session (2025-08-21)
- **Fixed core/ecs compilation errors** ✅
  - Fixed all Shared<T> migration issues
  - Fixed HashMap iteration (no .key()/.value() methods on tuples)
  - Fixed async/await propagation issues
  - Removed remaining dashmap references

- **Created systems/webgl package** ✅
  - Full WebGL2 renderer implementation
  - Implements core/rendering::Renderer trait
  - Vertex/index buffer batching system
  - Support for all RenderCommand types
  - Transform and clip rect stacks
  - Shader and texture management infrastructure

- **Fixed exports in core/rendering** ✅
  - Added Viewport export
  - Added RendererCapabilities export
  - Ensured all needed types are accessible

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
1. Update systems/ui to use core/rendering types
2. Integrate WebGLRenderer into systems/ui
3. Implement WebGL rendering in browser (update app.js)
4. Fix UI rendering (black screen issue)
5. Test render command generation and execution
6. Add pooling/recycling to GC for efficiency

### Build Command
```bash
cargo run -p playground-apps-editor  # Currently builds successfully
```

### Key Architecture Points
- Apps → Plugins → Systems → Core (strict layering)
- Browser is pure view, server has all logic
- Render commands sent via WebSocket channel 10
- 60fps frame batching for efficiency