use playground_core_rendering::Viewport;
use playground_core_types::{Shared, shared};
use playground_core_ecs::{System as EcsSystem, ExecutionStage, EcsResult};
use playground_core_ui::ElementId;
use crate::element::ElementGraph;
use crate::internal_storage::InternalElementStorage;
use crate::layout::LayoutEngine;
use crate::input::InputManager;
use crate::theme::{ThemeManager, ThemeId};
use crate::terminal::TerminalManager;
use crate::mobile::MobileFeatures;
use std::collections::HashMap;
use uuid::Uuid;

pub struct UiSystem {
    // Internal element storage (no ECS dependency)
    pub(super) storage: InternalElementStorage,
    
    // Element management
    pub(super) element_graph: Shared<ElementGraph>,
    pub(super) root_element: Option<ElementId>,
    
    // Layout
    pub(super) layout_engine: Shared<LayoutEngine>,
    
    // Input handling
    pub(super) input_manager: Shared<InputManager>,
    
    // Theme management
    pub(super) theme_manager: Shared<ThemeManager>,
    pub(super) current_theme: ThemeId,
    
    // Terminal support
    pub(super) terminal_manager: Shared<TerminalManager>,
    pub(super) terminal_connections: Shared<HashMap<Uuid, ElementId>>,
    
    // Mobile features
    pub(super) mobile_features: Shared<MobileFeatures>,
    
    // Rendering
    pub(super) viewport: Viewport,
    pub(super) frame_id: u64,
    
    // Channel for communication
    pub(super) channel_id: u16,
    
    // State
    pub(super) initialized: bool,
    pub(super) screen_size: [f32; 2],
}

impl UiSystem {
    pub fn new() -> Self {
        Self {
            storage: InternalElementStorage::new(),
            element_graph: shared(ElementGraph::new()),
            root_element: None,
            layout_engine: shared(LayoutEngine::new()),
            input_manager: shared(InputManager::new()),
            theme_manager: shared(ThemeManager::new()),
            current_theme: ThemeId::Dark,
            terminal_manager: shared(TerminalManager::new()),
            terminal_connections: shared(HashMap::new()),
            mobile_features: shared(MobileFeatures::new()),
            viewport: Viewport { x: 0, y: 0, width: 1920, height: 1080 },
            frame_id: 0,
            channel_id: 10,
            initialized: false,
            screen_size: [1920.0, 1080.0],
        }
    }
    
    pub async fn get_root_element(&self) -> Option<ElementId> {
        self.storage.get_root().await
    }
    
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    pub fn set_channel_id(&mut self, channel_id: u16) {
        self.channel_id = channel_id;
    }
    
    pub(super) async fn log(&self, _level: &str, _message: String) {
        // TODO: Systems need proper logging interface that doesn't violate architecture
        // For now, logging is disabled during refactoring
    }
}

// Implement the ECS System trait for UiSystem
#[async_trait::async_trait]
impl EcsSystem for UiSystem {
    fn name(&self) -> &str {
        "UiSystem"
    }
    
    fn stage(&self) -> ExecutionStage {
        ExecutionStage::Layout
    }
    
    async fn initialize(&mut self) -> EcsResult<()> {
        self.initialized = true;
        Ok(())
    }
    
    async fn update(&mut self, _delta_time: f32) -> EcsResult<()> {
        // In a proper implementation, this would:
        // 1. Query the World for entities with UI components
        // 2. Update internal element storage from ECS data
        // 3. Perform layout calculations
        // 4. Generate RenderCommandBatch for the render stage
        
        self.frame_id += 1;
        
        // TODO: Implement proper World querying when scheduler passes World reference
        
        Ok(())
    }
    
    async fn cleanup(&mut self) -> EcsResult<()> {
        self.storage.clear().await;
        self.initialized = false;
        Ok(())
    }
}