//! Console system implementation - ALL the logic for console operations
//! 
//! This implements the actual console functionality that core/console delegates to.

// Module declarations
pub mod registration;
pub mod vtable_handlers;
pub mod terminal;
pub mod dashboard;
pub mod file_logger;

// Re-exports - NO implementation here, just exports!
pub use registration::register;
pub use terminal::Terminal;
pub use dashboard::Dashboard;
pub use file_logger::FileLogger;