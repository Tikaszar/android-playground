//! Text element

use crate::element::{Element, ElementBase, ElementBounds, ElementId};
use crate::input::{InputEvent, InputResult, EventHandled};
use crate::layout::{LayoutConstraints, LayoutResult};
use crate::rendering::RenderData;
use crate::theme::Theme;
use crate::error::UiResult;
use std::any::Any;
use uuid::Uuid;

/// Text display element
pub struct Text {
    base: ElementBase,
    text: String,
    size: f32,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            base: ElementBase::new(),
            text: text.into(),
            size: 14.0,
        }
    }
}

impl Element for Text {
    fn id(&self) -> Uuid {
        self.base.id
    }
    
    fn type_name(&self) -> &str {
        "Text"
    }
    
    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult> {
        // TODO: Calculate text size
        Ok(LayoutResult::new(nalgebra::Vector2::new(100.0, 20.0), nalgebra::Vector2::zeros()))
    }
    
    fn handle_input(&mut self, _event: &InputEvent) -> InputResult {
        Ok(EventHandled::No)
    }
    
    fn render(&self, theme: &Theme) -> UiResult<RenderData> {
        // TODO: Render text with SDF
        Ok(RenderData::new())
    }
    
    fn update(&mut self, _delta_time: f32) {}
    
    fn children(&self) -> &[ElementId] {
        &self.base.children
    }
    
    fn children_mut(&mut self) -> &mut Vec<ElementId> {
        &mut self.base.children
    }
    
    fn is_dirty(&self) -> bool {
        self.base.dirty
    }
    
    fn mark_clean(&mut self) {
        self.base.dirty = false;
    }
    
    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
    
    fn bounds(&self) -> ElementBounds {
        self.base.bounds
    }
    
    fn set_bounds(&mut self, bounds: ElementBounds) {
        self.base.bounds = bounds;
        self.mark_dirty();
    }
    
    fn is_visible(&self) -> bool {
        self.base.visible
    }
    
    fn set_visible(&mut self, visible: bool) {
        self.base.visible = visible;
        self.mark_dirty();
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}