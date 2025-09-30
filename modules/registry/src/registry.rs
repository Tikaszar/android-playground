//! The main module registry that orchestrates everything

use crate::info::{ModuleInfo, ModuleState};
use crate::stats::RegistryStats;
use playground_modules_binding::BindingRegistry;
use playground_modules_loader::ModuleLoader;
use playground_modules_resolver::{AppModuleConfig, ModuleResolver};
use playground_modules_types::{Handle, ModuleError, ModuleResult, ModuleType, Shared};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// The main module registry that orchestrates everything
pub struct ModuleRegistry {
    /// Module loader (contains THE unsafe block)
    loader: Handle<ModuleLoader>,

    /// View-ViewModel binding registry
    binding: Handle<BindingRegistry>,

    /// Module dependency resolver
    resolver: Handle<RwLock<ModuleResolver>>,

    /// Registered modules and their state
    modules: Shared<HashMap<String, ModuleInfo>>,

    /// File watcher for hot-reload
    watcher: Option<Handle<RwLock<notify::RecommendedWatcher>>>,

    /// Paths being watched
    watched_paths: Shared<HashSet<PathBuf>>,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        Self {
            loader: Handle::new(ModuleLoader::new()),
            binding: Handle::new(BindingRegistry::new()),
            resolver: Handle::new(RwLock::new(ModuleResolver::new())),
            modules: Handle::new(RwLock::new(HashMap::new())),
            watcher: None,
            watched_paths: Handle::new(RwLock::new(HashSet::new())),
        }
    }

    /// Initialize from an App's Cargo.toml
    pub async fn initialize_from_app(&self, cargo_path: &Path) -> ModuleResult<()> {
        info!("Initializing module registry from app: {}", cargo_path.display());

        // Read app configuration
        let config = {
            let mut resolver = self.resolver.write().await;
            resolver.read_app_config(cargo_path)?
        };

        info!(
            "App {} declares {} Core modules and {} plugins",
            config.app_name,
            config.core_modules.len(),
            config.plugins.len()
        );

        // Find available System modules
        let system_paths = self.find_system_modules().await?;

        // Resolve and load modules
        for declaration in &config.core_modules {
            // Find which System implements this Core module
            let system_name = {
                let mut resolver = self.resolver.write().await;
                resolver.resolve_system(declaration, &system_paths)?
            };

            // Load Core module
            self.load_core_module(&declaration.name).await?;

            // Load System module
            self.load_system_module(&system_name).await?;
        }

        // Load plugins
        for plugin_name in &config.plugins {
            self.load_plugin_module(plugin_name).await?;
        }

        info!("Module registry initialized successfully");
        Ok(())
    }

    /// Load a Core module (Model + View)
    async fn load_core_module(&self, name: &str) -> ModuleResult<()> {
        info!("Loading Core module: {}", name);

        // Update state
        self.update_module_state(name, ModuleState::Loading).await;

        // Load the module
        self.loader.load_module(name).await?;

        // Register View API with binding registry
        // (The actual View API would be extracted from the loaded module)
        // For now, this is a placeholder

        // Update state
        self.update_module_state(name, ModuleState::Loaded).await;

        Ok(())
    }

    /// Load a System module (ViewModel)
    async fn load_system_module(&self, name: &str) -> ModuleResult<()> {
        info!("Loading System module: {}", name);

        // Update state
        self.update_module_state(name, ModuleState::Loading).await;

        // Load the module
        self.loader.load_module(name).await?;

        // Register ViewModel with binding registry
        // (The actual ViewModel would be extracted from the loaded module)
        // For now, this is a placeholder

        // Update state
        self.update_module_state(name, ModuleState::Bound).await;

        Ok(())
    }

    /// Load a Plugin module
    async fn load_plugin_module(&self, name: &str) -> ModuleResult<()> {
        info!("Loading Plugin module: {}", name);

        // Update state
        self.update_module_state(name, ModuleState::Loading).await;

        // Load the module
        self.loader.load_module(name).await?;

        // Update state
        self.update_module_state(name, ModuleState::Loaded).await;

        Ok(())
    }

    /// Find all System module Cargo.toml files
    async fn find_system_modules(&self) -> ModuleResult<Vec<PathBuf>> {
        let resolver = self.resolver.read().await;
        let system_dir = PathBuf::from("systems");

        if !system_dir.exists() {
            return Ok(Vec::new());
        }

        Ok(resolver.find_cargo_files(&system_dir))
    }

    /// Update module state
    async fn update_module_state(&self, name: &str, state: ModuleState) {
        let mut modules = self.modules.write().await;

        if let Some(info) = modules.get_mut(name) {
            info.state = state;
        } else {
            modules.insert(
                name.to_string(),
                ModuleInfo {
                    name: name.to_string(),
                    module_type: ModuleType::Core, // Would be determined from actual module
                    state,
                    path: None,
                    dependencies: Vec::new(),
                    features: Vec::new(),
                },
            );
        }

        debug!("Module {} state changed to {:?}", name, state);
    }

    /// Enable hot-reload watching
    pub async fn enable_hot_reload(&mut self) -> ModuleResult<()> {
        info!("Enabling hot-reload");

        let registry = Arc::new(self.clone_for_watcher());

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let registry = registry.clone();
                tokio::spawn(async move {
                    registry.handle_file_change(event).await;
                });
            }
        })
        .map_err(|e| ModuleError::Generic(format!("Failed to create watcher: {}", e)))?;

        // Watch target directories
        watcher
            .watch(&PathBuf::from("target/debug"), RecursiveMode::NonRecursive)
            .map_err(|e| ModuleError::Generic(format!("Failed to watch directory: {}", e)))?;

        self.watcher = Some(Handle::new(RwLock::new(watcher)));

        info!("Hot-reload enabled");
        Ok(())
    }

    /// Handle file change events
    async fn handle_file_change(&self, event: Event) {
        if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
            for path in &event.paths {
                if path.extension().and_then(|s| s.to_str()) == Some(std::env::consts::DLL_EXTENSION) {
                    if let Some(module_name) = path.file_stem().and_then(|s| s.to_str()) {
                        info!("Detected change in module: {}", module_name);

                        if let Err(e) = self.hot_reload_module(module_name).await {
                            error!("Failed to hot-reload {}: {}", module_name, e);
                        }
                    }
                }
            }
        }
    }

    /// Hot-reload a module
    async fn hot_reload_module(&self, name: &str) -> ModuleResult<()> {
        info!("Hot-reloading module: {}", name);

        // Update state
        self.update_module_state(name, ModuleState::Reloading).await;

        // Perform hot-reload
        self.loader.hot_reload(name).await?;

        // Re-bind if necessary
        // (Would need to re-establish View-ViewModel binding)

        // Update state
        self.update_module_state(name, ModuleState::Bound).await;

        info!("Successfully hot-reloaded module: {}", name);
        Ok(())
    }

    /// Get module information
    pub async fn get_module_info(&self, name: &str) -> ModuleResult<ModuleInfo> {
        let modules = self.modules.read().await;
        modules
            .get(name)
            .cloned()
            .ok_or_else(|| ModuleError::NotFound(name.to_string()))
    }

    /// List all modules
    pub async fn list_modules(&self) -> Vec<ModuleInfo> {
        let modules = self.modules.read().await;
        modules.values().cloned().collect()
    }

    /// Get registry statistics
    pub async fn get_stats(&self) -> RegistryStats {
        let modules = self.modules.read().await;
        let binding_stats = self.binding.get_stats().await;

        let mut stats = RegistryStats {
            total_modules: modules.len(),
            loaded_modules: 0,
            bound_modules: 0,
            failed_modules: 0,
            binding_stats,
        };

        for info in modules.values() {
            match info.state {
                ModuleState::Loaded => stats.loaded_modules += 1,
                ModuleState::Bound => stats.bound_modules += 1,
                ModuleState::Failed => stats.failed_modules += 1,
                _ => {}
            }
        }

        stats
    }

    /// Clone for use in watcher
    fn clone_for_watcher(&self) -> Self {
        Self {
            loader: self.loader.clone(),
            binding: self.binding.clone(),
            resolver: self.resolver.clone(),
            modules: self.modules.clone(),
            watcher: None,
            watched_paths: self.watched_paths.clone(),
        }
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}