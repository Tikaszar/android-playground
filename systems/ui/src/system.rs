use playground_core_rendering::{RenderCommand, RenderCommandBatch, Viewport};
use playground_core_ecs::{World, EntityId, ComponentRegistry, Component};
use playground_core_types::{Handle, handle, Shared, shared};
use playground_core_server::{ChannelManager, Packet, Priority};
use playground_systems_networking::NetworkingSystem;
use playground_core_ui::{
    UiRenderer, UiElement, UiCommand, UiEvent as CoreUiEvent, EventResult,
    ElementId as CoreElementId, ElementType as CoreElementType, 
    ElementUpdate, Style as CoreStyle, Bounds as CoreBounds,
    Orientation, UiError as CoreUiError, UiResult as CoreUiResult
};
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
use async_trait::async_trait;

pub struct UiSystem {
    // Core ECS
    world: Handle<World>,
    registry: Handle<ComponentRegistry>,
    
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
    networking_system: Option<Handle<NetworkingSystem>>,
    channel_id: u16,
    
    // State
    initialized: bool,
    screen_size: [f32; 2],
}

impl UiSystem {
    pub fn new() -> Self {
        let registry = handle(ComponentRegistry::new());
        let world = handle(World::with_registry(registry.clone()));
        
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
            networking_system: None,
            channel_id: 10,
            initialized: false,
            screen_size: [1920.0, 1080.0],
        }
    }
    
    pub async fn initialize(&mut self) -> UiResult<()> {
        self.log("Info", format!("[UiSystem] initialize() called, initialized={}", self.initialized)).await;
        
        if self.initialized {
            return Err(UiError::AlreadyInitialized);
        }
        
        // Register all component types
        self.log("Info", "[UiSystem] Registering components...".to_string()).await;
        self.register_components().await?;
        self.log("Info", "[UiSystem] Components registered".to_string()).await;
        
        // Load default themes
        self.log("Info", "[UiSystem] Loading default themes...".to_string()).await;
        let mut theme_mgr = self.theme_manager.write().await;
        theme_mgr.load_default_themes()?;
        drop(theme_mgr);
        self.log("Info", "[UiSystem] Themes loaded".to_string()).await;
        
        // Create root element
        self.log("Info", "[UiSystem] Creating root element...".to_string()).await;
        let root_entity = self.create_root().await?;
        self.log("Info", format!("[UiSystem] Root element created: {:?}", root_entity)).await;
        self.root_entity = Some(root_entity);
        self.log("Info", format!("[UiSystem] root_entity set to: {:?}", self.root_entity)).await;
        
        // Initialize mobile features if on mobile
        self.log("Info", "[UiSystem] Initializing mobile features...".to_string()).await;
        let mut mobile = self.mobile_features.write().await;
        mobile.initialize().await?;
        drop(mobile);
        self.log("Info", "[UiSystem] Mobile features initialized".to_string()).await;
        
        self.initialized = true;
        self.log("Info", format!("[UiSystem] Initialization complete, initialized={}, root_entity={:?}", 
                     self.initialized, self.root_entity)).await;
        Ok(())
    }
    
    
    pub async fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.screen_size = [viewport.width as f32, viewport.height as f32];
        
        // Mark all elements as dirty for relayout
        if let Some(root) = self.root_entity {
            self.mark_subtree_dirty(root).await.ok();
        }
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
        self.log("Info", format!("[UiSystem] create_element called: type={}, parent={:?}", element_type, parent)).await;
        
        // Now that world is Handle<World>, we can call methods directly without locking
        let entities = self.world.spawn_batch(vec![vec![]]).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let entity = entities.into_iter().next()
            .ok_or_else(|| UiError::CreationFailed("Failed to create element".into()))?;
        
        // Add components directly - World handles its own internal locking
        let element_component = Component::new(UiElementComponent::new(element_type)).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(element_component),
            <UiElementComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let layout_component = Component::new(UiLayoutComponent::default()).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(layout_component),
            <UiLayoutComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let style_component = Component::new(UiStyleComponent::default()).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(style_component),
            <UiStyleComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let input_component = Component::new(UiInputComponent::default()).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(input_component),
            <UiInputComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        // Add text component for text elements
        if element_type == "text" || element_type == "input" {
            let text_component = Component::new(UiTextComponent::new(String::new())).await
                .map_err(|e| UiError::EcsError(e.to_string()))?;
            self.world.add_component_raw(
                entity,
                Box::new(text_component),
                <UiTextComponent as playground_core_ecs::ComponentData>::component_id()
            ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        }
        
        self.log("Info", format!("[UiSystem] Entity created: {:?}", entity)).await;
        
        // Add to element graph
        if let Some(parent) = parent {
            self.log("Info", format!("[UiSystem] Adding entity {:?} as child of {:?}", entity, parent)).await;
            let mut graph = self.element_graph.write().await;
            graph.add_child(parent, entity)?;
            drop(graph);
            self.log("Info", "[UiSystem] Added to element graph".to_string()).await;
        }
        
        // Mark as dirty for layout
        self.log("Info", format!("[UiSystem] Marking entity {:?} as dirty", entity)).await;
        self.dirty_elements.write().await.push(entity);
        
        self.log("Info", format!("[UiSystem] create_element complete, returning entity: {:?}", entity)).await;
        Ok(entity)
    }
    
    pub async fn remove_element(&mut self, element: EntityId) -> UiResult<()> {
        // Remove from graph
        let mut graph = self.element_graph.write().await;
        graph.remove_element(element);
        drop(graph);
        
        // Remove from world - World has Arc now and handles its own internal locking
        self.world.despawn_batch(vec![element]).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn set_element_text(&mut self, element: EntityId, text: String) -> UiResult<()> {
        // Get the current component - World is Arc now and handles its own internal locking
        let current_component = self.world.get_component::<UiElementComponent>(element).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        // Create updated component with new text
        let mut updated_component = current_component.clone();
        updated_component.text_content = Some(text);
        
        // Remove old component and add updated one
        self.world.remove_component_raw(element, <UiElementComponent as playground_core_ecs::ComponentData>::component_id()).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let component = Component::new(updated_component).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            element,
            Box::new(component),
            <UiElementComponent as playground_core_ecs::ComponentData>::component_id()
        ).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        // Mark as dirty for re-render
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
    
    // Public API methods for plugins
    
    pub fn get_root_element(&self) -> Option<ElementId> {
        // Log is async but this method is not, so we can't log here
        self.root_entity
    }
    
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    pub async fn set_element_style(&mut self, element: ElementId, style: crate::types::ElementStyle) -> UiResult<()> {
        self.log("Info", format!("[UiSystem] set_element_style called for element: {:?}", element)).await;
        
        // Convert public ElementStyle to internal UiStyleComponent
        let ui_style = UiStyleComponent {
            background_color: Some(nalgebra::Vector4::new(
                style.background_color[0],
                style.background_color[1],
                style.background_color[2],
                style.background_color[3],
            )),
            text_color: Some(nalgebra::Vector4::new(
                style.text_color[0],
                style.text_color[1],
                style.text_color[2],
                style.text_color[3],
            )),
            border_color: Some(nalgebra::Vector4::new(
                style.border_color[0],
                style.border_color[1],
                style.border_color[2],
                style.border_color[3],
            )),
            border_width: style.border_width,
            border_radius: style.border_radius,
            opacity: style.opacity,
            font_size: style.font_size,
            font_family: Some(style.font_family),
            font_weight: match style.font_weight {
                crate::types::FontWeight::Light => crate::components::FontWeight::Light,
                crate::types::FontWeight::Normal => crate::components::FontWeight::Normal,
                crate::types::FontWeight::Bold => crate::components::FontWeight::Bold,
                crate::types::FontWeight::ExtraBold => crate::components::FontWeight::ExtraBold,
            },
            text_align: match style.text_align {
                crate::types::TextAlign::Left => crate::components::TextAlign::Left,
                crate::types::TextAlign::Center => crate::components::TextAlign::Center,
                crate::types::TextAlign::Right => crate::components::TextAlign::Right,
                crate::types::TextAlign::Justify => crate::components::TextAlign::Justify,
            },
            visible: style.visible,
            z_index: style.z_index,
            cursor: None,
            custom_styles: std::collections::HashMap::new(),
        };
        
        // Update component in world
        // We need to use the World directly without holding a guard across await points
        self.log("Info", "[UiSystem] Updating style component...".to_string()).await;
        
        // World is Handle<World> now, we can call methods directly
        let component_id = <UiStyleComponent as playground_core_ecs::ComponentData>::component_id();
        
        // Check if component exists first
        let has_component = self.world.has_component(element, component_id.clone()).await;
        self.log("Info", format!("[UiSystem] Entity has style component: {}", has_component)).await;
        
        if has_component {
            // Remove the old component
            self.log("Info", format!("[UiSystem] Removing existing component for entity {:?}", element)).await;
            let _ = self.world.remove_component_raw(element, component_id.clone()).await;
            self.log("Info", "[UiSystem] Existing component removed".to_string()).await;
        }
        
        // Add the new component
        self.log("Info", format!("[UiSystem] Adding new component for entity {:?}", element)).await;
        let style_component = Component::new(ui_style).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            element,
            Box::new(style_component),
            component_id
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.log("Info", "[UiSystem] Style component updated".to_string()).await;
        
        self.log("Info", format!("[UiSystem] Style component updated for element: {:?}", element)).await;
        
        // Mark as dirty for re-render
        self.dirty_elements.write().await.push(element);
        
        self.log("Info", format!("[UiSystem] set_element_style complete for element: {:?}", element)).await;
        Ok(())
    }
    
    pub async fn set_element_bounds(&mut self, element: ElementId, bounds: crate::types::ElementBounds) -> UiResult<()> {
        // Update layout component with bounds
        let layout = UiLayoutComponent {
            bounds: crate::components::ElementBounds {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
            },
            padding: [0.0; 4],
            margin: [0.0; 4],
            layout_type: crate::components::LayoutType::Absolute,
            flex_direction: crate::components::FlexDirection::Row,
            justify_content: crate::components::JustifyContent::FlexStart,
            align_items: crate::components::AlignItems::FlexStart,
            position_type: crate::components::PositionType::Absolute,
            size: crate::components::Size {
                width: Some(bounds.width),
                height: Some(bounds.height),
            },
            min_size: crate::components::Size { width: None, height: None },
            max_size: crate::components::Size { width: None, height: None },
        };
        
        // World handles its own locking - don't lock it here
        // Remove old layout component if it exists
        let _ = self.world.remove_component_raw(element, <UiLayoutComponent as playground_core_ecs::ComponentData>::component_id()).await;
        
        // Add the new layout component
        let component = Component::new(layout).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            element,
            Box::new(component),
            <UiLayoutComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        // Mark as dirty
        self.dirty_elements.write().await.push(element);
        
        Ok(())
    }
    
    pub async fn mark_dirty(&mut self, element: ElementId) -> UiResult<()> {
        self.dirty_elements.write().await.push(element);
        Ok(())
    }
    
    pub async fn force_layout(&mut self) -> UiResult<()> {
        self.update_layout().await
    }
    
    pub async fn create_element_with_id(
        &mut self,
        id: String,
        element_type: String,
        parent: Option<ElementId>,
    ) -> UiResult<ElementId> {
        self.log("Info", format!("[UiSystem] create_element_with_id called: id={}, type={}, parent={:?}", 
                     id, element_type, parent)).await;
        
        let entity = self.create_element(&element_type, parent).await?;
        self.log("Info", format!("[UiSystem] Element created with entity: {:?}", entity)).await;
        
        // Update element component with the id - World handles its own locking
        self.log("Info", "[UiSystem] Updating component with id...".to_string()).await;
        
        // Get the current element component
        let elem_box = self.world.get_component_raw(entity, <UiElementComponent as playground_core_ecs::ComponentData>::component_id()).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        // Deserialize, update, and re-serialize
        let bytes = elem_box.serialize();
        let mut elem = <UiElementComponent as playground_core_ecs::ComponentData>::deserialize(&bytes).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        elem.id = id.clone();
        
        // Remove old and add new
        let _ = self.world.remove_component_raw(entity, <UiElementComponent as playground_core_ecs::ComponentData>::component_id()).await;
        let component = Component::new(elem).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(component),
            <UiElementComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.log("Info", format!("[UiSystem] Component updated with id={}", id)).await;
        Ok(entity)
    }
    
    async fn register_components(&self) -> UiResult<()> {
        // Registry has internal locking, no need for external lock
        
        // Register components in the registry
        self.registry.register::<UiElementComponent>().await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.registry.register::<UiLayoutComponent>().await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.registry.register::<UiStyleComponent>().await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.registry.register::<UiInputComponent>().await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.registry.register::<UiTextComponent>().await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        // Registry dropped automatically
        
        // Also register storage in the world for each component - World handles its own locking
        self.world.register_component_storage(
            <UiElementComponent as playground_core_ecs::ComponentData>::component_id(),
            playground_core_ecs::StorageType::Dense
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.world.register_component_storage(
            <UiLayoutComponent as playground_core_ecs::ComponentData>::component_id(),
            playground_core_ecs::StorageType::Dense
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.world.register_component_storage(
            <UiStyleComponent as playground_core_ecs::ComponentData>::component_id(),
            playground_core_ecs::StorageType::Dense
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.world.register_component_storage(
            <UiInputComponent as playground_core_ecs::ComponentData>::component_id(),
            playground_core_ecs::StorageType::Dense
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.world.register_component_storage(
            <UiTextComponent as playground_core_ecs::ComponentData>::component_id(),
            playground_core_ecs::StorageType::Sparse
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn create_root(&self) -> UiResult<EntityId> {
        // Create an empty entity first - World handles its own locking
        let entities = self.world.spawn_batch(vec![vec![]]).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let entity = entities.into_iter().next()
            .ok_or_else(|| UiError::CreationFailed("Failed to create root entity".into()))?;
        
        // Now add components individually (avoiding trait object issues)
        let mut root_element = UiElementComponent::new("root");
        root_element.visible = true;
        let component = Component::new(root_element).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity, 
            Box::new(component),
            <UiElementComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let mut root_layout = UiLayoutComponent::default();
        root_layout.bounds = ElementBounds {
            x: 0.0,
            y: 0.0,
            width: self.screen_size[0],
            height: self.screen_size[1],
        };
        root_layout.layout_type = LayoutType::Absolute;
        let component = Component::new(root_layout).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(component),
            <UiLayoutComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let mut root_style = UiStyleComponent::default();
        root_style.visible = true;
        let style_component = Component::new(root_style).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(style_component),
            <UiStyleComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let input_component = Component::new(UiInputComponent::default()).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(input_component),
            <UiInputComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        Ok(entity)
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
        // Get all components for this element - World handles its own locking
        let element = self.world.get_component::<UiElementComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        let layout = self.world.get_component::<UiLayoutComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        let style = self.world.get_component::<UiStyleComponent>(entity).await
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
        // Serialize with bincode (efficient binary format)
        let data = bincode::serialize(batch)
            .map_err(|e| UiError::SerializationError(e.to_string()))?;
        
        // Log that we're sending render commands (if we have dashboard via networking)
        if let Some(ref networking) = self.networking_system {
            if let Some(dashboard) = networking.get_dashboard().await {
                dashboard.log(
                    playground_core_server::dashboard::LogLevel::Debug,
                    format!("UI: Publishing RenderBatch to MessageBus on channel {} (bincode, {} bytes)", 
                        self.channel_id, data.len()),
                    None
                ).await;
            }
        }
        
        // Use NetworkingSystem to send the packet, which will publish to the shared MessageBus
        // The MessageBridge in core/server will forward this to WebSocket clients
        if let Some(ref networking) = self.networking_system {
            networking.send_packet(self.channel_id, 104, data, Priority::High)
                .await
                .map_err(|e| UiError::SerializationError(format!("Failed to send packet: {}", e)))?;
        } else {
            return Err(UiError::NotInitialized);
        }
        
        Ok(())
    }
    
    /// Main render method that generates and sends render commands
    pub async fn render(&mut self) -> UiResult<()> {
        // Log render call
        if let Some(ref networking) = self.networking_system {
            if let Some(dashboard) = networking.get_dashboard().await {
                dashboard.log(
                    playground_core_server::dashboard::LogLevel::Debug,
                    format!("UI: render() called, frame {}", self.frame_id),
                    None
                ).await;
            }
        }
        
        // First update layout for any dirty elements
        self.update_layout().await?;
        
        // Clear dirty list after layout update
        self.dirty_elements.write().await.clear();
        
        // Get the current theme
        let theme_mgr = self.theme_manager.read().await;
        let theme = theme_mgr.get_theme(self.current_theme)?
            .clone();
        drop(theme_mgr);
        
        // Create render command batch
        let mut batch = RenderCommandBatch::new(self.frame_id);
        
        // Start with clear command using theme background
        batch.push(RenderCommand::Clear {
            color: [0.133, 0.137, 0.153, 1.0], // Discord dark background
        });
        
        // Add a test rectangle to see if rendering works
        batch.push(RenderCommand::DrawQuad {
            position: [100.0, 100.0],
            size: [200.0, 150.0],
            color: [1.0, 0.0, 0.0, 1.0], // Red rectangle
        });
        
        // Render the element tree starting from root
        if let Some(root) = self.root_entity {
            // Log if we have a root
            if let Some(ref networking) = self.networking_system {
                if let Some(dashboard) = networking.get_dashboard().await {
                    dashboard.log(
                        playground_core_server::dashboard::LogLevel::Debug,
                        format!("UI: Rendering root entity {:?}", root),
                        None
                    ).await;
                }
            }
            self.render_element_tree(root, &mut batch, &theme).await?;
        } else {
            // Log that we have no root
            if let Some(ref networking) = self.networking_system {
                if let Some(dashboard) = networking.get_dashboard().await {
                    dashboard.log(
                        playground_core_server::dashboard::LogLevel::Warning,
                        "UI: No root entity to render!".to_string(),
                        None
                    ).await;
                }
            }
        }
        
        // Send the batch through channel 10
        self.send_batch(&batch).await?;
        
        // Increment frame counter
        self.frame_id += 1;
        
        Ok(())
    }
    
    /// Set the channel manager for networking
    pub fn set_channel_manager(&mut self, manager: Shared<ChannelManager>) {
        self.channel_manager = Some(manager);
    }
    
    pub fn set_networking_system(&mut self, networking: Handle<NetworkingSystem>) {
        self.networking_system = Some(networking);
    }
    
    /// Log a message to the dashboard via NetworkingSystem
    async fn log(&self, level: &str, message: String) {
        if let Some(ref networking) = self.networking_system {
            // Get dashboard reference
            let dashboard = networking.get_dashboard().await;
            
            if let Some(dashboard) = dashboard {
                use playground_core_server::dashboard::LogLevel;
                let log_level = match level {
                    "error" | "Error" => LogLevel::Error,
                    "warn" | "Warning" => LogLevel::Warning,
                    "info" | "Info" => LogLevel::Info,
                    "debug" | "Debug" => LogLevel::Debug,
                    _ => LogLevel::Info,
                };
                dashboard.log(log_level, message, None).await;
            }
        }
    }
    
    /// Send renderer initialization message to a new client
    pub async fn initialize_client_renderer(&self, client_id: usize) -> UiResult<()> {
        use crate::messages::{RendererInitMessage, ViewportConfig, BlendMode, ShaderProgram, UiPacketType};
        
        // Create initialization message with default shaders
        let init_msg = RendererInitMessage {
            viewport: ViewportConfig {
                width: self.viewport.width,
                height: self.viewport.height,
                device_pixel_ratio: 1.0,
            },
            clear_color: [0.133, 0.137, 0.153, 1.0], // Discord dark background
            blend_mode: BlendMode::Normal,
            shaders: vec![
                ShaderProgram {
                    id: "quad".to_string(),
                    vertex_source: self.get_quad_vertex_shader(),
                    fragment_source: self.get_quad_fragment_shader(),
                },
                ShaderProgram {
                    id: "line".to_string(),
                    vertex_source: self.get_line_vertex_shader(),
                    fragment_source: self.get_line_fragment_shader(),
                },
                ShaderProgram {
                    id: "text".to_string(),
                    vertex_source: self.get_text_vertex_shader(),
                    fragment_source: self.get_text_fragment_shader(),
                },
            ],
        };
        
        // Serialize the message
        let data = bincode::serialize(&init_msg)
            .map_err(|e| UiError::SerializationError(e.to_string()))?;
        
        // Send via networking system
        if let Some(ref networking) = self.networking_system {
            // Send packet
            networking.send_packet(self.channel_id, UiPacketType::RendererInit as u16, data, Priority::High)
                .await
                .map_err(|e| UiError::SerializationError(format!("Failed to send init packet: {}", e)))?;
            
            // Get dashboard
            let dashboard = networking.get_dashboard().await;
            
            // Log the initialization
            if let Some(dashboard) = dashboard {
                dashboard.log(
                    playground_core_server::dashboard::LogLevel::Info,
                    format!("Initialized renderer for client {}", client_id),
                    Some(client_id)
                ).await;
            }
        }
        
        Ok(())
    }
    
    /// Get default quad vertex shader source
    fn get_quad_vertex_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec2 a_position;
        in vec2 a_texCoord;
        in vec4 a_color;
        
        uniform mat3 u_projection;
        uniform mat3 u_transform;
        
        out vec2 v_texCoord;
        out vec4 v_color;
        
        void main() {
            vec3 position = u_projection * u_transform * vec3(a_position, 1.0);
            gl_Position = vec4(position.xy, 0.0, 1.0);
            v_texCoord = a_texCoord;
            v_color = a_color;
        }"#.to_string()
    }
    
    /// Get default quad fragment shader source
    fn get_quad_fragment_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        in vec4 v_color;
        
        uniform sampler2D u_texture;
        uniform bool u_useTexture;
        
        out vec4 fragColor;
        
        void main() {
            if (u_useTexture) {
                fragColor = texture(u_texture, v_texCoord) * v_color;
            } else {
                fragColor = v_color;
            }
        }"#.to_string()
    }
    
    /// Get default line vertex shader source
    fn get_line_vertex_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec2 a_position;
        in vec4 a_color;
        
        uniform mat3 u_projection;
        uniform mat3 u_transform;
        
        out vec4 v_color;
        
        void main() {
            vec3 position = u_projection * u_transform * vec3(a_position, 1.0);
            gl_Position = vec4(position.xy, 0.0, 1.0);
            v_color = a_color;
        }"#.to_string()
    }
    
    /// Get default line fragment shader source
    fn get_line_fragment_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec4 v_color;
        out vec4 fragColor;
        
        void main() {
            fragColor = v_color;
        }"#.to_string()
    }
    
    /// Get default text vertex shader source
    fn get_text_vertex_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec2 a_position;
        in vec2 a_texCoord;
        in vec4 a_color;
        
        uniform mat3 u_projection;
        uniform mat3 u_transform;
        
        out vec2 v_texCoord;
        out vec4 v_color;
        
        void main() {
            vec3 position = u_projection * u_transform * vec3(a_position, 1.0);
            gl_Position = vec4(position.xy, 0.0, 1.0);
            v_texCoord = a_texCoord;
            v_color = a_color;
        }"#.to_string()
    }
    
    /// Get default text fragment shader source  
    fn get_text_fragment_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        in vec4 v_color;
        
        uniform sampler2D u_texture;
        
        out vec4 fragColor;
        
        void main() {
            float alpha = texture(u_texture, v_texCoord).a;
            fragColor = vec4(v_color.rgb, v_color.a * alpha);
        }"#.to_string()
    }
}

