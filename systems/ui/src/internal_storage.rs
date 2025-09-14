//! Internal storage for UI elements without using ECS
//! 
//! This module provides internal storage for UI elements that doesn't rely
//! on the ECS World. The UiSystem queries the World for UI components and
//! converts them to this internal representation for processing.

use std::collections::HashMap;
use playground_core_ui::{ElementId, ElementType, Style, Bounds};
use playground_core_types::{Shared, shared};

/// Internal representation of a UI element
#[derive(Clone)]
pub struct InternalElement {
    pub id: ElementId,
    pub element_type: ElementType,
    pub style: Style,
    pub bounds: Bounds,
    pub parent: Option<ElementId>,
    pub children: Vec<ElementId>,
    pub text: Option<String>,
    pub visible: bool,
    pub dirty: bool,
}

impl InternalElement {
    pub fn new(id: ElementId, element_type: ElementType) -> Self {
        Self {
            id,
            element_type,
            style: Style::default(),
            bounds: Bounds::new(0.0, 0.0, 100.0, 100.0),
            parent: None,
            children: Vec::new(),
            text: None,
            visible: true,
            dirty: true,
        }
    }
    
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    
    pub fn with_bounds(mut self, bounds: Bounds) -> Self {
        self.bounds = bounds;
        self
    }
    
    pub fn with_parent(mut self, parent: Option<ElementId>) -> Self {
        self.parent = parent;
        self
    }
    
    pub fn add_child(&mut self, child: ElementId) {
        if !self.children.contains(&child) {
            self.children.push(child);
            self.dirty = true;
        }
    }
    
    pub fn remove_child(&mut self, child: ElementId) {
        self.children.retain(|&id| id != child);
        self.dirty = true;
    }
}

/// Internal storage for UI elements
pub struct InternalElementStorage {
    elements: Shared<HashMap<ElementId, InternalElement>>,
    root_element: Shared<Option<ElementId>>,
    dirty_elements: Shared<Vec<ElementId>>,
    z_order: Shared<Vec<ElementId>>,
}

impl InternalElementStorage {
    pub fn new() -> Self {
        Self {
            elements: shared(HashMap::new()),
            root_element: shared(None),
            dirty_elements: shared(Vec::new()),
            z_order: shared(Vec::new()),
        }
    }
    
    pub async fn create_element(&self, element_type: ElementType) -> ElementId {
        let id = ElementId::new();
        let element = InternalElement::new(id, element_type);
        
        self.elements.write().await.insert(id, element);
        self.dirty_elements.write().await.push(id);
        
        // If no root, make this the root
        let mut root = self.root_element.write().await;
        if root.is_none() {
            *root = Some(id);
        }
        
        id
    }
    
    pub async fn get_element(&self, id: ElementId) -> Option<InternalElement> {
        self.elements.read().await.get(&id).cloned()
    }
    
    pub async fn update_element<F>(&self, id: ElementId, updater: F) -> bool 
    where
        F: FnOnce(&mut InternalElement)
    {
        let mut elements = self.elements.write().await;
        if let Some(element) = elements.get_mut(&id) {
            updater(element);
            element.dirty = true;
            self.dirty_elements.write().await.push(id);
            true
        } else {
            false
        }
    }
    
    pub async fn remove_element(&self, id: ElementId) -> bool {
        let mut elements = self.elements.write().await;
        
        // Get children to remove recursively
        let children = if let Some(element) = elements.get(&id) {
            element.children.clone()
        } else {
            return false;
        };
        
        // Remove from parent's children list
        if let Some(element) = elements.get(&id) {
            if let Some(parent_id) = element.parent {
                if let Some(parent) = elements.get_mut(&parent_id) {
                    parent.remove_child(id);
                }
            }
        }
        
        // Remove the element
        elements.remove(&id);
        
        // Remove from dirty list
        self.dirty_elements.write().await.retain(|&eid| eid != id);
        
        // Remove from z-order
        self.z_order.write().await.retain(|&eid| eid != id);
        
        // Update root if needed
        let mut root = self.root_element.write().await;
        if *root == Some(id) {
            *root = None;
        }
        
        // Recursively remove children
        drop(elements); // Release lock before recursive calls
        for child in children {
            let _ = self.remove_element(child).await;
        }
        
        true
    }
    
    pub async fn get_root(&self) -> Option<ElementId> {
        *self.root_element.read().await
    }
    
    pub async fn set_root(&self, id: Option<ElementId>) {
        *self.root_element.write().await = id;
    }
    
    pub async fn get_dirty_elements(&self) -> Vec<ElementId> {
        self.dirty_elements.read().await.clone()
    }
    
    pub async fn clear_dirty(&self) {
        self.dirty_elements.write().await.clear();
        
        // Clear dirty flags on all elements
        let mut elements = self.elements.write().await;
        for element in elements.values_mut() {
            element.dirty = false;
        }
    }
    
    pub async fn update_z_order(&self) {
        let elements = self.elements.read().await;
        let mut z_order: Vec<(ElementId, i32)> = elements
            .values()
            .map(|e| (e.id, e.style.z_index))
            .collect();
        
        z_order.sort_by_key(|&(_, z)| z);
        
        *self.z_order.write().await = z_order.into_iter().map(|(id, _)| id).collect();
    }
    
    pub async fn get_z_order(&self) -> Vec<ElementId> {
        self.z_order.read().await.clone()
    }
    
    pub async fn clear(&self) {
        self.elements.write().await.clear();
        *self.root_element.write().await = None;
        self.dirty_elements.write().await.clear();
        self.z_order.write().await.clear();
    }
}