use playground_core_rendering::{RenderCommand, RenderCommandBatch};
use crate::theme::Theme;
use crate::error::UiResult;
use nalgebra::Vector2;

pub struct MobileFeatures {
    floating_toolbar: Option<FloatingToolbar>,
    safe_area_insets: [f32; 4], // top, right, bottom, left
    keyboard_height: f32,
    is_mobile: bool,
}

impl MobileFeatures {
    pub fn new() -> Self {
        Self {
            floating_toolbar: None,
            safe_area_insets: [0.0; 4],
            keyboard_height: 0.0,
            is_mobile: Self::detect_mobile(),
        }
    }
    
    pub async fn initialize(&mut self) -> UiResult<()> {
        if self.is_mobile {
            self.floating_toolbar = Some(FloatingToolbar::new());
            // Would detect safe area insets here
            self.safe_area_insets = [44.0, 0.0, 34.0, 0.0]; // iPhone defaults
        }
        Ok(())
    }
    
    pub fn render(&self, batch: &mut RenderCommandBatch, theme: &Theme) -> UiResult<()> {
        if let Some(ref toolbar) = self.floating_toolbar {
            toolbar.render(batch, theme)?;
        }
        Ok(())
    }
    
    pub fn set_keyboard_height(&mut self, height: f32) {
        self.keyboard_height = height;
        if let Some(ref mut toolbar) = self.floating_toolbar {
            // Adjust toolbar position based on keyboard
            if height > 0.0 {
                toolbar.position.y -= height;
            }
        }
    }
    
    pub fn get_safe_area_insets(&self) -> [f32; 4] {
        self.safe_area_insets
    }
    
    fn detect_mobile() -> bool {
        // Simple detection - would use actual platform detection
        cfg!(target_os = "android") || cfg!(target_os = "ios")
    }
}

struct FloatingToolbar {
    position: Vector2<f32>,
    size: Vector2<f32>,
    visible: bool,
    actions: Vec<ToolbarAction>,
}

struct ToolbarAction {
    id: String,
    icon: String,
    label: String,
    enabled: bool,
}

impl FloatingToolbar {
    fn new() -> Self {
        Self {
            position: Vector2::new(10.0, 10.0),
            size: Vector2::new(300.0, 50.0),
            visible: true,
            actions: vec![
                ToolbarAction {
                    id: "copy".to_string(),
                    icon: "📋".to_string(),
                    label: "Copy".to_string(),
                    enabled: true,
                },
                ToolbarAction {
                    id: "paste".to_string(),
                    icon: "📄".to_string(),
                    label: "Paste".to_string(),
                    enabled: true,
                },
                ToolbarAction {
                    id: "undo".to_string(),
                    icon: "↩️".to_string(),
                    label: "Undo".to_string(),
                    enabled: true,
                },
                ToolbarAction {
                    id: "redo".to_string(),
                    icon: "↪️".to_string(),
                    label: "Redo".to_string(),
                    enabled: true,
                },
            ],
        }
    }
    
    fn render(&self, batch: &mut RenderCommandBatch, theme: &Theme) -> UiResult<()> {
        if !self.visible {
            return Ok(());
        }
        
        // Toolbar background
        batch.push(RenderCommand::DrawQuad {
            position: [self.position.x, self.position.y],
            size: [self.size.x, self.size.y],
            color: [
                theme.colors.surface.x,
                theme.colors.surface.y,
                theme.colors.surface.z,
                0.95, // Slightly transparent
            ],
        });
        
        // Toolbar border
        batch.push(RenderCommand::DrawLine {
            start: [self.position.x, self.position.y],
            end: [self.position.x + self.size.x, self.position.y],
            width: 1.0,
            color: [
                theme.colors.border.x,
                theme.colors.border.y,
                theme.colors.border.z,
                theme.colors.border.w,
            ],
        });
        
        // Render actions
        let action_width = self.size.x / self.actions.len() as f32;
        for (i, action) in self.actions.iter().enumerate() {
            let x = self.position.x + (i as f32 * action_width);
            
            // Action background if disabled
            if !action.enabled {
                batch.push(RenderCommand::DrawQuad {
                    position: [x, self.position.y],
                    size: [action_width, self.size.y],
                    color: [
                        theme.colors.surface_variant.x,
                        theme.colors.surface_variant.y,
                        theme.colors.surface_variant.z,
                        0.5,
                    ],
                });
            }
            
            // Action icon (using text for now)
            batch.push(RenderCommand::DrawText {
                text: action.icon.clone(),
                position: [x + action_width / 2.0, self.position.y + self.size.y / 2.0],
                size: 20.0,
                color: if action.enabled {
                    [
                        theme.colors.text.x,
                        theme.colors.text.y,
                        theme.colors.text.z,
                        theme.colors.text.w,
                    ]
                } else {
                    [
                        theme.colors.text_secondary.x,
                        theme.colors.text_secondary.y,
                        theme.colors.text_secondary.z,
                        theme.colors.text_secondary.w,
                    ]
                },
            });
        }
        
        Ok(())
    }
}