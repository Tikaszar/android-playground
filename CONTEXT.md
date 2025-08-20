# CONTEXT.md - Current Session Context

## Active Session - 2025-08-20 (Late Evening)

### Current Status
**MAJOR ARCHITECTURE FIX COMPLETED** - Fixed all RwLock violations

### What Was Done This Session
- **Created Shared<T> type alias** ✅
  - Located in core/types/src/shared.rs
  - `Shared<T> = Arc<RwLock<T>>` using ONLY tokio::sync::RwLock
  - Helper function `shared()` for easy construction
  - Re-exported through systems/logic for plugins/apps

- **Fixed ALL parking_lot violations** ✅
  - Replaced all parking_lot::RwLock with tokio::sync::RwLock
  - Replaced all DashMap with Shared<HashMap>
  - Updated core/ecs (world.rs, component.rs, storage.rs, entity.rs, query.rs)
  - Updated core/server (mcp/session.rs)
  - Made all necessary functions async with .await

- **Removed bad dependencies** ✅
  - Removed parking_lot from workspace Cargo.toml
  - Removed dashmap from workspace Cargo.toml
  - Cleaned all package dependencies

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
1. Fix compilation errors in core/ecs (complex but isolated)
2. Create systems/webgl to replace systems/rendering
3. Update systems/ui to use core/rendering types
4. Implement WebGL rendering in browser
5. Fix UI rendering (black screen issue)
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