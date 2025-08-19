# CONTEXT.md - Current Session Context

## Active Session - 2025-08-19

### Current Task
Reorganizing documentation and fixing architecture violations from last session.

### Where We Left Off (2025-12-21)
**Status**: Architecture violations identified, build partially working

#### Completed Last Session
- ✅ Package naming standardization (playground-core-*, playground-systems-*, etc.)
- ✅ ECS Query API - removed turbofish, uses `.with_component(ComponentId)`
- ✅ Plugin async traits with proper Context type
- ✅ Fixed many build issues (duplicate types, imports, Result alias)

#### Architecture Violations to Fix
1. **apps/playground-editor directly uses core/server** ❌
   - Currently imports and starts server directly
   - Should ONLY use systems/logic
   - NetworkingSystem should manage core/server internally

2. **NetworkingSystem expects server already running** ❌
   - Should start core/server internally during initialization
   - Apps shouldn't know about core/server

3. **systems/logic needs to expose all system APIs** ❌
   - SystemsManager should provide access to all systems
   - Plugins need these APIs through Context

4. **MCP router state type mismatch** ❌
   - Router state type issues between MCP and main router
   - Need proper WebSocketState passing

5. **Compilation errors remain** ❌
   - Handler trait bounds in playground-editor
   - WebSocketHandler constructor in ui-framework
   - Minor plugin issues

### Git Status
```
M core/server/src/mcp/sse_handler.rs
M core/server/src/mcp/streamable_http.rs
```

### Next Steps (This Session)
1. ✅ Compress CONTEXT.md to current session only
2. ⏳ Update CLAUDE.md with memory/rules/goals
3. ⏳ Update README.md with clear instructions
4. ⏳ Fix architecture violations
5. ⏳ Get build fully working

### Quick Reference
- **Correct Architecture**: Apps → systems/logic → all systems → core
- **ECS Usage**: core/ecs for Systems internal, systems/logic for Plugins/Apps
- **No Turbofish**: Use `.with_component(ComponentId)` everywhere
- **Thread Safety**: Arc<RwLock<>> pattern throughout