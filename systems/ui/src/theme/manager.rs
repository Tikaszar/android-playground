//! Theme manager

use crate::error::{UiError, UiResult};
use crate::theme::Theme;
use std::collections::HashMap;

/// Manages UI themes
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    current_theme: String,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(),
            current_theme: "Dark".to_string(),
        }
    }
    
    /// Load default themes
    pub fn load_default_themes(&mut self) -> UiResult<()> {
        self.add_theme(Theme::dark());
        self.add_theme(Theme::light());
        Ok(())
    }
    
    /// Add a theme
    pub fn add_theme(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }
    
    /// Get current theme
    pub fn current(&self) -> Option<&Theme> {
        self.themes.get(&self.current_theme)
    }
    
    /// Set current theme
    pub fn set_current(&mut self, name: &str) -> UiResult<()> {
        if !self.themes.contains_key(name) {
            return Err(UiError::ThemeError(format!("Theme '{}' not found", name)));
        }
        self.current_theme = name.to_string();
        Ok(())
    }
    
    /// Get theme by name
    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }
    
    /// List available themes
    pub fn list_themes(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }
}