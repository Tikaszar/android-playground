use crate::system::UiSystem;
use crate::error::{UiError, UiResult};
use playground_core_rendering::Viewport;
use playground_core_ui::ElementId;

impl UiSystem {
    pub async fn initialize_ui(&mut self) -> UiResult<()> {
        self.log("Info", format!("initialize_ui() called, initialized={}", self.initialized)).await;
        
        if self.initialized {
            return Err(UiError::AlreadyInitialized);
        }
        
        self.log("Info", "[UiSystem] Loading default themes...".to_string()).await;
        let mut theme_mgr = self.theme_manager.write().await;
        theme_mgr.load_default_themes()?;
        drop(theme_mgr);
        self.log("Info", "[UiSystem] Themes loaded".to_string()).await;
        
        self.log("Info", "[UiSystem] Creating root element...".to_string()).await;
        let root_id = self.storage.create_element(playground_core_ui::ElementType::Panel).await;
        self.storage.set_root(Some(root_id)).await;
        self.root_element = Some(root_id);
        self.log("Info", format!("[UiSystem] Root element created: {:?}", root_id)).await;
        
        self.log("Info", "[UiSystem] Initializing mobile features...".to_string()).await;
        let mut mobile = self.mobile_features.write().await;
        mobile.initialize().await?;
        drop(mobile);
        self.log("Info", "[UiSystem] Mobile features initialized".to_string()).await;
        
        self.initialized = true;
        self.log("Info", format!("[UiSystem] Initialization complete, initialized={}", self.initialized)).await;
        Ok(())
    }
    
    pub async fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.screen_size = [viewport.width as f32, viewport.height as f32];
        
        // Mark root as dirty for re-layout
        if let Some(root) = self.root_element {
            self.storage.update_element(root, |e| e.dirty = true).await;
        }
    }
    
    pub async fn shutdown(&mut self) -> UiResult<()> {
        self.log("Info", "[UiSystem] Shutting down...".to_string()).await;
        
        // Clear all elements
        self.storage.clear().await;
        self.root_element = None;
        
        // Shutdown mobile features
        let mut mobile = self.mobile_features.write().await;
        mobile.shutdown().await?;
        
        self.initialized = false;
        self.log("Info", "[UiSystem] Shutdown complete".to_string()).await;
        Ok(())
    }
}