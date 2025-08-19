# playground-systems-ui

Mobile-first UI system with flexbox layout, theming, gestures, and terminal integration.

## Overview

The UI System provides a complete framework for building responsive, touch-optimized user interfaces. It uses core/ecs internally for state management and integrates with the rendering system for display.

### Key Features
- Flexbox and absolute layout systems
- Docking panels for IDE-like interfaces
- Full touch gesture support with floating toolbar
- Theme system with hot-reload
- Terminal integration for Termux
- Text rendering with syntax highlighting
- WebSocket messaging for browser sync
- Component-based architecture via ECS
- Mobile-optimized with battery efficiency

## Architecture

### ECS-Based State Management
The UI system uses core/ecs internally to manage UI element state:

```rust
use playground_systems_ui::UiSystem;
use playground_systems_rendering::BaseRenderer;

// Create UI system with ECS backing
let mut ui = UiSystem::new();

// Initialize with a renderer
ui.initialize(renderer).await?;

// UI elements are entities with components
```

### Component System
All UI elements are entities with these components:
- `UiElementComponent` - Basic element properties
- `UiLayoutComponent` - Layout constraints and results
- `UiStyleComponent` - Visual styling
- `UiInputComponent` - Input handling
- `UiTextComponent` - Text content
- `UiDirtyComponent` - Change tracking

### Channel Communication
UI System uses channel 10 for WebSocket communication:
```
Channel 10: UI System
- Packet Type 1: CreateElement
- Packet Type 2: UpdateElement  
- Packet Type 3: DeleteElement
- Packet Type 4: InputEvent
- Packet Type 5: TerminalInput
- Packet Type 6: TerminalOutput
- Packet Type 10: RenderBatch
```

## Usage

### Basic Setup
```rust
use playground_systems_ui::{UiSystem, Element, ElementBounds};
use nalgebra::Vector2;

// Create and initialize UI system
let mut ui = UiSystem::new();
ui.initialize(renderer).await?;

// Set up WebSocket channel for browser sync
ui.set_channel(10, channel_manager, batcher)?;

// Create root container
let root = ui.create_element(Element::Container {
    bounds: ElementBounds {
        position: Vector2::new(0.0, 0.0),
        size: Vector2::new(1920.0, 1080.0),
    },
    children: vec![],
}).await?;
```

### Creating Elements
```rust
// Button element
let button = ui.create_element(Element::Button {
    text: "Click Me".to_string(),
    bounds: ElementBounds {
        position: Vector2::new(100.0, 100.0),
        size: Vector2::new(200.0, 50.0),
    },
    on_click: Some(Box::new(|| {
        println!("Button clicked!");
    })),
}).await?;

// Text element
let text = ui.create_element(Element::Text {
    content: "Hello World".to_string(),
    font_size: 16.0,
    color: Vector4::new(1.0, 1.0, 1.0, 1.0),
}).await?;

// Container with flexbox layout
let container = ui.create_element(Element::Container {
    layout: LayoutType::Flexbox {
        direction: FlexDirection::Row,
        justify: JustifyContent::SpaceBetween,
        align: AlignItems::Center,
    },
    children: vec![button, text],
}).await?;
```

### Layout Systems

#### Flexbox Layout
```rust
use playground_systems_ui::layout::{FlexboxLayout, FlexDirection, JustifyContent, AlignItems};

// Create flexbox container
let flex_container = FlexboxLayout {
    direction: FlexDirection::Column,
    justify: JustifyContent::Start,
    align: AlignItems::Stretch,
    wrap: FlexWrap::NoWrap,
    gap: 10.0,
};

// Apply to element
ui.set_layout(element_id, Layout::Flexbox(flex_container)).await?;

// Child elements can have flex properties
ui.set_flex_properties(child_id, FlexProperties {
    grow: 1.0,
    shrink: 0.0,
    basis: Auto,
    align_self: AlignSelf::Center,
}).await?;
```

#### Absolute Layout
```rust
use playground_systems_ui::layout::AbsoluteLayout;

// Position elements absolutely
let absolute = AbsoluteLayout {
    position: Position::Absolute,
    top: Some(100.0),
    left: Some(50.0),
    right: None,
    bottom: None,
    z_index: 10,
};

ui.set_layout(element_id, Layout::Absolute(absolute)).await?;
```

