//! Generic console and logging contracts
//! 
//! This module defines abstract contracts for console output and logging
//! that can be implemented by any backend (terminal, file, GUI, network, etc.)

mod contracts;
mod types;
mod commands;

pub use contracts::*;
pub use types::*;
pub use commands::*;