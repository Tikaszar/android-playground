//! Module Loader with Single Unsafe Exception
//!
//! This is the ONLY place in the entire codebase with unsafe code.
//! The single unsafe exception is for Library::new() to load dynamic libraries.
//! Everything else is wrapped in safe abstractions.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use libloading::Library;
use playground_api::*;
use playground_core_types::{CoreResult, CoreError};
use semver::{Version, VersionReq};
use notify::{Watcher, RecursiveMode, Result as NotifyResult, Event};
use tracing::{info, warn, error, debug};

mod dependency;
mod registry;
mod watcher;
mod unsafe_loader;

pub use dependency::DependencyGraph;
pub use registry::ModuleRegistry;
pub use watcher::ModuleWatcher;

/// Wrapper for module pointer to make it Send+Sync
/// This is safe because Module is 'static and immutable
struct ModulePtr(*const Module);

/// Wrapper for state pointer to make it Send+Sync
/// This is safe because we ensure exclusive access through ModuleLoader's locks
struct StatePtr(*mut u8);

/// A loaded module with its library and state
pub struct LoadedModule {
    /// The dynamic library handle
    library: Library,
    /// Module interface (wrapped for Send+Sync)
    module: ModulePtr,
    /// Module state pointer (wrapped for Send+Sync)
    state: StatePtr,
    /// Module name
    name: String,
    /// Module version
    version: Version,
    /// Modules that depend on this one
    dependents: Vec<String>,
    /// Current module state
    lifecycle_state: ModuleState,
}

impl LoadedModule {
    /// Get the module interface
    fn module(&self) -> &'static Module {
        unsafe { &*self.module.0 }
    }

    /// Get the state pointer
    fn state(&self) -> *mut u8 {
        self.state.0
    }
}

/// Module loader responsible for loading, unloading, and hot-reloading modules
pub struct ModuleLoader {
    /// Loaded modules indexed by name
    modules: Arc<RwLock<HashMap<String, LoadedModule>>>,
    /// Dependency graph for module resolution
    dependency_graph: Arc<RwLock<DependencyGraph>>,
    /// Module registry for inter-module communication
    registry: Arc<ModuleRegistry>,
    /// File watcher for hot-reload
    watcher: Option<ModuleWatcher>,
    /// Module search paths
    search_paths: Vec<PathBuf>,
}

impl ModuleLoader {
    /// Create a new module loader
    pub fn new() -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: Arc::new(RwLock::new(DependencyGraph::new())),
            registry: Arc::new(ModuleRegistry::new()),
            watcher: None,
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

    /// Enable hot-reload by watching module files
    pub async fn enable_hot_reload(&mut self) -> CoreResult<()> {
        let loader = self.clone_for_watcher();
        let watcher = ModuleWatcher::new(loader)?;
        self.watcher = Some(watcher);
        Ok(())
    }

    /// Load a module from a file path
    pub async fn load_module(&self, path: &Path) -> CoreResult<()> {
        info!("Loading module from: {}", path.display());

        // Find the module file
        let module_path = self.find_module_file(path)?;

        // Call the SINGLE UNSAFE function in the entire codebase
        let (library, module) = unsafe_loader::load_module_unsafe(&module_path)
            .map_err(|e| CoreError::ModuleLoadFailed(format!(
                "Failed to load module {}: {}",
                module_path.display(),
                e
            )))?;

        // Validate module metadata
        self.validate_module(module)?;

        // Check dependencies
        self.check_dependencies(module).await?;

        // Create module state
        let state = (module.vtable.create)();

        // Initialize module
        (module.vtable.initialize)(state, &[])
            .map_err(|e| CoreError::ModuleLoadFailed(format!(
                "Failed to initialize module {}: {}",
                module.metadata.name,
                e
            )))?;

        // Parse version
        let version = Version::parse(module.metadata.version)
            .map_err(|e| CoreError::ModuleLoadFailed(format!(
                "Invalid version {} for module {}: {}",
                module.metadata.version,
                module.metadata.name,
                e
            )))?;

        // Add to dependency graph
        let mut graph = self.dependency_graph.write().await;
        graph.add_module(module.metadata.name, &module.metadata.dependencies)?;

        // Create loaded module
        let loaded = LoadedModule {
            library,
            module: ModulePtr(module as *const Module),
            state: StatePtr(state),
            name: module.metadata.name.to_string(),
            version,
            dependents: Vec::new(),
            lifecycle_state: ModuleState::Ready,
        };

        // Register module
        self.registry.register_module(&loaded.name, loaded.module()).await?;

        // Store module
        let mut modules = self.modules.write().await;
        if modules.contains_key(&loaded.name) {
            return Err(CoreError::ModuleLoadFailed(format!(
                "Module {} already loaded",
                loaded.name
            )));
        }
        modules.insert(loaded.name.clone(), loaded);

        info!("Successfully loaded module: {}", module.metadata.name);
        Ok(())
    }