#### Docking Layout
```rust
use playground_systems_ui::layout::{DockingLayout, DockPosition};

// Create docking panels for IDE
let docking = DockingLayout::new();

// Add panels
docking.add_panel("sidebar", DockPosition::Left, 300.0)?;
docking.add_panel("terminal", DockPosition::Bottom, 200.0)?;
docking.add_panel("editor", DockPosition::Center, 0.0)?; // Fills remaining

// Enable drag-to-resize
docking.enable_resize_handles(true);
```

### Input Handling

#### Touch Gestures
```rust
use playground_systems_ui::input::{GestureRecognizer, GestureType};

// Register gesture handlers
ui.register_gesture(element_id, GestureType::Tap, |event| {
    println!("Tapped at {:?}", event.position);
}).await?;

ui.register_gesture(element_id, GestureType::Swipe, |event| {
    match event.direction {
        SwipeDirection::Left => println!("Swiped left"),
        SwipeDirection::Right => println!("Swiped right"),
        _ => {}
    }
}).await?;

ui.register_gesture(element_id, GestureType::Pinch, |event| {
    println!("Pinch scale: {}", event.scale);
}).await?;
```

#### Mobile Features
```rust
use playground_systems_ui::mobile::FloatingToolbar;

// Create floating toolbar for mobile
let toolbar = FloatingToolbar::new()
    .add_button("undo", "↶")
    .add_button("redo", "↷")
    .add_button("copy", "📋")
    .add_button("paste", "📄")
    .position(ToolbarPosition::Bottom)
    .auto_hide(true);

ui.add_floating_toolbar(toolbar).await?;
```

### Theming

#### Built-in Themes
```rust
use playground_systems_ui::theme::{Theme, ThemeManager};

// Load default themes
let mut theme_manager = ThemeManager::new();
theme_manager.load_default_themes()?;

// Switch theme
ui.set_theme(ThemeId::Dark)?;
ui.set_theme(ThemeId::Light)?;
ui.set_theme(ThemeId::HighContrast)?;
```

#### Custom Themes
```rust
// Define custom theme
let custom_theme = Theme {
    name: "Cyberpunk".to_string(),
    colors: ThemeColors {
        background: Color::from_hex("#0a0e27"),
        foreground: Color::from_hex("#00ff41"),
        primary: Color::from_hex("#ff006e"),
        secondary: Color::from_hex("#ffbe0b"),
        border: Color::from_hex("#3a506b"),
        // ... more colors
    },
    fonts: ThemeFonts {
        body: "Roboto Mono",
        heading: "Orbitron",
        code: "Fira Code",
    },
    spacing: ThemeSpacing {
        small: 4.0,
        medium: 8.0,
        large: 16.0,
    },
};

theme_manager.register_theme(custom_theme)?;
```

### Terminal Integration

#### Termux Terminal
```rust
use playground_systems_ui::terminal::{TerminalConnection, TerminalConfig};

// Connect to Termux terminal
let config = TerminalConfig {
    shell: "/data/data/com.termux/files/usr/bin/bash",
    env: HashMap::from([
        ("TERM", "xterm-256color"),
        ("HOME", "/data/data/com.termux/files/home"),
    ]),
    size: (80, 24), // cols, rows
};

let terminal = ui.create_terminal(config).await?;

// Send input
terminal.send_input("ls -la\n").await?;

// Receive output (via callback)
terminal.on_output(|output| {
    println!("Terminal: {}", output);
}).await?;
```

#### WebSocket Terminal
```rust
// Terminal over WebSocket for browser
ui.enable_websocket_terminal(channel_id).await?;

// Browser sends input via channel 10, packet type 5
// Server sends output via channel 10, packet type 6
```

### Text Rendering

#### Syntax Highlighting
```rust
use playground_systems_ui::rendering::TextRenderer;

// Create text element with syntax highlighting
let code_text = ui.create_text_element(TextConfig {
    content: r#"fn main() {
        println!("Hello, world!");
    }"#,
    language: "rust",
    theme: "monokai",
    line_numbers: true,
    highlight_line: Some(2),
}).await?;
```

#### Font Management
```rust
// Load custom fonts
ui.load_font("FiraCode", "/path/to/FiraCode.ttf").await?;

// Set font for element
ui.set_font(element_id, FontConfig {
    family: "FiraCode",
    size: 14.0,
    weight: FontWeight::Normal,
    style: FontStyle::Normal,
}).await?;
```

