//! Console data structure - NO LOGIC, just data fields!
//! 
//! This is like an abstract base class - defines structure only.
//! All actual implementation logic is in systems/console.

use std::collections::HashMap;
use playground_core_types::{Handle, handle, Shared, shared};
use playground_core_ecs::VTable;

/// The concrete Console struct - data fields only, no logic!
/// 
/// Like an abstract base class in OOP - structure but no behavior.
/// All actual console operations are implemented in systems/console.
pub struct Console {
    /// The VTable for system dispatch
    pub vtable: VTable,
    
    /// Console configuration data
    #[cfg(feature = "output")]
    pub output_buffer: Shared<Vec<String>>,
    
    /// Log entries storage
    #[cfg(feature = "logging")]
    pub log_entries: Shared<Vec<crate::LogEntry>>,
    
    /// Component-specific log storage
    #[cfg(feature = "logging")]
    pub component_logs: Shared<HashMap<String, Vec<crate::LogEntry>>>,
    
    /// Current log level
    #[cfg(feature = "logging")]
    pub log_level: Shared<crate::LogLevel>,
    
    /// Progress indicators
    #[cfg(feature = "progress")]
    pub progress_indicators: Shared<HashMap<String, crate::Progress>>,
    
    /// Input buffer
    #[cfg(feature = "input")]
    pub input_buffer: Shared<Vec<String>>,
    
    /// Console capabilities
    pub capabilities: crate::ConsoleCapabilities,
}

impl Console {
    /// Create a new Console instance - just data initialization, no logic!
    pub fn new() -> Handle<Self> {
        handle(Self {
            vtable: VTable::new(),
            
            #[cfg(feature = "output")]
            output_buffer: shared(Vec::new()),
            
            #[cfg(feature = "logging")]
            log_entries: shared(Vec::new()),
            
            #[cfg(feature = "logging")]
            component_logs: shared(HashMap::new()),
            
            #[cfg(feature = "logging")]
            log_level: shared(crate::LogLevel::Info),
            
            #[cfg(feature = "progress")]
            progress_indicators: shared(HashMap::new()),
            
            #[cfg(feature = "input")]
            input_buffer: shared(Vec::new()),
            
            capabilities: crate::ConsoleCapabilities::default(),
        })
    }
}