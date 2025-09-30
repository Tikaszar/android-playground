//! THE ONLY UNSAFE IN THE ENTIRE CODEBASE
//!
//! This module contains the single unsafe exception for Library::new()
//! as documented in CLAUDE.md

use libloading::Library;
use std::path::Path;

/// Load a dynamic library
///
/// This contains THE ONLY UNSAFE in the entire codebase.
/// It's the documented exception in CLAUDE.md for Library::new() only.
pub fn load_library_unsafe(path: &Path) -> Result<Library, String> {
    // THE ONLY UNSAFE - DOCUMENTED EXCEPTION IN CLAUDE.md
    unsafe {
        Library::new(path)
            .map_err(|e| format!("Failed to load library: {}", e))
    }
}