### WebSocket Messages

The UI system automatically syncs with browser clients:

```rust
// Server-side element creation
let button = ui.create_button("Click me").await?;

// Automatically sends to browser:
// {
//   "type": "CreateElement",
//   "id": "uuid",
//   "tag": "button",
//   "props": { "text": "Click me" }
// }

// Browser sends input events:
// {
//   "type": "InputEvent",
//   "element_id": "uuid",
//   "event": { "type": "click", "position": [100, 200] }
// }

// Server processes and updates UI
```

### Render Batching

UI updates are batched for efficiency:

```rust
// Multiple updates in one frame
ui.update_text(text1, "New text 1").await?;
ui.update_position(elem1, Vector2::new(10.0, 20.0)).await?;
ui.update_style(elem2, new_style).await?;

// All sent as single RenderBatch message
// Reduces network traffic by ~90%
```

## Components Reference

### UiElementComponent
```rust
pub struct UiElementComponent {
    pub id: Uuid,
    pub name: String,
    pub tag: String,              // html-like: div, button, text, etc.
    pub bounds: ElementBounds,
    pub children: Vec<EntityId>,
    pub parent: Option<EntityId>,
    pub visible: bool,
    pub interactive: bool,
    pub z_index: i32,
}
```

### UiLayoutComponent
```rust
pub struct UiLayoutComponent {
    pub constraints: LayoutConstraints,
    pub computed_size: Vector2<f32>,
    pub computed_position: Vector2<f32>,
    pub padding: Vector4<f32>,    // top, right, bottom, left
    pub margin: Vector4<f32>,     
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: f32,
    pub align_self: AlignSelf,
    pub justify_self: JustifySelf,
}
```

### UiStyleComponent
```rust
pub struct UiStyleComponent {
    pub background_color: Vector4<f32>,  // RGBA
    pub border_color: Vector4<f32>,
    pub border_width: f32,
    pub border_radius: f32,
    pub opacity: f32,
    pub shadow: Option<Shadow>,
    pub blend_mode: BlendMode,
    pub transform: Matrix4<f32>,
}
```

## Performance

- **ECS Backing**: Efficient entity queries and updates
- **Render Batching**: Groups updates per frame
- **Dirty Tracking**: Only updates changed elements
- **Layout Caching**: Reuses computed layouts
- **Virtual Scrolling**: For long lists
- **Gesture Debouncing**: Reduces event spam
- **WebSocket Compression**: Binary protocol

## Mobile Optimizations

- **Touch-First**: All interactions designed for touch
- **Floating Toolbar**: Contextual actions without screen clutter
- **Gesture Support**: Native feeling interactions
- **Battery Efficient**: Minimal redraws, event batching
- **Responsive**: Adapts to screen size/orientation
- **Offline First**: Works without network

## Testing

```rust
#[tokio::test]
async fn test_ui_system() {
    // Create mock renderer
    let renderer = MockRenderer::new();
    
    // Initialize UI
    let mut ui = UiSystem::new();
    ui.initialize(renderer).await.unwrap();
    
    // Create elements
    let button = ui.create_button("Test").await.unwrap();
    
    // Simulate input
    ui.simulate_click(button, Vector2::new(10.0, 10.0)).await.unwrap();
    
    // Verify state
    let element = ui.get_element(button).await.unwrap();
    assert!(element.clicked);
}
```

## Architecture Rules

- Uses core/ecs for internal state management
- Integrates with rendering system for display
- Thread-safe with Arc<RwLock<>> 
- All operations are async
- NO unsafe code
- Result<T, UiError> for all fallible operations
- Mobile-first design decisions

## Dependencies

- `playground-core-ecs`: Internal state management
- `playground-core-types`: Shared types
- `playground-systems-rendering`: Display output
- `nalgebra`: Vector/matrix math
- `uuid`: Element identifiers
- `tokio`: Async runtime
- `serde`: Serialization for themes
- `async-trait`: Async component traits

## See Also

- [systems/rendering](../rendering/README.md) - Rendering backend
- [plugins/ui-framework](../../plugins/ui-framework/README.md) - Conversational IDE
- [plugins/editor-core](../../plugins/editor-core/README.md) - Text editing