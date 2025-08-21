use playground_core_rendering::{RenderCommand, RenderCommandBatch};
use crate::components::{UiElementComponent, UiLayoutComponent, UiStyleComponent, TextAlign};
use crate::theme::Theme;
use crate::error::UiResult;

pub fn render_text(
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

pub fn render_button(
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

pub fn render_terminal(
    _element: &UiElementComponent,
    layout: &UiLayoutComponent,
    _style: &UiStyleComponent,
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

pub fn render_input(
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