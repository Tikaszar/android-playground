use playground_core_rendering::{RenderCommand, RenderCommandBatch};
use crate::components::{UiElementComponent, UiLayoutComponent, UiStyleComponent};
use crate::theme::Theme;
use crate::error::UiResult;
use super::element_renderer;

pub fn ui_to_render_commands(
    element: &UiElementComponent,
    layout: &UiLayoutComponent,
    style: &UiStyleComponent,
    theme: &Theme,
    batch: &mut RenderCommandBatch,
) -> UiResult<()> {
    // Skip invisible elements
    if !style.visible || !element.visible {
        return Ok(());
    }
    
    // Apply opacity if needed
    let has_opacity = style.opacity < 1.0;
    if has_opacity {
        batch.push(RenderCommand::PushState);
    }
    
    // Clip rect for bounds
    if layout.bounds.width > 0.0 && layout.bounds.height > 0.0 {
        batch.push(RenderCommand::SetClipRect {
            position: [layout.bounds.x, layout.bounds.y],
            size: [layout.bounds.width, layout.bounds.height],
        });
    }
    
    // Background
    if let Some(bg_color) = style.background_color {
        batch.push(RenderCommand::DrawQuad {
            position: [layout.bounds.x, layout.bounds.y],
            size: [layout.bounds.width, layout.bounds.height],
            color: [bg_color.x, bg_color.y, bg_color.z, bg_color.w * style.opacity],
        });
    }
    
    // Border
    if style.border_width > 0.0 {
        let border_color = style.border_color.unwrap_or(theme.colors.border);
        
        // Top border
        batch.push(RenderCommand::DrawLine {
            start: [layout.bounds.x, layout.bounds.y],
            end: [layout.bounds.x + layout.bounds.width, layout.bounds.y],
            width: style.border_width,
            color: [border_color.x, border_color.y, border_color.z, border_color.w * style.opacity],
        });
        
        // Right border
        batch.push(RenderCommand::DrawLine {
            start: [layout.bounds.x + layout.bounds.width, layout.bounds.y],
            end: [layout.bounds.x + layout.bounds.width, layout.bounds.y + layout.bounds.height],
            width: style.border_width,
            color: [border_color.x, border_color.y, border_color.z, border_color.w * style.opacity],
        });
        
        // Bottom border
        batch.push(RenderCommand::DrawLine {
            start: [layout.bounds.x + layout.bounds.width, layout.bounds.y + layout.bounds.height],
            end: [layout.bounds.x, layout.bounds.y + layout.bounds.height],
            width: style.border_width,
            color: [border_color.x, border_color.y, border_color.z, border_color.w * style.opacity],
        });
        
        // Left border
        batch.push(RenderCommand::DrawLine {
            start: [layout.bounds.x, layout.bounds.y + layout.bounds.height],
            end: [layout.bounds.x, layout.bounds.y],
            width: style.border_width,
            color: [border_color.x, border_color.y, border_color.z, border_color.w * style.opacity],
        });
    }
    
    // Element-specific rendering
    match element.element_type.as_str() {
        "text" => element_renderer::render_text(element, layout, style, theme, batch)?,
        "button" => element_renderer::render_button(element, layout, style, theme, batch)?,
        "terminal" => element_renderer::render_terminal(element, layout, style, theme, batch)?,
        "input" => element_renderer::render_input(element, layout, style, theme, batch)?,
        "panel" | "container" => {}, // Already rendered background
        "scrollview" => {
            // Scrollview is like a panel but with overflow handling
            // For now, just render as a panel
        },
        _ => {} // Unknown element type
    }
    
    // Clear clip rect
    batch.push(RenderCommand::ClearClipRect);
    
    // Restore state if we pushed
    if has_opacity {
        batch.push(RenderCommand::PopState);
    }
    
    Ok(())
}