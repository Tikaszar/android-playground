//! Resolves module dependencies from Cargo.toml

use crate::config::{AppModuleConfig, ModuleDeclaration, SystemProvides};
use playground_modules_types::{ModuleError, ModuleResult};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use toml::Value;
use tracing::{debug, info, warn};

/// Resolves module dependencies from Cargo.toml
pub struct ModuleResolver {
    /// Cache of parsed Cargo.toml files
    cargo_cache: HashMap<PathBuf, Value>,
}

impl ModuleResolver {
    /// Create a new module resolver
    pub fn new() -> Self {
        Self {
            cargo_cache: HashMap::new(),
        }
    }

    /// Read module declarations from an App's Cargo.toml
    pub fn read_app_config(&mut self, cargo_path: &Path) -> ModuleResult<AppModuleConfig> {
        info!("Reading app config from: {}", cargo_path.display());

        let content = std::fs::read_to_string(cargo_path).map_err(|e| {
            ModuleError::LoadFailed(format!("Failed to read Cargo.toml: {}", e))
        })?;

        let value: Value = toml::from_str(&content).map_err(|e| {
            ModuleError::LoadFailed(format!("Failed to parse Cargo.toml: {}", e))
        })?;

        // Cache the parsed value
        self.cargo_cache.insert(cargo_path.to_path_buf(), value.clone());

        // Extract package name
        let app_name = value
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .ok_or_else(|| ModuleError::LoadFailed("Missing package.name".to_string()))?
            .to_string();

        // Extract module declarations
        let core_modules = self.extract_core_modules(&value)?;
        let plugins = self.extract_plugins(&value)?;

        Ok(AppModuleConfig {
            app_name,
            core_modules,
            plugins,
        })
    }

    /// Extract Core module declarations from Cargo.toml
    fn extract_core_modules(&self, value: &Value) -> ModuleResult<Vec<ModuleDeclaration>> {
        let mut modules = Vec::new();

        // Look for [[package.metadata.modules.core]]
        if let Some(metadata) = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("modules"))
            .and_then(|m| m.get("core"))
        {
            if let Some(array) = metadata.as_array() {
                for item in array {
                    let name = item
                        .get("name")
                        .and_then(|n| n.as_str())
                        .ok_or_else(|| ModuleError::LoadFailed("Missing module name".to_string()))?
                        .to_string();

                    let features = item
                        .get("features")
                        .and_then(|f| f.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    let systems = item
                        .get("systems")
                        .and_then(|s| s.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    modules.push(ModuleDeclaration {
                        name,
                        features,
                        systems,
                    });
                }
            }
        }

        debug!("Found {} Core module declarations", modules.len());
        Ok(modules)
    }

    /// Extract plugin declarations from Cargo.toml
    fn extract_plugins(&self, value: &Value) -> ModuleResult<Vec<String>> {
        let mut plugins = Vec::new();

        // Look for [[package.metadata.modules.plugins]]
        if let Some(metadata) = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("modules"))
            .and_then(|m| m.get("plugins"))
        {
            if let Some(array) = metadata.as_array() {
                for item in array {
                    if let Some(name) = item.as_str() {
                        plugins.push(name.to_string());
                    }
                }
            }
        }

        debug!("Found {} plugin declarations", plugins.len());
        Ok(plugins)
    }

    /// Read what a System module provides from its Cargo.toml
    pub fn read_system_provides(&mut self, cargo_path: &Path) -> ModuleResult<SystemProvides> {
        info!("Reading system provides from: {}", cargo_path.display());

        let content = std::fs::read_to_string(cargo_path).map_err(|e| {
            ModuleError::LoadFailed(format!("Failed to read Cargo.toml: {}", e))
        })?;

        let value: Value = toml::from_str(&content).map_err(|e| {
            ModuleError::LoadFailed(format!("Failed to parse Cargo.toml: {}", e))
        })?;

        // Look for [package.metadata.provides]
        let provides = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("provides"))
            .ok_or_else(|| {
                ModuleError::LoadFailed("System module missing [package.metadata.provides]".to_string())
            })?;

        let core = provides
            .get("core")
            .and_then(|c| c.as_str())
            .ok_or_else(|| ModuleError::LoadFailed("Missing provides.core".to_string()))?
            .to_string();

        let features = provides
            .get("features")
            .and_then(|f| f.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(SystemProvides { core, features })
    }

    /// Validate that Systems provide required features
    pub fn validate_features(
        &self,
        required: &[String],
        provided: &[String],
    ) -> ModuleResult<()> {
        let required_set: HashSet<_> = required.iter().collect();
        let provided_set: HashSet<_> = provided.iter().collect();

        let missing: Vec<_> = required_set.difference(&provided_set).collect();

        if !missing.is_empty() {
            return Err(ModuleError::FeatureMissing(format!(
                "Missing features: {:?}",
                missing
            )));
        }

        Ok(())
    }

    /// Resolve which System to use for a Core module
    pub fn resolve_system(
        &mut self,
        declaration: &ModuleDeclaration,
        available_systems: &[PathBuf],
    ) -> ModuleResult<String> {
        // Try each system in priority order
        for system_name in &declaration.systems {
            // Find the system's Cargo.toml
            let system_cargo = available_systems
                .iter()
                .find(|p| p.to_string_lossy().contains(system_name));

            if let Some(cargo_path) = system_cargo {
                // Check what this system provides
                match self.read_system_provides(cargo_path) {
                    Ok(provides) => {
                        // Validate it implements the right Core module
                        if provides.core != declaration.name {
                            continue;
                        }

                        // Validate it provides required features
                        if self.validate_features(&declaration.features, &provides.features).is_ok() {
                            info!(
                                "Resolved {} to system {}",
                                declaration.name, system_name
                            );
                            return Ok(system_name.clone());
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read system {}: {}", system_name, e);
                        continue;
                    }
                }
            }
        }

        Err(ModuleError::LoadFailed(format!(
            "No suitable System found for Core module {}",
            declaration.name
        )))
    }

    /// Find all Cargo.toml files in a directory tree
    pub fn find_cargo_files(&self, root: &Path) -> Vec<PathBuf> {
        let mut cargo_files = Vec::new();

        for entry in walkdir::WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "Cargo.toml" {
                cargo_files.push(entry.path().to_path_buf());
            }
        }

        cargo_files
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}