    /// Hot-reload a module
    pub async fn hot_reload(&self, name: &str) -> CoreResult<()> {
        info!("Hot-reloading module: {}", name);

        // Save current state
        let saved_state = self.save_module_state(name).await?;

        // Find dependents
        let dependents = {
            let graph = self.dependency_graph.read().await;
            graph.get_dependents(name)?
        };

        // Save dependent states
        let mut dependent_states = HashMap::new();
        for dep in &dependents {
            dependent_states.insert(
                dep.clone(),
                self.save_module_state(dep).await?
            );
        }

        // Unload dependents
        for dep in dependents.iter().rev() {
            self.unload_module(dep).await?;
        }

        // Unload the module
        let module_path = {
            let modules = self.modules.read().await;
            let module = modules.get(name)
                .ok_or_else(|| CoreError::ModuleNotFound(name.to_string()))?;
            self.find_module_file(Path::new(name))?
        };
        self.unload_module(name).await?;

        // Reload the module
        self.load_module(&module_path).await?;

        // Restore module state
        self.restore_module_state(name, saved_state).await?;

        // Reload dependents
        for dep in &dependents {
            let dep_path = self.find_module_file(Path::new(dep))?;
            self.load_module(&dep_path).await?;

            // Restore dependent state
            if let Some(state) = dependent_states.get(dep) {
                self.restore_module_state(dep, state.clone()).await?;
            }
        }

        info!("Successfully hot-reloaded module: {}", name);
        Ok(())
    }

    /// Unload a module
    pub async fn unload_module(&self, name: &str) -> CoreResult<()> {
        info!("Unloading module: {}", name);

        let mut modules = self.modules.write().await;
        let mut module = modules.remove(name)
            .ok_or_else(|| CoreError::ModuleNotFound(name.to_string()))?;

        // Shutdown module
        (module.module().vtable.shutdown)(module.state())
            .map_err(|e| CoreError::Generic(format!(
                "Failed to shutdown module {}: {}",
                name,
                e
            )))?;

        // Destroy module state
        (module.module().vtable.destroy)(module.state());

        // Unregister from registry
        self.registry.unregister_module(name).await?;

        // Remove from dependency graph
        let mut graph = self.dependency_graph.write().await;
        graph.remove_module(name)?;

        // The Library will be dropped here, unloading the dynamic library
        info!("Successfully unloaded module: {}", name);
        Ok(())
    }

    /// Save module state for hot-reload
    async fn save_module_state(&self, name: &str) -> CoreResult<Vec<u8>> {
        let modules = self.modules.read().await;
        let module = modules.get(name)
            .ok_or_else(|| CoreError::ModuleNotFound(name.to_string()))?;

        let state = (module.module().vtable.save_state)(module.state());
        debug!("Saved {} bytes of state for module {}", state.len(), name);
        Ok(state)
    }

    /// Restore module state after hot-reload
    async fn restore_module_state(&self, name: &str, state: Vec<u8>) -> CoreResult<()> {
        let modules = self.modules.read().await;
        let module = modules.get(name)
            .ok_or_else(|| CoreError::ModuleNotFound(name.to_string()))?;

        (module.module().vtable.restore_state)(module.state(), &state)
            .map_err(|e| CoreError::Generic(format!(
                "Failed to restore state for module {}: {}",
                name,
                e
            )))?;

        debug!("Restored {} bytes of state for module {}", state.len(), name);
        Ok(())
    }

