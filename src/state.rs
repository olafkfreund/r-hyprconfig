// State management module
// Separates UI state from application logic state

use ratatui::widgets::ListState;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::app::FocusedPanel;
use crate::batch::BatchOperationType;
use crate::nixos::{NixConfigType, NixOSEnvironment};
use crate::ui::{BatchDialogMode, ConfigItem, EditMode, ExportFormatType, ImportExportMode, ImportSourceType};

/// Pure UI state - only visual rendering state
#[derive(Debug, Clone)]
pub struct UIState {
    // List states for navigation
    pub general_list_state: ListState,
    pub input_list_state: ListState,
    pub decoration_list_state: ListState,
    pub animations_list_state: ListState,
    pub gestures_list_state: ListState,
    pub binds_list_state: ListState,
    pub window_rules_list_state: ListState,
    pub layer_rules_list_state: ListState,
    pub misc_list_state: ListState,
    pub import_list_state: ListState,
    pub export_list_state: ListState,

    // Current tab navigation
    pub current_tab: FocusedPanel,

    // Visual state only
    pub help_scroll: usize,
    pub preview_scroll: usize,
    pub import_export_scroll: usize,

    // Virtualization and pagination
    pub current_page: HashMap<FocusedPanel, usize>,
    pub total_pages: HashMap<FocusedPanel, usize>,
    pub item_height: usize,
    pub page_size: usize,

    // Cache invalidation
    pub item_cache_generation: usize,

    // Theme (UI-specific)
    pub theme: crate::theme::Theme,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            general_list_state: ListState::default(),
            input_list_state: ListState::default(),
            decoration_list_state: ListState::default(),
            animations_list_state: ListState::default(),
            gestures_list_state: ListState::default(),
            binds_list_state: ListState::default(),
            window_rules_list_state: ListState::default(),
            layer_rules_list_state: ListState::default(),
            misc_list_state: ListState::default(),
            import_list_state: ListState::default(),
            export_list_state: ListState::default(),
            current_tab: FocusedPanel::General,
            help_scroll: 0,
            preview_scroll: 0,
            import_export_scroll: 0,
            current_page: HashMap::new(),
            total_pages: HashMap::new(),
            item_height: 2, // Height per item including description
            page_size: 50,
            item_cache_generation: 0,
            theme: crate::theme::Theme::default(),
        }
    }
}

impl UIState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the list state for the current panel
    pub fn get_current_list_state_mut(&mut self) -> &mut ListState {
        match self.current_tab {
            FocusedPanel::General => &mut self.general_list_state,
            FocusedPanel::Input => &mut self.input_list_state,
            FocusedPanel::Decoration => &mut self.decoration_list_state,
            FocusedPanel::Animations => &mut self.animations_list_state,
            FocusedPanel::Gestures => &mut self.gestures_list_state,
            FocusedPanel::Binds => &mut self.binds_list_state,
            FocusedPanel::WindowRules => &mut self.window_rules_list_state,
            FocusedPanel::LayerRules => &mut self.layer_rules_list_state,
            FocusedPanel::Misc => &mut self.misc_list_state,
            FocusedPanel::Import => &mut self.import_list_state,
            FocusedPanel::Export => &mut self.export_list_state,
        }
    }

    pub fn get_current_list_state(&self) -> &ListState {
        match self.current_tab {
            FocusedPanel::General => &self.general_list_state,
            FocusedPanel::Input => &self.input_list_state,
            FocusedPanel::Decoration => &self.decoration_list_state,
            FocusedPanel::Animations => &self.animations_list_state,
            FocusedPanel::Gestures => &self.gestures_list_state,
            FocusedPanel::Binds => &self.binds_list_state,
            FocusedPanel::WindowRules => &self.window_rules_list_state,
            FocusedPanel::LayerRules => &self.layer_rules_list_state,
            FocusedPanel::Misc => &self.misc_list_state,
            FocusedPanel::Import => &self.import_list_state,
            FocusedPanel::Export => &self.export_list_state,
        }
    }

    /// Invalidate cache by incrementing generation
    pub fn invalidate_cache(&mut self) {
        self.item_cache_generation = self.item_cache_generation.wrapping_add(1);
    }

    /// Reset all list states
    pub fn reset_list_states(&mut self) {
        self.general_list_state = ListState::default();
        self.input_list_state = ListState::default();
        self.decoration_list_state = ListState::default();
        self.animations_list_state = ListState::default();
        self.gestures_list_state = ListState::default();
        self.binds_list_state = ListState::default();
        self.window_rules_list_state = ListState::default();
        self.layer_rules_list_state = ListState::default();
        self.misc_list_state = ListState::default();
        self.import_list_state = ListState::default();
        self.export_list_state = ListState::default();
    }
}

