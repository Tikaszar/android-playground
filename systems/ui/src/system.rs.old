//! Simplified UI system implementation using core/ecs for internal state

use crate::error::{UiError, UiResult};
use crate::components::{
    UiElementComponent, UiLayoutComponent, UiStyleComponent, 
    UiDirtyComponent, UiInputComponent, UiWebSocketComponent, 
    UiTextComponent, AlignSelf, JustifySelf
};
use crate::element::ElementBounds;
use crate::layout::LayoutConstraints;
use crate::input::InputManager;
use crate::rendering::UiRenderer;
use crate::theme::{ThemeManager, ThemeId};
use std::collections::HashMap;
use std::default::Default;
use crate::messages::{
    UiPacketType, CreateElementMessage, UpdateElementMessage, InputEventMessage,
    TerminalInputMessage, TerminalOutputMessage, TerminalConnectMessage,
    TerminalStateMessage, RenderBatchMessage, RenderCommand, serialize_message, deserialize_message,
};
use nalgebra::{Vector2, Vector4};
use playground_core_ecs::{World, EntityId, ComponentRegistry, Component};
use playground_systems_rendering::BaseRenderer;
use playground_core_server::{ChannelManager, Packet, Priority, FrameBatcher};
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Simplified UI system struct using ECS for internal state management
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
    channel_manager: Option<Arc<RwLock<ChannelManager>>>,
    batcher: Option<Arc<FrameBatcher>>,
    terminal_connections: Arc<RwLock<HashMap<Uuid, EntityId>>>,
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
            batcher: None,
            terminal_connections: Arc::new(RwLock::new(HashMap::new())),
            current_frame: 0,
        }
    }

    /// Initialize the UI system without a renderer (for server-side command generation)
    pub async fn initialize_headless(&mut self) -> UiResult<()> {
        if self.initialized {
            return Err(UiError::InitializationFailed("Already initialized".to_string()));
        }
        
        // Register UI components with ECS
        self.register_components().await?;
        
        // Initialize theme manager with default themes
        self.theme_manager.load_default_themes()?;
        
        // Create root UI entity
        self.root_entity = Some(self.create_root_entity().await?);
        
        self.initialized = true;
        Ok(())
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
    
    /// Set up the WebSocket channel connection for sending render commands
    pub async fn setup_channel(&mut self, channel_manager: Arc<RwLock<ChannelManager>>, batcher: Arc<FrameBatcher>) {
        self.channel_manager = Some(channel_manager);
        self.batcher = Some(batcher);
        self.channel_id = Some(10); // UI system uses channel 10
    }
    
    /// Register all UI components with the ECS registry
    async fn register_components(&mut self) -> UiResult<()> {
        // Register components with the registry (async)
        self.registry.register::<UiElementComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiElementComponent: {}", e)))?;
        
        self.registry.register::<UiLayoutComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiLayoutComponent: {}", e)))?;
        
        self.registry.register::<UiStyleComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiStyleComponent: {}", e)))?;
        
        self.registry.register::<UiDirtyComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiDirtyComponent: {}", e)))?;
        
        self.registry.register::<UiInputComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiInputComponent: {}", e)))?;
        
        self.registry.register::<UiWebSocketComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiWebSocketComponent: {}", e)))?;
        
        self.registry.register::<UiTextComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiTextComponent: {}", e)))?;
        
        // Now register the components with the world's storage
        self.world.register_component::<UiElementComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiElementComponent with world: {}", e)))?;
        
        self.world.register_component::<UiLayoutComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiLayoutComponent with world: {}", e)))?;
        
        self.world.register_component::<UiStyleComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiStyleComponent with world: {}", e)))?;
        
        self.world.register_component::<UiDirtyComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiDirtyComponent with world: {}", e)))?;
        
        self.world.register_component::<UiInputComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiInputComponent with world: {}", e)))?;
        
        self.world.register_component::<UiWebSocketComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiWebSocketComponent with world: {}", e)))?;
        
        self.world.register_component::<UiTextComponent>().await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UiTextComponent with world: {}", e)))?;
        
        Ok(())
    }
    
    /// Create a UI element
    pub async fn create_element(&self, name: String, tag: String, parent: Option<EntityId>) -> UiResult<EntityId> {
        // Create empty entity
        let entities = self.world.spawn_batch(vec![vec![]]).await
            .map_err(|e| UiError::Other(format!("Failed to spawn entity: {}", e)))?;
        
        let entity = entities[0];
        
        // Add element component
        let element = UiElementComponent {
            id: Uuid::new_v4(),
            name,
            tag,
            bounds: ElementBounds {
                position: Vector2::new(0.0, 0.0),
                size: Vector2::new(100.0, 100.0),
            },
            children: Vec::new(),
            parent,
            visible: true,
            interactive: true,
            z_index: 0,
        };
        
        let component_box = Box::new(element) as Box<dyn playground_core_ecs::Component>;
        let component_id = <UiElementComponent as Component>::component_id();
        self.world.add_component_raw(entity, component_box, component_id).await
            .map_err(|e| UiError::Other(format!("Failed to add element component: {}", e)))?;
        
        // Mark as dirty to trigger render
        self.mark_dirty(entity).await?;
        
        Ok(entity)
    }
    
    /// Create the root UI entity
    async fn create_root_entity(&self) -> UiResult<EntityId> {
        // For now, just create an empty entity since spawn_batch has issues with boxed components
        // We'll add components later when we have proper typed methods
        let entities = self.world.spawn_batch(vec![vec![]]).await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to create root entity: {}", e)))?;
        
        let root_entity = entities[0];
        
        // Add root element component
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
        
        let component_box = Box::new(root_element) as Box<dyn playground_core_ecs::Component>;
        let component_id = <UiElementComponent as Component>::component_id();
        self.world.add_component_raw(root_entity, component_box, component_id).await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to add root element: {}", e)))?;
        
        // Add root style
        let root_style = UiStyleComponent {
            theme_id: ThemeId(0),
            background_color: Vector4::new(0.137, 0.145, 0.161, 1.0), // Discord dark
            border_color: Vector4::new(0.0, 0.0, 0.0, 0.0),
            text_color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            border_width: 0.0,
            border_radius: 0.0,
            opacity: 1.0,
            custom_properties: Default::default(),
        };
        
        let component_box = Box::new(root_style) as Box<dyn playground_core_ecs::Component>;
        let component_id = <UiStyleComponent as Component>::component_id();
        self.world.add_component_raw(root_entity, component_box, component_id).await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to add root style: {}", e)))?;
        
        Ok(root_entity)
    }
    
    /// Register UI system with core/server for WebSocket communication
    pub async fn register_with_server(&mut self, channel_manager: Arc<RwLock<ChannelManager>>) -> UiResult<()> {
        // Register UI system on channel 10
        let channel_id = channel_manager.write().await.register_system("ui".to_string(), 10).await
            .map_err(|e| UiError::InitializationFailed(format!("Failed to register UI channel: {}", e)))?;
        
        // Create frame batcher for packet batching (60fps)
        let batcher = Arc::new(FrameBatcher::new(1024, 60));
        
        self.channel_id = Some(channel_id);
        self.channel_manager = Some(channel_manager);
        self.batcher = Some(batcher);
        
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
        
        // For now, simplified layout without complex queries
        // This would need to be expanded with proper component access
        
        Ok(())
    }

    /// Render the UI
    pub async fn render(&mut self) -> UiResult<()> {
        if !self.initialized {
            return Err(UiError::InitializationFailed("UI system not initialized".to_string()));
        }
        
        self.current_frame += 1;
        
        // Build query for dirty elements
        let query = self.world.query()
            .with_component(<UiDirtyComponent as playground_core_ecs::Component>::component_id())
            .with_component(<UiElementComponent as playground_core_ecs::Component>::component_id())
            .build();
        
        let dirty_entities = self.world.execute_query(query.as_ref()).await
            .map_err(|e| UiError::Other(format!("Query failed: {}", e)))?;
        
        // Generate render commands
        let mut commands = Vec::new();
        
        // Clear screen with Discord dark background
        commands.push(RenderCommand::Clear { 
            color: Vector4::new(0.137, 0.145, 0.161, 1.0)  // #2f3136
        });
        
        // Render each dirty element
        for entity in dirty_entities {
            // Get element component
            if let Ok(element) = self.world.get_component::<UiElementComponent>(entity).await {
                if !element.visible {
                    continue;
                }
                
                // Get style component if it exists
                let style = self.world.get_component::<UiStyleComponent>(entity).await.ok();
                
                // Draw element background
                if let Some(ref style) = style {
                    commands.push(RenderCommand::DrawQuad {
                        position: element.bounds.position,
                        size: element.bounds.size,
                        color: style.background_color,
                    });
                }
                
                // Draw text if present
                if let Ok(text) = self.world.get_component::<UiTextComponent>(entity).await {
                    let text_color = style.as_ref()
                        .map(|s| s.text_color)
                        .unwrap_or_else(|| Vector4::new(1.0, 1.0, 1.0, 1.0));
                    
                    commands.push(RenderCommand::DrawText {
                        position: element.bounds.position + Vector2::new(10.0, 10.0), // Padding
                        text: text.text.clone(),
                        size: text.font_size,
                        color: text_color,
                    });
                }
            }
            
            // Clear dirty flag after rendering
            // TODO: Add remove_component method to World
            // self.world.remove_component::<UiDirtyComponent>(entity).await
            //     .ok(); // Ignore error if component doesn't exist
        }
        
        // Send render batch if we have commands
        if !commands.is_empty() {
            let batch = RenderBatchMessage {
                frame_id: self.current_frame,
                commands,
            };
            
            let payload = serialize_message(&batch)?;
            self.send_packet(UiPacketType::RenderBatch, payload).await?;
        }
        
        Ok(())
    }

    /// Update the UI
    pub async fn update(&mut self, _delta_time: f32) -> UiResult<()> {
        if !self.initialized {
            return Err(UiError::InitializationFailed("UI system not initialized".to_string()));
        }
        
        // Run garbage collection on ECS
        self.world.run_gc().await
            .map_err(|e| UiError::Other(format!("GC failed: {}", e)))?;
        
        // Process WebSocket messages if connected
        if let Some(_channel_id) = self.channel_id {
            // Process messages through channel manager
        }
        
        // Render the UI each frame
        self.render().await?;
        
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
    pub async fn memory_stats(&self) -> UiResult<playground_core_ecs::MemoryStats> {
        let stats = self.world.memory_stats().await;
        Ok(stats)
    }
    
    /// Mark an element as dirty for rendering
    pub async fn mark_dirty(&self, entity: EntityId) -> UiResult<()> {
        let dirty = UiDirtyComponent {
            layout_dirty: true,
            style_dirty: true,
            content_dirty: true,
            last_render_frame: self.current_frame,
        };
        
        // Use add_component_raw with the component boxed
        let component_box = Box::new(dirty) as Box<dyn playground_core_ecs::Component>;
        let component_id = <UiDirtyComponent as Component>::component_id();
        
        self.world.add_component_raw(entity, component_box, component_id).await
            .map_err(|e| UiError::Other(format!("Failed to mark dirty: {}", e)))?;
        
        Ok(())
    }
    
    /// Send a packet through the WebSocket channel
    pub async fn send_packet(&self, packet_type: UiPacketType, payload: Bytes) -> UiResult<()> {
        let channel_id = self.channel_id
            .ok_or_else(|| UiError::ChannelError("UI system not registered with server".to_string()))?;
        
        let batcher = self.batcher.as_ref()
            .ok_or_else(|| UiError::ChannelError("Batcher not initialized".to_string()))?;
        
        let packet = Packet::new(
            channel_id,
            packet_type as u16,
            Priority::Medium,
            payload,
        );
        
        batcher.queue_packet(packet).await;
        Ok(())
    }
    
    /// Process incoming WebSocket message
    pub async fn handle_message(&mut self, packet: Packet) -> UiResult<()> {
        let packet_type = UiPacketType::try_from(packet.packet_type)?;
        
        match packet_type {
            UiPacketType::CreateElement => {
                let msg: CreateElementMessage = deserialize_message(&packet.payload)?;
                self.handle_create_element(msg).await?;
            }
            UiPacketType::UpdateElement => {
                let msg: UpdateElementMessage = deserialize_message(&packet.payload)?;
                self.handle_update_element(msg).await?;
            }
            UiPacketType::InputEvent => {
                let msg: InputEventMessage = deserialize_message(&packet.payload)?;
                self.handle_input_event(msg).await?;
            }
            UiPacketType::TerminalInput => {
                let msg: TerminalInputMessage = deserialize_message(&packet.payload)?;
                self.handle_terminal_input(msg).await?;
            }
            UiPacketType::TerminalConnect => {
                let msg: TerminalConnectMessage = deserialize_message(&packet.payload)?;
                self.handle_terminal_connect(msg).await?;
            }
            _ => {
                // Ignore unhandled packet types for now
            }
        }
        
        Ok(())
    }
    
    /// Handle create element message
    async fn handle_create_element(&mut self, msg: CreateElementMessage) -> UiResult<()> {
        // Find parent entity if specified
        let parent_entity = if let Some(_parent_id) = msg.parent_id {
            // TODO: Implement UUID to EntityId mapping
            self.root_entity
        } else {
            self.root_entity
        };
        
        // Clone values we need to reuse
        let element_type = msg.element_type.clone();
        let name = msg.name.clone();
        
        // Create new element
        let entity = self.create_element(
            name,
            element_type.clone(),
            parent_entity,
        ).await?;
        
        // Send response
        let response = CreateElementMessage {
            parent_id: msg.parent_id,
            element_type,
            name: format!("Created entity: {:?}", entity),
            position: msg.position,
            size: msg.size,
        };
        
        let payload = serialize_message(&response)?;
        self.send_packet(UiPacketType::ElementCreated, payload).await?;
        
        Ok(())
    }
    
    /// Handle update element message
    async fn handle_update_element(&mut self, _msg: UpdateElementMessage) -> UiResult<()> {
        // TODO: Implement element updates via ECS components
        Ok(())
    }
    
    /// Handle input event message
    async fn handle_input_event(&mut self, _msg: InputEventMessage) -> UiResult<()> {
        // TODO: Route input events to appropriate elements
        Ok(())
    }
    
    /// Handle terminal input message
    async fn handle_terminal_input(&mut self, msg: TerminalInputMessage) -> UiResult<()> {
        // Forward terminal input to the terminal connection
        let connections = self.terminal_connections.read().await;
        if let Some(_entity_id) = connections.get(&msg.terminal_id) {
            // TODO: Send input to actual terminal process
            
            // For now, echo the input back as output
            let output = TerminalOutputMessage {
                terminal_id: msg.terminal_id,
                output: format!("$ {}", msg.input),
                is_error: false,
            };
            
            let payload = serialize_message(&output)?;
            self.send_packet(UiPacketType::TerminalOutput, payload).await?;
        }
        
        Ok(())
    }
    
    /// Handle terminal connect message
    async fn handle_terminal_connect(&mut self, msg: TerminalConnectMessage) -> UiResult<()> {
        // Create a terminal entity
        let entity = self.create_element(
            format!("terminal-{}", msg.terminal_id),
            "terminal".to_string(),
            self.root_entity,
        ).await?;
        
        // Store the terminal connection
        let mut connections = self.terminal_connections.write().await;
        connections.insert(msg.terminal_id, entity);
        
        // Send connection status
        let state = TerminalStateMessage {
            terminal_id: msg.terminal_id,
            connected: true,
            ready: true,
        };
        
        let payload = serialize_message(&state)?;
        self.send_packet(UiPacketType::TerminalState, payload).await?;
        
        Ok(())
    }
    
    /// Send render batch to clients
    pub async fn send_render_batch(&self, batch: RenderBatchMessage) -> UiResult<()> {
        let payload = serialize_message(&batch)?;
        self.send_packet(UiPacketType::RenderBatch, payload).await?;
        Ok(())
    }
}

impl Default for UiSystem {
    fn default() -> Self {
        Self::new()
    }
}