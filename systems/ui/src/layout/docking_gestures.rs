//! Gesture support for the docking system

use nalgebra::Vector2;
use uuid::Uuid;
use crate::input::{GestureType, SwipeDirection};
use super::docking::{DockingLayout, DockOrientation};

/// Gesture handler for the docking system
pub struct DockingGestureHandler {
    /// Enable swipe to switch tabs
    pub swipe_tabs_enabled: bool,
    
    /// Enable pinch to maximize/minimize panels
    pub pinch_panels_enabled: bool,
    
    /// Enable double tap to maximize panel
    pub double_tap_maximize_enabled: bool,
    
    /// Swipe velocity threshold for tab switching
    pub swipe_velocity_threshold: f32,
    
    /// Currently maximized panel
    maximized_panel: Option<Uuid>,
    
    /// Previous layout before maximizing
    saved_layout: Option<String>,
}

impl Default for DockingGestureHandler {
    fn default() -> Self {
        Self {
            swipe_tabs_enabled: true,
            pinch_panels_enabled: true,
            double_tap_maximize_enabled: true,
            swipe_velocity_threshold: 200.0,
            maximized_panel: None,
            saved_layout: None,
        }
    }
}

impl DockingGestureHandler {
    /// Create a new gesture handler
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Handle a gesture for the docking system
    pub fn handle_gesture(
        &mut self,
        gesture: &GestureType,
        docking: &mut DockingLayout,
        position: Vector2<f32>,
    ) -> bool {
        match gesture {
            GestureType::Swipe { direction, velocity, .. } => {
                if self.swipe_tabs_enabled && velocity.magnitude() > self.swipe_velocity_threshold {
                    return self.handle_swipe(docking, *direction, position);
                }
            }
            
            GestureType::DoubleTap { position, .. } => {
                if self.double_tap_maximize_enabled {
                    return self.handle_double_tap(docking, *position);
                }
            }
            
            GestureType::Pinch { scale, .. } => {
                if self.pinch_panels_enabled {
                    return self.handle_pinch(docking, *scale, position);
                }
            }
            
            GestureType::LongPress { position, .. } => {
                return self.handle_long_press(docking, *position);
            }
            
            _ => {}
        }
        
        false
    }
    
    /// Handle swipe gesture for tab switching
    fn handle_swipe(
        &mut self,
        _docking: &mut DockingLayout,
        _direction: SwipeDirection,
        _position: Vector2<f32>,
    ) -> bool {
        // TODO: Implement when DockingLayout provides necessary methods
        // Would switch between tabs based on swipe direction
        false
    }
    
    /// Handle double tap for panel maximize/restore
    fn handle_double_tap(&mut self, docking: &mut DockingLayout, position: Vector2<f32>) -> bool {
        if let Some(dock_id) = self.find_dock_at_position_helper(docking, position) {
            if self.maximized_panel == Some(dock_id) {
                // Restore from maximized state
                if let Some(layout_json) = &self.saved_layout {
                    if docking.load_layout(layout_json).is_ok() {
                        self.maximized_panel = None;
                        self.saved_layout = None;
                        return true;
                    }
                }
            } else {
                // Maximize the panel
                if let Ok(layout_json) = docking.save_layout() {
                    self.saved_layout = Some(layout_json);
                    
                    // Maximize by hiding all other panels
                    if self.maximize_panel(docking, dock_id) {
                        self.maximized_panel = Some(dock_id);
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    /// Handle pinch gesture for panel zoom
    fn handle_pinch(&mut self, docking: &mut DockingLayout, scale: f32, position: Vector2<f32>) -> bool {
        if let Some(dock_id) = self.find_dock_at_position_helper(docking, position) {
            if scale > 1.5 {
                // Pinch out - maximize panel
                if self.maximized_panel != Some(dock_id) {
                    if let Ok(layout_json) = docking.save_layout() {
                        self.saved_layout = Some(layout_json);
                        if self.maximize_panel(docking, dock_id) {
                            self.maximized_panel = Some(dock_id);
                            return true;
                        }
                    }
                }
            } else if scale < 0.7 {
                // Pinch in - restore panel
                if self.maximized_panel == Some(dock_id) {
                    if let Some(layout_json) = &self.saved_layout {
                        if docking.load_layout(layout_json).is_ok() {
                            self.maximized_panel = None;
                            self.saved_layout = None;
                            return true;
                        }
                    }
                }
            }
        }
        
        false
    }
    
    /// Handle long press for context menu
    fn handle_long_press(&mut self, docking: &mut DockingLayout, position: Vector2<f32>) -> bool {
        if let Some(_dock_id) = self.find_dock_at_position_helper(docking, position) {
            // In a real implementation, this would show a context menu
            // This is where you'd show options like:
            // - Close panel
            // - Split horizontal/vertical
            // - Move to new window
            // - Pin/unpin
            return true;
        }
        
        false
    }
    
    /// Maximize a panel by making it fill the entire layout
    fn maximize_panel(&self, _docking: &mut DockingLayout, _dock_id: Uuid) -> bool {
        // TODO: Implement when DockingLayout provides necessary methods
        // Would adjust layout to maximize the specified panel
        false
    }
    
    /// Check if a panel is currently maximized
    pub fn is_maximized(&self, dock_id: Uuid) -> bool {
        self.maximized_panel == Some(dock_id)
    }
    
    /// Restore from maximized state
    pub fn restore_layout(&mut self, docking: &mut DockingLayout) -> bool {
        if let Some(layout_json) = &self.saved_layout {
            if docking.load_layout(layout_json).is_ok() {
                self.maximized_panel = None;
                self.saved_layout = None;
                return true;
            }
        }
        false
    }
}

// These extension methods would be implemented on DockingLayout in the actual docking.rs file
// For now, we'll provide helper functions that work with the public API

impl DockingGestureHandler {
    /// Helper to find dock at position (would be implemented in DockingLayout)
    fn find_dock_at_position_helper(&self, _docking: &DockingLayout, _position: Vector2<f32>) -> Option<Uuid> {
        // Placeholder - would traverse dock tree to find dock at position
        None
    }
    
    /// Helper to find parent dock (would be implemented in DockingLayout) 
    fn find_parent_dock_helper(&self, _docking: &DockingLayout, _dock_id: Uuid) -> Option<Uuid> {
        // Placeholder - would traverse dock tree to find parent
        None
    }
}