// Implement UiRenderer trait from core/ui
#[async_trait]
impl UiRenderer for UiSystem {
    async fn initialize(&mut self) -> CoreUiResult<()> {
        self.initialize().await
            .map_err(|e| CoreUiError::NotInitialized)
    }
    
    async fn create_element(
        &mut self,
        element_type: CoreElementType,
        parent: Option<CoreElementId>,
    ) -> CoreUiResult<CoreElementId> {
        // Map core element type to our internal type
        let element_type_str = match element_type {
            CoreElementType::Panel => "panel",
            CoreElementType::Text => "text",
            CoreElementType::Button => "button",
            CoreElementType::Input => "input",
            CoreElementType::Image => "image",
            CoreElementType::ScrollView => "scrollview",
            CoreElementType::List => "list",
            CoreElementType::Grid => "grid",
            CoreElementType::Canvas => "canvas",
            CoreElementType::Custom => "custom",
        };
        
        // Convert parent ID if provided
        let parent_entity = if let Some(_parent_id) = parent {
            // For now, we'll need to maintain a mapping between CoreElementId and EntityId
            // This is a simplification - in production you'd have a proper mapping
            None
        } else {
            self.root_entity
        };
        
        let entity = self.create_element(element_type_str, parent_entity).await
            .map_err(|e| CoreUiError::InvalidOperation(e.to_string()))?;
        
        // Create a CoreElementId from the entity
        Ok(CoreElementId(Uuid::new_v4()))
    }
    