    /// Find a module file in search paths
    fn find_module_file(&self, path: &Path) -> CoreResult<PathBuf> {
        // Try exact path first
        if path.exists() {
            return Ok(path.to_path_buf());
        }

        // Try with platform-specific extension
        let with_ext = path.with_extension(std::env::consts::DLL_EXTENSION);
        if with_ext.exists() {
            return Ok(with_ext);
        }

        // Search in search paths
        let file_name = path.file_name()
            .ok_or_else(|| CoreError::ModuleLoadFailed(format!(
                "Invalid module path: {}",
                path.display()
            )))?;

        for search_path in &self.search_paths {
            let full_path = search_path.join(file_name);
            if full_path.exists() {
                return Ok(full_path);
            }

            let with_ext = full_path.with_extension(std::env::consts::DLL_EXTENSION);
            if with_ext.exists() {
                return Ok(with_ext);
            }

            // Try with lib prefix (Unix convention)
            let with_prefix = search_path.join(format!("lib{}", file_name.to_string_lossy()));
            if with_prefix.exists() {
                return Ok(with_prefix);
            }

            let with_both = with_prefix.with_extension(std::env::consts::DLL_EXTENSION);
            if with_both.exists() {
                return Ok(with_both);
            }
        }

        Err(CoreError::ModuleLoadFailed(format!(
            "Module file not found: {}",
            path.display()
        )))
    }

    /// Validate module metadata
    fn validate_module(&self, module: &Module) -> CoreResult<()> {
        if module.metadata.name.is_empty() {
            return Err(CoreError::ModuleLoadFailed(
                "Module name cannot be empty".to_string()
            ));
        }

        if module.metadata.version.is_empty() {
            return Err(CoreError::ModuleLoadFailed(
                "Module version cannot be empty".to_string()
            ));
        }

        // Validate version format
        Version::parse(module.metadata.version)
            .map_err(|e| CoreError::ModuleLoadFailed(format!(
                "Invalid version {} for module {}: {}",
                module.metadata.version,
                module.metadata.name,
                e
            )))?;

        Ok(())
    }

    /// Check module dependencies
    async fn check_dependencies(&self, module: &Module) -> CoreResult<()> {
        let modules = self.modules.read().await;

        for dep in module.metadata.dependencies {
            // Check if dependency is loaded
            let loaded = modules.get(dep.name)
                .ok_or_else(|| CoreError::ModuleLoadFailed(format!(
                    "Dependency {} not loaded for module {}",
                    dep.name,
                    module.metadata.name
                )))?;

            // Check version requirement
            let version_req = VersionReq::parse(dep.version_req)
                .map_err(|e| CoreError::ModuleLoadFailed(format!(
                    "Invalid version requirement {} for dependency {}: {}",
                    dep.version_req,
                    dep.name,
                    e
                )))?;

            if !version_req.matches(&loaded.version) {
                return Err(CoreError::ModuleLoadFailed(format!(
                    "Version mismatch for dependency {}: required {}, found {}",
                    dep.name,
                    dep.version_req,
                    loaded.version
                )));
            }

            // Check required features
            for feature in dep.features {
                if !loaded.module.metadata.features.contains(feature) {
                    return Err(CoreError::ModuleLoadFailed(format!(
                        "Required feature {} not found in dependency {}",
                        feature,
                        dep.name
                    )));
                }
            }
        }

        Ok(())
    }

    /// List all loaded modules
    pub async fn list_modules(&self) -> Vec<String> {
        let modules = self.modules.read().await;
        modules.keys().cloned().collect()
    }

    /// Get module info
    pub async fn get_module_info(&self, name: &str) -> CoreResult<ModuleInfo> {
        let modules = self.modules.read().await;
        let module = modules.get(name)
            .ok_or_else(|| CoreError::ModuleNotFound(name.to_string()))?;

        Ok(ModuleInfo {
            name: module.name.clone(),
            version: module.version.to_string(),
            module_type: module.module().metadata.module_type,
            features: module.module().metadata.features.iter().map(|s| s.to_string()).collect(),
            dependents: module.dependents.clone(),
            state: module.lifecycle_state,
        })
    }

    /// Call a method on a module
    pub async fn call_module(
        &self,
        module_name: &str,
        method: &str,
        args: &[u8]
    ) -> CoreResult<Vec<u8>> {
        self.registry.call(module_name, method, args).await
    }

    /// Clone for use in watcher thread
    fn clone_for_watcher(&self) -> Arc<Self> {
        Arc::new(Self {
            modules: self.modules.clone(),
            dependency_graph: self.dependency_graph.clone(),
            registry: self.registry.clone(),
            watcher: None,
            search_paths: self.search_paths.clone(),
        })
    }
}

/// Information about a loaded module
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub version: String,
    pub module_type: ModuleType,
    pub features: Vec<String>,
    pub dependents: Vec<String>,
    pub state: ModuleState,
}

// LoadedModule is now automatically Send + Sync because all its fields are

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_module_loader_creation() {
        let loader = ModuleLoader::new();
        assert_eq!(loader.list_modules().await.len(), 0);
    }
}