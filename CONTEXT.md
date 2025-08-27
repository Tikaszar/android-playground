# CONTEXT.md - Current Session Context

## Active Session - 2025-08-27 (Session 30)

### Current Status
**Build**: 🔴 IN PROGRESS - Splitting system.rs to comply with 1000-line rule
**Architecture**: ✅ Complete compliance achieved  
**Rendering**: ✅ UiSystem::render() sends RenderCommandBatch to browser

### Session 30 Accomplishments

#### Fixed Packet Queue Buildup Issue
- **Diagnosed root cause**: batcher.rs `get_batch()` drains queue, only one client gets packets
- **Issue**: 1489 packets queuing on channel 10, browser gets nothing
- **Solution needed**: Change from queue/drain to broadcast model for render packets

#### Splitting system.rs for Architecture Compliance
- **Violation**: system.rs was 1190 lines (exceeds 1000-line rule)
- **Created directory structure**: `/systems/ui/src/system/`
- **Files created**:
  - `mod.rs` - Exports only (per rules)
  - `core.rs` - UiSystem struct and basic methods
  - `init.rs` - Initialization and setup
  - `elements.rs` - Element management
  - `layout.rs` - Layout and dirty tracking
- **Still need**: rendering.rs, shaders.rs, ui_renderer_impl.rs
- **Old system.rs**: Needs deletion after split complete

### Key Architecture Rules Enforced
- **NO Search/Grep** - Read files directly only
- **Read files FULLY** in one pass
- **mod.rs/lib.rs ONLY export** - No implementation
- **Files under 1000 lines**
- **NO dyn, NO any, NO turbofish, NO unsafe**
- **Shared<> for internal, Handle<> for external**

### Next Steps
1. **Complete system.rs split**:
   - Create rendering.rs, shaders.rs, ui_renderer_impl.rs
   - Delete old system.rs
   - Verify compilation

2. **Fix packet broadcasting**:
   - Modify batcher to not consume packets
   - Implement proper broadcast for render channel
   - Test with multiple browser clients

3. **Verify Discord UI rendering**:
   - Check UI Framework Plugin creates elements
   - Ensure render_element_tree() works
   - Remove test red quad

### Running the IDE
```bash
cargo run -p playground-apps-editor
```

Then browse to: http://localhost:8080/playground-editor/

### Expected Behavior
- Browser should connect via WebSocket
- Currently: Packets queue up, nothing renders
- Goal: All clients receive render packets via broadcast