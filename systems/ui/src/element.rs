//! Core UI element trait and types

use crate::UiResult;
use crate::layout::{LayoutConstraints, LayoutResult};
use crate::input::{InputEvent, InputResult};
use crate::rendering::RenderData;
use crate::theme::Theme;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use std::any::Any;
use uuid::Uuid;

new_key_type! {
    /// Unique key for UI elements in the graph
    pub struct ElementId;
}

/// Core trait that all UI elements must implement
pub trait Element: Any + Send + Sync {
    /// Get the element's unique ID
    fn id(&self) -> Uuid;
    
    /// Get the element's type name
    fn type_name(&self) -> &str;
    
    /// Perform layout calculation
    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult>;
    
    /// Handle input events
    fn handle_input(&mut self, event: &InputEvent) -> InputResult;
    
    /// Generate render data for batching
    fn render(&self, theme: &Theme) -> UiResult<RenderData>;
    
    /// Update element state
    fn update(&mut self, delta_time: f32);
    
    /// Get child elements
    fn children(&self) -> &[ElementId];
    
    /// Get mutable child elements
    fn children_mut(&mut self) -> &mut Vec<ElementId>;
    
    /// Check if element is dirty and needs re-rendering
    fn is_dirty(&self) -> bool;
    
    /// Mark element as clean
    fn mark_clean(&mut self);
    
    /// Mark element as dirty
    fn mark_dirty(&mut self);
    
    /// Get element bounds in screen space
    fn bounds(&self) -> ElementBounds;
    
    /// Set element bounds
    fn set_bounds(&mut self, bounds: ElementBounds);
    
    /// Check if point is inside element
    fn contains_point(&self, point: Vector2<f32>) -> bool {
        self.bounds().contains(point)
    }
    
    /// Get element visibility
    fn is_visible(&self) -> bool;
    
    /// Set element visibility
    fn set_visible(&mut self, visible: bool);
    
    /// As any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// As mutable any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Bounding box for UI elements
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ElementBounds {
    /// Position (x, y) in screen space
    pub position: Vector2<f32>,
    /// Size (width, height)
    pub size: Vector2<f32>,
}

impl ElementBounds {
    /// Create new bounds
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Vector2::new(x, y),
            size: Vector2::new(width, height),
        }
    }
    
    /// Check if point is inside bounds
    pub fn contains(&self, point: Vector2<f32>) -> bool {
        point.x >= self.position.x &&
        point.x <= self.position.x + self.size.x &&
        point.y >= self.position.y &&
        point.y <= self.position.y + self.size.y
    }
    
    /// Get the center point
    pub fn center(&self) -> Vector2<f32> {
        self.position + self.size * 0.5
    }
    
    /// Get top-left corner
    pub fn top_left(&self) -> Vector2<f32> {
        self.position
    }
    
    /// Get bottom-right corner
    pub fn bottom_right(&self) -> Vector2<f32> {
        self.position + self.size
    }
    
    /// Check if bounds intersect
    pub fn intersects(&self, other: &ElementBounds) -> bool {
        let self_br = self.bottom_right();
        let other_br = other.bottom_right();
        
        !(self.position.x > other_br.x ||
          other.position.x > self_br.x ||
          self.position.y > other_br.y ||
          other.position.y > self_br.y)
    }
}

/// Base implementation for common element functionality
pub struct ElementBase {
    pub id: Uuid,
    pub bounds: ElementBounds,
    pub children: Vec<ElementId>,
    pub dirty: bool,
    pub visible: bool,
}

impl ElementBase {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            bounds: ElementBounds::new(0.0, 0.0, 0.0, 0.0),
            children: Vec::new(),
            dirty: true,
            visible: true,
        }
    }
    
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

/// UI element graph that manages all elements
pub struct ElementGraph {
    elements: SlotMap<ElementId, Box<dyn Element>>,
    root: Option<ElementId>,
}

impl ElementGraph {
    pub fn new() -> Self {
        Self {
            elements: SlotMap::with_key(),
            root: None,
        }
    }
    
    /// Add element to graph
    pub fn add_element(&mut self, element: Box<dyn Element>) -> ElementId {
        self.elements.insert(element)
    }
    
    /// Remove element from graph
    pub fn remove_element(&mut self, id: ElementId) -> Option<Box<dyn Element>> {
        self.elements.remove(id)
    }
    
    /// Get element reference
    pub fn get(&self, id: ElementId) -> Option<&dyn Element> {
        self.elements.get(id).map(|e| e.as_ref())
    }
    
    /// Get mutable element reference
    pub fn get_mut(&mut self, id: ElementId) -> Option<&mut dyn Element> {
        self.elements.get_mut(id).map(|e| e.as_mut())
    }
    
    /// Set root element
    pub fn set_root(&mut self, id: ElementId) {
        self.root = Some(id);
    }
    
    /// Get root element
    pub fn root(&self) -> Option<ElementId> {
        self.root
    }
    
    /// Iterate all elements
    pub fn iter(&self) -> impl Iterator<Item = (ElementId, &dyn Element)> {
        self.elements.iter().map(|(id, e)| (id, e.as_ref()))
    }
    
    /// Get dirty elements that need re-rendering
    pub fn dirty_elements(&self) -> Vec<ElementId> {
        self.elements
            .iter()
            .filter(|(_, e)| e.is_dirty())
            .map(|(id, _)| id)
            .collect()
    }
}

/// Element state for hot-reload and persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementState {
    pub id: Uuid,
    pub type_name: String,
    pub bounds: ElementBounds,
    pub visible: bool,
    pub children: Vec<Uuid>,
    pub custom_state: serde_json::Value,
}