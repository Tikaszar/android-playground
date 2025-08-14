# playground-server

Web server for the Android Playground in-browser development environment.

## Overview

This crate provides an Axum-based web server that hosts the browser-based code editor and development tools for Android Playground. It enables touch-friendly development directly on Android devices through a web interface.

## Features

- **Web-based Code Editor** - Full-featured code editing in the browser
- **File Management** - Browse, create, edit, and delete project files
- **Plugin Hot Reload** - Trigger plugin rebuilds and reloads from the browser
- **Live Preview** - See game changes in real-time
- **WebSocket Support** - Real-time communication with running plugins
- **Touch Optimized** - UI designed for mobile browsers and touch input

## Architecture

The server consists of:
- **HTTP Server** - Serves the editor UI and static assets
- **API Endpoints** - RESTful APIs for file operations and plugin management
- **WebSocket Handler** - Real-time updates and live reload
- **File Watcher** - Monitors changes for auto-reload

## Running the Server

```bash
# Start the development server
cargo run -p playground-server

# The server will be available at:
# http://localhost:8080
```

## API Endpoints

- `GET /` - Serve the editor UI
- `GET /api/files` - List project files
- `GET /api/files/:path` - Read file contents
- `POST /api/files/:path` - Save file contents
- `DELETE /api/files/:path` - Delete a file
- `POST /api/plugins/reload` - Trigger plugin hot reload
- `WS /ws` - WebSocket connection for live updates

## Configuration

The server can be configured via environment variables:
- `PORT` - Server port (default: 8080)
- `HOST` - Bind address (default: 0.0.0.0)
- `WATCH_DIR` - Directory to watch for changes

## Dependencies

- `axum` - Web framework
- `tower` - Service utilities
- `tokio` - Async runtime
- `serde_json` - JSON serialization