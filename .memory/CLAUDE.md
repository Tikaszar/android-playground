# Claude Memory File

## Project Overview
Mobile-first plugin-based game engine built entirely on Android in Termux.

## Architecture
- **Core crates**: Foundational functionality
  - `types`: Shared types and traits
  - `android`: Android JNI bindings
  - `server`: Web server (Axum-based)
  - `plugin`: Plugin system with hot-reload
  
- **Systems layer**: Game engine components
  - `ui`: Immediate mode GUI / DOM rendering
  - `networking`: WebSocket, WebRTC
  - `physics`: 2D/3D physics engine
  - `logic`: ECS, state machines
  - `rendering`: WebGL/Canvas abstraction
  
- **Plugins**: Actual games/apps
  - `idle-game`: First game to publish
  - `playground-editor`: In-browser development

## Development Environment
- 100% mobile development in Termux
- Hot-reload for rapid iteration
- Browser-based code editor
- Git/GitHub integration

## Current Status
- Repository created: https://github.com/Tikaszar/android-playground
- Initial structure in place
- Ready to implement core crates