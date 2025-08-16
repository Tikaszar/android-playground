//! Gesture-aware element wrapper

use nalgebra::{Vector2, Vector4};
use crate::element::{Element, ElementBounds};
use uuid::Uuid;
use std::any::Any;
use std::sync::{Arc, RwLock};
use crate::layout::{LayoutConstraints, LayoutResult};
use crate::rendering::RenderData;
use super::event::{InputEvent, InputResult, EventHandled};
use super::gestures::{GestureRecognizer, GestureType, GestureConfig, SwipeDirection};

/// Callback function for gesture events
pub type GestureCallback = Arc<RwLock<dyn FnMut(&GestureType) -> bool + Send + Sync>>;

/// Gesture-aware wrapper for UI elements
pub struct GestureElement {
    id: Uuid,
    inner: Box<dyn Element>,
    recognizer: GestureRecognizer,
    callbacks: Vec<(GestureType, GestureCallback)>,
    enabled: bool,
    bounds: ElementBounds,
    dirty: bool,
    visible: bool,
}

impl GestureElement {
    /// Create a new gesture-aware element
    pub fn new(inner: Box<dyn Element>) -> Self {
        let bounds = inner.bounds();
        let visible = inner.is_visible();
        Self {
            id: Uuid::new_v4(),
            inner,
            recognizer: GestureRecognizer::default(),
            callbacks: Vec::new(),
            enabled: true,
            bounds,
            dirty: false,
            visible,
        }
    }
    
    /// Create with custom gesture configuration
    pub fn with_config(inner: Box<dyn Element>, config: GestureConfig) -> Self {
        let bounds = inner.bounds();
        let visible = inner.is_visible();
        Self {
            id: Uuid::new_v4(),
            inner,
            recognizer: GestureRecognizer::new(config),
            callbacks: Vec::new(),
            enabled: true,
            bounds,
            dirty: false,
            visible,
        }
    }
    
    /// Enable or disable gesture recognition
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.recognizer.reset();
        }
    }
    
    /// Register a callback for tap gestures
    pub fn on_tap<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnMut(&GestureType) -> bool + Send + Sync + 'static,
    {
        self.callbacks.push((
            GestureType::Tap {
                position: Vector2::zeros(),
                touch_id: 0,
            },
            Arc::new(RwLock::new(callback)),
        ));
        self
    }
    
    /// Register a callback for double tap gestures
    pub fn on_double_tap<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnMut(&GestureType) -> bool + Send + Sync + 'static,
    {
        self.callbacks.push((
            GestureType::DoubleTap {
                position: Vector2::zeros(),
                touch_id: 0,
            },
            Arc::new(RwLock::new(callback)),
        ));
        self
    }
    
    /// Register a callback for long press gestures
    pub fn on_long_press<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnMut(&GestureType) -> bool + Send + Sync + 'static,
    {
        self.callbacks.push((
            GestureType::LongPress {
                position: Vector2::zeros(),
                touch_id: 0,
                duration: std::time::Duration::from_secs(0),
            },
            Arc::new(RwLock::new(callback)),
        ));
        self
    }
    
    /// Register a callback for swipe gestures
    pub fn on_swipe<F>(&mut self, direction: Option<SwipeDirection>, callback: F) -> &mut Self
    where
        F: FnMut(&GestureType) -> bool + Send + Sync + 'static,
    {
        if let Some(dir) = direction {
            self.callbacks.push((
                GestureType::Swipe {
                    start: Vector2::zeros(),
                    end: Vector2::zeros(),
                    velocity: Vector2::zeros(),
                    direction: dir,
                    touch_id: 0,
                },
                Arc::new(RwLock::new(callback)),
            ));
        } else {
            // For all directions, we need to share the callback
            let cb = Arc::new(RwLock::new(callback));
            for dir in [SwipeDirection::Up, SwipeDirection::Down, 
                       SwipeDirection::Left, SwipeDirection::Right] {
                self.callbacks.push((
                    GestureType::Swipe {
                        start: Vector2::zeros(),
                        end: Vector2::zeros(),
                        velocity: Vector2::zeros(),
                        direction: dir,
                        touch_id: 0,
                    },
                    cb.clone(),
                ));
            }
        }
        self
    }
    
    /// Register a callback for pinch gestures
    pub fn on_pinch<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnMut(&GestureType) -> bool + Send + Sync + 'static,
    {
        self.callbacks.push((
            GestureType::Pinch {
                center: Vector2::zeros(),
                scale: 1.0,
                velocity: 0.0,
            },
            Arc::new(RwLock::new(callback)),
        ));
        self
    }
    
    /// Register a callback for pan gestures
    pub fn on_pan<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnMut(&GestureType) -> bool + Send + Sync + 'static,
    {
        self.callbacks.push((
            GestureType::Pan {
                position: Vector2::zeros(),
                delta: Vector2::zeros(),
                velocity: Vector2::zeros(),
                touch_id: 0,
            },
            Arc::new(RwLock::new(callback)),
        ));
        self
    }
    
    /// Register a callback for any gesture type
    pub fn on_gesture<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnMut(&GestureType) -> bool + Send + Sync + 'static,
    {
        // This will be called for any gesture
        self.callbacks.push((
            GestureType::Tap {
                position: Vector2::zeros(),
                touch_id: 0,
            },
            Arc::new(RwLock::new(callback)),
        ));
        self
    }
    
    /// Process gestures and trigger callbacks
    fn process_gestures(&mut self, gestures: Vec<GestureType>) -> bool {
        let mut handled = false;
        
        for gesture in gestures {
            for (pattern, callback) in &self.callbacks {
                if self.matches_gesture_pattern(&gesture, pattern) {
                    if let Ok(mut cb) = callback.write() {
                        if cb(&gesture) {
                            handled = true;
                        }
                    }
                }
            }
        }
        
        handled
    }
    
    /// Check if a gesture matches a pattern
    fn matches_gesture_pattern(&self, gesture: &GestureType, pattern: &GestureType) -> bool {
        use GestureType::*;
        
        match (gesture, pattern) {
            (Tap { .. }, Tap { .. }) => true,
            (DoubleTap { .. }, DoubleTap { .. }) => true,
            (LongPress { .. }, LongPress { .. }) => true,
            (Swipe { direction: d1, .. }, Swipe { direction: d2, .. }) => d1 == d2,
            (Pinch { .. }, Pinch { .. }) => true,
            (Rotate { .. }, Rotate { .. }) => true,
            (Pan { .. }, Pan { .. }) => true,
            (Fling { direction: d1, .. }, Fling { direction: d2, .. }) => d1 == d2,
            _ => false,
        }
    }
}