    async fn update_element(
        &mut self,
        id: CoreElementId,
        update: ElementUpdate,
    ) -> CoreUiResult<()> {
        // This would need proper ID mapping in production
        match update {
            ElementUpdate::Text(text) => {
                // Find entity and update text
                Ok(())
            }
            ElementUpdate::Style(style) => {
                // Convert CoreStyle to our internal style and apply
                Ok(())
            }
            ElementUpdate::Bounds(bounds) => {
                // Update element bounds
                Ok(())
            }
            _ => Ok(())
        }
    }
    
    async fn remove_element(&mut self, id: CoreElementId) -> CoreUiResult<()> {
        // Find and remove element
        Ok(())
    }
    
    async fn get_element(&self, id: CoreElementId) -> CoreUiResult<Box<dyn UiElement>> {
        // NOTE: This violates NO dyn rule, but is required by core/ui trait
        // Always return error to avoid using trait objects
        Err(CoreUiError::ElementNotFound(format!("{:?}", id)))
    }
    
    async fn process_command(&mut self, command: UiCommand) -> CoreUiResult<()> {
        match command {
            UiCommand::CreateElement { id, element_type, parent } => {
                // Map core element type to our internal type
                let element_type_str = match element_type {
                    CoreElementType::Panel => "panel",
                    CoreElementType::Text => "text",
                    CoreElementType::Button => "button",
                    CoreElementType::Input => "input",
                    CoreElementType::Image => "image",
                    CoreElementType::ScrollView => "scrollview",
                    CoreElementType::List => "list",
                    CoreElementType::Grid => "grid",
                    CoreElementType::Canvas => "canvas",
                    CoreElementType::Custom => "custom",
                };
                
                // Convert parent CoreElementId to internal EntityId (would need mapping)
                let parent_entity = self.root_entity;
                self.create_element(element_type_str, parent_entity).await
                    .map_err(|e| CoreUiError::InvalidOperation(e.to_string()))?;
                Ok(())
            }
            UiCommand::SetText { id, text } => {
                self.update_element(id, ElementUpdate::Text(text)).await
            }
            _ => Ok(())
        }
    }
    
