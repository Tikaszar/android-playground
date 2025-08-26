# CONTEXT.md - Current Session Context

## Active Session - 2025-08-26 (Session 28)

### Current Status
**Build**: ✅ FULLY COMPILING (warnings only)
**Architecture**: ✅ Complete compliance achieved
**Channel System**: 🔄 Planning dynamic channel allocation

### Major Architecture Discovery - Session 28

#### App and Browser are ONE Application
- Server-side: playground-editor (Rust app) 
- Client-side: Browser (WebGL renderer)
- They are two sides of the SAME distributed application
- Browser is the App's frontend, not a separate entity

#### Dynamic Channel Architecture Planned
- **Channel 0**: ONLY hardcoded channel (control/discovery)
- **All other channels**: Dynamically allocated by SystemsManager
- **No ranges, no categories**: Pure sequential assignment
- **Browser learns channels**: Via manifest on channel 0

### Current Architecture State

#### Plugin System 
- All 9 IDE plugins properly implement `systems/logic::System`
- Plugins are self-contained with no inter-dependencies
- App (playground-editor) coordinates all plugins
- Channels will be dynamically assigned (not hardcoded)

#### Planned Channel System
- Control channel: 0 (hardcoded)
- All systems/plugins: Dynamically assigned (1, 2, 3, ...)
- Browser discovers all channels on connect
- Complete flexibility for adding/removing components

### Implementation Plan for Session

1. **Update SystemsManager** - Add dynamic channel registry
2. **Implement Channel Discovery** - Protocol on channel 0
3. **Fix UiSystem Rendering** - Send RenderCommandBatch on assigned channel
4. **Update Browser Client** - Dynamic channel subscription
5. **Remove Hardcoded Channels** - From all plugins and systems

### What's Working
- Complete 4-layer architecture: Apps → Plugins → Systems → Core
- All architectural rules enforced (NO dyn, NO unsafe, Handle/Shared pattern)
- playground-editor loads and coordinates all 9 IDE plugins
- Systems initialization through SystemsManager
- 60fps update loop with proper System execution

### Next Steps After Channel System
1. Complete render pipeline (UiSystem → Browser)
2. Implement actual plugin functionality
3. Wire up Discord-style UI rendering
4. Add terminal PTY support
5. Implement MCP integration in Chat Assistant

### Key Understanding This Session
- Browser uses channels to communicate with ALL systems/plugins
- systems/logic is the orchestrator for Apps
- Browser connects via core/server but then uses channels
- UI Framework Plugin is just another plugin (not special)

### Running the IDE
```bash
cargo run -p playground-apps-editor
```

Then browse to: http://localhost:8080/playground-editor/