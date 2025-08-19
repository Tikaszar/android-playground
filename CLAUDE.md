# CLAUDE.md - AI Agent Memory

This file contains critical memory for Claude Code when working with this repository.

## #strict-rules
- **NO unsafe code** - Never use `unsafe` blocks anywhere
- **NO std::any::Any** - Avoid runtime type casting
- **NO turbofish** - Use `.with_component(ComponentId)` instead of `::<T>`
- **NO TODOs** - Complete all implementations fully
- **NO incomplete code** - Everything must compile and work
- **Arc<RwLock<>>** - Use consistently for thread safety
- **Result everywhere** - All fallible operations return Result<T, Error>
- **Async by default** - All I/O must be async
- **Batch operations** - APIs operate on collections, not singles
- **NO dangling imports** - Remove unused imports immediately

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
1. apps/playground-editor directly uses core/server (WRONG)
2. NetworkingSystem expects server already running (should start internally)
3. systems/logic needs to expose all system APIs
4. MCP router state type mismatch issues
5. Compilation errors in Handler traits and WebSocketHandler

## #immediate-goals
1. Fix architecture violations above
2. Get playground-apps-editor fully compiling
3. Test MCP integration with Claude Code
4. Verify UI Framework Plugin receives channel 1200 messages

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
- WebSocket multiplexer with binary protocol
- MCP server with tool system
- UI Framework Plugin (3000+ lines)

🟡 **Partial**:
- Package naming (done but build issues)
- ECS Query API (no turbofish done)
- Plugin async traits (done)
- Build system (many fixes, some remaining)

🔴 **Needs Work**:
- Architecture violations (5 identified)
- Full compilation of all packages
- MCP-LLM connection testing

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