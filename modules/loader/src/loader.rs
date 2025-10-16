//! Module Loader with THE Single Unsafe Block

use crate::loaded_module::LoadedModule;
use playground_modules_types::{
    Handle, ModelTypeInfo, Module, ModuleError, ModuleMetadata, ModuleResult, ModuleType,
    Shared, ViewModelTrait, ViewTrait,
};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::info;

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

            // 4. Get View trait object for Core modules
            let view = if module.module_type == ModuleType::Core {
                let view_symbol: Symbol<&'static Handle<dyn ViewTrait>> = library
                    .get(b"PLAYGROUND_VIEW\0")
                    .map_err(|e| {
                        ModuleError::LoadFailed(format!("Failed to find PLAYGROUND_VIEW symbol: {}", e))
                    })?;
                Some((*view_symbol).clone())
            } else {
                None
            };

            // 5. Get Model type info for Core modules
            let models = if module.module_type == ModuleType::Core {
                let models_symbol: Symbol<*const &'static [ModelTypeInfo]> = library
                    .get(b"PLAYGROUND_MODELS\0")
                    .map_err(|e| {
                        ModuleError::LoadFailed(format!("Failed to find PLAYGROUND_MODELS symbol: {}", e))
                    })?;
                Some(**models_symbol)
            } else {
                None
            };

            // 6. Get ViewModel trait object for System modules
            let viewmodel = if module.module_type == ModuleType::System {
                let vm_symbol: Symbol<&'static Handle<dyn ViewModelTrait>> = library
                    .get(b"PLAYGROUND_VIEWMODEL\0")
                    .map_err(|e| {
                        ModuleError::LoadFailed(format!("Failed to find PLAYGROUND_VIEWMODEL symbol: {}", e))
                    })?;
                Some((*vm_symbol).clone())
            } else {
                None
            };

            // 7. Initialize the module
            (module.lifecycle.initialize)(&[]).map_err(|e| {
                ModuleError::LoadFailed(format!("Failed to initialize module {}: {}", name, e))
            })?;

            // Create the loaded module struct
            LoadedModule {
                _library: library,
                metadata,
                module_type: module.module_type,
                path: module_path.clone(),
                view,
                models,
                viewmodel,
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

    /// Hot-reload a module with optional state preservation
    pub async fn hot_reload(&self, name: &str) -> ModuleResult<()> {
        info!("Hot-reloading module: {}", name);

        let saved_state = {
            let modules = self.modules.read().await;
            let module = modules
                .get(name)
                .ok_or_else(|| ModuleError::NotFound(name.to_string()))?;

            if let Some(ref viewmodel) = module.viewmodel {
                viewmodel.save_state().await.transpose()?
            } else {
                None
            }
        };

        self.unload_module(name).await?;
        self.load_module(name).await?;

        if let Some(state_bytes) = saved_state {
            let modules = self.modules.read().await;
            if let Some(module) = modules.get(name) {
                if let Some(ref viewmodel) = module.viewmodel {
                    if let Some(result) = viewmodel.restore_state(state_bytes).await {
                        result?;
                    }
                }
            }
        }

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

    /// Get loaded module (for accessing View/ViewModel/Models)
    pub async fn get_module(&self, name: &str) -> ModuleResult<LoadedModule> {
        let modules = self.modules.read().await;
        let _module = modules
            .get(name)
            .ok_or_else(|| ModuleError::NotFound(name.to_string()))?;

        // Clone the module data (Library can't be cloned, so create minimal copy)
        // This is a placeholder - actual implementation would need to handle this better
        Err(ModuleError::Generic(
            "get_module not yet implemented - use direct registry access".to_string()
        ))
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
