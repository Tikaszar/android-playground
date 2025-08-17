//! Main UI system implementation using core/ecs for internal state

use crate::error::{UiError, UiResult};
use crate::components::*;
use crate::element::{ElementBounds, ElementId};
use crate::layout::LayoutConstraints;
use crate::input::InputManager;
use crate::rendering::UiRenderer;
use crate::theme::{ThemeManager, ThemeId};
use nalgebra::{Vector2, Vector4};
use playground_ecs::{World, EntityId, ComponentRegistry, QueryBuilder};
use playground_rendering::BaseRenderer;
use playground_server::channel::ChannelManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Main UI system struct using ECS for internal state management
pub struct UiSystem {
    initialized: bool,
    world: Arc<World>,
    registry: Arc<ComponentRegistry>,
    input_manager: InputManager,
    renderer: Option<Box<dyn UiRenderer>>,
    theme_manager: ThemeManager,
    screen_size: Vector2<f32>,
    root_entity: Option<EntityId>,
    channel_id: Option<u16>,
    channel_manager: Option<Arc<ChannelManager>>,
    current_frame: u64,
}

impl UiSystem {
    /// Create a new UI system with ECS backing
    pub fn new() -> Self {
        let registry = Arc::new(ComponentRegistry::new());
        let world = Arc::new(World::with_registry(Arc::clone(&registry)));
        
        Self {
            initialized: false,
            world,
            registry,
            input_manager: InputManager::new(),
            renderer: None,
            theme_manager: ThemeManager::new(),
            screen_size: Vector2::new(1920.0, 1080.0),
            root_entity: None,
            channel_id: None,
            channel_manager: None,
            current_frame: 0,
        }
    }

    /// Initialize the UI system with a renderer and register components
    pub async fn initialize<R: BaseRenderer + 'static>(&mut self, renderer: R) -> UiResult<()> {
        if self.initialized {
            return Err(UiError::InitializationFailed("Already initialized".to_string()));
        }
        
        // Register UI components with ECS
        self.register_components().await?;
        
        // Initialize theme manager with default themes
        self.theme_manager.load_default_themes()?;
        
        // Set up the renderer
        // self.renderer = Some(Box::new(UiRendererImpl::new(renderer)));
        
        // Create root UI entity
        self.root_entity = Some(self.create_root_entity().await?);
        
