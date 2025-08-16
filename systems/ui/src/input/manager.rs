//! Input manager for handling events

use crate::element::{ElementGraph, ElementId};
use crate::error::UiResult;
use crate::input::{InputEvent, EventHandled, InputResult};
use nalgebra::Vector2;
use std::collections::VecDeque;

/// Manages input events and routing
pub struct InputManager {
    event_queue: VecDeque<InputEvent>,
    focused_element: Option<ElementId>,
    pointer_position: Vector2<f32>,
    captured_element: Option<ElementId>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            focused_element: None,
            pointer_position: Vector2::zeros(),
            captured_element: None,
        }
    }
    
    /// Queue an input event
    pub fn queue_event(&mut self, event: InputEvent) {
        self.event_queue.push_back(event);
    }
    
    /// Process all queued events
    pub fn process_events(&mut self, graph: &mut ElementGraph) -> UiResult<()> {
        while let Some(event) = self.event_queue.pop_front() {
            self.process_event(graph, event)?;
        }
        Ok(())
    }
    
    /// Process a single event
    fn process_event(&mut self, graph: &mut ElementGraph, event: InputEvent) -> UiResult<()> {
        match &event {
            InputEvent::PointerMove { position, .. } => {
                self.pointer_position = *position;
            }
            InputEvent::PointerDown { position, .. } => {
                // Find element under pointer
                if let Some(element_id) = self.find_element_at(graph, *position) {
                    self.captured_element = Some(element_id);
                }
            }
            InputEvent::PointerUp { .. } => {
                self.captured_element = None;
            }
            _ => {}
        }
        
        // Route event to appropriate element
        if let Some(element_id) = self.get_target_element(&event) {
            if let Some(element) = graph.get_mut(element_id) {
                let result = element.handle_input(&event);
                if result.handled == EventHandled::Yes {
                    if result.request_focus {
                        self.focused_element = Some(element_id);
                    }
                    return Ok(());
                }
            }
        }
        
        // Bubble event up through parents
        // TODO: Implement event bubbling
        
        Ok(())
    }
    
    /// Find element at position
    fn find_element_at(&self, graph: &ElementGraph, position: Vector2<f32>) -> Option<ElementId> {
        // TODO: Implement hit testing with z-order
        for (id, element) in graph.iter() {
            if element.contains_point(position) && element.is_visible() {
                return Some(id);
            }
        }
        None
    }
    
    /// Get target element for event
    fn get_target_element(&self, event: &InputEvent) -> Option<ElementId> {
        match event {
            InputEvent::KeyDown { .. } | InputEvent::KeyUp { .. } | InputEvent::TextInput { .. } => {
                self.focused_element
            }
            InputEvent::PointerMove { .. } | InputEvent::PointerDown { .. } | InputEvent::PointerUp { .. } => {
                self.captured_element.or(self.focused_element)
            }
            _ => None,
        }
    }
    
    /// Set focused element
    pub fn set_focus(&mut self, element_id: Option<ElementId>) {
        self.focused_element = element_id;
    }
    
    /// Get focused element
    pub fn focused_element(&self) -> Option<ElementId> {
        self.focused_element
    }
    
    /// Get current pointer position
    pub fn pointer_position(&self) -> Vector2<f32> {
        self.pointer_position
    }
}