# CLAUDE.md - AI Agent Memory

This file contains critical memory for Claude Code when working with this repository.

## #strict-rules
- **NO unsafe code** - Never use `unsafe` blocks anywhere
- **NO std::any::Any** - Avoid runtime type casting
- **NO super** - Explicit trait implementations only
- **NO turbofish** - Use `.with_component(ComponentId)` instead of `::<T>`
- **NO TODOs** - Complete all implementations fully
- **NO incomplete code** - Everything must compile and work
- **Shared<T> for concurrency** - Use our Shared<T> type (Arc<tokio::sync::RwLock<T>>)
- **tokio::sync::RwLock ONLY** - NEVER use parking_lot::RwLock (Send issues)
- **NO DashMap** - Use Shared<HashMap> instead
- **Result everywhere** - All fallible operations return Result<T, Error>
- **Async by default** - All I/O must be async
- **Batch operations** - APIs operate on collections, not singles
- **NO dangling imports** - Remove unused imports immediately
- **Async propagates** - If a function uses .await, it must be async
- **Files under 1000 lines** - Split large files into directories
- **lib.rs/mod.rs exports only** - No implementation code in these files

## #architecture-rules
- Apps → Plugins → Systems → Core (STRICT LAYERING)
- Apps are THE AUTHORITY - control flow, state, and timing
- Apps create systems/logic which initializes ALL other systems
- Plugins provide reusable features using Systems APIs
- Systems provide generic engine capabilities
- Core provides foundation (server, ecs, types)
- Plugins customize Systems for specific use cases (e.g., Discord UI)

## #package-naming
- Core: `playground-core-*` (e.g., playground-core-ecs)
- Systems: `playground-systems-*` (e.g., playground-systems-ui)
- Plugins: `playground-plugins-*` (e.g., playground-plugins-inventory)
- Apps: `playground-apps-*` (e.g., playground-apps-editor)

## #ecs-rules
- Systems use `core/ecs` for internal state management
- Plugins/Apps use `systems/logic` for game logic
- NO playground_ecs (doesn't exist)
- Query API: `.with_component(ComponentId)` - NO TURBOFISH!
- systems/logic is SPECIAL - initializes all other systems
- Plugins ARE systems/logic Systems (NOT core/ecs) - implement systems/logic::System trait
- **Plugins MUST NOT use core/ecs directly** - only through systems/logic
- UiSystem internal ECS is private - plugins use UiInterface from systems/logic

## #current-violations
None! All architecture violations fixed ✅

## #immediate-goals
1. ~~Fix UI rendering~~ - COMPLETED! ✅
   - WebGL Clear command working (grey background)
   - WebGL DrawQuad working (red rectangle)
   - Server-controlled renderer initialization
   - Resource caching for shaders/textures
   - Clean shutdown protocol
2. Implement Discord UI layout (next priority)
   - Fix ECS entity spawning for UI elements
   - Generate proper render commands from UI tree
3. Fix client tracking in Dashboard (remove disconnected)
4. Design and implement idle-mmo-rpg game mechanics

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
// Shared type for concurrency
use playground_systems_logic::{Shared, shared}; // For plugins/apps
use playground_core_types::{Shared, shared};    // For core/systems

let data: Shared<HashMap<String, Value>> = shared(HashMap::new());
let guard = data.read().await;  // or write().await

// ECS Query (NO TURBOFISH!)
query.with_component(Position::component_id())
     .with_component(Velocity::component_id())

// Plugin async trait
#[async_trait]
impl Plugin for MyPlugin {
    async fn update(&mut self, ctx: &Context) -> Result<()>
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
- 8 IDE plugins created with structure
- WebSocket multiplexer with binary protocol (FIXED!)
- MCP server with tool system (WORKING!)
- UI Framework Plugin (3000+ lines) - NOW USES systems/logic!
- Package naming (all standardized)
- ECS Query API (no turbofish)
- Plugin async traits (all async)
- Build system (FULLY COMPILING!)
- All RwLock converted to tokio::sync
- 100+ functions made async
- Browser WebSocket connection (FIXED!)
- Terminal dashboard (unified in core/server)
- Dashboard owned by server, accessed by systems
- File logging for verbose output
- playground-editor IDE builds and runs!
- UI Framework Plugin architecture fixed (uses UiInterface)
- Render pipeline architecture established

🟡 **Needs Testing**:
- Discord UI layout implementation  
- Multiple concurrent clients
- IDE plugin functionality

🔴 **Not Started**:
- Game plugins (inventory, combat, etc.)
- idle-mmo-rpg game design and implementation
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
- Only the User csn execute cargo run. Always include usage instructions