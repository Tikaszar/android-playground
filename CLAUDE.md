# CLAUDE.md - AI Agent Memory

This file contains critical memory for Claude Code when working with this repository.

## #strict-rules
- **NO unsafe code** - Never use `unsafe` blocks anywhere
- **NO std::any::Any** - Avoid runtime type casting
- **NO super** - Explicit trait implementations only
- **NO turbofish** - Use `.with_component(ComponentId)` instead of `::<T>`
- **NO TODOs** - Complete all implementations fully
- **NO incomplete code** - Everything must compile and work
- **NO dyn** - Use concrete types/base classes instead of trait objects
- **NO enums for type erasure** - Use concrete wrapper types instead
- **Handle<T> for external refs** - Use Handle<T> (Arc<T>) for referencing objects with internal state
- **Shared<T> for internal state** - Use Shared<T> (Arc<RwLock<T>>) ONLY for private fields
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
**NetworkingSystem/UiSystem interface** - Handle vs Shared mismatch 🟡 (last remaining)

## #ecs-architecture-fix
**CRITICAL**: UiSystem uses `Arc<World>` not `Shared<World>`
- World internally has Shared<> fields for its own locking
- Adding another layer with Shared<World> causes nested lock deadlocks
- Solution: `world: Arc<World>` in UiSystem
- World's methods handle all internal locking

## #immediate-goals
1. ~~Fix systems/logic dyn violations~~ ✅ COMPLETED (Session 21-22)
   - ALL files fixed: archetype, entity, event, messaging, query, storage, system, etc.
   - Concrete wrapper pattern applied consistently
2. ~~Fix systems/networking type aliases~~ ✅ COMPLETED (Session 18)
3. ~~Fix Component/ComponentData pattern~~ ✅ COMPLETED (Session 20)
4. ~~Fix systems/logic NO turbofish~~ ✅ COMPLETED (Session 22)
   - ALL TypeId usage removed, replaced with string-based IDs
   - No more turbofish syntax anywhere
5. ~~Fix core/ecs NO dyn violations~~ ✅ COMPLETED (Session 23)
   - messaging.rs completely refactored
   - All other files already compliant
6. ~~Fix core/server Handle/Shared compliance~~ ✅ COMPLETED (Session 24)
   - All Arc usage replaced with Handle/Shared type aliases
   - Documentation updated with proper patterns
7. **Fix NetworkingSystem/UiSystem interface mismatch** 🔴
   - Resolve Handle vs Shared type mismatch
8. Implement Discord UI layout
   - Fix ECS entity spawning for UI elements
   - Generate proper render commands from UI tree
9. Fix client tracking in Dashboard (remove disconnected)

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
// Handle<T> for external references, Shared<T> for internal state
use playground_core_types::{Handle, handle, Shared, shared};

// External reference to object with internal state
let world: Handle<World> = handle(World::new());
world.some_method().await;  // No .read().await needed!

// Internal mutable state (private fields only)
struct MyStruct {
    data: Shared<HashMap<String, Value>>,  // INTERNAL state
}
let guard = self.data.read().await;  // or write().await

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
- Core infrastructure (all 7 modules including core/ui)
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
- core/ui abstraction layer for UI systems
- Mobile Discord UI layout implementation

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
- Only the User can execute cargo run. Always include usage instructions
- ask the User to cargo run, never cargo run as an AI Agent
- No DYN or ENUMs in place of DYN
- **NEVER add migration ANYTHING - no migration code, notes, plans, or functions EVER**