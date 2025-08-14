# playground-plugin

Plugin system infrastructure for the Android Playground game engine.

## Overview

This crate provides the core plugin system that enables dynamic loading, hot-reload, and lifecycle management for Android Playground plugins.

## Features

- **Dynamic Loading** - Load plugins from `.so` files at runtime
- **Hot Reload** - Replace plugin implementations without restarting
- **State Preservation** - Save and restore plugin state across reloads
- **Lifecycle Hooks** - Structured initialization and cleanup
- **Message Passing** - Inter-plugin communication via context

## Core Traits

### Plugin
```rust
pub trait Plugin {
    fn metadata(&self) -> &PluginMetadata;
    fn on_load(&mut self, ctx: &mut Context) -> Result<(), PluginError>;
    fn on_unload(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context, delta_time: f32);
    fn render(&mut self, ctx: &mut RenderContext);
    fn on_event(&mut self, event: &Event) -> bool;
}
```

### Stateful
```rust
pub trait Stateful {
    type State: Serialize + DeserializeOwned;
    fn save_state(&self) -> Self::State;
    fn restore_state(&mut self, state: Self::State);
}
```

## Plugin Entry Point

Every plugin must export a `create_plugin` function:

```rust
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(MyPlugin::new())
}
```

## Usage

```rust
use playground_plugin::{Plugin, Stateful};
use playground_types::{Context, Event};

struct MyPlugin {
    // Plugin state
}

impl Plugin for MyPlugin {
    // Implement required methods
}
```