use playground_core_rendering::{RenderCommand, RenderCommandBatch, Viewport};
use playground_core_ecs::{World, EntityId, ComponentRegistry};
use playground_core_types::{Shared, shared};
use playground_core_server::{ChannelManager, Packet, Priority};
use crate::error::{UiError, UiResult};
use crate::element::{ElementGraph, ElementId};
use crate::components::*;
use crate::layout::LayoutEngine;
use crate::input::{InputManager, InputEvent};
use crate::theme::{ThemeManager, Theme, ThemeId};
use crate::terminal::TerminalManager;
use crate::mobile::MobileFeatures;
use crate::rendering::ui_to_render_commands;
use std::collections::HashMap;
use uuid::Uuid;

pub struct UiSystem {
    // Core ECS
    world: Shared<World>,
    registry: Shared<ComponentRegistry>,
    
    // Element management
    element_graph: Shared<ElementGraph>,
    root_entity: Option<EntityId>,
    
    // Layout
    layout_engine: Shared<LayoutEngine>,
    
    // Input handling
    input_manager: Shared<InputManager>,
    
    // Theme management
    theme_manager: Shared<ThemeManager>,
    current_theme: ThemeId,
    
    // Terminal support
    terminal_manager: Shared<TerminalManager>,
    terminal_connections: Shared<HashMap<Uuid, EntityId>>,
    
    // Mobile features
    mobile_features: Shared<MobileFeatures>,
    
    // Rendering
    viewport: Viewport,
    frame_id: u64,
    dirty_elements: Shared<Vec<EntityId>>,
    
    // Networking
    channel_manager: Option<Shared<ChannelManager>>,
    channel_id: u16,
    
    // State
    initialized: bool,
    screen_size: [f32; 2],
}

impl UiSystem {
    pub fn new() -> Self {
        let registry = shared(ComponentRegistry::new());
        let world = shared(World::with_registry(registry.clone()));
        
        Self {
            world,
            registry,
            element_graph: shared(ElementGraph::new()),
            root_entity: None,
            layout_engine: shared(LayoutEngine::new()),
            input_manager: shared(InputManager::new()),
            theme_manager: shared(ThemeManager::new()),
            current_theme: ThemeId::Dark,
            terminal_manager: shared(TerminalManager::new()),
            terminal_connections: shared(HashMap::new()),
            mobile_features: shared(MobileFeatures::new()),
            viewport: Viewport { x: 0, y: 0, width: 1920, height: 1080 },
            frame_id: 0,
            dirty_elements: shared(Vec::new()),
            channel_manager: None,
            channel_id: 10,
            initialized: false,
            screen_size: [1920.0, 1080.0],
        }
    }
    
    pub async fn initialize(&mut self) -> UiResult<()> {
        if self.initialized {
            return Err(UiError::AlreadyInitialized);
        }
        
        // Register all component types
        self.register_components().await?;
        
        // Load default themes
        let mut theme_mgr = self.theme_manager.write().await;
        theme_mgr.load_default_themes()?;
        drop(theme_mgr);
        
        // Create root element
        self.root_entity = Some(self.create_root().await?);
        
        // Initialize mobile features if on mobile
        let mut mobile = self.mobile_features.write().await;
        mobile.initialize().await?;
        drop(mobile);
        
        self.initialized = true;
        Ok(())
    }
    
    pub async fn set_channel_manager(&mut self, manager: Shared<ChannelManager>) {
        self.channel_manager = Some(manager);
    }
    
    pub async fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.screen_size = [viewport.width as f32, viewport.height as f32];
        
