//! UI renderer trait

use crate::element::{ElementGraph, ElementId};
use crate::error::UiResult;
use crate::theme::ThemeManager;

/// Trait for UI renderers
pub trait UiRenderer: Send + Sync {
    /// Render the given elements
    fn render_elements(
        &mut self,
        graph: &ElementGraph,
        elements: &[ElementId],
        theme_manager: &ThemeManager,
    ) -> UiResult<()>;
    
    /// Begin a new frame
    fn begin_frame(&mut self) -> UiResult<()>;
    
    /// End the current frame
    fn end_frame(&mut self) -> UiResult<()>;
    
    /// Clear the render target
    fn clear(&mut self) -> UiResult<()>;
}