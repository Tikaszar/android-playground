# playground-types

Core type definitions and traits for the Android Playground game engine.

## Overview

This crate provides the fundamental types and traits used throughout the Android Playground ecosystem. It has zero dependencies to ensure maximum compatibility and minimal compilation overhead.

## Key Types

- `PluginId` - Unique identifier for plugins
- `PluginMetadata` - Plugin information (id, name, version)
- `Version` - Semantic versioning support
- `Context` - Shared runtime context for plugins
- `Event` - Event system types
- `RenderContext` - Rendering context for UI operations
- `PluginError` - Error handling types

## Usage

```rust
use playground_types::{PluginId, Version, Event};

// Create a plugin ID
let id = PluginId("my-plugin".to_string());

// Define a version
let version = Version {
    major: 1,
    minor: 0,
    patch: 0,
};
```

## Design Principles

- Zero dependencies for fast compilation
- Shared by all plugins and systems
- Serializable for state persistence
- Thread-safe where necessary