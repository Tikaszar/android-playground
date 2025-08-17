# Idle Game Plugin

A simple idle/incremental game plugin for Android Playground.

## Overview

This plugin implements a basic idle game where players accumulate currency over time and can purchase generators to increase their production rate. It demonstrates the plugin system's capabilities including state management, hot-reload support, and UI rendering.

## Features

- **Passive Income** - Currency accumulates automatically over time
- **Generators** - Purchase upgrades to increase production
- **Multipliers** - Boost overall production rates
- **State Persistence** - Progress saves automatically
- **Hot Reload Support** - Game state preserved during development

## Game Mechanics

### Currency
The primary resource that accumulates over time based on owned generators.

### Generators
Automated production units that can be purchased and upgraded:
- Each generator has a base cost and production rate
- Cost increases with each purchase
- Production stacks with quantity owned

### Multipliers
Global modifiers that affect all production rates.

## Plugin Structure

```
src/
├── lib.rs      # Plugin entry point and exports
├── plugin.rs   # Main plugin implementation
├── state.rs    # Game state and data structures
└── mod.rs      # Module declarations
```

## Building

```bash
# Build the plugin as a dynamic library
cargo build -p idle-game --release

# The plugin .so file will be in:
# target/release/libdle_game.so
```

## Integration

The plugin implements the standard `Plugin` trait and can be loaded by the Android Playground engine:

```rust
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(IdleGame::new())
}
```

## State Management

Game state is automatically serialized and can be restored during hot-reloads:
- Currency balance
- Owned generators
- Active multipliers
- All progress is preserved

## Dependencies

- `playground-plugin` - Plugin trait definitions
- `playground-types` - Core type system
- `serde` - State serialization
- `tracing` - Logging support