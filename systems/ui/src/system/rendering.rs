use crate::{
    error::{UiResult, UiError},
    messages::{RendererInitMessage, ViewportConfig, BlendMode, ShaderProgram, UiPacketType},
    theme::Theme,
    types::ElementBounds,
    components::{UiElementComponent, UiLayoutComponent, UiStyleComponent},
    rendering::ui_to_render_commands,
    system::UiSystem,
};
use playground_core_rendering::{RenderCommandBatch, RenderCommand};
use playground_core_types::Shared;
use playground_systems_networking::NetworkingSystem;
use playground_core_types::Priority;
use playground_core_ecs::EntityId;

impl UiSystem {
    pub async fn render_element_tree(
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
    
    pub async fn send_batch(&self, batch: &RenderCommandBatch) -> UiResult<()> {
        // Serialize with bincode (efficient binary format)
        let data = bincode::serialize(batch)
            .map_err(|e| UiError::SerializationError(e.to_string()))?;
        
        // Log that we're sending render commands (if we have dashboard via networking)
        if let Some(ref networking) = self.networking_system {
            let networking = networking.read().await;
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
            let networking = networking.read().await;
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
            let networking = networking.read().await;
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
                let networking = networking.read().await;
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
                let networking = networking.read().await;
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
    
    /// Send renderer initialization message to a new client
    pub async fn initialize_client_renderer(&self, client_id: usize) -> UiResult<()> {
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
            let networking = networking.read().await;
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
}