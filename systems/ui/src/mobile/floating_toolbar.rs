//! Floating toolbar for mobile interactions

use nalgebra::{Vector2, Vector4};
use uuid::Uuid;
use crate::element::{Element, ElementBounds};
use std::any::Any;
use crate::layout::{LayoutConstraints, LayoutResult};
use crate::rendering::RenderData;
use crate::input::{InputEvent, InputResult, EventHandled};
use crate::theme::Theme;

/// Toolbar action
#[derive(Debug, Clone)]
pub struct ToolbarAction {
    pub id: String,
    pub icon: String,
    pub label: String,
    pub enabled: bool,
    pub callback: fn(),
}

/// Floating toolbar position
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolbarPosition {
    Top,
    Bottom,
    Left,
    Right,
    Center,
    Custom(Vector2<f32>),
}

/// Floating toolbar visibility state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolbarVisibility {
    Hidden,
    Showing(f32), // Animation progress 0-1
    Visible,
    Hiding(f32), // Animation progress 0-1
}

/// Floating toolbar for mobile
pub struct FloatingToolbar {
    id: Uuid,
    actions: Vec<ToolbarAction>,
    position: Vector2<f32>,
    size: Vector2<f32>,
    toolbar_position: ToolbarPosition,
    visibility: ToolbarVisibility,
    animation_speed: f32,
    auto_hide_delay: f32,
    auto_hide_timer: f32,
    selected_action: Option<usize>,
    theme: Theme,
    visible: bool,
    bounds: ElementBounds,
    dirty: bool,
    children: Vec<crate::element::ElementId>,
}

impl FloatingToolbar {
    /// Create a new floating toolbar
    pub fn new() -> Self {
        let position = Vector2::new(0.0, 0.0);
        let size = Vector2::new(300.0, 60.0);
        Self {
            id: Uuid::new_v4(),
            actions: Vec::new(),
            position,
            size,
            toolbar_position: ToolbarPosition::Bottom,
            visibility: ToolbarVisibility::Hidden,
            animation_speed: 3.0, // Animation takes ~0.3 seconds
            auto_hide_delay: 3.0, // Hide after 3 seconds
            auto_hide_timer: 0.0,
            selected_action: None,
            theme: Theme::dark(),
            visible: true,
            bounds: ElementBounds { position, size },
            dirty: false,
            children: Vec::new(),
        }
    }
    
    /// Add an action to the toolbar
    pub fn add_action(&mut self, action: ToolbarAction) {
        self.actions.push(action);
        self.update_size();
    }
    
    /// Remove an action by ID
    pub fn remove_action(&mut self, id: &str) {
        self.actions.retain(|a| a.id != id);
        self.update_size();
    }
    
    /// Clear all actions
    pub fn clear_actions(&mut self) {
        self.actions.clear();
        self.selected_action = None;
        self.update_size();
    }
    
    /// Set toolbar position
    pub fn set_toolbar_position(&mut self, position: ToolbarPosition) {
        self.toolbar_position = position;
    }
    
    /// Show the toolbar
    pub fn show(&mut self) {
        if self.visibility == ToolbarVisibility::Hidden {
            self.visibility = ToolbarVisibility::Showing(0.0);
            self.auto_hide_timer = self.auto_hide_delay;
        }
    }
    
    /// Hide the toolbar
    pub fn hide(&mut self) {
        if self.visibility == ToolbarVisibility::Visible {
            self.visibility = ToolbarVisibility::Hiding(1.0);
        }
    }
    
    /// Toggle toolbar visibility
    pub fn toggle(&mut self) {
        match self.visibility {
            ToolbarVisibility::Hidden => self.show(),
            ToolbarVisibility::Visible => self.hide(),
            _ => {} // Don't interrupt animations
        }
    }
    
    /// Set auto-hide delay (0 to disable)
    pub fn set_auto_hide_delay(&mut self, delay: f32) {
        self.auto_hide_delay = delay;
        if delay > 0.0 {
            self.auto_hide_timer = delay;
        }
    }
    
    /// Update toolbar size based on actions
    fn update_size(&mut self) {
        let action_width = 60.0;
        let padding = 10.0;
        let width = (self.actions.len() as f32 * action_width) + (padding * 2.0);
        self.size.x = width.min(400.0); // Max width
    }
    