/// Application logic state - business logic and data
#[derive(Debug, Clone)]
pub struct ApplicationState {
    // Configuration data
    pub config_items: HashMap<FocusedPanel, Vec<ConfigItem>>,

    // NixOS integration
    pub nixos_env: NixOSEnvironment,

    // System state
    pub last_tick: Instant,
    pub tick_rate: Duration,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            config_items: HashMap::new(),
            nixos_env: NixOSEnvironment::detect(),
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl ApplicationState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get configuration items for a specific panel
    pub fn get_config_items(&self, panel: &FocusedPanel) -> Option<&Vec<ConfigItem>> {
        self.config_items.get(panel)
    }

    /// Get mutable configuration items for a specific panel
    pub fn get_config_items_mut(&mut self, panel: &FocusedPanel) -> Option<&mut Vec<ConfigItem>> {
        self.config_items.get_mut(panel)
    }

    /// Set configuration items for a panel
    pub fn set_config_items(&mut self, panel: FocusedPanel, items: Vec<ConfigItem>) {
        self.config_items.insert(panel, items);
    }

    /// Update a specific configuration item
    pub fn update_config_item(&mut self, panel: &FocusedPanel, key: &str, new_value: String) -> bool {
        if let Some(items) = self.config_items.get_mut(panel) {
            if let Some(item) = items.iter_mut().find(|item| item.key == key) {
                item.value = new_value;
                return true;
            }
        }
        false
    }

    /// Add a new configuration item
    pub fn add_config_item(&mut self, panel: FocusedPanel, item: ConfigItem) {
        self.config_items.entry(panel).or_default().push(item);
    }

    /// Remove a configuration item
    pub fn remove_config_item(&mut self, panel: &FocusedPanel, key: &str) -> bool {
        if let Some(items) = self.config_items.get_mut(panel) {
            if let Some(pos) = items.iter().position(|item| item.key == key) {
                items.remove(pos);
                return true;
            }
        }
        false
    }
}

/// Dialog states - separate from UI and application logic
#[derive(Debug, Clone)]
pub struct DialogState {
    // Basic dialogs
    pub show_popup: bool,
    pub popup_message: String,
    pub show_save_dialog: bool,
    pub show_reload_dialog: bool,
    pub show_help: bool,

    // NixOS export dialog
    pub show_nixos_export_dialog: bool,
    pub nixos_export_config_type: NixConfigType,
    pub nixos_export_preview: Option<String>,

    // Batch management dialog
    pub show_batch_dialog: bool,
    pub batch_dialog_mode: BatchDialogMode,
    pub batch_selected_profile: Option<String>,
    pub batch_operation_type: BatchOperationType,

    // Preview dialog
    pub show_preview_dialog: bool,
    pub preview_before: Option<String>,
    pub preview_after: Option<String>,
    pub preview_setting_name: String,

    // Import/Export dialog
    pub show_import_dialog: bool,
    pub show_export_dialog: bool,
    pub import_export_mode: ImportExportMode,
    pub selected_import_source: ImportSourceType,
    pub selected_export_format: ExportFormatType,
    pub import_preview: Option<String>,
    pub export_preview: Option<String>,
}

impl Default for DialogState {
    fn default() -> Self {
        Self {
            show_popup: false,
            popup_message: String::new(),
            show_save_dialog: false,
            show_reload_dialog: false,
            show_help: false,
            show_nixos_export_dialog: false,
            nixos_export_config_type: NixConfigType::SystemConfig,
            nixos_export_preview: None,
            show_batch_dialog: false,
            batch_dialog_mode: BatchDialogMode::ManageProfiles,
            batch_selected_profile: None,
            batch_operation_type: BatchOperationType::Apply,
            show_preview_dialog: false,
            preview_before: None,
            preview_after: None,
            preview_setting_name: String::new(),
            show_import_dialog: false,
            show_export_dialog: false,
            import_export_mode: ImportExportMode::SelectSource,
            selected_import_source: ImportSourceType::LocalFile,
            selected_export_format: ExportFormatType::HyprlandConf,
            import_preview: None,
            export_preview: None,
        }
    }
}

