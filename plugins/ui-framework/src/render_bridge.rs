use playground_systems_logic::{UiInterface, LogicResult, LogicError};
use playground_core_types::{Shared, shared};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Bridges UI Framework state changes to trigger renders
pub struct RenderBridge {
    ui_interface: UiInterface,
    last_render: Instant,
    min_frame_time: Duration,
    pending_updates: Shared<Vec<UiUpdate>>,
    dirty_elements: Shared<Vec<playground_systems_ui::ElementId>>,
}

#[derive(Debug, Clone)]
pub enum UiUpdate {
    ElementCreated(playground_systems_ui::ElementId),
    ElementModified(playground_systems_ui::ElementId),
    ElementDeleted(playground_systems_ui::ElementId),
    LayoutChanged,
    ThemeChanged,
}

impl RenderBridge {
    pub fn new(ui_interface: UiInterface) -> Self {
        Self {
            ui_interface,
            last_render: Instant::now(),
            min_frame_time: Duration::from_millis(16), // 60fps
            pending_updates: shared(Vec::new()),
            dirty_elements: shared(Vec::new()),
        }
    }
    
    /// Queue an update for the next render
    pub async fn queue_update(&mut self, update: UiUpdate) {
        let mut updates = self.pending_updates.write().await;
        updates.push(update.clone());
        
        // Mark affected elements as dirty
        match update {
            UiUpdate::ElementCreated(id) | 
            UiUpdate::ElementModified(id) => {
                let mut dirty = self.dirty_elements.write().await;
                if !dirty.contains(&id) {
                    dirty.push(id);
                }
            }
            UiUpdate::ElementDeleted(_) => {
                // Deletion handled differently
            }
            UiUpdate::LayoutChanged => {
                // Force full layout recalculation
                self.ui_interface.force_layout().await.ok();
            }
            UiUpdate::ThemeChanged => {
                // Theme change affects all elements
                if let Some(root) = self.ui_interface.get_root().await {
                    self.ui_interface.mark_dirty(root).await.ok();
                }
            }
        }
    }
    
    /// Process pending updates and trigger render if needed
    pub async fn process_updates(&mut self) -> LogicResult<bool> {
        // Check if enough time has passed for next frame
        if self.last_render.elapsed() < self.min_frame_time {
            return Ok(false);
        }
        
        let updates = self.pending_updates.read().await;
        if updates.is_empty() {
            return Ok(false);
        }
        drop(updates);
        
        // Process all pending updates
        let mut updates = self.pending_updates.write().await;
        let dirty = self.dirty_elements.write().await;
        
        debug!("Processing {} UI updates, {} dirty elements", 
               updates.len(), dirty.len());
        
        // Clear pending updates
        updates.clear();
        drop(updates);
        drop(dirty);
        
        // Trigger render through UI interface
        // The actual render() call will be made by SystemsManager
        self.mark_needs_render().await?;
        
        self.last_render = Instant::now();
        Ok(true)
    }
    
    /// Mark that a render is needed
    async fn mark_needs_render(&mut self) -> LogicResult<()> {
        // Mark dirty elements in the UI system
        let dirty = self.dirty_elements.read().await;
        for &element in dirty.iter() {
            self.ui_interface.mark_dirty(element).await?;
        }
        drop(dirty);
        
        // Clear dirty list after marking
        self.dirty_elements.write().await.clear();
        
        Ok(())
    }
    
    /// Set the target frame rate
    pub fn set_target_fps(&mut self, fps: u32) {
        self.min_frame_time = Duration::from_millis(1000 / fps as u64);
    }
    
    /// Check if render is needed based on timing
    pub fn should_render(&self) -> bool {
        self.last_render.elapsed() >= self.min_frame_time
    }
    
    /// Force an immediate render on next update
    pub fn force_render(&mut self) {
        self.last_render = Instant::now() - self.min_frame_time;
    }
    
    /// Get statistics about the render bridge
    pub async fn get_stats(&self) -> RenderBridgeStats {
        let updates = self.pending_updates.read().await;
        let dirty = self.dirty_elements.read().await;
        
        RenderBridgeStats {
            pending_updates: updates.len(),
            dirty_elements: dirty.len(),
            time_since_render: self.last_render.elapsed(),
            target_frame_time: self.min_frame_time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderBridgeStats {
    pub pending_updates: usize,
    pub dirty_elements: usize,
    pub time_since_render: Duration,
    pub target_frame_time: Duration,
}

impl RenderBridgeStats {
    pub fn needs_render(&self) -> bool {
        self.time_since_render >= self.target_frame_time && 
        (self.pending_updates > 0 || self.dirty_elements > 0)
    }
}