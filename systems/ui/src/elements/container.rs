//! Container element

use crate::element::{Element, ElementBase, ElementBounds, ElementId};
use crate::input::{InputEvent, InputResult, EventHandled};
use crate::layout::{LayoutEngine, LayoutConstraints, LayoutResult, FlexLayout};
use crate::rendering::RenderData;
use crate::theme::Theme;
use crate::error::UiResult;
use std::any::Any;
use uuid::Uuid;

/// Container element that holds child elements
pub struct Container {
    base: ElementBase,
    layout: Box<dyn LayoutEngine>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            base: ElementBase::new(),
            layout: Box::new(FlexLayout::default()),
        }
    }
    
    pub fn with_layout(mut self, layout: Box<dyn LayoutEngine>) -> Self {
        self.layout = layout;
        self
    }
}

impl Element for Container {
    fn id(&self) -> Uuid {
        self.base.id
    }
    
    fn type_name(&self) -> &str {
        "Container"
    }
    
    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult> {
        // TODO: Implement layout
        Ok(LayoutResult::new(constraints.available_size, nalgebra::Vector2::zeros()))
    }
    
    fn handle_input(&mut self, event: &InputEvent) -> InputResult {
        Ok(EventHandled::No)
    }
    
    fn render(&self, theme: &Theme) -> UiResult<RenderData> {
        let mut data = RenderData::new();
        data.add_quad(
            self.base.bounds.position,
            self.base.bounds.size,
            theme.colors.surface,
        );
        Ok(data)
    }
    
    fn update(&mut self, delta_time: f32) {
        // Update logic
    }
    
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