impl DialogState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any dialog is currently open
    pub fn has_active_dialog(&self) -> bool {
        self.show_popup
            || self.show_save_dialog
            || self.show_reload_dialog
            || self.show_help
            || self.show_nixos_export_dialog
            || self.show_batch_dialog
            || self.show_preview_dialog
            || self.show_import_dialog
            || self.show_export_dialog
    }

    /// Close all dialogs
    pub fn close_all(&mut self) {
        self.show_popup = false;
        self.show_save_dialog = false;
        self.show_reload_dialog = false;
        self.show_help = false;
        self.show_nixos_export_dialog = false;
        self.show_batch_dialog = false;
        self.show_preview_dialog = false;
        self.show_import_dialog = false;
        self.show_export_dialog = false;
        
        // Clear associated data
        self.popup_message.clear();
        self.nixos_export_preview = None;
        self.preview_before = None;
        self.preview_after = None;
        self.preview_setting_name.clear();
        self.import_preview = None;
        self.export_preview = None;
    }

    /// Show popup with message
    pub fn show_popup(&mut self, message: impl Into<String>) {
        self.popup_message = message.into();
        self.show_popup = true;
    }
}

/// Interactive state - user input and editing
#[derive(Debug, Clone)]
pub struct InteractiveState {
    // Edit mode
    pub edit_mode: EditMode,
    pub editing_item: Option<(FocusedPanel, String)>,

    // Search functionality  
    pub search_mode: bool,
    pub search_query: String,
    pub search_cursor: usize,

    // Debounced search
    pub search_debounce_delay: Duration,
    pub last_search_input: Instant,
    pub pending_search_query: String,
    pub debounced_search_active: bool,

    // Search result caching
    pub search_cache: HashMap<String, Vec<ConfigItem>>,
    pub search_cache_max_size: usize,

    // Progressive search
    pub progressive_search_threshold: usize,
    pub progressive_search_chunk_size: usize,

    // Real-time preview
    pub preview_mode: bool,
    pub preview_debounce_delay: Duration,
    pub last_preview_time: Instant,
    pub pending_preview_change: Option<(String, String)>, // (key, value)
    pub preview_original_value: Option<String>,

    // Pending operations
    pub pending_deletion: Option<(FocusedPanel, String)>, // (panel, key)
}

impl Default for InteractiveState {
    fn default() -> Self {
        Self {
            edit_mode: EditMode::None,
            editing_item: None,
            search_mode: false,
            search_query: String::new(),
            search_cursor: 0,
            search_debounce_delay: Duration::from_millis(300),
            last_search_input: Instant::now(),
            pending_search_query: String::new(),
            debounced_search_active: false,
            search_cache: HashMap::new(),
            search_cache_max_size: 100,
            progressive_search_threshold: 1000,
            progressive_search_chunk_size: 100,
            preview_mode: true,
            preview_debounce_delay: Duration::from_millis(500),
            last_preview_time: Instant::now(),
            pending_preview_change: None,
            preview_original_value: None,
            pending_deletion: None,
        }
    }
}

impl InteractiveState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enter edit mode for a specific item
    pub fn start_editing(&mut self, panel: FocusedPanel, key: String, edit_mode: EditMode) {
        self.editing_item = Some((panel, key));
        self.edit_mode = edit_mode;
    }

    /// Exit edit mode
    pub fn stop_editing(&mut self) {
        self.editing_item = None;
        self.edit_mode = EditMode::None;
        self.preview_original_value = None;
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.editing_item.is_some() && self.edit_mode != EditMode::None
    }

    /// Start search mode
    pub fn start_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_cursor = 0;
    }

    /// Stop search mode
    pub fn stop_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.search_cursor = 0;
        self.pending_search_query.clear();
        self.debounced_search_active = false;
    }

    /// Update search query
    pub fn update_search_query(&mut self, query: String) {
        self.search_query = query.clone();
        self.pending_search_query = query;
        self.last_search_input = Instant::now();
        self.debounced_search_active = true;
    }

    /// Check if search debounce period has elapsed
    pub fn should_execute_search(&self) -> bool {
        self.debounced_search_active
            && self.last_search_input.elapsed() >= self.search_debounce_delay
    }

    /// Get cached search result
    pub fn get_cached_search(&self, query: &str) -> Option<&Vec<ConfigItem>> {
        self.search_cache.get(query)
    }

    /// Cache search result
    pub fn cache_search_result(&mut self, query: String, result: Vec<ConfigItem>) {
        if self.search_cache.len() >= self.search_cache_max_size {
            // Simple cache eviction - remove first entry
            if let Some(first_key) = self.search_cache.keys().next().cloned() {
                self.search_cache.remove(&first_key);
            }
        }
        self.search_cache.insert(query, result);
    }
}

