pub mod event;
pub mod manager;
pub mod gestures;

pub use event::{InputEvent, MouseButton, Key, Modifiers};
pub use manager::InputManager;
pub use gestures::{GestureRecognizer, GestureEvent};