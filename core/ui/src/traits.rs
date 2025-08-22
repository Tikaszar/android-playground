use async_trait::async_trait;
use crate::{
    UiResult, ElementId, ElementType, ElementUpdate, UiCommand, UiEvent, 
    EventResult, Style, Bounds, LayoutType
};
use playground_core_rendering::RenderCommandBatch;

/// Base trait for UI elements
#[async_trait]
pub trait UiElement: Send + Sync {
    /// Get the element's unique identifier
    fn id(&self) -> ElementId;
    
    /// Get the element's type
    fn element_type(&self) -> ElementType;
    
    /// Get the element's current bounds
    async fn get_bounds(&self) -> Bounds;
    
    /// Get the element's current style
    async fn get_style(&self) -> Style;
    
    /// Update the element
    async fn update(&mut self, update: ElementUpdate) -> UiResult<()>;
    
    /// Handle an event
    async fn handle_event(&mut self, event: &UiEvent) -> UiResult<EventResult>;
    
    /// Generate UI commands for this element
    async fn generate_commands(&self) -> UiResult<Vec<UiCommand>>;
    
    /// Check if element contains a point (for hit testing)
    async fn contains_point(&self, x: f32, y: f32) -> bool {
        let bounds = self.get_bounds().await;
        bounds.contains(x, y)
    }
    
    /// Check if element is visible
    async fn is_visible(&self) -> bool {
        let style = self.get_style().await;
        style.visible && style.opacity > 0.0
    }
}

/// Trait for container elements that can have children
#[async_trait]
pub trait UiContainer: UiElement {
    /// Add a child element
    async fn add_child(&mut self, child: ElementId, index: Option<usize>) -> UiResult<()>;
    
    /// Remove a child element
    async fn remove_child(&mut self, child: ElementId) -> UiResult<()>;
    
    /// Get all child element IDs
    async fn get_children(&self) -> Vec<ElementId>;
    
    /// Clear all children
    async fn clear_children(&mut self) -> UiResult<()>;
    
    /// Get the layout type for this container
    async fn get_layout(&self) -> LayoutType;
    
    /// Set the layout type for this container
    async fn set_layout(&mut self, layout: LayoutType) -> UiResult<()>;
}

/// Main UI renderer trait that systems must implement
#[async_trait]
pub trait UiRenderer: Send + Sync {
    /// Initialize the renderer
    async fn initialize(&mut self) -> UiResult<()>;
    
    /// Create a new UI element
    async fn create_element(
        &mut self,
        element_type: ElementType,
        parent: Option<ElementId>,
    ) -> UiResult<ElementId>;
    
    /// Update an existing element
    async fn update_element(
        &mut self,
        id: ElementId,
        update: ElementUpdate,
    ) -> UiResult<()>;
    
    /// Remove an element and all its children
    async fn remove_element(&mut self, id: ElementId) -> UiResult<()>;
    
    /// Get an element by ID
    async fn get_element(&self, id: ElementId) -> UiResult<Box<dyn UiElement>>;
    
    /// Process a UI command
    async fn process_command(&mut self, command: UiCommand) -> UiResult<()>;
    
    /// Handle a UI event
    async fn handle_event(&mut self, event: UiEvent) -> UiResult<EventResult>;
    
    /// Perform layout calculation
    async fn calculate_layout(&mut self) -> UiResult<()>;
    
    /// Render the UI tree to render commands
    async fn render_frame(&mut self, frame_id: u64) -> UiResult<RenderCommandBatch>;
    
    /// Get the root element ID
    async fn get_root(&self) -> Option<ElementId>;
    
    /// Set the viewport size (mobile screen dimensions)
    async fn set_viewport(&mut self, width: f32, height: f32) -> UiResult<()>;
    
    /// Mobile-specific: Set safe area insets
    async fn set_safe_area_insets(
        &mut self,
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
    ) -> UiResult<()>;
    
    /// Mobile-specific: Handle orientation change
    async fn handle_orientation_change(&mut self, orientation: crate::events::Orientation) -> UiResult<()>;
}

/// Trait for UI systems that support themes
#[async_trait]
pub trait ThemedRenderer: UiRenderer {
    /// Set the active theme
    async fn set_theme(&mut self, theme_id: String) -> UiResult<()>;
    
    /// Get the current theme ID
    async fn get_theme(&self) -> String;
    
    /// Register a new theme
    async fn register_theme(&mut self, theme_id: String, theme_data: Vec<u8>) -> UiResult<()>;
}

/// Trait for UI systems that support animations
#[async_trait]
pub trait AnimatedRenderer: UiRenderer {
    /// Start an animation on an element
    async fn animate_element(
        &mut self,
        element: ElementId,
        animation: Animation,
    ) -> UiResult<AnimationId>;
    
    /// Stop an animation
    async fn stop_animation(&mut self, animation_id: AnimationId) -> UiResult<()>;
    
    /// Update animations (called per frame)
    async fn update_animations(&mut self, delta_time: f32) -> UiResult<()>;
}

/// Animation definition
#[derive(Debug, Clone)]
pub struct Animation {
    pub duration_ms: u32,
    pub easing: EasingFunction,
    pub properties: Vec<AnimatedProperty>,
    pub repeat: AnimationRepeat,
}

/// Animation ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimationId(pub uuid::Uuid);

/// Properties that can be animated
#[derive(Debug, Clone)]
pub enum AnimatedProperty {
    Opacity(f32, f32),           // from, to
    Position(f32, f32, f32, f32), // from_x, from_y, to_x, to_y
    Scale(f32, f32),              // from, to
    Rotation(f32, f32),           // from, to (in radians)
    Color([f32; 4], [f32; 4]),   // from, to
}

/// Easing functions for animations
#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32),
}

/// Animation repeat behavior
#[derive(Debug, Clone, Copy)]
pub enum AnimationRepeat {
    None,
    Count(u32),
    Infinite,
    PingPong(u32),
}