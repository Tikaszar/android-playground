//! Module Loader with THE Single Unsafe Block

use crate::loaded_module::LoadedModule;
use playground_modules_types::{
    Handle, Module, ModuleError, ModuleMetadata, ModuleResult, ModuleType, Shared, ViewAPI,
    ViewModelImpl,
};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Module loader responsible for loading and managing modules
pub struct ModuleLoader {
    /// Loaded modules indexed by name
    modules: Shared<HashMap<String, LoadedModule>>,

    /// Module search paths
    search_paths: Vec<PathBuf>,
}

impl ModuleLoader {
    /// Create a new module loader
    pub fn new() -> Self {
        Self {
            modules: Handle::new(RwLock::new(HashMap::new())),
            search_paths: vec![
                PathBuf::from("target/debug"),
                PathBuf::from("target/release"),
                PathBuf::from("modules"),
            ],
        }
    }

    /// Add a search path for modules
    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) {
        self.search_paths.push(path.into());
    }

    /// Load a module from a file path
    ///
    /// This function contains THE ONLY unsafe block in the entire codebase.
    /// All unsafe operations (Library::new, symbol loading, dereferencing)
    /// happen in a single, well-documented block.
    pub async fn load_module(&self, name: &str) -> ModuleResult<()> {
        info!("Loading module: {}", name);

        // Find the module file
        let module_path = self.find_module_file(name)?;

        // Check if already loaded
        {
            let modules = self.modules.read().await;
            if modules.contains_key(name) {
                return Err(ModuleError::AlreadyLoaded(name.to_string()));
            }
        }

        // ================================================================
        // THE ONLY UNSAFE BLOCK IN THE ENTIRE CODEBASE
        // ================================================================
        // This block handles all unsafe operations required for loading
        // a dynamic library and extracting module symbols.
        // ================================================================
        let loaded_module = unsafe {
            // 1. Load the dynamic library
            let library = Library::new(&module_path).map_err(|e| {
                ModuleError::LoadFailed(format!(
                    "Failed to load library {}: {}",
                    module_path.display(),
                    e
                ))
            })?;

            // 2. Get the module symbol
            let module_symbol: Symbol<*const Module> =
                library.get(b"PLAYGROUND_MODULE\0").map_err(|e| {
                    ModuleError::LoadFailed(format!(
                        "Failed to find PLAYGROUND_MODULE symbol: {}",
                        e
                    ))
                })?;
            let module = &**module_symbol;

            // 3. Clone metadata (to avoid lifetime issues)
            let metadata = ModuleMetadata {
                name: module.metadata.name,
                version: module.metadata.version,
                description: module.metadata.description,
                features: module.metadata.features,
                dependencies: module.metadata.dependencies,
            };

            // 4. Get View API for Core modules
            let view_api = if module.module_type == ModuleType::Core {
                let view_symbol: Symbol<*const ViewAPI> =
                    library.get(b"PLAYGROUND_VIEW_API\0").ok().map(|s| &**s).cloned()
            } else {
                None
            };

            // 5. Get ViewModel implementation for System modules
            let viewmodel_impl = if module.module_type == ModuleType::System {
                let vm_symbol: Symbol<*const ViewModelImpl> = library
                    .get(b"PLAYGROUND_VIEWMODEL_IMPL\0")
                    .ok()
                    .map(|s| &**s)
                    .cloned()
            } else {
                None
            };

            // 6. Initialize the module
            (module.lifecycle.initialize)(&[]).map_err(|e| {
                ModuleError::LoadFailed(format!("Failed to initialize module {}: {}", name, e))
            })?;

            // Create the loaded module struct
            LoadedModule {
                _library: library,
                metadata,
                module_type: module.module_type,
                path: module_path.clone(),
                view_api,
                viewmodel_impl,
            }
        };
        // ================================================================
        // END OF UNSAFE BLOCK
        // ================================================================

        // Store the loaded module
        let mut modules = self.modules.write().await;
        modules.insert(name.to_string(), loaded_module);

        info!("Successfully loaded module: {}", name);
        Ok(())
    }

    /// Unload a module
    pub async fn unload_module(&self, name: &str) -> ModuleResult<()> {
        info!("Unloading module: {}", name);

        let mut modules = self.modules.write().await;
        let module = modules
            .remove(name)
            .ok_or_else(|| ModuleError::NotFound(name.to_string()))?;

        // Module's destructor will run when dropped
        drop(module);

        info!("Successfully unloaded module: {}", name);
        Ok(())
    }

    /// Hot-reload a module
    pub async fn hot_reload(&self, name: &str) -> ModuleResult<()> {
        info!("Hot-reloading module: {}", name);

        // Get the module path
        let module_path = {
            let modules = self.modules.read().await;
            let module = modules
                .get(name)
                .ok_or_else(|| ModuleError::NotFound(name.to_string()))?;
            module.path.clone()
        };

        // Unload the old version
        self.unload_module(name).await?;

        // Load the new version
        self.load_module(name).await?;

        info!("Successfully hot-reloaded module: {}", name);
        Ok(())
    }

    /// List all loaded modules
    pub async fn list_modules(&self) -> Vec<String> {
        let modules = self.modules.read().await;
        modules.keys().cloned().collect()
    }

    /// Get module metadata
    pub async fn get_module_info(&self, name: &str) -> ModuleResult<ModuleMetadata> {
        let modules = self.modules.read().await;
        let module = modules
            .get(name)
            .ok_or_else(|| ModuleError::NotFound(name.to_string()))?;
        Ok(module.metadata.clone())
    }

    /// Find a module file in search paths
    fn find_module_file(&self, name: &str) -> ModuleResult<PathBuf> {
        // Try with platform-specific extension
        let filename = format!("lib{}.{}", name, std::env::consts::DLL_EXTENSION);

        // Search in all paths
        for search_path in &self.search_paths {
            let full_path = search_path.join(&filename);
            if full_path.exists() {
                return Ok(full_path);
            }

            // Also try without lib prefix
            let alt_filename = format!("{}.{}", name, std::env::consts::DLL_EXTENSION);
            let alt_path = search_path.join(&alt_filename);
            if alt_path.exists() {
                return Ok(alt_path);
            }
        }

        Err(ModuleError::NotFound(format!(
            "Module file not found for: {}",
            name
        )))
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}