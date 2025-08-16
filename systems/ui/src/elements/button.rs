//! Button element

use crate::element::{Element, ElementBase, ElementBounds, ElementId};
use crate::input::{InputEvent, InputResult, EventHandled};
use crate::layout::{LayoutConstraints, LayoutResult};
use crate::rendering::RenderData;
use crate::theme::Theme;
use crate::error::UiResult;
use std::any::Any;
use uuid::Uuid;

/// Button element
pub struct Button {
    base: ElementBase,
    text: String,
    pressed: bool,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            base: ElementBase::new(),
            text: text.into(),
            pressed: false,
            on_click: None,
        }
    }
    
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(callback));
        self
    }
}

impl Element for Button {
    fn id(&self) -> Uuid {
        self.base.id
    }
    
    fn type_name(&self) -> &str {
        "Button"
    }
    
    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult> {
        // TODO: Calculate button size based on text
        Ok(LayoutResult::new(nalgebra::Vector2::new(120.0, 40.0), nalgebra::Vector2::zeros()))
    }
    
    fn handle_input(&mut self, event: &InputEvent) -> InputResult {
        match event {
            InputEvent::PointerDown { .. } => {
                self.pressed = true;
                self.mark_dirty();
                InputResult { handled: EventHandled::Yes, request_focus: false }
            }
            InputEvent::PointerUp { .. } => {
                if self.pressed {
                    if let Some(callback) = &self.on_click {
                        callback();
                    }
                    self.pressed = false;
                    self.mark_dirty();
                }
                InputResult { handled: EventHandled::Yes, request_focus: false }
            }
            _ => InputResult { handled: EventHandled::No, request_focus: false },
        }
    }
    
    fn render(&self, theme: &Theme) -> UiResult<RenderData> {
        let mut data = RenderData::new();
        let color = if self.pressed {
            theme.colors.primary * 0.8
        } else {
            theme.colors.primary
        };
        data.add_quad(self.base.bounds.position, self.base.bounds.size, color);
        Ok(data)
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