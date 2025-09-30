//! Helper macros for module definition

/// Helper macro for defining a module
#[macro_export]
macro_rules! define_module {
    (
        name: $name:expr,
        version: $version:expr,
        type: $mod_type:expr,
        dependencies: [$($dep:expr),* $(,)?],
        features: [$($feat:expr),* $(,)?],
        vtable: $vtable:ident
    ) => {
        #[no_mangle]
        pub static PLAYGROUND_MODULE: $crate::Module = $crate::Module {
            metadata: &MODULE_METADATA,
            vtable: &$vtable,
        };

        static MODULE_METADATA: $crate::ModuleMetadata = $crate::ModuleMetadata {
            name: $name,
            version: $version,
            module_type: $mod_type,
            dependencies: &[$($dep),*],
            features: &[$($feat),*],
        };
    };
}