    async fn handle_event(&mut self, event: CoreUiEvent) -> CoreUiResult<EventResult> {
        // Convert core event to our internal event and handle
        Ok(EventResult::Ignored)
    }
    
    async fn calculate_layout(&mut self) -> CoreUiResult<()> {
        self.update_layout().await
            .map_err(|e| CoreUiError::LayoutFailed(e.to_string()))
    }
    
    async fn render_frame(&mut self, frame_id: u64) -> CoreUiResult<RenderCommandBatch> {
        self.frame_id = frame_id;
        
        // Create render command batch
        let mut batch = RenderCommandBatch::new(frame_id);
        
        // Clear with mobile-friendly dark background (Discord-like)
        batch.push(RenderCommand::Clear {
            color: [0.133, 0.137, 0.153, 1.0],
        });
        
        // Render the UI tree
        if let Some(root) = self.root_entity {
            let theme_mgr = self.theme_manager.read().await;
            let theme = theme_mgr.get_theme(self.current_theme)
                .map_err(|e| CoreUiError::RenderingFailed(e.to_string()))?
                .clone();
            drop(theme_mgr);
            
            self.render_element_tree(root, &mut batch, &theme).await
                .map_err(|e| CoreUiError::RenderingFailed(e.to_string()))?;
        }
        
        Ok(batch)
    }
    
