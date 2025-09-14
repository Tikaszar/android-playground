//! Wrapper types for avoiding dyn trait objects

use bytes::Bytes;
use crate::{ElementId, ElementType, Style, Bounds};
use serde::{Serialize, Deserialize};

/// Concrete wrapper for UI elements that avoids dyn
/// This follows the NO dyn pattern used throughout the codebase
#[derive(Clone)]
pub struct UiElementWrapper {
    /// Serialized element data
    data: Bytes,
    /// Element ID
    id: ElementId,
    /// Element type
    element_type: ElementType,
    /// Current bounds
    bounds: Bounds,
    /// Current style
    style: Style,
    /// Whether element is visible
    visible: bool,
    /// Parent element ID if any
    parent: Option<ElementId>,
    /// Child element IDs
    children: Vec<ElementId>,
}

impl UiElementWrapper {
    /// Create a new UI element wrapper
    pub fn new(
        data: Bytes,
        id: ElementId,
        element_type: ElementType,
        bounds: Bounds,
        style: Style,
    ) -> Self {
        Self {
            data,
            id,
            element_type,
            bounds,
            style,
            visible: true,
            parent: None,
            children: Vec::new(),
        }
    }
    
    /// Get the element ID
    pub fn id(&self) -> ElementId {
        self.id
    }
    
    /// Get the element type
    pub fn element_type(&self) -> ElementType {
        self.element_type
    }
    
    /// Get the bounds
    pub fn bounds(&self) -> &Bounds {
        &self.bounds
    }
    
    /// Get mutable bounds
    pub fn bounds_mut(&mut self) -> &mut Bounds {
        &mut self.bounds
    }
    
    /// Get the style
    pub fn style(&self) -> &Style {
        &self.style
    }
    
    /// Get mutable style
    pub fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
    
    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible && self.style.visible && self.style.opacity > 0.0
    }
    
    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    
    /// Get parent
    pub fn parent(&self) -> Option<ElementId> {
        self.parent
    }
    
    /// Set parent
    pub fn set_parent(&mut self, parent: Option<ElementId>) {
        self.parent = parent;
    }
    
    /// Get children
    pub fn children(&self) -> &[ElementId] {
        &self.children
    }
    
    /// Add child
    pub fn add_child(&mut self, child: ElementId) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }
    
    /// Remove child
    pub fn remove_child(&mut self, child: ElementId) {
        self.children.retain(|&id| id != child);
    }
    
    /// Clear children
    pub fn clear_children(&mut self) {
        self.children.clear();
    }
    
    /// Get the serialized data
    pub fn data(&self) -> &Bytes {
        &self.data
    }
    
    /// Update the serialized data
    pub fn set_data(&mut self, data: Bytes) {
        self.data = data;
    }
    
    /// Check if point is contained in bounds
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        self.bounds.contains(x, y)
    }
}

/// Element metadata for UI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiElementInfo {
    pub id: ElementId,
    pub element_type: ElementType,
    pub bounds: Bounds,
    pub visible: bool,
    pub z_index: i32,
    pub parent: Option<ElementId>,
    pub children_count: usize,
    pub has_event_handlers: bool,
    pub is_interactive: bool,
}

impl UiElementInfo {
    /// Create from wrapper
    pub fn from_wrapper(wrapper: &UiElementWrapper) -> Self {
        Self {
            id: wrapper.id,
            element_type: wrapper.element_type,
            bounds: wrapper.bounds,
            visible: wrapper.is_visible(),
            z_index: wrapper.style.z_index,
            parent: wrapper.parent,
            children_count: wrapper.children.len(),
            has_event_handlers: false, // Would be set by implementation
            is_interactive: matches!(
                wrapper.element_type,
                ElementType::Button | ElementType::Input | ElementType::ScrollView
            ),
        }
    }
}