    /// Calculate toolbar position based on screen and position setting
    fn calculate_position(&self, screen_size: Vector2<f32>) -> Vector2<f32> {
        let margin = 20.0;
        
        match self.toolbar_position {
            ToolbarPosition::Top => {
                Vector2::new(
                    (screen_size.x - self.size.x) / 2.0,
                    margin,
                )
            }
            ToolbarPosition::Bottom => {
                Vector2::new(
                    (screen_size.x - self.size.x) / 2.0,
                    screen_size.y - self.size.y - margin,
                )
            }
            ToolbarPosition::Left => {
                Vector2::new(
                    margin,
                    (screen_size.y - self.size.y) / 2.0,
                )
            }
            ToolbarPosition::Right => {
                Vector2::new(
                    screen_size.x - self.size.x - margin,
                    (screen_size.y - self.size.y) / 2.0,
                )
            }
            ToolbarPosition::Center => {
                Vector2::new(
                    (screen_size.x - self.size.x) / 2.0,
                    (screen_size.y - self.size.y) / 2.0,
                )
            }
            ToolbarPosition::Custom(pos) => pos,
        }
    }
    
    /// Get animation progress for rendering
    fn get_animation_progress(&self) -> f32 {
        match self.visibility {
            ToolbarVisibility::Hidden => 0.0,
            ToolbarVisibility::Showing(progress) => progress,
            ToolbarVisibility::Visible => 1.0,
            ToolbarVisibility::Hiding(progress) => progress,
        }
    }
    
    /// Check if point is inside an action button
    fn get_action_at_position(&self, position: Vector2<f32>) -> Option<usize> {
        let relative = position - self.position;
        
        if relative.x < 0.0 || relative.y < 0.0 ||
           relative.x > self.size.x || relative.y > self.size.y {
            return None;
        }
        
        let action_width = 60.0;
        let padding = 10.0;
        let button_x = relative.x - padding;
        
        if button_x >= 0.0 {
            let index = (button_x / action_width) as usize;
            if index < self.actions.len() {
                return Some(index);
            }
        }
        
        None
    }
}

impl Element for FloatingToolbar {
    fn id(&self) -> Uuid {
        self.id
    }
    
    fn type_name(&self) -> &str {
        "FloatingToolbar"
    }
    
    fn layout(&mut self, constraints: &LayoutConstraints) -> crate::UiResult<LayoutResult> {
        let screen_size = constraints.max_size;
        
        self.position = self.calculate_position(screen_size);
        self.bounds = ElementBounds { position: self.position, size: self.size };
        
        Ok(LayoutResult {
            size: self.size,
            position: self.position,
        })
    }
    
    fn render(&self, theme: &Theme) -> crate::UiResult<RenderData> {
        let mut vertices = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();
        
        let progress = self.get_animation_progress();
        if progress <= 0.0 {
            return Ok(RenderData {
                vertices,
                colors,
                indices,
                uvs: Vec::new(),
                texture_id: None,
                z_order: 1000.0, // High z-order to float above everything
                scissor_rect: None,
            });
        }
        
        // Apply animation
        let animated_pos = Vector2::new(
            self.position.x,
            match self.toolbar_position {
                ToolbarPosition::Bottom => {
                    self.position.y + (self.size.y * (1.0 - progress))
                }
                ToolbarPosition::Top => {
                    self.position.y - (self.size.y * (1.0 - progress))
                }
                _ => self.position.y,
            },
        );
        
        // Toolbar background with shadow
        let shadow_offset = 2.0;
        let shadow_color = Vector4::new(0.0, 0.0, 0.0, 0.3 * progress);
        
        // Shadow
        let shadow_base = vertices.len() as u32;
        vertices.extend_from_slice(&[
            animated_pos + Vector2::new(shadow_offset, shadow_offset),
            animated_pos + Vector2::new(self.size.x + shadow_offset, shadow_offset),
            animated_pos + Vector2::new(self.size.x + shadow_offset, self.size.y + shadow_offset),
            animated_pos + Vector2::new(shadow_offset, self.size.y + shadow_offset),
        ]);
        colors.extend_from_slice(&[shadow_color; 4]);
        indices.extend_from_slice(&[
            shadow_base, shadow_base + 1, shadow_base + 2,
            shadow_base, shadow_base + 2, shadow_base + 3,
        ]);
        
        // Toolbar background
        let bg_base = vertices.len() as u32;
        let bg_color = self.theme.colors.surface;
        let mut bg_with_alpha = bg_color;
        bg_with_alpha.w *= progress;
        
        vertices.extend_from_slice(&[
            animated_pos,
            animated_pos + Vector2::new(self.size.x, 0.0),
            animated_pos + Vector2::new(self.size.x, self.size.y),
            animated_pos + Vector2::new(0.0, self.size.y),
        ]);
        colors.extend_from_slice(&[bg_with_alpha; 4]);
        indices.extend_from_slice(&[
            bg_base, bg_base + 1, bg_base + 2,
            bg_base, bg_base + 2, bg_base + 3,
        ]);
        
        // Render action buttons
        let action_width = 60.0;
        let padding = 10.0;
        let button_height = 40.0;
        let button_y = (self.size.y - button_height) / 2.0;
        
        for (i, action) in self.actions.iter().enumerate() {
            let button_x = padding + (i as f32 * action_width);
            let button_pos = animated_pos + Vector2::new(button_x, button_y);
            
            let is_selected = self.selected_action == Some(i);
            let button_color = if !action.enabled {
                self.theme.colors.text_secondary
            } else if is_selected {
                self.theme.colors.primary
            } else {
                self.theme.colors.surface_variant
            };
            
            let mut color_with_alpha = button_color;
            color_with_alpha.w *= progress;
            
            // Button background
            let button_base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                button_pos,
                button_pos + Vector2::new(action_width - 5.0, 0.0),
                button_pos + Vector2::new(action_width - 5.0, button_height),
                button_pos + Vector2::new(0.0, button_height),
            ]);
            colors.extend_from_slice(&[color_with_alpha; 4]);
            indices.extend_from_slice(&[
                button_base, button_base + 1, button_base + 2,
                button_base, button_base + 2, button_base + 3,
            ]);
            
