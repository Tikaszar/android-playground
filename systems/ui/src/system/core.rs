use playground_core_rendering::Viewport;
use playground_core_ecs::{World, EntityId, ComponentRegistry};
use playground_core_types::{Handle, handle, Shared, shared};
use playground_core_server::ChannelManager;
use playground_systems_networking::NetworkingSystem;
use crate::element::{ElementGraph, ElementId};
use crate::layout::LayoutEngine;
use crate::input::InputManager;
use crate::theme::{ThemeManager, ThemeId};
use crate::terminal::TerminalManager;
use crate::mobile::MobileFeatures;
use std::collections::HashMap;
use uuid::Uuid;

pub struct UiSystem {
    // Core ECS
    pub(super) world: Handle<World>,
    pub(super) registry: Handle<ComponentRegistry>,
    
    // Element management
    pub(super) element_graph: Shared<ElementGraph>,
    pub(super) root_entity: Option<EntityId>,
    
    // Layout
    pub(super) layout_engine: Shared<LayoutEngine>,
    
    // Input handling
    pub(super) input_manager: Shared<InputManager>,
    
    // Theme management
    pub(super) theme_manager: Shared<ThemeManager>,
    pub(super) current_theme: ThemeId,
    
    // Terminal support
    pub(super) terminal_manager: Shared<TerminalManager>,
    pub(super) terminal_connections: Shared<HashMap<Uuid, EntityId>>,
    
    // Mobile features
    pub(super) mobile_features: Shared<MobileFeatures>,
    
    // Rendering
    pub(super) viewport: Viewport,
    pub(super) frame_id: u64,
    pub(super) dirty_elements: Shared<Vec<EntityId>>,
    
    // Networking
    pub(super) channel_manager: Option<Shared<ChannelManager>>,
    pub(super) networking_system: Option<Shared<NetworkingSystem>>,
    pub(super) channel_id: u16,
    
    // State
    pub(super) initialized: bool,
    pub(super) screen_size: [f32; 2],
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
    
    pub fn get_root_element(&self) -> Option<ElementId> {
        self.root_entity
    }
    
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    pub fn set_channel_manager(&mut self, manager: Shared<ChannelManager>) {
        self.channel_manager = Some(manager);
    }
    
    pub fn set_networking_system(&mut self, networking: Shared<NetworkingSystem>) {
        self.networking_system = Some(networking);
    }
    
    pub fn set_networking_system_shared(&mut self, networking: Shared<NetworkingSystem>) {
        self.set_networking_system(networking);
    }
    
    pub fn set_channel_id(&mut self, channel_id: u16) {
        self.channel_id = channel_id;
    }
    
    pub(super) async fn log(&self, level: &str, message: String) {
        if let Some(ref networking) = self.networking_system {
            let networking = networking.read().await;
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
                dashboard.log_component("systems/ui", log_level, message, None).await;
            }
        }
    }
}