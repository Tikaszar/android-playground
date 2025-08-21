use playground_core_rendering::{RenderCommand, RenderCommandBatch};
use crate::components::*;
use crate::theme::Theme;
use crate::error::UiResult;

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
        "text" => render_text(element, layout, style, theme, batch)?,
        "button" => render_button(element, layout, style, theme, batch)?,
        "terminal" => render_terminal(element, layout, style, theme, batch)?,
        "input" => render_input(element, layout, style, theme, batch)?,
        "container" => {}, // Already rendered background
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

fn render_text(
    element: &UiElementComponent,
    layout: &UiLayoutComponent,
    style: &UiStyleComponent,
    theme: &Theme,
    batch: &mut RenderCommandBatch,
) -> UiResult<()> {
    if let Some(text) = &element.text_content {
        let text_color = style.text_color.unwrap_or(theme.colors.text);
        
        // Calculate text position based on alignment
        let text_x = match style.text_align {
            TextAlign::Left => layout.bounds.x + layout.padding[3],
            TextAlign::Center => layout.bounds.x + layout.bounds.width / 2.0,
            TextAlign::Right => layout.bounds.x + layout.bounds.width - layout.padding[1],
            TextAlign::Justify => layout.bounds.x + layout.padding[3],
        };
        
        let text_y = layout.bounds.y + layout.padding[0] + style.font_size;
        
        batch.push(RenderCommand::DrawText {
            text: text.clone(),
            position: [text_x, text_y],
            size: style.font_size,
            color: [text_color.x, text_color.y, text_color.z, text_color.w * style.opacity],
        });
    }
    
    Ok(())
}

fn render_button(
    element: &UiElementComponent,
    layout: &UiLayoutComponent,
    style: &UiStyleComponent,
    theme: &Theme,
    batch: &mut RenderCommandBatch,
) -> UiResult<()> {
    // Button gets special hover/pressed styling
    let bg_color = if element.disabled {
        theme.colors.surface_variant
    } else if element.focused {
        theme.colors.primary
    } else if element.hovered {
        theme.colors.hover
    } else {
        style.background_color.unwrap_or(theme.colors.surface)
    };
    
    // Draw button background
    batch.push(RenderCommand::DrawQuad {
        position: [layout.bounds.x, layout.bounds.y],
        size: [layout.bounds.width, layout.bounds.height],
        color: [bg_color.x, bg_color.y, bg_color.z, bg_color.w * style.opacity],
    });
    
    // Draw button text if present
    if let Some(text) = &element.text_content {
        let text_color = if element.disabled {
            theme.colors.text_secondary
        } else {
            style.text_color.unwrap_or(theme.colors.text)
        };
        
        // Center text in button
        let text_x = layout.bounds.x + layout.bounds.width / 2.0;
        let text_y = layout.bounds.y + layout.bounds.height / 2.0;
        
        batch.push(RenderCommand::DrawText {
            text: text.clone(),
            position: [text_x, text_y],
            size: style.font_size,
            color: [text_color.x, text_color.y, text_color.z, text_color.w * style.opacity],
        });
    }
    
    Ok(())
}

fn render_terminal(
    element: &UiElementComponent,
    layout: &UiLayoutComponent,
    style: &UiStyleComponent,
    theme: &Theme,
    batch: &mut RenderCommandBatch,
) -> UiResult<()> {
    // Terminal gets special background
    let bg_color = theme.colors.editor_background;
    
    batch.push(RenderCommand::DrawQuad {
        position: [layout.bounds.x, layout.bounds.y],
        size: [layout.bounds.width, layout.bounds.height],
        color: [bg_color.x, bg_color.y, bg_color.z, bg_color.w],
    });
    
    // Terminal content would be rendered here
    // This would include terminal lines, cursor, etc.
    
    Ok(())
}

fn render_input(
    element: &UiElementComponent,
    layout: &UiLayoutComponent,
    style: &UiStyleComponent,
    theme: &Theme,
    batch: &mut RenderCommandBatch,
) -> UiResult<()> {
    // Input field background
    let bg_color = if element.focused {
        theme.colors.surface_variant
    } else {
        style.background_color.unwrap_or(theme.colors.surface)
    };
    
    batch.push(RenderCommand::DrawQuad {
        position: [layout.bounds.x, layout.bounds.y],
        size: [layout.bounds.width, layout.bounds.height],
        color: [bg_color.x, bg_color.y, bg_color.z, bg_color.w * style.opacity],
    });
    
    // Draw input text
    if let Some(text) = &element.text_content {
        let text_color = style.text_color.unwrap_or(theme.colors.text);
        
        batch.push(RenderCommand::DrawText {
            text: text.clone(),
            position: [layout.bounds.x + layout.padding[3], layout.bounds.y + layout.padding[0] + style.font_size],
            size: style.font_size,
            color: [text_color.x, text_color.y, text_color.z, text_color.w * style.opacity],
        });
    }
    
    // Draw cursor if focused
    if element.focused {
        let cursor_x = layout.bounds.x + layout.padding[3] + 
            (element.text_content.as_ref().map(|t| t.len()).unwrap_or(0) as f32 * style.font_size * 0.6);
        
        batch.push(RenderCommand::DrawLine {
            start: [cursor_x, layout.bounds.y + layout.padding[0]],
            end: [cursor_x, layout.bounds.y + layout.bounds.height - layout.padding[2]],
            width: 2.0,
            color: [theme.colors.cursor.x, theme.colors.cursor.y, theme.colors.cursor.z, theme.colors.cursor.w],
        });
    }
    
    Ok(())
}