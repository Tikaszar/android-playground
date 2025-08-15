use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;

use playground_types::error::PluginError;
use playground_types::plugin_metadata::PluginId;

use crate::r#trait::{CreatePluginFn, Plugin};

pub struct PluginLoader {
    loaded_plugins: HashMap<PluginId, LoadedPlugin>,
}

struct LoadedPlugin {
    plugin: Box<dyn Plugin>,
    _library: libloading::Library,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            loaded_plugins: HashMap::new(),
        }
    }

    pub fn load_plugin(&mut self, path: &Path) -> Result<PluginId, PluginError> {
        if !path.exists() {
            return Err(PluginError::NotFound("Plugin file does not exist".to_string()));
        }

        if path.extension() != Some(OsStr::new("so")) {
            return Err(PluginError::LoadFailed("Plugin must be a .so file".to_string()));
        }

        unsafe {
            let library = libloading::Library::new(path)
                .map_err(|e| PluginError::LoadFailed(format!("Failed to load library: {}", e)))?;

            let create_fn: libloading::Symbol<CreatePluginFn> = library
                .get(b"create_plugin")
                .map_err(|e| PluginError::LoadFailed(format!("Failed to find create_plugin: {}", e)))?;

            let plugin_ptr = create_fn();
            let plugin = Box::from_raw(plugin_ptr);
            let metadata = plugin.metadata();
            let plugin_id = metadata.id.clone();

            if self.loaded_plugins.contains_key(&plugin_id) {
                return Err(PluginError::LoadFailed("Plugin already loaded".to_string()));
            }

            self.loaded_plugins.insert(
                plugin_id.clone(),
                LoadedPlugin { plugin, _library: library },
            );

            Ok(plugin_id)
        }
    }

    pub fn unload_plugin(&mut self, id: &PluginId) -> Result<(), PluginError> {
        self.loaded_plugins
            .remove(id)
            .ok_or_else(|| PluginError::NotFound("Plugin not found".to_string()))?;
        Ok(())
    }

    pub fn get_plugin(&self, id: &PluginId) -> Option<&dyn Plugin> {
        self.loaded_plugins
            .get(id)
            .map(|loaded| loaded.plugin.as_ref())
    }

    pub fn get_plugin_mut(&mut self, id: &PluginId) -> Option<&mut dyn Plugin> {
        self.loaded_plugins
            .get_mut(id)
            .map(|loaded| loaded.plugin.as_mut())
    }

    pub fn list_plugins(&self) -> Vec<PluginId> {
        self.loaded_plugins.keys().cloned().collect()
    }
}