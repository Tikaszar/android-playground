//! No-op vtable for data-only core modules

use crate::ModuleVTable;

/// Helper for creating a no-op vtable for data-only core modules
pub const NOOP_VTABLE: ModuleVTable = ModuleVTable {
    create: || std::ptr::null_mut(),
    destroy: |_| {},
    initialize: |_, _| Ok(()),
    shutdown: |_| Ok(()),
    call: |_, _, _| Err("Core modules are data-only, no operations".to_string()),
    save_state: |_| vec![],
    restore_state: |_, _| Ok(()),
    get_capabilities: || vec![],
};