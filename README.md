# Android Playground

This is a personal playground for building and testing game engine concepts on Android using AI agents. The project is designed for rapid prototyping and experimentation with a mobile-first, plugin-based architecture.

## Purpose

This repository is used as a development environment for AI agents to build and test designs. It is not intended for public use or collaboration.

## Getting Started

### Prerequisites

- [Termux](https://termux.com/) for a terminal environment on Android.
- [Rust](https://rustup.rs/) for building the engine and plugins.

### Building the Project

```bash
# Clone the repository
git clone https://github.com/Tikaszar/android-playground.git
cd android-playground

# Build all crates
cargo build --workspace
```

## Architecture

The engine is built with a modular, crate-based architecture. The core components are separated into a `core` layer, with various `systems` providing engine functionality. Games and applications are implemented as `plugins`.

For more detailed information, please see the `GEMINI.md` file.
