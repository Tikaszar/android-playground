# CLAUDE.md - AI Agent Memory

This file contains critical memory for Claude Code when working with this repository.

## #strict-rules
- **NO unsafe code** - Never use `unsafe` blocks anywhere
- **NO std::any::Any** - Avoid runtime type casting
- **NO turbofish** - Use `.with_component(ComponentId)` instead of `::<T>`
- **NO TODOs** - Complete all implementations fully
- **NO incomplete code** - Everything must compile and work
- **tokio::sync::RwLock ONLY** - NEVER use parking_lot::RwLock (Send issues)
- **Result everywhere** - All fallible operations return Result<T, Error>
- **Async by default** - All I/O must be async
- **Batch operations** - APIs operate on collections, not singles
- **NO dangling imports** - Remove unused imports immediately
- **Async propagates** - If a function uses .await, it must be async

## #architecture-rules
- Apps → Plugins → Systems → Core (STRICT LAYERING)
- Apps create systems/logic which initializes ALL other systems
- Plugins use Systems APIs from App (NEVER create systems)
- Systems use Core APIs only (including core/ecs for internal state)
- Plugins may implement custom Systems internally that use Core

## #package-naming
- Core: `playground-core-*` (e.g., playground-core-ecs)
- Systems: `playground-systems-*` (e.g., playground-systems-ui)
- Plugins: `playground-plugins-*` (e.g., playground-plugins-inventory)
- Apps: `playground-apps-*` (e.g., playground-apps-editor)

## #ecs-rules
- Systems use `core/ecs` for internal state
- Plugins/Apps use `systems/logic` for game logic
- NO playground_ecs (doesn't exist)
- Query API: `.with_component(ComponentId)` - NO TURBOFISH!
- systems/logic is SPECIAL - initializes all other systems

## #current-violations
None! All architecture violations fixed ✅

## #immediate-goals
1. Test running application ✅ COMPILES!
2. Verify MCP integration with Claude Code
3. Test browser connection at http://localhost:8080/playground-editor/
4. Implement actual UI Framework Plugin functionality

## #architecture-fixed
✅ Plugins ARE Systems - no separate Plugin trait
✅ Plugin trait doesn't exist in Core (removed core/plugin)
✅ NetworkingSystem starts server internally
✅ Apps only use systems/logic, never core/server directly
✅ Axum version unified to 0.8 workspace-wide

## #channel-allocation
- 0: Control channel
- 1-999: Systems (UI on 10)
- 1000-1079: IDE plugins
- 1100-1199: Game plugins
- 1200-1209: UI Framework Plugin
- 2000-2999: LLM sessions via MCP

## #key-apis
```rust
// ECS Query (NO TURBOFISH!)
query.with_component(Position::component_id())
     .with_component(Velocity::component_id())

// Plugin async trait
#[async_trait]
impl Plugin for MyPlugin {
    async fn update(&mut self, ctx: &Context) -> Result<()>

// Thread-safe pattern
Arc<RwLock<HashMap<K, V>>>
```

## #test-commands
```bash
# Run IDE (everything in one command)
cargo run -p playground-apps-editor

# Browser URL
http://localhost:8080/playground-editor/

# MCP endpoint for LLMs
http://localhost:8080/mcp
```

## #documentation-structure
- **CONTEXT.md** - Current session state only
- **HISTORY.md** - Detailed session history and bug fixes
- **DESIGN_DECISIONS.md** - Architecture evolution and why
- **README.md** - User guide, build instructions, features
- **CLAUDE.md** - This file, AI agent memory

## #implementation-status
✅ **Complete**:
- Core infrastructure (all 6 modules)
- Systems (all 5 with ECS integration) 
- 18 plugins created with structure
- WebSocket multiplexer with binary protocol (FIXED!)
- MCP server with tool system (WORKING!)
- UI Framework Plugin (3000+ lines)
- Package naming (all standardized)
- ECS Query API (no turbofish)
- Plugin async traits (all async)
- Build system (FULLY COMPILING!)
- All RwLock converted to tokio::sync
- 100+ functions made async
- Browser WebSocket connection (FIXED!)
- Terminal dashboard with real-time monitoring
- File logging for verbose output

🟡 **Needs Testing**:
- UI Framework Plugin rendering
- Multiple concurrent clients
- Game plugin functionality

🔴 **Not Started**:
- Actual game logic implementation
- Production deployment

## #workflow
1. Read CONTEXT.md for current state
2. Check `git status` and recent commits
3. Review #strict-rules and #architecture-rules
4. Use TodoWrite to plan tasks
5. Complete implementations fully
6. Update CONTEXT.md with progress
7. Mark todos completed immediately

## #remember
- Always check current date with `date` command
- Check project age with `git log --reverse --format="%ai" | head -1`
- All development in Termux on Android
- Mobile-first, battery-efficient design
- Server-side authority (browser is pure view)
- Conversational IDE is primary interface