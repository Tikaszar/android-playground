use std::collections::HashMap;
use crate::error::{UiError, UiResult};
use super::types::{Theme, ThemeId};

pub struct ThemeManager {
    themes: HashMap<ThemeId, Theme>,
    current: ThemeId,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        themes.insert(ThemeId::Dark, Theme::dark());
        themes.insert(ThemeId::Light, Theme::light());
        
        Self {
            themes,
            current: ThemeId::Dark,
        }
    }
    
    pub fn load_default_themes(&mut self) -> UiResult<()> {
        self.themes.insert(ThemeId::Dark, Theme::dark());
        self.themes.insert(ThemeId::Light, Theme::light());
        Ok(())
    }
    
    pub fn add_theme(&mut self, theme: Theme) -> ThemeId {
        let id = ThemeId::Custom(self.themes.len() as u32);
        self.themes.insert(id, theme);
        id
    }
    
    pub fn get_theme(&self, id: ThemeId) -> UiResult<&Theme> {
        self.themes.get(&id)
            .ok_or_else(|| UiError::ThemeNotFound(id.to_string()))
    }
    
    pub fn has_theme(&self, id: ThemeId) -> bool {
        self.themes.contains_key(&id)
    }
    
    pub fn set_current(&mut self, id: ThemeId) -> UiResult<()> {
        if self.has_theme(id) {
            self.current = id;
            Ok(())
        } else {
            Err(UiError::ThemeNotFound(id.to_string()))
        }
    }
    
    pub fn current(&self) -> ThemeId {
        self.current
    }
}