//! Mobile gesture recognition system

use std::collections::HashMap;
use std::time::{Duration, Instant};
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use super::event::{InputEvent, PointerButton};

/// Gesture types that can be recognized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureType {
    /// Single tap
    Tap {
        position: Vector2<f32>,
        touch_id: u32,
    },
    
    /// Double tap
    DoubleTap {
        position: Vector2<f32>,
        touch_id: u32,
    },
    
    /// Long press
    LongPress {
        position: Vector2<f32>,
        touch_id: u32,
        duration: Duration,
    },
    
    /// Swipe gesture
    Swipe {
        start: Vector2<f32>,
        end: Vector2<f32>,
        velocity: Vector2<f32>,
        direction: SwipeDirection,
        touch_id: u32,
    },
    
    /// Pinch gesture for zooming
    Pinch {
        center: Vector2<f32>,
        scale: f32,
        velocity: f32,
    },
    
    /// Rotation gesture
    Rotate {
        center: Vector2<f32>,
        angle: f32,
        velocity: f32,
    },
    
    /// Pan/drag gesture
    Pan {
        position: Vector2<f32>,
        delta: Vector2<f32>,
        velocity: Vector2<f32>,
        touch_id: u32,
    },
    
    /// Fling gesture (fast swipe)
    Fling {
        position: Vector2<f32>,
        velocity: Vector2<f32>,
        direction: SwipeDirection,
        touch_id: u32,
    },
}

/// Swipe direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Touch point state
#[derive(Debug, Clone)]
struct TouchPoint {
    id: u32,
    start_position: Vector2<f32>,
    current_position: Vector2<f32>,
    previous_position: Vector2<f32>,
    start_time: Instant,
    last_move_time: Instant,
    velocity: Vector2<f32>,
    is_moving: bool,
    total_distance: f32,
}

impl TouchPoint {
    fn new(id: u32, position: Vector2<f32>) -> Self {
        let now = Instant::now();
        Self {
            id,
            start_position: position,
            current_position: position,
            previous_position: position,
            start_time: now,
            last_move_time: now,
            velocity: Vector2::zeros(),
            is_moving: false,
            total_distance: 0.0,
        }
    }
    
    fn update_position(&mut self, position: Vector2<f32>) {
        let now = Instant::now();
        let dt = (now - self.last_move_time).as_secs_f32();
        
        if dt > 0.0 {
            let delta = position - self.current_position;
            self.velocity = delta / dt;
            self.total_distance += delta.magnitude();
        }
        
        self.previous_position = self.current_position;
        self.current_position = position;
        self.last_move_time = now;
        self.is_moving = true;
    }
    
    fn get_duration(&self) -> Duration {
        Instant::now() - self.start_time
    }
}

/// Gesture recognizer configuration
#[derive(Debug, Clone)]
pub struct GestureConfig {
    /// Maximum time between taps for double tap (ms)
    pub double_tap_time: u64,
    
    /// Maximum distance between taps for double tap (pixels)
    pub double_tap_distance: f32,
    
    /// Minimum time for long press (ms)
    pub long_press_time: u64,
    
    /// Maximum movement allowed during long press (pixels)
    pub long_press_tolerance: f32,
    
    /// Minimum distance for swipe (pixels)
    pub swipe_min_distance: f32,
    
    /// Minimum velocity for swipe (pixels/second)
    pub swipe_min_velocity: f32,
    
    /// Minimum velocity for fling (pixels/second)
    pub fling_min_velocity: f32,
    
    /// Minimum scale change for pinch
    pub pinch_threshold: f32,
    
    /// Minimum rotation angle for rotate gesture (radians)
    pub rotate_threshold: f32,
    
    /// Enable haptic feedback
    pub haptic_feedback: bool,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            double_tap_time: 300,
            double_tap_distance: 30.0,
            long_press_time: 500,
            long_press_tolerance: 10.0,
            swipe_min_distance: 50.0,
            swipe_min_velocity: 100.0,
            fling_min_velocity: 500.0,
            pinch_threshold: 0.05,
            rotate_threshold: 0.1,
            haptic_feedback: true,
        }
    }
}

/// Gesture recognizer state
pub struct GestureRecognizer {
    config: GestureConfig,
    active_touches: HashMap<u32, TouchPoint>,
    last_tap: Option<(Vector2<f32>, Instant)>,
    long_press_timer: Option<(u32, Instant)>,
    gesture_in_progress: Option<GestureType>,
    pinch_initial_distance: Option<f32>,
    rotate_initial_angle: Option<f32>,
}

