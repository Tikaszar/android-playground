use playground_core_ui::ElementId;
use playground_core_types::{Shared, shared};
use crate::error::{UiError, UiResult};
use std::collections::HashMap;

// ElementId is now from core/ui, not an alias

// Element graph for managing UI hierarchy
pub struct ElementGraph {
    children: HashMap<ElementId, Vec<ElementId>>,
    parents: HashMap<ElementId, ElementId>,
}

impl ElementGraph {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            parents: HashMap::new(),
        }
    }
    
    pub fn add_child(&mut self, parent: ElementId, child: ElementId) -> UiResult<()> {
        // Remove from previous parent if exists
        if let Some(old_parent) = self.parents.get(&child) {
            if let Some(siblings) = self.children.get_mut(old_parent) {
                siblings.retain(|&id| id != child);
            }
        }
        
        // Add to new parent
        self.children.entry(parent).or_insert_with(Vec::new).push(child);
        self.parents.insert(child, parent);
        
        Ok(())
    }
    
    pub fn remove_child(&mut self, parent: ElementId, child: ElementId) -> UiResult<()> {
        if let Some(children) = self.children.get_mut(&parent) {
            children.retain(|&id| id != child);
        }
        self.parents.remove(&child);
        Ok(())
    }
    
    pub fn get_children(&self, parent: ElementId) -> Option<&Vec<ElementId>> {
        self.children.get(&parent)
    }
    
    pub fn get_parent(&self, child: ElementId) -> Option<ElementId> {
        self.parents.get(&child).copied()
    }
    
    pub fn remove_element(&mut self, element: ElementId) {
        // Remove as child from parent
        if let Some(parent) = self.parents.remove(&element) {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|&id| id != element);
            }
        }
        
        // Remove all children recursively
        if let Some(children) = self.children.remove(&element) {
            for child in children {
                self.remove_element(child);
            }
        }
    }
    
    pub fn iter_depth_first(&self, root: ElementId) -> DepthFirstIterator {
        DepthFirstIterator::new(self, root)
    }
}

pub struct DepthFirstIterator<'a> {
    graph: &'a ElementGraph,
    stack: Vec<ElementId>,
}

impl<'a> DepthFirstIterator<'a> {
    fn new(graph: &'a ElementGraph, root: ElementId) -> Self {
        Self {
            graph,
            stack: vec![root],
        }
    }
}

impl<'a> Iterator for DepthFirstIterator<'a> {
    type Item = ElementId;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(element) = self.stack.pop() {
            // Add children to stack in reverse order for correct traversal
            if let Some(children) = self.graph.get_children(element) {
                for &child in children.iter().rev() {
                    self.stack.push(child);
                }
            }
            Some(element)
        } else {
            None
        }
    }
}