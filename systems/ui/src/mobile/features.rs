use playground_core_rendering::RenderCommandBatch;
use crate::theme::Theme;
use crate::error::UiResult;
use super::floating_toolbar::FloatingToolbar;

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
                toolbar.adjust_for_keyboard(height);
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