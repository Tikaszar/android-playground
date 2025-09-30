//! Complete module configuration from an App

use super::declaration::ModuleDeclaration;

/// Complete module configuration from an App
#[derive(Debug, Clone)]
pub struct AppModuleConfig {
    /// App package name
    pub app_name: String,

    /// Core modules this app needs
    pub core_modules: Vec<ModuleDeclaration>,

    /// Plugin modules this app uses
    pub plugins: Vec<String>,
}