use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Manages UI panels in the browser (editor, terminal, file browser, etc.)
pub struct PanelManager {
    panels: HashMap<Uuid, Panel>,
    layout: PanelLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panel {
    pub id: Uuid,
    pub panel_type: PanelType,
    pub title: String,
    pub is_visible: bool,
    pub is_focused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    Editor,
    Terminal,
    FileBrowser,
    Chat,
    Diff,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelLayout {
    Single(Uuid),
    SplitHorizontal { left: Box<PanelLayout>, right: Box<PanelLayout> },
    SplitVertical { top: Box<PanelLayout>, bottom: Box<PanelLayout> },
    Tabs(Vec<Uuid>),
}

impl PanelManager {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
            layout: PanelLayout::Tabs(Vec::new()),
        }
    }
    
    pub async fn initialize_default_panels(&mut self) {
        // Create default panels
        self.create_panel(PanelType::Chat, "Chat".to_string());
        self.create_panel(PanelType::Editor, "Editor".to_string());
        self.create_panel(PanelType::FileBrowser, "Files".to_string());
    }
    
    pub async fn handle_panel_update(&mut self, msg: serde_json::Value) {
        // Handle panel update messages from browser
        if let Some(panel_id) = msg.get("panel_id").and_then(|v| v.as_str()) {
            if let Ok(id) = Uuid::parse_str(panel_id) {
                if let Some(panel) = self.panels.get_mut(&id) {
                    if let Some(is_focused) = msg.get("is_focused").and_then(|v| v.as_bool()) {
                        panel.is_focused = is_focused;
                    }
                }
            }
        }
    }

    pub fn create_panel(&mut self, panel_type: PanelType, title: String) -> Uuid {
        let panel = Panel {
            id: Uuid::new_v4(),
            panel_type,
            title,
            is_visible: true,
            is_focused: false,
        };
        
        let panel_id = panel.id;
        self.panels.insert(panel_id, panel);
        
        // Add to layout if it's tabs
        if let PanelLayout::Tabs(ref mut tabs) = self.layout {
            tabs.push(panel_id);
        }
        
        panel_id
    }

    pub fn show_panel(&mut self, panel_id: Uuid) -> Result<()> {
        if let Some(panel) = self.panels.get_mut(&panel_id) {
            panel.is_visible = true;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Panel {} not found", panel_id))
        }
    }

    pub fn hide_panel(&mut self, panel_id: Uuid) -> Result<()> {
        if let Some(panel) = self.panels.get_mut(&panel_id) {
            panel.is_visible = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Panel {} not found", panel_id))
        }
    }

    pub fn focus_panel(&mut self, panel_id: Uuid) -> Result<()> {
        // Unfocus all panels first
        for panel in self.panels.values_mut() {
            panel.is_focused = false;
        }
        
        // Focus the requested panel
        if let Some(panel) = self.panels.get_mut(&panel_id) {
            panel.is_focused = true;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Panel {} not found", panel_id))
        }
    }

    pub fn get_layout(&self) -> &PanelLayout {
        &self.layout
    }

    pub fn set_layout(&mut self, layout: PanelLayout) {
        self.layout = layout;
    }
}