        self.initialized = true;
        Ok(())
    }
    
    /// Register all UI components with the ECS registry
    async fn register_components(&mut self) -> UiResult<()> {
        self.registry.register::<UiElementComponent>()
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiElementComponent: {}", e)))?;
        
        self.registry.register::<UiLayoutComponent>()
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiLayoutComponent: {}", e)))?;
        
        self.registry.register::<UiStyleComponent>()
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiStyleComponent: {}", e)))?;
        
        self.registry.register::<UiDirtyComponent>()
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiDirtyComponent: {}", e)))?;
        
        self.registry.register::<UiInputComponent>()
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiInputComponent: {}", e)))?;
        
        self.registry.register::<UiWebSocketComponent>()
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiWebSocketComponent: {}", e)))?;
        
        self.registry.register::<UiTextComponent>()
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiTextComponent: {}", e)))?;
        
        Ok(())
    }
    
    /// Create the root UI entity
    async fn create_root_entity(&self) -> UiResult<EntityId> {
        let root_element = UiElementComponent {
            id: Uuid::new_v4(),
            name: "root".to_string(),
            tag: "div".to_string(),
            bounds: ElementBounds {
                position: Vector2::new(0.0, 0.0),
                size: self.screen_size,
            },
            children: Vec::new(),
            parent: None,
            visible: true,
            interactive: false,
            z_index: 0,
        };
        
        let root_layout = UiLayoutComponent {
            constraints: LayoutConstraints::new(self.screen_size),
            computed_size: self.screen_size,
            computed_position: Vector2::new(0.0, 0.0),
            padding: Vector4::zeros(),
            margin: Vector4::zeros(),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            flex_basis: 0.0,
            align_self: AlignSelf::Auto,
            justify_self: JustifySelf::Auto,
        };
        
        let root_style = UiStyleComponent {
            theme_id: ThemeId(0),
            background_color: Vector4::new(0.0, 0.0, 0.0, 1.0),
            border_color: Vector4::new(0.0, 0.0, 0.0, 0.0),
            text_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            border_width: 0.0,
            border_radius: 0.0,
            opacity: 1.0,
            custom_properties: Default::default(),
        };
        
        let entities = self.world.spawn_batch(vec![
            vec![
                Box::new(root_element) as Box<dyn playground_ecs::Component>,
                Box::new(root_layout) as Box<dyn playground_ecs::Component>,
                Box::new(root_style) as Box<dyn playground_ecs::Component>,
            ],
        ]).await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to create root entity: {}", e)))?;
        
        Ok(entities[0])
    }
    
    /// Register UI system with core/server for WebSocket communication
    pub async fn register_with_server(&mut self, channel_manager: Arc<ChannelManager>) -> UiResult<()> {
        // Register UI system on channel 10
        let channel_id = channel_manager.register_system("ui".to_string(), 10)
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UI channel: {}", e)))?;
        
        self.channel_id = Some(channel_id);
        self.channel_manager = Some(channel_manager);
        
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
    
    /// Create a new UI element entity
    pub async fn create_element(
        &self,
        name: String,
        tag: String,
        parent: Option<EntityId>,
    ) -> UiResult<EntityId> {
        let element = UiElementComponent {
            id: Uuid::new_v4(),
            name,
            tag,
            bounds: ElementBounds {
                position: Vector2::zeros(),
                size: Vector2::new(100.0, 100.0),
            },
            children: Vec::new(),
            parent,
            visible: true,
            interactive: true,
            z_index: 0,
        };
        
        let layout = UiLayoutComponent {
            constraints: LayoutConstraints::new(self.screen_size),
            computed_size: Vector2::new(100.0, 100.0),
            computed_position: Vector2::zeros(),
            padding: Vector4::zeros(),
            margin: Vector4::zeros(),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: 0.0,
            align_self: AlignSelf::Auto,
            justify_self: JustifySelf::Auto,
        };
        
        let style = UiStyleComponent {
            theme_id: ThemeId(0),
            background_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            border_color: Vector4::new(0.2, 0.2, 0.2, 1.0),
            text_color: Vector4::new(0.0, 0.0, 0.0, 1.0),
            border_width: 1.0,
            border_radius: 0.0,
            opacity: 1.0,
            custom_properties: Default::default(),
        };
        
        let dirty = UiDirtyComponent {
            layout_dirty: true,
            style_dirty: true,
            content_dirty: true,
            last_render_frame: 0,
        };
        
        let entities = self.world.spawn_batch(vec![
            vec![
                Box::new(element) as Box<dyn playground_ecs::Component>,
                Box::new(layout) as Box<dyn playground_ecs::Component>,
                Box::new(style) as Box<dyn playground_ecs::Component>,
                Box::new(dirty) as Box<dyn playground_ecs::Component>,
            ],
        ]).await
            .map_err(|e| UiError::Other(format!("Failed to create element: {}", e)))?;
        
        let entity = entities[0];
        
        // Add to parent's children if specified
        if let Some(parent_id) = parent {
            self.add_child(parent_id, entity).await?;
        }
        
        Ok(entity)
    }
    
    /// Add a child to a parent element
    async fn add_child(&self, parent: EntityId, child: EntityId) -> UiResult<()> {
        // Query for parent's UiElementComponent and update children
        let query = self.world.query::<&mut UiElementComponent>()
            .with_entity(parent)
            .build();
        
        let mut results = query.execute().await
            .map_err(|e| UiError::Other(format!("Failed to query parent: {}", e)))?;
        
        if let Some((_, component)) = results.first_mut() {
            component.children.push(child);
        }
        
        // Update child's parent reference
        let query = self.world.query::<&mut UiElementComponent>()
            .with_entity(child)
            .build();
        
        let mut results = query.execute().await
            .map_err(|e| UiError::Other(format!("Failed to query child: {}", e)))?;
        
        if let Some((_, component)) = results.first_mut() {
            component.parent = Some(parent);
        }
        
        Ok(())
    }

    /// Perform layout for all elements
    pub async fn perform_layout(&mut self) -> UiResult<()> {
        if !self.initialized {
            return Err(UiError::InitializationFailed("UI system not initialized".to_string()));
        }
        
        let constraints = LayoutConstraints::new(self.screen_size);
        
        // Start from root and recursively layout
        if let Some(root_id) = self.root_entity {
            self.layout_element_recursive(root_id, &constraints).await?;
        }
        
        Ok(())
    }
    
    /// Recursively layout an element and its children
    async fn layout_element_recursive(
        &self,
        entity: EntityId,
        constraints: &LayoutConstraints,
    ) -> UiResult<()> {
        // Query for element and layout components
        let query = self.world.query::<(&UiElementComponent, &mut UiLayoutComponent)>()
            .with_entity(entity)
            .build();
        
        let results = query.execute().await
            .map_err(|e| UiError::Other(format!("Failed to query element for layout: {}", e)))?;
        
        if let Some((_, (element, layout))) = results.first() {
            // Update layout constraints
            layout.constraints = *constraints;
            
            // Compute layout (simplified for now)
            layout.computed_size = constraints.max_size;
            layout.computed_position = Vector2::zeros();
            
            // Layout children recursively
            for child_id in element.children.clone() {
                let child_constraints = LayoutConstraints::new(layout.computed_size);
                self.layout_element_recursive(child_id, &child_constraints).await?;
            }
        }
        
        Ok(())
    }

    /// Mark elements as dirty for re-rendering
    pub async fn mark_dirty(&self, entity: EntityId, layout: bool, style: bool, content: bool) -> UiResult<()> {
        let query = self.world.query::<&mut UiDirtyComponent>()
            .with_entity(entity)
            .build();
        
        let mut results = query.execute().await
            .map_err(|e| UiError::Other(format!("Failed to query dirty component: {}", e)))?;
        
        if let Some((_, dirty)) = results.first_mut() {
            if layout {
                dirty.layout_dirty = true;
            }
            if style {
                dirty.style_dirty = true;
            }
            if content {
                dirty.content_dirty = true;
            }
        }
        
        Ok(())
    }

    /// Render the UI
    pub async fn render(&mut self) -> UiResult<()> {
        if !self.initialized {
            return Err(UiError::InitializationFailed("UI system not initialized".to_string()));
        }
        
        self.current_frame += 1;
        
        // Query for dirty elements
        let query = self.world.query::<(&UiElementComponent, &UiStyleComponent, &mut UiDirtyComponent)>()
            .build();
        
        let results = query.execute().await
            .map_err(|e| UiError::Other(format!("Failed to query dirty elements: {}", e)))?;
        
        // Collect dirty elements for rendering
        let mut dirty_elements = Vec::new();
        for (entity, (element, style, dirty)) in results {
            if dirty.layout_dirty || dirty.style_dirty || dirty.content_dirty {
                dirty_elements.push((entity, element.clone(), style.clone()));
                
                // Clear dirty flags
                dirty.layout_dirty = false;
                dirty.style_dirty = false;
                dirty.content_dirty = false;
                dirty.last_render_frame = self.current_frame;
            }
        }
        
        // Render dirty elements
        if let Some(renderer) = &mut self.renderer {
            // renderer.render_elements(&dirty_elements, &self.theme_manager)?;
        }
        
        Ok(())
    }

    /// Update the UI
    pub async fn update(&mut self, delta_time: f32) -> UiResult<()> {
        if !self.initialized {
            return Err(UiError::InitializationFailed("UI system not initialized".to_string()));
        }
        
        // Run garbage collection on ECS
        self.world.gc().collect_incremental(&self.world).await
            .map_err(|e| UiError::Other(format!("GC failed: {}", e)))?;
        
        // Process input events through ECS queries
        // self.input_manager.process_events(&self.world).await?;
        
        // Process WebSocket messages if connected
        if let Some(channel_id) = self.channel_id {
            self.process_websocket_messages().await?;
        }
        
        Ok(())
    }
    
    /// Process incoming WebSocket messages from core/server
    async fn process_websocket_messages(&self) -> UiResult<()> {
        // Query for WebSocket components and process their messages
        let query = self.world.query::<&mut UiWebSocketComponent>()
            .build();
        
        let mut results = query.execute().await
            .map_err(|e| UiError::Other(format!("Failed to query WebSocket components: {}", e)))?;
        
        for (_, ws_component) in results {
            // Process pending messages
            for message in ws_component.pending_messages.drain(..) {
                // Handle message based on type
                // This will be implemented when we integrate with core/server
            }
        }
        
        Ok(())
    }
    
    /// Get the root entity
    pub fn root_entity(&self) -> Option<EntityId> {
        self.root_entity
    }
    
    /// Get the ECS world for advanced queries
    pub fn world(&self) -> Arc<World> {
        Arc::clone(&self.world)
    }
    
    /// Get the input manager
    pub fn input_manager(&mut self) -> &mut InputManager {
        &mut self.input_manager
    }
    
    /// Get the theme manager
    pub fn theme_manager(&mut self) -> &mut ThemeManager {
        &mut self.theme_manager
    }
    
    /// Get memory statistics from the ECS
    pub async fn memory_stats(&self) -> UiResult<playground_ecs::MemoryStats> {
        let stats = self.world.memory_stats().await
            .map_err(|e| UiError::Other(format!("Failed to get memory stats: {}", e)))?;
        Ok(stats)
    }
}

impl Default for UiSystem {
    fn default() -> Self {
        Self::new()
    }
}