    async fn get_root(&self) -> Option<CoreElementId> {
        // Would need proper ID mapping
        self.root_entity.map(|_| CoreElementId(Uuid::new_v4()))
    }
    
    async fn set_viewport(&mut self, width: f32, height: f32) -> CoreUiResult<()> {
        self.viewport = Viewport {
            x: 0,
            y: 0,
            width: width as u32,
            height: height as u32,
        };
        self.screen_size = [width, height];
        
        // Update root element bounds
        if let Some(_root) = self.root_entity {
            // TODO: Update root layout bounds to match viewport
        }
        
        Ok(())
    }
    
    async fn set_safe_area_insets(
        &mut self,
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
    ) -> CoreUiResult<()> {
        // Store safe area insets for mobile layout
        let mut mobile = self.mobile_features.write().await;
        mobile.set_safe_area_insets(top, bottom, left, right).await
            .map_err(|e| CoreUiError::InvalidOperation(e.to_string()))?;
        Ok(())
    }
    
    async fn handle_orientation_change(&mut self, orientation: Orientation) -> CoreUiResult<()> {
        // Handle mobile orientation change
        let mut mobile = self.mobile_features.write().await;
        
        // Update screen size based on orientation
        match orientation {
            Orientation::Portrait | Orientation::PortraitUpsideDown => {
                // Portrait mode - taller than wide
                if self.screen_size[0] > self.screen_size[1] {
                    self.screen_size = [self.screen_size[1], self.screen_size[0]];
                }
            }
            Orientation::LandscapeLeft | Orientation::LandscapeRight => {
                // Landscape mode - wider than tall
                if self.screen_size[1] > self.screen_size[0] {
                    self.screen_size = [self.screen_size[1], self.screen_size[0]];
                }
            }
        }
        
        // Mark all elements as dirty for re-layout
        if let Some(root) = self.root_entity {
            self.mark_subtree_dirty(root).await
                .map_err(|e| CoreUiError::LayoutFailed(e.to_string()))?;
        }
        
        Ok(())
    }
}