impl Element for GestureElement {
    fn id(&self) -> Uuid {
        self.id
    }
    
    fn type_name(&self) -> &str {
        "GestureElement"
    }
    
    fn layout(&mut self, constraints: &LayoutConstraints) -> crate::UiResult<LayoutResult> {
        let result = self.inner.layout(constraints)?;
        self.bounds.size = result.size;
        Ok(result)
    }
    
    fn render(&self, theme: &crate::theme::Theme) -> crate::UiResult<RenderData> {
        self.inner.render(theme)
    }
    
    fn handle_input(&mut self, event: &InputEvent) -> InputResult {
        if !self.enabled {
            return self.inner.handle_input(event);
        }
        
        // First, let the gesture recognizer process the event
        let gestures = self.recognizer.process_event(event);
        
        // Process any recognized gestures
        if !gestures.is_empty() && self.process_gestures(gestures) {
            return InputResult {
                handled: EventHandled::Yes,
                request_focus: false,
            };
        }
        
        // Pass through to inner element
        self.inner.handle_input(event)
    }
    
    fn update(&mut self, delta_time: f32) {
        self.inner.update(delta_time);
    }
    
    fn children(&self) -> &[crate::element::ElementId] {
        self.inner.children()
    }
    
    fn children_mut(&mut self) -> &mut Vec<crate::element::ElementId> {
        self.inner.children_mut()
    }
    
    fn is_dirty(&self) -> bool {
        self.dirty || self.inner.is_dirty()
    }
    
    fn mark_clean(&mut self) {
        self.dirty = false;
        self.inner.mark_clean();
    }
    
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    fn bounds(&self) -> ElementBounds {
        self.bounds
    }
    
    fn set_bounds(&mut self, bounds: ElementBounds) {
        self.bounds = bounds;
        self.inner.set_bounds(bounds);
    }
    
    fn is_visible(&self) -> bool {
        self.visible && self.inner.is_visible()
    }
    
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        self.inner.set_visible(visible);
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Extension trait to easily add gesture support to any element
pub trait GestureExt: Element + Sized {
    /// Wrap this element with gesture recognition
    fn with_gestures(self) -> GestureElement {
        GestureElement::new(Box::new(self))
    }
    
    /// Wrap with custom gesture configuration
    fn with_gesture_config(self, config: GestureConfig) -> GestureElement {
        GestureElement::with_config(Box::new(self), config)
    }
}

// Implement GestureExt for all Element types
impl<T: Element + 'static> GestureExt for T {}