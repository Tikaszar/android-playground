use serde::{Serialize, Deserialize};
use std::time::{Instant, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureEvent {
    Tap { x: f32, y: f32 },
    DoubleTap { x: f32, y: f32 },
    LongPress { x: f32, y: f32 },
    Pan { delta_x: f32, delta_y: f32 },
    Pinch { scale: f32, center_x: f32, center_y: f32 },
    Rotate { angle: f32, center_x: f32, center_y: f32 },
    Swipe { direction: SwipeDirection, velocity: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

pub struct GestureRecognizer {
    last_tap_time: Option<Instant>,
    last_tap_position: Option<[f32; 2]>,
    press_start_time: Option<Instant>,
    press_start_position: Option<[f32; 2]>,
    double_tap_threshold: Duration,
    long_press_threshold: Duration,
    tap_distance_threshold: f32,
}

impl GestureRecognizer {
    pub fn new() -> Self {
        Self {
            last_tap_time: None,
            last_tap_position: None,
            press_start_time: None,
            press_start_position: None,
            double_tap_threshold: Duration::from_millis(300),
            long_press_threshold: Duration::from_millis(500),
            tap_distance_threshold: 10.0,
        }
    }
    
    pub fn on_touch_start(&mut self, x: f32, y: f32) {
        self.press_start_time = Some(Instant::now());
        self.press_start_position = Some([x, y]);
    }
    
    pub fn on_touch_end(&mut self, x: f32, y: f32) -> Option<GestureEvent> {
        let Some(start_time) = self.press_start_time else {
            return None;
        };
        
        let Some(start_pos) = self.press_start_position else {
            return None;
        };
        
        let elapsed = start_time.elapsed();
        let distance = ((x - start_pos[0]).powi(2) + (y - start_pos[1]).powi(2)).sqrt();
        
        // Check if it's a tap (short duration and small movement)
        if elapsed < self.long_press_threshold && distance < self.tap_distance_threshold {
            // Check for double tap
            if let (Some(last_time), Some(last_pos)) = (self.last_tap_time, self.last_tap_position) {
                let tap_interval = Instant::now().duration_since(last_time);
                let tap_distance = ((x - last_pos[0]).powi(2) + (y - last_pos[1]).powi(2)).sqrt();
                
                if tap_interval < self.double_tap_threshold && tap_distance < self.tap_distance_threshold {
                    self.last_tap_time = None;
                    self.last_tap_position = None;
                    return Some(GestureEvent::DoubleTap { x, y });
                }
            }
            
            // Single tap
            self.last_tap_time = Some(Instant::now());
            self.last_tap_position = Some([x, y]);
            return Some(GestureEvent::Tap { x, y });
        }
        
        // Check for long press
        if elapsed >= self.long_press_threshold && distance < self.tap_distance_threshold {
            return Some(GestureEvent::LongPress { x, y });
        }
        
        // Reset state
        self.press_start_time = None;
        self.press_start_position = None;
        
        None
    }
    
    pub fn on_touch_move(&mut self, x: f32, y: f32, prev_x: f32, prev_y: f32) -> Option<GestureEvent> {
        let delta_x = x - prev_x;
        let delta_y = y - prev_y;
        
        // Simple pan gesture
        if delta_x.abs() > 0.1 || delta_y.abs() > 0.1 {
            return Some(GestureEvent::Pan { delta_x, delta_y });
        }
        
        None
    }
    
    pub fn recognize_swipe(&self, start_x: f32, start_y: f32, end_x: f32, end_y: f32, duration: Duration) -> Option<GestureEvent> {
        let delta_x = end_x - start_x;
        let delta_y = end_y - start_y;
        let distance = (delta_x.powi(2) + delta_y.powi(2)).sqrt();
        
        // Calculate velocity
        let velocity = distance / duration.as_secs_f32();
        
        // Minimum velocity for swipe detection
        if velocity < 100.0 {
            return None;
        }
        
        // Determine direction
        let direction = if delta_x.abs() > delta_y.abs() {
            if delta_x > 0.0 {
                SwipeDirection::Right
            } else {
                SwipeDirection::Left
            }
        } else {
            if delta_y > 0.0 {
                SwipeDirection::Down
            } else {
                SwipeDirection::Up
            }
        };
        
        Some(GestureEvent::Swipe { direction, velocity })
    }
}