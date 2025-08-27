// Export all modules
mod core;
mod init;
mod elements;
mod rendering;
mod shaders;
mod ui_renderer_impl;
mod layout;

// Re-export the main type
pub use core::UiSystem;