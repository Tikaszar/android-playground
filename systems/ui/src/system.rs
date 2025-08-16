//! Main UI system implementation

use crate::error::{UiError, UiResult};
use crate::element::{ElementGraph, ElementId};
use crate::layout::LayoutConstraints;
use crate::input::InputManager;
use crate::rendering::UiRenderer;
use crate::theme::ThemeManager;
use nalgebra::Vector2;
use playground_rendering::BaseRenderer;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main UI system struct
pub struct UiSystem {
    initialized: bool,
    element_graph: Arc<RwLock<ElementGraph>>,
    input_manager: InputManager,
    renderer: Option<Box<dyn UiRenderer>>,
    theme_manager: ThemeManager,
    screen_size: Vector2<f32>,
}

impl UiSystem {
    /// Create a new UI system
    pub fn new() -> Self {
        Self {
            initialized: false,
            element_graph: Arc::new(RwLock::new(ElementGraph::new())),
            input_manager: InputManager::new(),
            renderer: None,
            theme_manager: ThemeManager::new(),
            screen_size: Vector2::new(1920.0, 1080.0),
        }
    }

    /// Initialize the UI system with a renderer
    pub async fn initialize<R: BaseRenderer + 'static>(&mut self, renderer: R) -> UiResult<()> {
        if self.initialized {
            return Err(UiError::InitializationFailed("Already initialized".to_string()));
        }
        
        // Initialize theme manager with default themes
        self.theme_manager.load_default_themes()?;
        
        // Set up the renderer
        // self.renderer = Some(Box::new(UiRendererImpl::new(renderer)));
        
        self.initialized = true;
        Ok(())
    }

    /// Check if the UI system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Set screen size for layout calculations
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_size = Vector2::new(width, height);
    }

    /// Perform layout for all elements
    pub async fn perform_layout(&mut self) -> UiResult<()> {
        if !self.initialized {
            return Err(UiError::InitializationFailed("UI system not initialized".to_string()));
        }
        
        let constraints = LayoutConstraints::new(self.screen_size);
        let mut graph = self.element_graph.write().await;
        
        // Start from root and recursively layout
        if let Some(root_id) = graph.root() {
            self.layout_element(&mut graph, root_id, &constraints)?;
        }
        
        Ok(())
    }
    
    fn layout_element(
        &self,
        graph: &mut ElementGraph,
        id: ElementId,
        constraints: &LayoutConstraints,
    ) -> UiResult<()> {
        // Get element and perform layout
        if let Some(element) = graph.get_mut(id) {
            element.layout(constraints)?;
            
            // Layout children recursively
            let children = element.children().to_vec();
            for child_id in children {
                self.layout_element(graph, child_id, constraints)?;
            }
        }
        
        Ok(())
    }

    /// Render the UI
    pub async fn render(&mut self) -> UiResult<()> {
        if !self.initialized {
            return Err(UiError::InitializationFailed("UI system not initialized".to_string()));
        }
        
        // Get dirty elements and render them
        let graph = self.element_graph.read().await;
        let dirty = graph.dirty_elements();
        
        if let Some(renderer) = &mut self.renderer {
            renderer.render_elements(&graph, &dirty, &self.theme_manager)?;
        }
        
        Ok(())
    }

    /// Update the UI
    pub async fn update(&mut self, delta_time: f32) -> UiResult<()> {
        if !self.initialized {
            return Err(UiError::InitializationFailed("UI system not initialized".to_string()));
        }
        
        // Update all elements
        let mut graph = self.element_graph.write().await;
        for (_, element) in graph.iter() {
            // Elements update themselves through the Element trait
        }
        
        // Process input events
        self.input_manager.process_events(&mut graph)?;
        
        Ok(())
    }
    
    /// Get the element graph for manipulation
    pub fn element_graph(&self) -> Arc<RwLock<ElementGraph>> {
        Arc::clone(&self.element_graph)
    }
    
    /// Get the input manager
    pub fn input_manager(&mut self) -> &mut InputManager {
        &mut self.input_manager
    }
    
    /// Get the theme manager
    pub fn theme_manager(&mut self) -> &mut ThemeManager {
        &mut self.theme_manager
    }
}

impl Default for UiSystem {
    fn default() -> Self {
        Self::new()
    }
}