impl GestureRecognizer {
    /// Create a new gesture recognizer
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config,
            active_touches: HashMap::new(),
            last_tap: None,
            long_press_timer: None,
            gesture_in_progress: None,
            pinch_initial_distance: None,
            rotate_initial_angle: None,
        }
    }
    
    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(GestureConfig::default())
    }
    
    /// Process an input event and return recognized gestures
    pub fn process_event(&mut self, event: &InputEvent) -> Vec<GestureType> {
        let mut gestures = Vec::new();
        
        match event {
            InputEvent::PointerDown { position, button } => {
                if let PointerButton::Touch(id) = button {
                    self.handle_touch_down(*id, *position, &mut gestures);
                }
            }
            
            InputEvent::PointerMove { position, .. } => {
                self.handle_touch_move(*position, &mut gestures);
            }
            
            InputEvent::PointerUp { position, button } => {
                if let PointerButton::Touch(id) = button {
                    self.handle_touch_up(*id, *position, &mut gestures);
                }
            }
            
            _ => {}
        }
        
        // Check for long press timeout
        if let Some((id, start_time)) = self.long_press_timer {
            if start_time.elapsed().as_millis() >= self.config.long_press_time as u128 {
                if let Some(touch) = self.active_touches.get(&id) {
                    if !touch.is_moving && touch.total_distance < self.config.long_press_tolerance {
                        gestures.push(GestureType::LongPress {
                            position: touch.current_position,
                            touch_id: id,
                            duration: touch.get_duration(),
                        });
                        self.long_press_timer = None;
                    }
                }
            }
        }
        
        gestures
    }
    
    /// Handle touch down event
    fn handle_touch_down(&mut self, id: u32, position: Vector2<f32>, gestures: &mut Vec<GestureType>) {
        let touch = TouchPoint::new(id, position);
        self.active_touches.insert(id, touch);
        
        // Start long press timer for single touch
        if self.active_touches.len() == 1 {
            self.long_press_timer = Some((id, Instant::now()));
        }
        
        // Check for multi-touch gestures
        if self.active_touches.len() == 2 {
            self.init_multi_touch_gesture();
        }
    }
    
    /// Handle touch move event
    fn handle_touch_move(&mut self, position: Vector2<f32>, gestures: &mut Vec<GestureType>) {
        // Get the first touch ID and check count
        let first_id = self.active_touches.keys().next().copied();
        let touch_count = self.active_touches.len();
        
        // Update the primary touch
        if let Some(id) = first_id {
            if let Some(touch) = self.active_touches.get_mut(&id) {
                let old_pos = touch.current_position;
                touch.update_position(position);
                
                // Cancel long press if moved too much
                if touch.total_distance > self.config.long_press_tolerance {
                    self.long_press_timer = None;
                }
                
                // Emit pan gesture
                if touch.is_moving && touch_count == 1 {
                    gestures.push(GestureType::Pan {
                        position: touch.current_position,
                        delta: touch.current_position - old_pos,
                        velocity: touch.velocity,
                        touch_id: id,
                    });
                }
            }
        }
        
        // Handle multi-touch gestures
        if touch_count == 2 {
            self.update_multi_touch_gesture(gestures);
        }
    }
    
    /// Handle touch up event
    fn handle_touch_up(&mut self, id: u32, position: Vector2<f32>, gestures: &mut Vec<GestureType>) {
        if let Some(mut touch) = self.active_touches.remove(&id) {
            touch.update_position(position);
            let duration = touch.get_duration();
            
            // Cancel long press
            if let Some((timer_id, _)) = self.long_press_timer {
                if timer_id == id {
                    self.long_press_timer = None;
                }
            }
            
            // Check for tap or swipe
            if duration.as_millis() < 300 && touch.total_distance < 10.0 {
                // Check for double tap
                let now = Instant::now();
                if let Some((last_pos, last_time)) = self.last_tap {
                    let time_diff = (now - last_time).as_millis();
                    let distance = (position - last_pos).magnitude();
                    
                    if time_diff < self.config.double_tap_time as u128 
                        && distance < self.config.double_tap_distance {
                        gestures.push(GestureType::DoubleTap {
                            position,
                            touch_id: id,
                        });
                        self.last_tap = None;
                    } else {
                        gestures.push(GestureType::Tap {
                            position,
                            touch_id: id,
                        });
                        self.last_tap = Some((position, now));
                    }
                } else {
                    gestures.push(GestureType::Tap {
                        position,
                        touch_id: id,
                    });
                    self.last_tap = Some((position, now));
                }
            } else if touch.total_distance >= self.config.swipe_min_distance {
                let velocity_magnitude = touch.velocity.magnitude();
                let direction = self.get_swipe_direction(&touch);
                
                if velocity_magnitude >= self.config.fling_min_velocity {
                    gestures.push(GestureType::Fling {
                        position: touch.current_position,
                        velocity: touch.velocity,
                        direction,
                        touch_id: id,
                    });
                } else if velocity_magnitude >= self.config.swipe_min_velocity {
                    gestures.push(GestureType::Swipe {
                        start: touch.start_position,
                        end: touch.current_position,
                        velocity: touch.velocity,
                        direction,
                        touch_id: id,
                    });
                }
            }
        }
        
        // Reset multi-touch state if no longer have 2 touches
        if self.active_touches.len() < 2 {
            self.pinch_initial_distance = None;
            self.rotate_initial_angle = None;
            self.gesture_in_progress = None;
        }
    }
    
    /// Initialize multi-touch gesture tracking
    fn init_multi_touch_gesture(&mut self) {
        if self.active_touches.len() == 2 {
            let points: Vec<_> = self.active_touches.values().collect();
            let distance = (points[0].current_position - points[1].current_position).magnitude();
            let angle = (points[1].current_position - points[0].current_position).y
                .atan2((points[1].current_position - points[0].current_position).x);
            
            self.pinch_initial_distance = Some(distance);
            self.rotate_initial_angle = Some(angle);
        }
    }
    
    /// Update multi-touch gesture
    fn update_multi_touch_gesture(&mut self, gestures: &mut Vec<GestureType>) {
        if self.active_touches.len() != 2 {
            return;
        }
        
        let points: Vec<_> = self.active_touches.values().collect();
        let center = (points[0].current_position + points[1].current_position) * 0.5;
        let current_distance = (points[0].current_position - points[1].current_position).magnitude();
        let current_angle = (points[1].current_position - points[0].current_position).y
            .atan2((points[1].current_position - points[0].current_position).x);
        
        // Check for pinch
        if let Some(initial_distance) = self.pinch_initial_distance {
            let scale = current_distance / initial_distance;
            if (scale - 1.0).abs() > self.config.pinch_threshold {
                let velocity = if points[0].velocity.magnitude() > 0.0 {
                    (current_distance - initial_distance) / 
                    points[0].get_duration().as_secs_f32()
                } else {
                    0.0
                };
                
                gestures.push(GestureType::Pinch {
                    center,
                    scale,
                    velocity,
                });
            }
        }
        
        // Check for rotation
        if let Some(initial_angle) = self.rotate_initial_angle {
            let angle_diff = current_angle - initial_angle;
            if angle_diff.abs() > self.config.rotate_threshold {
                let velocity = if points[0].velocity.magnitude() > 0.0 {
                    angle_diff / points[0].get_duration().as_secs_f32()
                } else {
                    0.0
                };
                
                gestures.push(GestureType::Rotate {
                    center,
                    angle: angle_diff,
                    velocity,
                });
            }
        }
    }
    
    /// Get swipe direction from touch point
    fn get_swipe_direction(&self, touch: &TouchPoint) -> SwipeDirection {
        let delta = touch.current_position - touch.start_position;
        
        if delta.x.abs() > delta.y.abs() {
            if delta.x > 0.0 {
                SwipeDirection::Right
            } else {
                SwipeDirection::Left
            }
        } else {
            if delta.y > 0.0 {
                SwipeDirection::Down
            } else {
                SwipeDirection::Up
            }
        }
    }
    
    /// Reset all gesture state
    pub fn reset(&mut self) {
        self.active_touches.clear();
        self.last_tap = None;
        self.long_press_timer = None;
        self.gesture_in_progress = None;
        self.pinch_initial_distance = None;
        self.rotate_initial_angle = None;
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: GestureConfig) {
        self.config = config;
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> &GestureConfig {
        &self.config
    }
    
    /// Check if a gesture is in progress
    pub fn is_gesture_active(&self) -> bool {
        !self.active_touches.is_empty() || self.gesture_in_progress.is_some()
    }
    
    /// Get active touch count
    pub fn active_touch_count(&self) -> usize {
        self.active_touches.len()
    }
}