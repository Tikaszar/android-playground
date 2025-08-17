use playground_ui::layout::docking::{DockingLayout, DockNode, DockOrientation, TabInfo};
use uuid::Uuid;

/// IDE layout configuration
pub struct IdeLayout {
    pub docking: DockingLayout,
    pub file_browser_id: Uuid,
    pub editor_id: Uuid,
    pub terminal_id: Uuid,
    pub chat_id: Uuid,
    pub debugger_id: Uuid,
    pub version_control_id: Uuid,
}

impl IdeLayout {
    pub fn new() -> Self {
        let mut docking = DockingLayout::new();
        
        // Create element IDs
        let file_browser_id = Uuid::new_v4();
        let editor_id = Uuid::new_v4();
        let terminal_id = Uuid::new_v4();
        let chat_id = Uuid::new_v4();
        let debugger_id = Uuid::new_v4();
        let version_control_id = Uuid::new_v4();
        
        // Build default layout:
        // +------------------+------------------+------------------+
        // |                  |                  |                  |
        // |   File Browser   |      Editor      |       Chat       |
        // |                  |                  |                  |
        // |                  |                  |                  |
        // +------------------+------------------+------------------+
        // |  Version Control |     Terminal     |     Debugger     |
        // +------------------+------------------+------------------+
        
        let root = docking.get_root();
        
        // Split horizontally first (top/bottom)
        let (top, bottom) = docking.split_dock(root, DockOrientation::Horizontal, 0.7)
            .expect("Failed to split root");
        
        // Split top into three columns
        let (top_left, top_middle_right) = docking.split_dock(top, DockOrientation::Vertical, 0.2)
            .expect("Failed to split top");
        let (top_middle, top_right) = docking.split_dock(top_middle_right, DockOrientation::Vertical, 0.7)
            .expect("Failed to split top middle");
        
        // Split bottom into three columns  
        let (bottom_left, bottom_middle_right) = docking.split_dock(bottom, DockOrientation::Vertical, 0.2)
            .expect("Failed to split bottom");
        let (bottom_middle, bottom_right) = docking.split_dock(bottom_middle_right, DockOrientation::Vertical, 0.6)
            .expect("Failed to split bottom middle");
        
        // Add panels
        docking.add_panel(top_left, TabInfo {
            id: file_browser_id,
            title: "Files".to_string(),
            element_id: file_browser_id,
            closable: false,
            icon: Some("folder"),
        }).expect("Failed to add file browser");
        
        docking.add_panel(top_middle, TabInfo {
            id: editor_id,
            title: "Editor".to_string(),
            element_id: editor_id,
            closable: false,
            icon: Some("file-code"),
        }).expect("Failed to add editor");
        
        docking.add_panel(top_right, TabInfo {
            id: chat_id,
            title: "Assistant".to_string(),
            element_id: chat_id,
            closable: true,
            icon: Some("message-circle"),
        }).expect("Failed to add chat");
        
        docking.add_panel(bottom_left, TabInfo {
            id: version_control_id,
            title: "Git".to_string(),
            element_id: version_control_id,
            closable: true,
            icon: Some("git-branch"),
        }).expect("Failed to add version control");
        
        docking.add_panel(bottom_middle, TabInfo {
            id: terminal_id,
            title: "Terminal".to_string(),
            element_id: terminal_id,
            closable: false,
            icon: Some("terminal"),
        }).expect("Failed to add terminal");
        
        docking.add_panel(bottom_right, TabInfo {
            id: debugger_id,
            title: "Debug".to_string(),
            element_id: debugger_id,
            closable: true,
            icon: Some("bug"),
        }).expect("Failed to add debugger");
        
        Self {
            docking,
            file_browser_id,
            editor_id,
            terminal_id,
            chat_id,
            debugger_id,
            version_control_id,
        }
    }
    
    /// Get the default mobile layout (simplified)
    pub fn new_mobile() -> Self {
        let mut docking = DockingLayout::new();
        
        let file_browser_id = Uuid::new_v4();
        let editor_id = Uuid::new_v4();
        let terminal_id = Uuid::new_v4();
        let chat_id = Uuid::new_v4();
        let debugger_id = Uuid::new_v4();
        let version_control_id = Uuid::new_v4();
        
        // Mobile layout: Single dock with tabs
        let root = docking.get_root();
        
        // Add all panels as tabs in the root
        docking.add_panel(root, TabInfo {
            id: editor_id,
            title: "Editor".to_string(),
            element_id: editor_id,
            closable: false,
            icon: Some("file-code"),
        }).expect("Failed to add editor");
        
        docking.add_panel(root, TabInfo {
            id: file_browser_id,
            title: "Files".to_string(),
            element_id: file_browser_id,
            closable: false,
            icon: Some("folder"),
        }).expect("Failed to add file browser");
        
        docking.add_panel(root, TabInfo {
            id: terminal_id,
            title: "Terminal".to_string(),
            element_id: terminal_id,
            closable: false,
            icon: Some("terminal"),
        }).expect("Failed to add terminal");
        
        docking.add_panel(root, TabInfo {
            id: chat_id,
            title: "Chat".to_string(),
            element_id: chat_id,
            closable: false,
            icon: Some("message-circle"),
        }).expect("Failed to add chat");
        
        Self {
            docking,
            file_browser_id,
            editor_id,
            terminal_id,
            chat_id,
            debugger_id,
            version_control_id,
        }
    }
}