            // Icon placeholder (would be actual icon rendering in production)
            let icon_size = 24.0;
            let icon_pos = button_pos + Vector2::new(
                (action_width - 5.0 - icon_size) / 2.0,
                (button_height - icon_size) / 2.0,
            );
            
            let icon_base = vertices.len() as u32;
            let mut icon_color = self.theme.colors.text;
            icon_color.w *= progress;
            
            vertices.extend_from_slice(&[
                icon_pos,
                icon_pos + Vector2::new(icon_size, 0.0),
                icon_pos + Vector2::new(icon_size, icon_size),
                icon_pos + Vector2::new(0.0, icon_size),
            ]);
            colors.extend_from_slice(&[icon_color; 4]);
            indices.extend_from_slice(&[
                icon_base, icon_base + 1, icon_base + 2,
                icon_base, icon_base + 2, icon_base + 3,
            ]);
        }
        
        Ok(RenderData {
            vertices,
            colors,
            indices,
            uvs: Vec::new(),
            texture_id: None,
            z_order: 1000.0,
            scissor_rect: None,
        })
    }
    
    fn handle_input(&mut self, event: &InputEvent) -> InputResult {
        match event {
            InputEvent::PointerDown { position, .. } => {
                if let Some(index) = self.get_action_at_position(*position) {
                    if self.actions[index].enabled {
                        self.selected_action = Some(index);
                        return InputResult {
                            handled: EventHandled::Yes,
                            request_focus: false,
                        };
                    }
                }
            }
            
            InputEvent::PointerUp { position, .. } => {
                if let Some(selected) = self.selected_action {
                    if let Some(index) = self.get_action_at_position(*position) {
                        if index == selected && self.actions[index].enabled {
                            // Execute action
                            (self.actions[index].callback)();
                            self.selected_action = None;
                            
                            // Reset auto-hide timer
                            if self.auto_hide_delay > 0.0 {
                                self.auto_hide_timer = self.auto_hide_delay;
                            }
                            
                            return InputResult {
                                handled: EventHandled::Yes,
                                request_focus: false,
                            };
                        }
                    }
                    self.selected_action = None;
                }
            }
            
            _ => {}
        }
        
        InputResult::default()
    }
    
    fn children(&self) -> &[crate::element::ElementId] {
        &self.children
    }
    
    fn children_mut(&mut self) -> &mut Vec<crate::element::ElementId> {
        &mut self.children
    }
    
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    fn mark_clean(&mut self) {
        self.dirty = false;
    }
    
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    fn bounds(&self) -> ElementBounds {
        self.bounds
    }
    
    fn set_bounds(&mut self, bounds: ElementBounds) {
        self.bounds = bounds;
        self.position = bounds.position;
        self.size = bounds.size;
    }
    
    fn is_visible(&self) -> bool {
        self.visible && self.visibility != ToolbarVisibility::Hidden
    }
    
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn update(&mut self, delta_time: f32) {
        // Update visibility animation
        match self.visibility {
            ToolbarVisibility::Showing(progress) => {
                let new_progress = (progress + delta_time * self.animation_speed).min(1.0);
                if new_progress >= 1.0 {
                    self.visibility = ToolbarVisibility::Visible;
                } else {
                    self.visibility = ToolbarVisibility::Showing(new_progress);
                }
            }
            
            ToolbarVisibility::Hiding(progress) => {
                let new_progress = (progress - delta_time * self.animation_speed).max(0.0);
                if new_progress <= 0.0 {
                    self.visibility = ToolbarVisibility::Hidden;
                } else {
                    self.visibility = ToolbarVisibility::Hiding(new_progress);
                }
            }
            
            ToolbarVisibility::Visible => {
                // Handle auto-hide
                if self.auto_hide_delay > 0.0 {
                    self.auto_hide_timer -= delta_time;
                    if self.auto_hide_timer <= 0.0 {
                        self.hide();
                    }
                }
            }
            
            _ => {}
        }
    }
}