        // Mark all elements as dirty for relayout
        if let Some(root) = self.root_entity {
            self.mark_subtree_dirty(root).await.ok();
        }
    }
    
    pub async fn render(&mut self) -> UiResult<RenderCommandBatch> {
        if !self.initialized {
            return Err(UiError::NotInitialized);
        }
        
        let mut batch = RenderCommandBatch::new(self.frame_id);
        batch.set_viewport(self.viewport);
        
        // Get current theme
        let theme_mgr = self.theme_manager.read().await;
        let theme = theme_mgr.get_theme(self.current_theme)?;
        let theme_clone = theme.clone();
        drop(theme_mgr);
        
        // Clear with theme background
        batch.push(RenderCommand::Clear {
            color: [
                theme_clone.colors.background.x,
                theme_clone.colors.background.y,
                theme_clone.colors.background.z,
                theme_clone.colors.background.w,
            ]
        });
        
        // Update layout for dirty elements
        self.update_layout().await?;
        
        // Render element tree
        if let Some(root) = self.root_entity {
            self.render_element_tree(root, &mut batch, &theme_clone).await?;
        }
        
        // Render mobile UI if active
        let mobile = self.mobile_features.read().await;
        mobile.render(&mut batch, &theme_clone)?;
        drop(mobile);
        
        // Send via WebSocket
        if let Some(ref manager) = self.channel_manager {
            self.send_batch(&batch).await?;
        }
        
        // Clear dirty list
        self.dirty_elements.write().await.clear();
        
        self.frame_id += 1;
        Ok(batch)
    }
    
    pub async fn handle_input(&mut self, event: InputEvent) -> UiResult<bool> {
        let mut input_mgr = self.input_manager.write().await;
        let handled = input_mgr.process_event(event, &self.element_graph, &self.world).await?;
        
        // If input changed element state, mark as dirty
        if handled {
            if let Some(focused) = input_mgr.get_focused_element() {
                self.dirty_elements.write().await.push(focused);
            }
        }
        
        Ok(handled)
    }
    
    pub async fn create_element(
        &mut self,
        element_type: &str,
        parent: Option<EntityId>,
    ) -> UiResult<EntityId> {
        let mut world = self.world.write().await;
        
        // Create element with all components
        let components: Vec<Box<dyn playground_core_ecs::Component>> = vec![
            Box::new(UiElementComponent::new(element_type)),
            Box::new(UiLayoutComponent::default()),
            Box::new(UiStyleComponent::default()),
            Box::new(UiInputComponent::default()),
        ];
        
        // Add text component for text elements
        let components = if element_type == "text" || element_type == "input" {
            let mut comps = components;
            comps.push(Box::new(UiTextComponent::new(String::new())));
            comps
        } else {
            components
        };
        
        let entities = world.spawn_batch(vec![components]).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let entity = entities.into_iter().next()
            .ok_or_else(|| UiError::CreationFailed("Failed to create element".into()))?;
        
        drop(world);
        
        // Add to element graph
        if let Some(parent) = parent {
            let mut graph = self.element_graph.write().await;
            graph.add_child(parent, entity)?;
        }
        
        // Mark as dirty for layout
        self.dirty_elements.write().await.push(entity);
        
        Ok(entity)
    }
    
    pub async fn remove_element(&mut self, element: EntityId) -> UiResult<()> {
        // Remove from graph
        let mut graph = self.element_graph.write().await;
        graph.remove_element(element);
        drop(graph);
        
        // Remove from world
        self.world.write().await.despawn_batch(vec![element]).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn set_element_text(&mut self, element: EntityId, text: String) -> UiResult<()> {
        // For now, just mark as dirty - proper implementation would update components
        // The ECS doesn't provide mutable component access, need to remove and re-add
        
        // Mark as dirty
        self.dirty_elements.write().await.push(element);
        
        Ok(())
    }
    
    pub async fn create_terminal(&mut self, parent: EntityId) -> UiResult<Uuid> {
        let terminal_id = Uuid::new_v4();
        let entity = self.create_element("terminal", Some(parent)).await?;
        
        // Register terminal connection
        self.terminal_connections.write().await.insert(terminal_id, entity);
        
        // Create terminal instance
        let mut term_mgr = self.terminal_manager.write().await;
        term_mgr.create_terminal(terminal_id).await?;
        
        Ok(terminal_id)
    }
    
    pub async fn set_theme(&mut self, theme_id: ThemeId) -> UiResult<()> {
        let theme_mgr = self.theme_manager.read().await;
        if !theme_mgr.has_theme(theme_id) {
            return Err(UiError::ThemeNotFound(theme_id.to_string()));
        }
        drop(theme_mgr);
        
        self.current_theme = theme_id;
        
        // Mark all elements as dirty to re-render with new theme
        if let Some(root) = self.root_entity {
            self.mark_subtree_dirty(root).await?;
        }
        
        Ok(())
    }
    
    async fn register_components(&self) -> UiResult<()> {
        let mut registry = self.registry.write().await;
        
        registry.register::<UiElementComponent>();
        registry.register::<UiLayoutComponent>();
        registry.register::<UiStyleComponent>();
        registry.register::<UiInputComponent>();
        registry.register::<UiTextComponent>();
        
        Ok(())
    }
    
    async fn create_root(&self) -> UiResult<EntityId> {
        let mut world = self.world.write().await;
        
        let mut root_element = UiElementComponent::new("root");
        root_element.visible = true;
        
        let mut root_layout = UiLayoutComponent::default();
        root_layout.bounds = ElementBounds {
            x: 0.0,
            y: 0.0,
            width: self.screen_size[0],
            height: self.screen_size[1],
        };
        root_layout.layout_type = LayoutType::Absolute;
        
        let mut root_style = UiStyleComponent::default();
        root_style.visible = true;
        
        let components: Vec<Box<dyn playground_core_ecs::Component>> = vec![
            Box::new(root_element),
            Box::new(root_layout),
            Box::new(root_style),
            Box::new(UiInputComponent::default()),
        ];
        
        let entities = world.spawn_batch(vec![components]).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        entities.into_iter().next()
            .ok_or_else(|| UiError::CreationFailed("Failed to create root".into()))
    }
    
    async fn update_layout(&mut self) -> UiResult<()> {
        let dirty = self.dirty_elements.read().await.clone();
        
        if dirty.is_empty() {
            return Ok(());
        }
        
        let mut layout_engine = self.layout_engine.write().await;
        
        for entity in dirty {
            layout_engine.calculate_layout(
                entity,
                &self.element_graph,
                &self.world,
                self.screen_size,
            ).await?;
        }
        
        Ok(())
    }
    
    async fn render_element_tree(
        &self,
        entity: EntityId,
        batch: &mut RenderCommandBatch,
        theme: &Theme,
    ) -> UiResult<()> {
        let world = self.world.read().await;
        
        // Get all components for this element
        let element = world.get_component::<UiElementComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        let layout = world.get_component::<UiLayoutComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        let style = world.get_component::<UiStyleComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        // Convert to render commands
        ui_to_render_commands(&element, &layout, &style, theme, batch)?;
        
        // Render children
        let graph = self.element_graph.read().await;
        if let Some(children) = graph.get_children(entity) {
            for &child in children {
                Box::pin(self.render_element_tree(child, batch, theme)).await?;
            }
        }
        
        Ok(())
    }
    
    async fn mark_subtree_dirty(&self, root: EntityId) -> UiResult<()> {
        let graph = self.element_graph.read().await;
        let mut dirty = self.dirty_elements.write().await;
        
        // Use depth-first iterator to mark entire subtree
        for element in graph.iter_depth_first(root) {
            dirty.push(element);
        }
        
        Ok(())
    }
    
    async fn send_batch(&self, batch: &RenderCommandBatch) -> UiResult<()> {
        if let Some(ref manager) = self.channel_manager {
            let data = bincode::serialize(batch)
                .map_err(|e| UiError::SerializationError(e.to_string()))?;
            
            let packet = Packet {
                channel_id: self.channel_id,
                packet_type: 100, // RenderBatch type
                priority: Priority::High,
                payload: bytes::Bytes::from(data),
            };
            
            // TODO: Send packet through proper channel
            // For now, just skip sending until networking is properly connected
            // manager.read().await.send_to_channel(self.channel_id, packet.payload.clone()).await
            //     .map_err(|e| UiError::NetworkError(e.to_string()))?;
        }
        
        Ok(())
    }
}