/// Central state manager that composes all state types
#[derive(Debug, Clone)]
pub struct StateManager {
    pub ui: UIState,
    pub application: ApplicationState,
    pub dialogs: DialogState,
    pub interactive: InteractiveState,
}

impl Default for StateManager {
    fn default() -> Self {
        Self {
            ui: UIState::default(),
            application: ApplicationState::default(),
            dialogs: DialogState::default(),
            interactive: InteractiveState::default(),
        }
    }
}

impl StateManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all state to defaults
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Get the currently focused panel
    pub fn current_panel(&self) -> FocusedPanel {
        self.ui.current_tab
    }

    /// Change the current panel
    pub fn set_current_panel(&mut self, panel: FocusedPanel) {
        self.ui.current_tab = panel;
    }

    /// Check if any modal dialog is open
    pub fn has_modal_open(&self) -> bool {
        self.dialogs.has_active_dialog() || self.interactive.is_editing()
    }

    /// Close all open modals
    pub fn close_all_modals(&mut self) {
        self.dialogs.close_all();
        self.interactive.stop_editing();
        self.interactive.stop_search();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state_list_navigation() {
        let mut ui_state = UIState::new();
        
        // Test initial state
        assert_eq!(ui_state.current_tab, FocusedPanel::General);
        
        // Test tab switching
        ui_state.current_tab = FocusedPanel::Input;
        assert_eq!(ui_state.current_tab, FocusedPanel::Input);
        
        // Test list state access
        let list_state = ui_state.get_current_list_state_mut();
        assert!(list_state.selected().is_none());
    }

    #[test]
    fn test_application_state_config_management() {
        let mut app_state = ApplicationState::new();
        
        // Test adding config items
        let item = ConfigItem::new(
            "test_key".to_string(),
            "test_value".to_string(),
            "Test description".to_string(),
            crate::ui::ConfigDataType::String,
        );
        
        app_state.add_config_item(FocusedPanel::General, item);
        
        // Test retrieval
        let items = app_state.get_config_items(&FocusedPanel::General);
        assert!(items.is_some());
        assert_eq!(items.unwrap().len(), 1);
        assert_eq!(items.unwrap()[0].key, "test_key");
    }

    #[test]
    fn test_dialog_state_management() {
        let mut dialog_state = DialogState::new();
        
        // Test initial state
        assert!(!dialog_state.has_active_dialog());
        
        // Test showing popup
        dialog_state.show_popup("Test message");
        assert!(dialog_state.has_active_dialog());
        assert!(dialog_state.show_popup);
        assert_eq!(dialog_state.popup_message, "Test message");
        
        // Test closing all dialogs
        dialog_state.close_all();
        assert!(!dialog_state.has_active_dialog());
        assert!(dialog_state.popup_message.is_empty());
    }

    #[test]
    fn test_interactive_state_editing() {
        let mut interactive_state = InteractiveState::new();
        
        // Test initial state
        assert!(!interactive_state.is_editing());
        
        // Test starting edit
        interactive_state.start_editing(
            FocusedPanel::General,
            "test_key".to_string(),
            EditMode::Text {
                current_value: "test".to_string(),
                cursor_pos: 4,
            }
        );
        
        assert!(interactive_state.is_editing());
        assert!(interactive_state.editing_item.is_some());
        
        // Test stopping edit
        interactive_state.stop_editing();
        assert!(!interactive_state.is_editing());
        assert!(interactive_state.editing_item.is_none());
    }

    #[test]
    fn test_state_manager_composition() {
        let mut state_manager = StateManager::new();
        
        // Test initial state
        assert_eq!(state_manager.current_panel(), FocusedPanel::General);
        assert!(!state_manager.has_modal_open());
        
        // Test panel change
        state_manager.set_current_panel(FocusedPanel::Input);
        assert_eq!(state_manager.current_panel(), FocusedPanel::Input);
        
        // Test modal detection
        state_manager.dialogs.show_popup("Test");
        assert!(state_manager.has_modal_open());
        
        // Test closing all modals
        state_manager.close_all_modals();
        assert!(!state_manager.has_modal_open());
    }
}