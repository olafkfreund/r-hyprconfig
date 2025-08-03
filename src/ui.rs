use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::app::FocusedPanel;
use crate::nixos::NixOSEnvironment;

#[derive(Debug, Clone, PartialEq)]
pub enum EditMode {
    None,
    Text {
        current_value: String,
        cursor_pos: usize,
    },
    Slider {
        current_value: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    Select {
        options: Vec<String>,
        selected: usize,
    },
    Boolean {
        current_value: bool,
    },
    Keybind {
        modifiers: Vec<String>,
        key: String,
        dispatcher: String,
        args: String,
        editing_field: KeybindField,
    },
    Rule {
        rule_type: RuleType,
        pattern: String,
        action: String,
        editing_field: RuleField,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeybindField {
    Modifiers,
    Key,
    Dispatcher,
    Args,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleType {
    Window,
    Layer,
    Workspace,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleField {
    Pattern,
    Action,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatchDialogMode {
    ManageProfiles,
    SelectOperation,
    ExecuteOperation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportExportMode {
    SelectSource,
    SelectFormat,
    Preview,
    Execute,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportSourceType {
    LocalFile,
    LocalFolder,
    GitHubRepository,
    UrlDownload,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormatType {
    HyprlandConf,
    Json,
    Toml,
    Yaml,
    RHyprConfig,
    NixOS,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigItem {
    pub key: String,
    pub value: String,
    pub description: String,
    pub data_type: ConfigDataType,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ConfigDataType {
    Integer { min: Option<i32>, max: Option<i32> },
    Float { min: Option<f32>, max: Option<f32> },
    Boolean,
    String,
    Color,
    Keyword { options: Vec<String> },
}

pub struct UI {
    pub general_list_state: ListState,
    pub input_list_state: ListState,
    pub decoration_list_state: ListState,
    pub animations_list_state: ListState,
    pub gestures_list_state: ListState,
    pub binds_list_state: ListState,
    pub window_rules_list_state: ListState,
    pub layer_rules_list_state: ListState,
    pub misc_list_state: ListState,

    // Tab navigation state
    pub current_tab: FocusedPanel,

    // New editing state
    pub edit_mode: EditMode,
    pub editing_item: Option<(FocusedPanel, String)>,
    pub show_popup: bool,
    pub popup_message: String,
    pub show_save_dialog: bool,
    pub show_reload_dialog: bool,

    // Search functionality
    pub search_mode: bool,
    pub search_query: String,
    pub search_cursor: usize,

    // Help system
    pub show_help: bool,
    pub help_scroll: usize,

    // Debounced search
    pub search_debounce_delay: std::time::Duration,
    pub last_search_input: std::time::Instant,
    pub pending_search_query: String,
    pub debounced_search_active: bool,

    // Search result caching
    pub search_cache: std::collections::HashMap<String, Vec<ConfigItem>>,
    pub search_cache_max_size: usize,

    // Progressive search for large datasets
    pub progressive_search_threshold: usize,
    pub progressive_search_chunk_size: usize,

    // Theme
    pub theme: crate::theme::Theme,

    // Real-time preview functionality
    pub preview_mode: bool,
    pub preview_debounce_delay: std::time::Duration,
    pub last_preview_time: std::time::Instant,
    pub pending_preview_change: Option<(String, String)>, // (key, value)
    pub preview_original_value: Option<String>,           // Store original value for rollback
    pub pending_deletion: Option<(FocusedPanel, String)>, // (panel, key) for items pending deletion

    // Lazy loading / pagination support
    pub page_size: usize,
    pub current_page: std::collections::HashMap<FocusedPanel, usize>,
    pub total_pages: std::collections::HashMap<FocusedPanel, usize>,

    // Virtualization support
    pub item_height: usize, // Height per item (including description line)
    pub item_cache_generation: usize, // Cache invalidation counter

    pub config_items: std::collections::HashMap<FocusedPanel, Vec<ConfigItem>>,

    // NixOS environment information
    pub nixos_env: NixOSEnvironment,

    // NixOS export dialog state
    pub show_nixos_export_dialog: bool,
    pub nixos_export_config_type: crate::nixos::NixConfigType,
    pub nixos_export_preview: Option<String>,

    // Batch management dialog state
    pub show_batch_dialog: bool,
    pub batch_dialog_mode: BatchDialogMode,
    pub batch_selected_profile: Option<String>,
    pub batch_operation_type: crate::batch::BatchOperationType,

    // Visual preview system for configuration changes
    pub show_preview_dialog: bool,
    pub preview_before: Option<String>,
    pub preview_after: Option<String>,
    pub preview_setting_name: String,
    pub preview_scroll: usize,

    // Import/Export dialog state
    pub show_import_dialog: bool,
    pub show_export_dialog: bool,
    pub import_export_mode: ImportExportMode,
    pub selected_import_source: ImportSourceType,
    pub selected_export_format: ExportFormatType,
    pub import_preview: Option<String>,
    pub export_preview: Option<String>,
    pub import_export_scroll: usize,
    pub import_list_state: ListState,
    pub export_list_state: ListState,
}

impl UI {
    pub fn new() -> Self {
        let mut ui = Self {
            general_list_state: ListState::default(),
            input_list_state: ListState::default(),
            decoration_list_state: ListState::default(),
            animations_list_state: ListState::default(),
            gestures_list_state: ListState::default(),
            binds_list_state: ListState::default(),
            window_rules_list_state: ListState::default(),
            layer_rules_list_state: ListState::default(),
            misc_list_state: ListState::default(),

            current_tab: FocusedPanel::General,

            edit_mode: EditMode::None,
            editing_item: None,
            show_popup: false,
            popup_message: String::new(),
            show_save_dialog: false,
            show_reload_dialog: false,

            // Search functionality
            search_mode: false,
            search_query: String::new(),
            search_cursor: 0,

            // Help system
            show_help: false,
            help_scroll: 0,

            // Debounced search
            search_debounce_delay: std::time::Duration::from_millis(300), // 300ms debounce
            last_search_input: std::time::Instant::now(),
            pending_search_query: String::new(),
            debounced_search_active: false,

            // Search result caching
            search_cache: std::collections::HashMap::new(),
            search_cache_max_size: 50, // Cache up to 50 recent searches

            // Progressive search
            progressive_search_threshold: 1000, // Use progressive search for 1000+ items
            progressive_search_chunk_size: 100, // Process 100 items per chunk

            // Theme
            theme: crate::theme::Theme::default(),

            // Real-time preview functionality
            preview_mode: false, // Start with preview mode disabled
            preview_debounce_delay: std::time::Duration::from_millis(100), // 100ms debounce for responsive previews
            last_preview_time: std::time::Instant::now(),
            pending_preview_change: None,
            preview_original_value: None,
            pending_deletion: None,

            // Lazy loading / pagination
            page_size: 50, // Show 50 items per page for smooth performance
            current_page: std::collections::HashMap::new(),
            total_pages: std::collections::HashMap::new(),

            // Virtualization
            item_height: 3, // Each item takes 3 lines (key+value, description, spacing)
            item_cache_generation: 0,

            config_items: std::collections::HashMap::new(),

            // NixOS environment detection
            nixos_env: NixOSEnvironment::detect(),

            // NixOS export dialog
            show_nixos_export_dialog: false,
            nixos_export_config_type: crate::nixos::NixConfigType::HomeManager,
            nixos_export_preview: None,

            // Batch management dialog
            show_batch_dialog: false,
            batch_dialog_mode: BatchDialogMode::ManageProfiles,
            batch_selected_profile: None,
            batch_operation_type: crate::batch::BatchOperationType::Apply,

            // Visual preview system
            show_preview_dialog: false,
            preview_before: None,
            preview_after: None,
            preview_setting_name: String::new(),
            preview_scroll: 0,

            // Import/Export dialog system
            show_import_dialog: false,
            show_export_dialog: false,
            import_export_mode: ImportExportMode::SelectSource,
            selected_import_source: ImportSourceType::LocalFile,
            selected_export_format: ExportFormatType::HyprlandConf,
            import_preview: None,
            export_preview: None,
            import_export_scroll: 0,
            import_list_state: ListState::default(),
            export_list_state: ListState::default(),
        };

        // Initialize with first item selected for each panel
        ui.general_list_state.select(Some(0));
        ui.input_list_state.select(Some(0));
        ui.decoration_list_state.select(Some(0));
        ui.animations_list_state.select(Some(0));
        ui.gestures_list_state.select(Some(0));
        ui.binds_list_state.select(Some(0));
        ui.window_rules_list_state.select(Some(0));
        ui.layer_rules_list_state.select(Some(0));
        ui.misc_list_state.select(Some(0));
        ui.import_list_state.select(Some(0));
        ui.export_list_state.select(Some(0));

        ui.initialize_config_items();

        ui
    }

    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = self.current_tab.previous();
    }

    pub fn get_current_list_state(&mut self) -> &mut ListState {
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

    fn initialize_config_items(&mut self) {
        // Initialize with default/placeholder values
        // These will be updated with real values from hyprctl in load_current_config
        self.initialize_default_config_items();
    }

    fn initialize_default_config_items(&mut self) {
        // General configuration items
        let general_items = vec![
            ConfigItem {
                key: "gaps_in".to_string(),
                value: "5".to_string(),
                description: "Inner gaps between windows".to_string(),
                data_type: ConfigDataType::Integer {
                    min: Some(0),
                    max: Some(50),
                },
                suggestions: vec![
                    "0".to_string(),
                    "5".to_string(),
                    "10".to_string(),
                    "15".to_string(),
                ],
            },
            ConfigItem {
                key: "gaps_out".to_string(),
                value: "20".to_string(),
                description: "Outer gaps between windows and monitor edges".to_string(),
                data_type: ConfigDataType::Integer {
                    min: Some(0),
                    max: Some(100),
                },
                suggestions: vec!["10".to_string(), "20".to_string(), "30".to_string()],
            },
            ConfigItem {
                key: "border_size".to_string(),
                value: "2".to_string(),
                description: "Border width in pixels".to_string(),
                data_type: ConfigDataType::Integer {
                    min: Some(0),
                    max: Some(20),
                },
                suggestions: vec![
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                    "4".to_string(),
                ],
            },
            ConfigItem {
                key: "col.active_border".to_string(),
                value: "rgba(33ccffee)".to_string(),
                description: "Active window border color".to_string(),
                data_type: ConfigDataType::Color,
                suggestions: vec![
                    "rgba(33ccffee)".to_string(),
                    "rgba(ff6666ee)".to_string(),
                    "rgba(66ff66ee)".to_string(),
                    "rgba(ffff66ee)".to_string(),
                    "rgb(255, 255, 255)".to_string(),
                ],
            },
            ConfigItem {
                key: "col.inactive_border".to_string(),
                value: "rgba(595959aa)".to_string(),
                description: "Inactive window border color".to_string(),
                data_type: ConfigDataType::Color,
                suggestions: vec![
                    "rgba(595959aa)".to_string(),
                    "rgba(333333aa)".to_string(),
                    "rgba(666666aa)".to_string(),
                ],
            },
        ];

        // Input configuration items
        let input_items = vec![
            ConfigItem {
                key: "kb_layout".to_string(),
                value: "us".to_string(),
                description: "Keyboard layout".to_string(),
                data_type: ConfigDataType::Keyword {
                    options: vec![
                        "us".to_string(),
                        "uk".to_string(),
                        "de".to_string(),
                        "fr".to_string(),
                        "es".to_string(),
                    ],
                },
                suggestions: vec!["us".to_string(), "uk".to_string(), "de".to_string()],
            },
            ConfigItem {
                key: "follow_mouse".to_string(),
                value: "1".to_string(),
                description: "Follow mouse focus behavior".to_string(),
                data_type: ConfigDataType::Keyword {
                    options: vec![
                        "0".to_string(),
                        "1".to_string(),
                        "2".to_string(),
                        "3".to_string(),
                    ],
                },
                suggestions: vec!["0".to_string(), "1".to_string(), "2".to_string()],
            },
            ConfigItem {
                key: "sensitivity".to_string(),
                value: "0.0".to_string(),
                description: "Mouse sensitivity (-1.0 to 1.0)".to_string(),
                data_type: ConfigDataType::Float {
                    min: Some(-1.0),
                    max: Some(1.0),
                },
                suggestions: vec!["-0.5".to_string(), "0.0".to_string(), "0.5".to_string()],
            },
        ];

        // Decoration items
        let decoration_items = vec![
            ConfigItem {
                key: "rounding".to_string(),
                value: "10".to_string(),
                description: "Window corner rounding in pixels".to_string(),
                data_type: ConfigDataType::Integer {
                    min: Some(0),
                    max: Some(50),
                },
                suggestions: vec![
                    "0".to_string(),
                    "5".to_string(),
                    "10".to_string(),
                    "15".to_string(),
                ],
            },
            ConfigItem {
                key: "blur.enabled".to_string(),
                value: "true".to_string(),
                description: "Enable window blur effect".to_string(),
                data_type: ConfigDataType::Boolean,
                suggestions: vec!["true".to_string(), "false".to_string()],
            },
            ConfigItem {
                key: "blur.size".to_string(),
                value: "3".to_string(),
                description: "Blur effect radius".to_string(),
                data_type: ConfigDataType::Integer {
                    min: Some(1),
                    max: Some(20),
                },
                suggestions: vec![
                    "1".to_string(),
                    "3".to_string(),
                    "5".to_string(),
                    "8".to_string(),
                ],
            },
        ];

        self.config_items
            .insert(FocusedPanel::General, general_items);
        self.config_items.insert(FocusedPanel::Input, input_items);
        self.config_items
            .insert(FocusedPanel::Decoration, decoration_items);

        // Animations configuration items
        let animations_items = vec![
            ConfigItem {
                key: "animations.enabled".to_string(),
                value: "true".to_string(),
                description: "Enable/disable animations globally".to_string(),
                data_type: ConfigDataType::Boolean,
                suggestions: vec!["true".to_string(), "false".to_string()],
            },
            ConfigItem {
                key: "bezier.myBezier".to_string(),
                value: "0.05, 0.9, 0.1, 1.05".to_string(),
                description: "Custom bezier curve for animations".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "0.05, 0.9, 0.1, 1.05".to_string(),
                    "0.25, 0.46, 0.45, 0.94".to_string(),
                    "0.16, 1, 0.3, 1".to_string(),
                ],
            },
            ConfigItem {
                key: "animation.windows".to_string(),
                value: "1, 7, myBezier".to_string(),
                description: "Window open/close animation settings".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "1, 7, myBezier".to_string(),
                    "1, 5, default".to_string(),
                    "0".to_string(),
                ],
            },
            ConfigItem {
                key: "animation.fade".to_string(),
                value: "1, 7, default".to_string(),
                description: "Fade in/out animation settings".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "1, 7, default".to_string(),
                    "1, 5, default".to_string(),
                    "0".to_string(),
                ],
            },
            ConfigItem {
                key: "animation.workspaces".to_string(),
                value: "1, 6, default".to_string(),
                description: "Workspace switching animation".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "1, 6, default".to_string(),
                    "1, 4, default, slidevert".to_string(),
                    "1, 8, default, slide".to_string(),
                ],
            },
        ];

        // Gestures configuration items
        let gestures_items = vec![
            ConfigItem {
                key: "gestures.workspace_swipe".to_string(),
                value: "false".to_string(),
                description: "Enable workspace swiping with touchpad".to_string(),
                data_type: ConfigDataType::Boolean,
                suggestions: vec!["true".to_string(), "false".to_string()],
            },
            ConfigItem {
                key: "gestures.workspace_swipe_fingers".to_string(),
                value: "3".to_string(),
                description: "Number of fingers for workspace swipe".to_string(),
                data_type: ConfigDataType::Integer {
                    min: Some(3),
                    max: Some(5),
                },
                suggestions: vec!["3".to_string(), "4".to_string(), "5".to_string()],
            },
            ConfigItem {
                key: "gestures.workspace_swipe_distance".to_string(),
                value: "300".to_string(),
                description: "Distance in pixels to trigger swipe".to_string(),
                data_type: ConfigDataType::Integer {
                    min: Some(100),
                    max: Some(1000),
                },
                suggestions: vec!["200".to_string(), "300".to_string(), "400".to_string()],
            },
            ConfigItem {
                key: "gestures.workspace_swipe_invert".to_string(),
                value: "true".to_string(),
                description: "Invert swipe direction".to_string(),
                data_type: ConfigDataType::Boolean,
                suggestions: vec!["true".to_string(), "false".to_string()],
            },
        ];

        // Key binds configuration items
        let binds_items = vec![
            ConfigItem {
                key: "$mainMod".to_string(),
                value: "SUPER".to_string(),
                description: "Main modifier key for keybindings".to_string(),
                data_type: ConfigDataType::Keyword {
                    options: vec![
                        "SUPER".to_string(),
                        "ALT".to_string(),
                        "CTRL".to_string(),
                        "SHIFT".to_string(),
                    ],
                },
                suggestions: vec!["SUPER".to_string(), "ALT".to_string(), "CTRL".to_string()],
            },
            ConfigItem {
                key: "bind[terminal]".to_string(),
                value: "$mainMod, Q, exec, kitty".to_string(),
                description: "Open terminal application".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "$mainMod, Q, exec, kitty".to_string(),
                    "$mainMod, Return, exec, alacritty".to_string(),
                    "$mainMod, T, exec, wezterm".to_string(),
                ],
            },
            ConfigItem {
                key: "bind[kill]".to_string(),
                value: "$mainMod, C, killactive,".to_string(),
                description: "Close active window".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "$mainMod, C, killactive,".to_string(),
                    "$mainMod, Q, killactive,".to_string(),
                    "$mainMod SHIFT, C, killactive,".to_string(),
                ],
            },
            ConfigItem {
                key: "bind[launcher]".to_string(),
                value: "$mainMod, R, exec, wofi --show drun".to_string(),
                description: "Application launcher".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "$mainMod, R, exec, wofi --show drun".to_string(),
                    "$mainMod, D, exec, rofi -show drun".to_string(),
                    "$mainMod, Space, exec, bemenu-run".to_string(),
                ],
            },
            ConfigItem {
                key: "bind[floating]".to_string(),
                value: "$mainMod, V, togglefloating,".to_string(),
                description: "Toggle floating mode for active window".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "$mainMod, V, togglefloating,".to_string(),
                    "$mainMod, F, togglefloating,".to_string(),
                    "$mainMod SHIFT, Space, togglefloating,".to_string(),
                ],
            },
        ];

        // Window rules configuration items
        let window_rules_items = vec![
            ConfigItem {
                key: "windowrule[float_kitty]".to_string(),
                value: "float, ^(kitty)$".to_string(),
                description: "Make kitty terminal windows float".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "float, ^(kitty)$".to_string(),
                    "tile, ^(kitty)$".to_string(),
                    "fullscreen, ^(kitty)$".to_string(),
                ],
            },
            ConfigItem {
                key: "windowrule[opacity_alacritty]".to_string(),
                value: "opacity 0.8 0.8, ^(Alacritty)$".to_string(),
                description: "Set Alacritty terminal opacity".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "opacity 0.8 0.8, ^(Alacritty)$".to_string(),
                    "opacity 0.9 0.9, ^(Alacritty)$".to_string(),
                    "opacity 1.0 1.0, ^(Alacritty)$".to_string(),
                ],
            },
            ConfigItem {
                key: "windowrule[size_pavucontrol]".to_string(),
                value: "size 800 600, ^(pavucontrol)$".to_string(),
                description: "Set fixed size for PulseAudio Volume Control".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "size 800 600, ^(pavucontrol)$".to_string(),
                    "size 600 400, ^(pavucontrol)$".to_string(),
                    "size 1000 700, ^(pavucontrol)$".to_string(),
                ],
            },
            ConfigItem {
                key: "windowrulev2[firefox_pip]".to_string(),
                value: "float, class:^(firefox)$, title:^(Picture-in-Picture)$".to_string(),
                description: "Float Firefox picture-in-picture windows".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "float, class:^(firefox)$, title:^(Picture-in-Picture)$".to_string(),
                    "pin, class:^(firefox)$, title:^(Picture-in-Picture)$".to_string(),
                    "size 400 300, class:^(firefox)$, title:^(Picture-in-Picture)$".to_string(),
                ],
            },
        ];

        // Layer rules configuration items
        let layer_rules_items = vec![
            ConfigItem {
                key: "layerrule[blur_rofi]".to_string(),
                value: "blur, rofi".to_string(),
                description: "Apply blur effect to rofi launcher".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "blur, rofi".to_string(),
                    "noblur, rofi".to_string(),
                    "ignorezero, rofi".to_string(),
                ],
            },
            ConfigItem {
                key: "layerrule[blur_notifications]".to_string(),
                value: "blur, notifications".to_string(),
                description: "Apply blur to notification layer".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "blur, notifications".to_string(),
                    "noblur, notifications".to_string(),
                    "ignorealpha 0.8, notifications".to_string(),
                ],
            },
            ConfigItem {
                key: "layerrule[blur_waybar]".to_string(),
                value: "blur, waybar".to_string(),
                description: "Apply blur effect to waybar".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "blur, waybar".to_string(),
                    "noblur, waybar".to_string(),
                    "ignorezero, waybar".to_string(),
                ],
            },
        ];

        // Misc configuration items
        let misc_items = vec![
            ConfigItem {
                key: "misc.disable_hyprland_logo".to_string(),
                value: "false".to_string(),
                description: "Disable Hyprland logo on empty workspace".to_string(),
                data_type: ConfigDataType::Boolean,
                suggestions: vec!["true".to_string(), "false".to_string()],
            },
            ConfigItem {
                key: "misc.disable_splash_rendering".to_string(),
                value: "false".to_string(),
                description: "Disable splash screen on startup".to_string(),
                data_type: ConfigDataType::Boolean,
                suggestions: vec!["true".to_string(), "false".to_string()],
            },
            ConfigItem {
                key: "misc.mouse_move_enables_dpms".to_string(),
                value: "true".to_string(),
                description: "Wake up displays on mouse movement".to_string(),
                data_type: ConfigDataType::Boolean,
                suggestions: vec!["true".to_string(), "false".to_string()],
            },
            ConfigItem {
                key: "misc.vfr".to_string(),
                value: "true".to_string(),
                description: "Variable Frame Rate - saves power".to_string(),
                data_type: ConfigDataType::Boolean,
                suggestions: vec!["true".to_string(), "false".to_string()],
            },
            ConfigItem {
                key: "misc.vrr".to_string(),
                value: "0".to_string(),
                description: "Variable Refresh Rate (0=off, 1=on, 2=fullscreen only)".to_string(),
                data_type: ConfigDataType::Keyword {
                    options: vec!["0".to_string(), "1".to_string(), "2".to_string()],
                },
                suggestions: vec!["0".to_string(), "1".to_string(), "2".to_string()],
            },
        ];

        // Insert configuration items only if they don't already exist
        // This prevents overwriting dynamically loaded data from hyprctl
        self.config_items
            .entry(FocusedPanel::Animations)
            .or_insert(animations_items);
        self.config_items
            .entry(FocusedPanel::Gestures)
            .or_insert(gestures_items);
        self.config_items
            .entry(FocusedPanel::Binds)
            .or_insert(binds_items);
        self.config_items
            .entry(FocusedPanel::WindowRules)
            .or_insert(window_rules_items);
        self.config_items
            .entry(FocusedPanel::LayerRules)
            .or_insert(layer_rules_items);
        self.config_items
            .entry(FocusedPanel::Misc)
            .or_insert(misc_items);
    }

    pub fn collect_all_config_changes(&self) -> std::collections::HashMap<String, String> {
        let mut options = std::collections::HashMap::new();

        // Collect changes from all panels
        for (panel, items) in &self.config_items {
            for item in items {
                // Only include items that have hyprctl mappings (skip keybinds/rules for now)
                if let Some(hypr_key) = self.get_hyprctl_key(panel, &item.key) {
                    options.insert(hypr_key, item.value.clone());
                }
            }
        }

        options
    }

    pub fn collect_keybinds(&self) -> Vec<String> {
        let mut keybinds = Vec::new();

        if let Some(bind_items) = self.config_items.get(&crate::app::FocusedPanel::Binds) {
            for item in bind_items {
                // Convert display format back to config format
                // Display format: "SUPER + q → exec [kitty]"
                // Config format: "bind = SUPER, q, exec, kitty"

                if let Some(config_line) = self.display_value_to_config_line(&item.value) {
                    keybinds.push(config_line);
                }
            }
        }

        keybinds
    }

    pub fn collect_window_rules(&self) -> Vec<String> {
        let mut rules = Vec::new();

        if let Some(rule_items) = self
            .config_items
            .get(&crate::app::FocusedPanel::WindowRules)
        {
            for item in rule_items {
                rules.push(item.value.clone());
            }
        }

        rules
    }

    pub fn collect_layer_rules(&self) -> Vec<String> {
        let mut rules = Vec::new();

        if let Some(rule_items) = self.config_items.get(&crate::app::FocusedPanel::LayerRules) {
            for item in rule_items {
                rules.push(item.value.clone());
            }
        }

        rules
    }

    fn display_value_to_config_line(&self, display_value: &str) -> Option<String> {
        // Convert display format "SUPER + q → exec [kitty]" back to config format
        // "bind = SUPER, q, exec, kitty"

        if let Some((key_part, command_part)) = display_value.split_once(" → ") {
            let key_part = key_part.trim();
            let command_part = command_part.trim();

            // Parse key part "SUPER + q" or just "q"
            let (modifiers, key) = if key_part.contains(" + ") {
                let parts: Vec<&str> = key_part.split(" + ").collect();
                let key = parts.last().unwrap_or(&"").to_string();
                let mods = parts[..parts.len() - 1].join(" ");
                (mods, key)
            } else {
                (String::new(), key_part.to_string())
            };

            // Parse command part "exec [kitty]" or "killactive"
            let (dispatcher, args) = if command_part.contains(" [") && command_part.ends_with(']') {
                let parts: Vec<&str> = command_part.splitn(2, " [").collect();
                let dispatcher = parts[0].to_string();
                let args = parts
                    .get(1)
                    .map(|s| s.trim_end_matches(']'))
                    .unwrap_or("")
                    .to_string();
                (dispatcher, Some(args))
            } else {
                (command_part.to_string(), None)
            };

            // Format as config line
            let mod_part = if modifiers.is_empty() {
                String::new()
            } else {
                format!("{modifiers}, ")
            };

            let args_part = if let Some(args) = args {
                format!(", {args}")
            } else {
                String::new()
            };

            Some(format!("bind = {mod_part}{key}, {dispatcher}{args_part}"))
        } else {
            None
        }
    }

    pub async fn load_current_config(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        // Try to load from hyprctl first
        let _hyprctl_success = match hyprctl.get_all_options().await {
            Ok(all_options) => {
                self.populate_config_from_options(all_options);
                true
            }
            Err(e) => {
                eprintln!("Warning: Failed to load all options from hyprctl: {e}");
                // Fall back to loading individual sections
                let mut sections_loaded = 0;
                if self.load_general_config(hyprctl).await.is_ok() {
                    sections_loaded += 1;
                }
                if self.load_input_config(hyprctl).await.is_ok() {
                    sections_loaded += 1;
                }
                if self.load_decoration_config(hyprctl).await.is_ok() {
                    sections_loaded += 1;
                }
                if self.load_animations_config(hyprctl).await.is_ok() {
                    sections_loaded += 1;
                }
                if self.load_gestures_config(hyprctl).await.is_ok() {
                    sections_loaded += 1;
                }
                if self.load_misc_config(hyprctl).await.is_ok() {
                    sections_loaded += 1;
                }
                sections_loaded > 0
            }
        };

        // Try to load keybinds, window rules, and layer rules from hyprctl
        let binds_success = self.load_binds_config(hyprctl).await.is_ok();
        let window_rules_success = self.load_window_rules_config(hyprctl).await.is_ok();
        let layer_rules_success = self.load_layer_rules_config(hyprctl).await.is_ok();

        // If hyprctl failed for rules, try to load from config file
        if !binds_success || !window_rules_success || !layer_rules_success {
            eprintln!("Debug: hyprctl failed (binds: {binds_success}, window_rules: {window_rules_success}, layer_rules: {layer_rules_success}), trying config file");
            if let Err(e) = self.load_from_config_file().await {
                eprintln!("Warning: Failed to load from config file: {e}");
                // As a last resort, add placeholder data
                self.add_fallback_placeholder_data();
            }
        } else {
            eprintln!("Debug: hyprctl succeeded, not loading from config file");
        }

        // Update pagination for all panels after loading config
        self.update_all_pagination();

        // Optimize memory usage after loading large configuration sets
        self.optimize_memory_usage();

        Ok(())
    }

    async fn load_from_config_file(&mut self) -> Result<(), anyhow::Error> {
        let config = crate::config::Config::load().await?;
        eprintln!(
            "Debug: Config loaded, path: {:?}",
            config.hyprland_config_path
        );

        let hyprland_config = config.parse_hyprland_config().await?;
        eprintln!(
            "Debug: Parsed {} keybinds, {} window rules, {} layer rules",
            hyprland_config.keybinds.len(),
            hyprland_config.window_rules.len(),
            hyprland_config.layer_rules.len()
        );

        // Load keybinds from config file
        if !hyprland_config.keybinds.is_empty() {
            let mut bind_items = Vec::new();

            for (i, keybind) in hyprland_config.keybinds.iter().enumerate() {
                let key = format!("bind_{i}");
                let display_value = format!(
                    "{} {} → {} {}",
                    keybind.modifiers,
                    keybind.key,
                    keybind.dispatcher,
                    if keybind.args.is_empty() {
                        "".to_string()
                    } else {
                        format!("[{}]", keybind.args)
                    }
                );

                bind_items.push(crate::ui::ConfigItem {
                    key: key.clone(),
                    value: display_value,
                    description: format!(
                        "Keybind: {} {} -> {}",
                        keybind.modifiers, keybind.key, keybind.dispatcher
                    ),
                    data_type: crate::ui::ConfigDataType::String,
                    suggestions: self.get_keybind_suggestions(&keybind.dispatcher),
                });
            }

            if !bind_items.is_empty() {
                self.config_items
                    .insert(crate::app::FocusedPanel::Binds, bind_items);
            }
        }

        // Load window rules from config file
        if !hyprland_config.window_rules.is_empty() {
            let mut rule_items = Vec::new();

            for (i, rule) in hyprland_config.window_rules.iter().enumerate() {
                let key = format!("window_rule_{i}");

                rule_items.push(crate::ui::ConfigItem {
                    key: key.clone(),
                    value: rule.clone(),
                    description: format!("Window rule: {rule}"),
                    data_type: crate::ui::ConfigDataType::String,
                    suggestions: self.get_window_rule_suggestions(),
                });
            }

            if !rule_items.is_empty() {
                self.config_items
                    .insert(crate::app::FocusedPanel::WindowRules, rule_items);
            }
        }

        // Load layer rules from config file
        if !hyprland_config.layer_rules.is_empty() || !hyprland_config.workspace_rules.is_empty() {
            let mut rule_items = Vec::new();

            // Add layer rules
            for (i, rule) in hyprland_config.layer_rules.iter().enumerate() {
                let key = format!("layer_rule_{i}");

                rule_items.push(crate::ui::ConfigItem {
                    key: key.clone(),
                    value: rule.clone(),
                    description: format!("Layer rule: {rule}"),
                    data_type: crate::ui::ConfigDataType::String,
                    suggestions: self.get_layer_rule_suggestions(),
                });
            }

            // Add workspace rules
            for (i, rule) in hyprland_config.workspace_rules.iter().enumerate() {
                let key = format!("workspace_rule_{i}");

                rule_items.push(crate::ui::ConfigItem {
                    key: key.clone(),
                    value: rule.clone(),
                    description: format!("Workspace rule: {rule}"),
                    data_type: crate::ui::ConfigDataType::String,
                    suggestions: self.get_workspace_rule_suggestions(),
                });
            }

            if !rule_items.is_empty() {
                self.config_items
                    .insert(crate::app::FocusedPanel::LayerRules, rule_items);
            }
        }

        Ok(())
    }

    fn add_fallback_placeholder_data(&mut self) {
        // Add placeholder keybinds if not already present
        self.config_items
            .entry(FocusedPanel::Binds)
            .or_insert_with(|| {
                let placeholder_binds = vec![
                    ConfigItem {
                        key: "bind_example_1".to_string(),
                        value: "SUPER + Q → exec [kitty]".to_string(),
                        description: "Example: Open terminal with Super+Q".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: vec![
                            "exec".to_string(),
                            "killactive".to_string(),
                            "togglefloating".to_string(),
                        ],
                    },
                    ConfigItem {
                        key: "hyprland_not_running".to_string(),
                        value: "⚠️  Configuration not available".to_string(),
                        description: "Could not load from hyprctl or config file".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: vec![],
                    },
                ];
                placeholder_binds
            });

        // Add placeholder window rules if not already present
        if !self.config_items.contains_key(&FocusedPanel::WindowRules) {
            let placeholder_rules = vec![
                ConfigItem {
                    key: "window_rule_example_1".to_string(),
                    value: "windowrule = float, ^(kitty)$".to_string(),
                    description: "Example: Float kitty terminal windows".to_string(),
                    data_type: ConfigDataType::String,
                    suggestions: self.get_window_rule_suggestions(),
                },
                ConfigItem {
                    key: "hyprland_not_running".to_string(),
                    value: "⚠️  Configuration not available".to_string(),
                    description: "Could not load from hyprctl or config file".to_string(),
                    data_type: ConfigDataType::String,
                    suggestions: vec![],
                },
            ];
            self.config_items
                .insert(FocusedPanel::WindowRules, placeholder_rules);
        }

        // Add placeholder layer rules if not already present
        if !self.config_items.contains_key(&FocusedPanel::LayerRules) {
            let placeholder_layer_rules = vec![
                ConfigItem {
                    key: "layer_rule_example_1".to_string(),
                    value: "layerrule = blur, waybar".to_string(),
                    description: "Example: Apply blur effect to waybar".to_string(),
                    data_type: ConfigDataType::String,
                    suggestions: self.get_layer_rule_suggestions(),
                },
                ConfigItem {
                    key: "hyprland_not_running".to_string(),
                    value: "⚠️  Configuration not available".to_string(),
                    description: "Could not load from hyprctl or config file".to_string(),
                    data_type: ConfigDataType::String,
                    suggestions: vec![],
                },
            ];
            self.config_items
                .insert(FocusedPanel::LayerRules, placeholder_layer_rules);
        }
    }

    fn populate_config_from_options(&mut self, options: std::collections::HashMap<String, String>) {
        // Clear existing items and populate from actual hyprctl data
        self.config_items.clear();

        // Initialize empty vectors for each panel
        let mut general_items = Vec::new();
        let mut input_items = Vec::new();
        let mut decoration_items = Vec::new();
        let mut animation_items = Vec::new();
        let mut gesture_items = Vec::new();
        let mut misc_items = Vec::new();

        // Process all options and categorize them
        for (key, value) in options {
            let parsed_value = Self::parse_hyprctl_value(&value).unwrap_or(value.clone());

            let config_item = ConfigItem {
                key: key.clone(),
                value: parsed_value,
                description: self.get_option_description(&key),
                data_type: self.infer_data_type(&key, &value),
                suggestions: self.get_option_suggestions(&key),
            };

            // Categorize based on option prefix
            if key.starts_with("general:") {
                general_items.push(config_item);
            } else if key.starts_with("input:") {
                input_items.push(config_item);
            } else if key.starts_with("decoration:") {
                decoration_items.push(config_item);
            } else if key.starts_with("animations:") {
                animation_items.push(config_item);
            } else if key.starts_with("gestures:") {
                gesture_items.push(config_item);
            } else if key.starts_with("misc:") {
                misc_items.push(config_item);
            }
        }

        // Sort items by key name for consistent display
        general_items.sort_by(|a, b| a.key.cmp(&b.key));
        input_items.sort_by(|a, b| a.key.cmp(&b.key));
        decoration_items.sort_by(|a, b| a.key.cmp(&b.key));
        animation_items.sort_by(|a, b| a.key.cmp(&b.key));
        gesture_items.sort_by(|a, b| a.key.cmp(&b.key));
        misc_items.sort_by(|a, b| a.key.cmp(&b.key));

        // Insert into config_items
        if !general_items.is_empty() {
            self.config_items
                .insert(FocusedPanel::General, general_items);
        }
        if !input_items.is_empty() {
            self.config_items.insert(FocusedPanel::Input, input_items);
        }
        if !decoration_items.is_empty() {
            self.config_items
                .insert(FocusedPanel::Decoration, decoration_items);
        }
        if !animation_items.is_empty() {
            self.config_items
                .insert(FocusedPanel::Animations, animation_items);
        }
        if !gesture_items.is_empty() {
            self.config_items
                .insert(FocusedPanel::Gestures, gesture_items);
        }
        if !misc_items.is_empty() {
            self.config_items.insert(FocusedPanel::Misc, misc_items);

            // Import configuration panel items
            let import_items = vec![
            ConfigItem {
                key: "local_file".to_string(),
                value: "Select a file to import".to_string(),
                description: "Import configuration from a local Hyprland config file (.conf, .json, .toml, .yaml)".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "~/.config/hypr/hyprland.conf".to_string(),
                    "./hyprland.conf".to_string(),
                    "./config.json".to_string(),
                ],
            },
            ConfigItem {
                key: "local_folder".to_string(),
                value: "Select a folder to scan".to_string(),
                description: "Scan and import from a directory containing Hyprland configuration files".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "~/.config/hypr/".to_string(),
                    "./dotfiles/hypr/".to_string(),
                    "~/Downloads/hypr-configs/".to_string(),
                ],
            },
            ConfigItem {
                key: "github_repo".to_string(),
                value: "Enter GitHub repository URL".to_string(),
                description: "Import configuration from a GitHub repository (dotfiles, configs, etc.)".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "https://github.com/user/dotfiles".to_string(),
                    "https://github.com/user/hyprland-config".to_string(),
                    "user/dotfiles".to_string(),
                ],
            },
            ConfigItem {
                key: "url_download".to_string(),
                value: "Enter direct URL".to_string(),
                description: "Import configuration from a direct URL (pastebin, gist, raw file)".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![
                    "https://pastebin.com/raw/...".to_string(),
                    "https://gist.githubusercontent.com/...".to_string(),
                    "https://raw.githubusercontent.com/...".to_string(),
                ],
            },
        ];
            self.config_items.insert(FocusedPanel::Import, import_items);

            // Export configuration panel items
            let export_items = vec![
                ConfigItem {
                    key: "hyprland_conf".to_string(),
                    value: "Standard Hyprland format".to_string(),
                    description: "Export as standard hyprland.conf file compatible with Hyprland"
                        .to_string(),
                    data_type: ConfigDataType::String,
                    suggestions: vec![
                        "hyprland_export.conf".to_string(),
                        "my_hyprland_config.conf".to_string(),
                    ],
                },
                ConfigItem {
                    key: "json_format".to_string(),
                    value: "Structured JSON format".to_string(),
                    description: "Export as JSON with hierarchical structure and metadata"
                        .to_string(),
                    data_type: ConfigDataType::String,
                    suggestions: vec![
                        "config_export.json".to_string(),
                        "hyprland_backup.json".to_string(),
                    ],
                },
                ConfigItem {
                    key: "toml_format".to_string(),
                    value: "Human-readable TOML".to_string(),
                    description: "Export as TOML configuration file for easy editing".to_string(),
                    data_type: ConfigDataType::String,
                    suggestions: vec![
                        "config_export.toml".to_string(),
                        "hyprland_settings.toml".to_string(),
                    ],
                },
                ConfigItem {
                    key: "yaml_format".to_string(),
                    value: "Clean YAML format".to_string(),
                    description: "Export as YAML with clean indentation and comments".to_string(),
                    data_type: ConfigDataType::String,
                    suggestions: vec![
                        "config_export.yaml".to_string(),
                        "hyprland_config.yml".to_string(),
                    ],
                },
                ConfigItem {
                    key: "rhypr_format".to_string(),
                    value: "R-Hyprconfig native format".to_string(),
                    description: "Export in r-hyprconfig native format with full feature support"
                        .to_string(),
                    data_type: ConfigDataType::String,
                    suggestions: vec![
                        "config_backup.rhypr".to_string(),
                        "my_hyprland_setup.rhypr".to_string(),
                    ],
                },
                ConfigItem {
                    key: "nixos_format".to_string(),
                    value: "NixOS declarative module".to_string(),
                    description:
                        "Export as NixOS configuration module for declarative system management"
                            .to_string(),
                    data_type: ConfigDataType::String,
                    suggestions: vec![
                        "hyprland_module.nix".to_string(),
                        "home_manager_hyprland.nix".to_string(),
                    ],
                },
            ];
            self.config_items.insert(FocusedPanel::Export, export_items);
        }
    }

    fn get_option_description(&self, key: &str) -> String {
        match key {
            // General options
            "general:gaps_in" => "Inner gaps between windows".to_string(),
            "general:gaps_out" => "Outer gaps between windows and monitor edges".to_string(),
            "general:border_size" => "Window border thickness".to_string(),
            "general:col.active_border" => "Color of active window border".to_string(),
            "general:col.inactive_border" => "Color of inactive window border".to_string(),
            "general:resize_on_border" => "Enable resizing by dragging border".to_string(),
            "general:extend_border_grab_area" => "Extended border grab area".to_string(),
            "general:hover_icon_on_border" => "Show resize cursor on border hover".to_string(),

            // Input options
            "input:kb_layout" => "Keyboard layout".to_string(),
            "input:kb_variant" => "Keyboard layout variant".to_string(),
            "input:kb_model" => "Keyboard model".to_string(),
            "input:kb_options" => "Keyboard options".to_string(),
            "input:kb_rules" => "Keyboard rules".to_string(),
            "input:follow_mouse" => "Mouse focus behavior".to_string(),
            "input:mouse_refocus" => "Refocus on mouse move".to_string(),
            "input:sensitivity" => "Mouse sensitivity".to_string(),
            "input:accel_profile" => "Mouse acceleration profile".to_string(),
            "input:natural_scroll" => "Natural scrolling".to_string(),

            // Decoration options
            "decoration:rounding" => "Window corner rounding".to_string(),
            "decoration:blur:enabled" => "Enable blur effects".to_string(),
            "decoration:blur:size" => "Blur effect size".to_string(),
            "decoration:blur:passes" => "Blur effect passes".to_string(),
            "decoration:drop_shadow" => "Enable drop shadows".to_string(),
            "decoration:shadow_range" => "Shadow range".to_string(),
            "decoration:shadow_render_power" => "Shadow render power".to_string(),
            "decoration:col.shadow" => "Shadow color".to_string(),
            "decoration:dim_inactive" => "Dim inactive windows".to_string(),
            "decoration:dim_strength" => "Dim strength".to_string(),

            // Animation options
            "animations:enabled" => "Enable animations".to_string(),

            // Gesture options
            "gestures:workspace_swipe" => "Enable workspace swiping".to_string(),
            "gestures:workspace_swipe_fingers" => "Fingers for workspace swipe".to_string(),
            "gestures:workspace_swipe_distance" => "Swipe distance threshold".to_string(),
            "gestures:workspace_swipe_invert" => "Invert swipe direction".to_string(),
            "gestures:workspace_swipe_min_speed_to_force" => "Min speed to force swipe".to_string(),
            "gestures:workspace_swipe_cancel_ratio" => "Swipe cancel ratio".to_string(),
            "gestures:workspace_swipe_create_new" => "Create new workspace on swipe".to_string(),
            "gestures:workspace_swipe_forever" => "Enable infinite workspace swipe".to_string(),

            // Misc options
            "misc:disable_hyprland_logo" => "Disable Hyprland logo".to_string(),
            "misc:disable_splash_rendering" => "Disable splash screen".to_string(),
            "misc:mouse_move_enables_dpms" => "Mouse movement enables DPMS".to_string(),
            "misc:key_press_enables_dpms" => "Key press enables DPMS".to_string(),
            "misc:always_follow_on_dnd" => "Always follow on drag and drop".to_string(),
            "misc:layers_hog_keyboard_focus" => "Layers hog keyboard focus".to_string(),
            "misc:animate_manual_resizes" => "Animate manual resizes".to_string(),
            "misc:animate_mouse_windowdragging" => "Animate mouse window dragging".to_string(),
            "misc:disable_autoreload" => "Disable auto-reload".to_string(),
            "misc:enable_swallow" => "Enable window swallowing".to_string(),
            "misc:swallow_regex" => "Swallow regex pattern".to_string(),

            _ => format!("Configuration option: {key}"),
        }
    }

    fn infer_data_type(&self, key: &str, value: &str) -> ConfigDataType {
        // Infer data type based on key patterns and value content
        match key {
            // Color options
            k if k.contains("col.") || k.contains("color") => ConfigDataType::Color,

            // Boolean options
            k if k.contains("enable")
                || k.contains("disable")
                || k.ends_with("_on_border")
                || k.contains("natural_scroll")
                || k.contains("mouse_refocus")
                || k.contains("resize_on_border") =>
            {
                ConfigDataType::Boolean
            }

            // Float options
            "input:sensitivity"
            | "decoration:dim_strength"
            | "gestures:workspace_swipe_cancel_ratio" => ConfigDataType::Float {
                min: Some(0.0),
                max: Some(10.0),
            },

            // Integer options with ranges
            "general:gaps_in" | "general:gaps_out" => ConfigDataType::Integer {
                min: Some(0),
                max: Some(100),
            },
            "general:border_size" => ConfigDataType::Integer {
                min: Some(0),
                max: Some(20),
            },
            "decoration:rounding" => ConfigDataType::Integer {
                min: Some(0),
                max: Some(50),
            },

            k if k.contains("size")
                || k.contains("range")
                || k.contains("passes")
                || k.contains("fingers")
                || k.contains("distance") =>
            {
                ConfigDataType::Integer {
                    min: Some(0),
                    max: Some(100),
                }
            }

            // Try to infer from value
            _ => {
                let trimmed = value.trim();
                if trimmed == "true" || trimmed == "false" || trimmed == "1" || trimmed == "0" {
                    ConfigDataType::Boolean
                } else if trimmed.parse::<i32>().is_ok() {
                    ConfigDataType::Integer {
                        min: None,
                        max: None,
                    }
                } else if trimmed.parse::<f32>().is_ok() {
                    ConfigDataType::Float {
                        min: None,
                        max: None,
                    }
                } else if trimmed.starts_with('#') || trimmed.starts_with("rgb") {
                    ConfigDataType::Color
                } else {
                    ConfigDataType::String
                }
            }
        }
    }

    fn get_option_suggestions(&self, key: &str) -> Vec<String> {
        match key {
            "input:follow_mouse" => vec!["0".to_string(), "1".to_string(), "2".to_string()],
            "input:accel_profile" => vec!["flat".to_string(), "adaptive".to_string()],
            "input:kb_layout" => vec![
                "us".to_string(),
                "de".to_string(),
                "fr".to_string(),
                "uk".to_string(),
            ],
            "general:col.active_border" => vec![
                "0xffff0000".to_string(),
                "0xff00ff00".to_string(),
                "0xff0000ff".to_string(),
                "0xffffffff".to_string(),
            ],
            "general:col.inactive_border" => vec![
                "0x66333333".to_string(),
                "0x66666666".to_string(),
                "0x66999999".to_string(),
            ],
            _ => vec![],
        }
    }

    fn get_window_rule_suggestions(&self) -> Vec<String> {
        vec![
            "float".to_string(),
            "tile".to_string(),
            "fullscreen".to_string(),
            "maximize".to_string(),
            "pin".to_string(),
            "workspace 2".to_string(),
            "workspace special".to_string(),
            "size 800 600".to_string(),
            "move 100 100".to_string(),
            "opacity 0.8".to_string(),
            "opaque".to_string(),
            "animation slide".to_string(),
            "bordercolor rgb(255,0,0)".to_string(),
            "idleinhibit focus".to_string(),
            "suppressevent maximize".to_string(),
        ]
    }

    fn get_layer_rule_suggestions(&self) -> Vec<String> {
        vec![
            "blur".to_string(),
            "ignorealpha".to_string(),
            "ignorezero".to_string(),
            "noanim".to_string(),
            "dimaround".to_string(),
            "xray 0".to_string(),
            "xray 1".to_string(),
            "animation slide".to_string(),
            "animation fade".to_string(),
        ]
    }

    fn get_workspace_rule_suggestions(&self) -> Vec<String> {
        vec![
            "monitor:DP-1".to_string(),
            "monitor:HDMI-A-1".to_string(),
            "default:true".to_string(),
            "gapsout:20".to_string(),
            "gapsin:10".to_string(),
            "bordersize:2".to_string(),
            "border:false".to_string(),
            "shadow:false".to_string(),
            "rounding:false".to_string(),
            "decorate:false".to_string(),
        ]
    }

    async fn load_general_config(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        if let Some(items) = self.config_items.get_mut(&FocusedPanel::General) {
            for item in items.iter_mut() {
                let hypr_key = match item.key.as_str() {
                    "gaps_in" => "general:gaps_in",
                    "gaps_out" => "general:gaps_out",
                    "border_size" => "general:border_size",
                    "col.active_border" => "general:col.active_border",
                    "col.inactive_border" => "general:col.inactive_border",
                    _ => continue,
                };

                match hyprctl.get_option(hypr_key).await {
                    Ok(value) => {
                        // Parse hyprctl output - it usually returns "option = value"
                        if let Some(parsed_value) = Self::parse_hyprctl_value(&value) {
                            item.value = parsed_value;
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to get {hypr_key}: {e}");
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_input_config(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        if let Some(items) = self.config_items.get_mut(&FocusedPanel::Input) {
            for item in items.iter_mut() {
                let hypr_key = match item.key.as_str() {
                    "kb_layout" => "input:kb_layout",
                    "follow_mouse" => "input:follow_mouse",
                    "sensitivity" => "input:sensitivity",
                    _ => continue,
                };

                match hyprctl.get_option(hypr_key).await {
                    Ok(value) => {
                        if let Some(parsed_value) = Self::parse_hyprctl_value(&value) {
                            item.value = parsed_value;
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to get {hypr_key}: {e}");
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_decoration_config(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        if let Some(items) = self.config_items.get_mut(&FocusedPanel::Decoration) {
            for item in items.iter_mut() {
                let hypr_key = match item.key.as_str() {
                    "rounding" => "decoration:rounding",
                    "blur.enabled" => "decoration:blur:enabled",
                    "blur.size" => "decoration:blur:size",
                    _ => continue,
                };

                match hyprctl.get_option(hypr_key).await {
                    Ok(value) => {
                        if let Some(parsed_value) = Self::parse_hyprctl_value(&value) {
                            item.value = parsed_value;
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to get {hypr_key}: {e}");
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_animations_config(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        if let Some(items) = self.config_items.get_mut(&FocusedPanel::Animations) {
            for item in items.iter_mut() {
                let hypr_key = match item.key.as_str() {
                    "animations.enabled" => "animations:enabled",
                    _ => continue,
                };

                match hyprctl.get_option(hypr_key).await {
                    Ok(value) => {
                        if let Some(parsed_value) = Self::parse_hyprctl_value(&value) {
                            item.value = parsed_value;
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to get {hypr_key}: {e}");
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_gestures_config(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        if let Some(items) = self.config_items.get_mut(&FocusedPanel::Gestures) {
            for item in items.iter_mut() {
                let hypr_key = match item.key.as_str() {
                    "gestures.workspace_swipe" => "gestures:workspace_swipe",
                    "gestures.workspace_swipe_fingers" => "gestures:workspace_swipe_fingers",
                    "gestures.workspace_swipe_distance" => "gestures:workspace_swipe_distance",
                    "gestures.workspace_swipe_invert" => "gestures:workspace_swipe_invert",
                    _ => continue,
                };

                match hyprctl.get_option(hypr_key).await {
                    Ok(value) => {
                        if let Some(parsed_value) = Self::parse_hyprctl_value(&value) {
                            item.value = parsed_value;
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to get {hypr_key}: {e}");
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_misc_config(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        if let Some(items) = self.config_items.get_mut(&FocusedPanel::Misc) {
            for item in items.iter_mut() {
                let hypr_key = match item.key.as_str() {
                    "misc.disable_hyprland_logo" => "misc:disable_hyprland_logo",
                    "misc.disable_splash_rendering" => "misc:disable_splash_rendering",
                    "misc.mouse_move_enables_dpms" => "misc:mouse_move_enables_dpms",
                    "misc.vfr" => "misc:vfr",
                    "misc.vrr" => "misc:vrr",
                    _ => continue,
                };

                match hyprctl.get_option(hypr_key).await {
                    Ok(value) => {
                        if let Some(parsed_value) = Self::parse_hyprctl_value(&value) {
                            item.value = parsed_value;
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to get {hypr_key}: {e}");
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_binds_config(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        match hyprctl.get_binds().await {
            Ok(keybinds) => {
                // If hyprctl succeeds but returns empty keybinds, treat it as a failure
                // This likely means Hyprland isn't running and we should try config file parsing
                if keybinds.is_empty() {
                    eprintln!(
                        "Debug: hyprctl.get_binds() returned empty results, treating as failure"
                    );
                    return Err(anyhow::anyhow!("No keybinds found via hyprctl"));
                }

                let mut bind_items = Vec::new();

                for (i, keybind) in keybinds.iter().enumerate() {
                    // Create a unique key for each keybind
                    let key = format!("bind_{i}");

                    bind_items.push(ConfigItem {
                        key: key.clone(),
                        value: keybind.display_string(),
                        description: format!(
                            "Keybind: {} {}",
                            keybind.dispatcher,
                            keybind.args.as_deref().unwrap_or("")
                        ),
                        data_type: ConfigDataType::String,
                        suggestions: self.get_keybind_suggestions(&keybind.dispatcher),
                    });
                }

                // Insert the loaded keybinds
                self.config_items.insert(FocusedPanel::Binds, bind_items);
                Ok(())
            }
            Err(e) => {
                eprintln!("Warning: Failed to load keybinds: {e}");
                // Don't insert placeholder data here - let the config file loading handle it
                Err(e)
            }
        }
    }

    fn get_keybind_suggestions(&self, dispatcher: &str) -> Vec<String> {
        match dispatcher {
            "exec" => vec![
                "kitty".to_string(),
                "firefox".to_string(),
                "code".to_string(),
                "rofi -show drun".to_string(),
                "grim -g \"$(slurp)\"".to_string(),
            ],
            "workspace" => vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "7".to_string(),
                "8".to_string(),
                "9".to_string(),
                "10".to_string(),
            ],
            "movetoworkspace" => vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "7".to_string(),
                "8".to_string(),
                "9".to_string(),
                "10".to_string(),
            ],
            "movefocus" => vec![
                "l".to_string(),
                "r".to_string(),
                "u".to_string(),
                "d".to_string(),
                "left".to_string(),
                "right".to_string(),
                "up".to_string(),
                "down".to_string(),
            ],
            "movewindow" => vec![
                "l".to_string(),
                "r".to_string(),
                "u".to_string(),
                "d".to_string(),
                "left".to_string(),
                "right".to_string(),
                "up".to_string(),
                "down".to_string(),
            ],
            "resizeactive" => vec![
                "10 0".to_string(),
                "-10 0".to_string(),
                "0 10".to_string(),
                "0 -10".to_string(),
                "50 0".to_string(),
                "-50 0".to_string(),
                "0 50".to_string(),
                "0 -50".to_string(),
            ],
            _ => vec![],
        }
    }

    async fn load_window_rules_config(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        match hyprctl.get_window_rules().await {
            Ok(window_rules) => {
                // If hyprctl succeeds but returns empty window rules, treat it as a failure
                // This likely means Hyprland isn't running and we should try config file parsing
                if window_rules.is_empty() {
                    eprintln!("Debug: hyprctl.get_window_rules() returned empty results, treating as failure");
                    return Err(anyhow::anyhow!("No window rules found via hyprctl"));
                }

                let mut rule_items = Vec::new();

                for (i, rule) in window_rules.iter().enumerate() {
                    let key = format!("window_rule_{i}");

                    // Parse rule to extract description
                    let description = if rule.contains("windowrule") {
                        let parts: Vec<&str> = rule.splitn(3, " = ").collect();
                        if parts.len() >= 2 {
                            format!("Window rule: {}", parts[1])
                        } else {
                            "Window rule configuration".to_string()
                        }
                    } else {
                        format!("Window pattern: {rule}")
                    };

                    rule_items.push(ConfigItem {
                        key: key.clone(),
                        value: rule.clone(),
                        description,
                        data_type: ConfigDataType::String,
                        suggestions: self.get_window_rule_suggestions(),
                    });
                }

                // Insert the loaded window rules
                self.config_items
                    .insert(FocusedPanel::WindowRules, rule_items);
                Ok(())
            }
            Err(e) => {
                eprintln!("Warning: Failed to load window rules: {e}");
                Err(e)
            }
        }
    }

    async fn load_layer_rules_config(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        match hyprctl.get_layer_rules().await {
            Ok(layer_rules) => {
                // If hyprctl succeeds but returns empty layer rules, treat it as a failure
                // This likely means Hyprland isn't running and we should try config file parsing
                if layer_rules.is_empty() {
                    eprintln!("Debug: hyprctl.get_layer_rules() returned empty results, treating as failure");
                    return Err(anyhow::anyhow!("No layer rules found via hyprctl"));
                }

                let mut rule_items = Vec::new();

                for (i, rule) in layer_rules.iter().enumerate() {
                    let key = format!("layer_rule_{i}");

                    // Parse rule to extract description
                    let description = if rule.contains("layerrule") {
                        let parts: Vec<&str> = rule.splitn(3, " = ").collect();
                        if parts.len() >= 2 {
                            format!("Layer rule: {}", parts[1])
                        } else {
                            "Layer rule configuration".to_string()
                        }
                    } else {
                        format!("Layer configuration: {rule}")
                    };

                    rule_items.push(ConfigItem {
                        key: key.clone(),
                        value: rule.clone(),
                        description,
                        data_type: ConfigDataType::String,
                        suggestions: self.get_layer_rule_suggestions(),
                    });
                }

                // Also load workspace rules if available
                match hyprctl.get_workspace_rules().await {
                    Ok(workspace_rules) => {
                        // Add workspace rules to the layer rules panel for now
                        for (i, rule) in workspace_rules.iter().enumerate() {
                            let key = format!("workspace_rule_{i}");

                            rule_items.push(ConfigItem {
                                key: key.clone(),
                                value: rule.clone(),
                                description: format!("Workspace rule: {rule}"),
                                data_type: ConfigDataType::String,
                                suggestions: self.get_workspace_rule_suggestions(),
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load workspace rules: {e}");
                    }
                }

                // Insert the loaded layer rules
                self.config_items
                    .insert(FocusedPanel::LayerRules, rule_items);
                Ok(())
            }
            Err(e) => {
                eprintln!("Warning: Failed to load layer rules: {e}");
                Err(e)
            }
        }
    }

    fn parse_hyprctl_value(raw_value: &str) -> Option<String> {
        // hyprctl usually returns output like "option = value" or just "value"
        // Handle different formats:

        if raw_value.contains(" = ") {
            // Format: "option = value"
            if let Some(value_part) = raw_value.split(" = ").nth(1) {
                return Some(value_part.trim().to_string());
            }
        } else if raw_value.contains(": ") {
            // Format: "option: value"
            if let Some(value_part) = raw_value.split(": ").nth(1) {
                return Some(value_part.trim().to_string());
            }
        } else {
            // Format: just "value"
            let trimmed = raw_value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }

        None
    }

    fn parse_rule_for_editing(&self, rule_value: &str, panel: &FocusedPanel) -> EditMode {
        // Parse different rule formats

        let rule_type = match panel {
            FocusedPanel::WindowRules => RuleType::Window,
            FocusedPanel::LayerRules => {
                if rule_value.contains("layerrule") {
                    RuleType::Layer
                } else {
                    RuleType::Workspace
                }
            }
            _ => RuleType::Window,
        };

        // Try to parse rule format: "ruletype = action, pattern" or similar
        if let Some((action_part, pattern_part)) = rule_value.split_once(", ") {
            // Format: "windowrule = float, ^(kitty)$"
            let action = if let Some((_, action)) = action_part.split_once(" = ") {
                action.to_string()
            } else {
                action_part.to_string()
            };

            EditMode::Rule {
                rule_type,
                pattern: pattern_part.to_string(),
                action,
                editing_field: RuleField::Pattern,
            }
        } else if rule_value.contains("class:") || rule_value.contains("title:") {
            // Format: "class:^(kitty)$" - just a pattern
            EditMode::Rule {
                rule_type,
                pattern: rule_value.to_string(),
                action: "float".to_string(),
                editing_field: RuleField::Pattern,
            }
        } else {
            // Fallback to text editing
            EditMode::Text {
                current_value: rule_value.to_string(),
                cursor_pos: rule_value.len(),
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, app_state: (FocusedPanel, bool)) {
        let size = f.area();
        let (_, debug) = app_state;

        // Create main layout with tabs
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Header
                Constraint::Length(3), // Tab bar
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ])
            .margin(1)
            .split(size);

        self.render_enhanced_header(f, main_chunks[0], debug);

        // Render tab bar
        self.render_tab_bar(f, main_chunks[1]);

        // Render current tab content
        self.render_current_tab(f, main_chunks[2]);

        self.render_enhanced_footer(f, main_chunks[3]);

        // Render popups and dialogs on top
        if self.show_popup {
            self.render_popup(f, size);
        }

        if self.show_save_dialog {
            self.render_save_dialog(f, size);
        }

        if self.show_reload_dialog {
            self.render_reload_dialog(f, size);
        }

        if self.show_import_dialog {
            self.render_import_dialog(f, size);
        }

        if self.show_export_dialog {
            self.render_export_dialog(f, size);
        }

        if self.show_nixos_export_dialog {
            self.render_nixos_export_dialog(f, size);
        }

        if self.show_batch_dialog {
            self.render_batch_dialog(f, size);
        }

        if self.edit_mode != EditMode::None {
            self.render_edit_popup(f, size);
        }

        if self.show_help {
            self.render_help_overlay(f, size);
        }
        if self.show_preview_dialog {
            self.render_preview_dialog(f, size);
        }
    }

    fn render_enhanced_header(&self, f: &mut Frame, area: Rect, debug: bool) {
        let _title_text = if debug {
            "🦀 R-Hyprconfig - Debug Mode 🔧"
        } else {
            "🎨 R-Hyprconfig - Hyprland Configuration Manager ⚡"
        };

        // Create gradient-like effect with theme colors
        let mut title_spans = vec![
            Span::styled("R-Hyprconfig", self.theme.header_style().bold()),
            Span::raw(" - "),
            Span::styled(
                "Hyprland Configuration Manager",
                Style::default().fg(self.theme.accent_secondary).bold(),
            ),
        ];

        // Add NixOS indicator if detected
        if self.nixos_env.is_nixos {
            title_spans.push(Span::raw(" | "));
            title_spans.push(Span::styled(
                "NixOS",
                Style::default().fg(self.theme.accent_info).bold(),
            ));

            if let Some(config_loc) = self.nixos_env.get_primary_config_location() {
                title_spans.push(Span::raw(" ("));
                let config_type_str = config_loc.config_type.to_string();
                title_spans.push(Span::styled(
                    config_type_str,
                    Style::default().fg(self.theme.fg_muted),
                ));
                title_spans.push(Span::raw(")"));
            }
        }

        // Add live preview status indicator
        let preview_status = self.get_preview_status();
        title_spans.push(Span::raw(" | "));
        if self.preview_mode {
            title_spans.push(Span::styled(
                preview_status,
                Style::default().fg(self.theme.accent_primary).bold(),
            ));
        } else {
            title_spans.push(Span::styled(
                preview_status,
                Style::default().fg(self.theme.fg_muted),
            ));
        }

        let header_content = vec![
            Line::from(title_spans),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(self.theme.fg_muted)),
                Span::styled("Enter", self.theme.warning_style().bold()),
                Span::styled(" to edit • ", Style::default().fg(self.theme.fg_muted)),
                Span::styled("Tab", self.theme.warning_style().bold()),
                Span::styled(
                    " to switch tabs • ",
                    Style::default().fg(self.theme.fg_muted),
                ),
                Span::styled("↑↓", self.theme.info_style().bold()),
                Span::styled(" to navigate • ", Style::default().fg(self.theme.fg_muted)),
                Span::styled("S", self.theme.success_style().bold()),
                Span::styled(" to save • ", Style::default().fg(self.theme.fg_muted)),
                Span::styled("R", self.theme.info_style().bold()),
                Span::styled(" to reload", Style::default().fg(self.theme.fg_muted)),
            ]),
        ];

        let header = Paragraph::new(header_content)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style(true))
                    .border_type(BorderType::Double)
                    .title(" Hyprland TUI ")
                    .title_style(self.theme.header_style().bold()),
            );

        f.render_widget(header, area);
    }

    fn render_tab_bar(&self, f: &mut Frame, area: Rect) {
        let tabs = [
            FocusedPanel::General,
            FocusedPanel::Input,
            FocusedPanel::Decoration,
            FocusedPanel::Animations,
            FocusedPanel::Gestures,
            FocusedPanel::Binds,
            FocusedPanel::WindowRules,
            FocusedPanel::LayerRules,
            FocusedPanel::Misc,
            FocusedPanel::Import,
            FocusedPanel::Export,
        ];

        let tab_spans: Vec<Span> = tabs
            .iter()
            .enumerate()
            .flat_map(|(i, &panel)| {
                let base_name = match panel {
                    FocusedPanel::General => "General",
                    FocusedPanel::Input => "Input",
                    FocusedPanel::Decoration => "Decoration",
                    FocusedPanel::Animations => "Animations",
                    FocusedPanel::Gestures => "Gestures",
                    FocusedPanel::Binds => "Binds",
                    FocusedPanel::WindowRules => "Win Rules",
                    FocusedPanel::LayerRules => "Layers",
                    FocusedPanel::Misc => "Misc",
                    FocusedPanel::Import => "Import",
                    FocusedPanel::Export => "Export",
                };

                // Add pagination info to current tab
                let tab_name = if panel == self.current_tab {
                    let (current_page, total_pages, _) = self.get_pagination_info();
                    if total_pages > 1 {
                        format!("{base_name} ({current_page}/{total_pages})")
                    } else {
                        base_name.to_string()
                    }
                } else {
                    base_name.to_string()
                };

                let style = self.theme.tab_style(panel == self.current_tab);

                let mut result = vec![Span::styled(tab_name, style)];
                if i < tabs.len() - 1 {
                    result.push(Span::raw(" │ "));
                }
                result
            })
            .collect();

        let tabs_paragraph = Paragraph::new(Line::from(tab_spans))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style(false)),
            );

        f.render_widget(tabs_paragraph, area);
    }

    fn render_current_tab(&mut self, f: &mut Frame, area: Rect) {
        // Split area to make room for search bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if self.search_mode || !self.search_query.is_empty() {
                vec![
                    Constraint::Length(3), // Search bar
                    Constraint::Min(0),    // Config list
                ]
            } else {
                vec![
                    Constraint::Min(0), // Full area for list
                ]
            })
            .split(area);

        // Render search bar if in search mode or has query
        if self.search_mode || !self.search_query.is_empty() {
            self.render_search_bar(f, chunks[0]);
        }

        // Get config items for current tab and filter them
        let config_items = self
            .config_items
            .get(&self.current_tab)
            .cloned()
            .unwrap_or_default();
        let filtered_items = self.filter_items_progressive(&config_items);

        // Update pagination for current filtered items
        self.update_pagination(self.current_tab, filtered_items.len());

        // Apply pagination to the filtered items for performance
        let paginated_items = self.get_paginated_items(&filtered_items);

        // Apply virtualization to only render visible items
        let content_area_height = if self.search_mode || !self.search_query.is_empty() {
            area.height.saturating_sub(3) // Account for search bar
        } else {
            area.height
        };
        let (virtualized_items, _start_idx, _end_idx) =
            self.get_virtualized_items(&paginated_items, content_area_height as usize);

        // Extract theme before creating items to avoid borrowing conflicts
        let theme = self.theme.clone();
        let current_tab = self.current_tab;

        let items = Self::create_optimized_list_items(&virtualized_items, &theme);

        // Panel title
        let title = match current_tab {
            FocusedPanel::General => "🏠 General Configuration",
            FocusedPanel::Input => "⌨️ Input Configuration",
            FocusedPanel::Decoration => "✨ Decoration Configuration",
            FocusedPanel::Animations => "🎬 Animation Configuration",
            FocusedPanel::Gestures => "👆 Gesture Configuration",
            FocusedPanel::Binds => "🔗 Key Bindings Configuration",
            FocusedPanel::WindowRules => "📏 Window Rules Configuration",
            FocusedPanel::LayerRules => "📐 Layer Rules Configuration",
            FocusedPanel::Misc => "⚙️ Miscellaneous Configuration",
            FocusedPanel::Import => "📥 Import Configuration",
            FocusedPanel::Export => "📤 Export Configuration",
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .title_style(Style::default().fg(Color::Cyan).bold())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(60, 60, 120))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        // Determine list area before getting mutable reference
        let has_search = self.search_mode || !self.search_query.is_empty();
        let list_area = if has_search {
            chunks[1] // Use second chunk when search bar is present
        } else {
            chunks[0] // Use first (and only) chunk when no search bar
        };

        let current_list_state = self.get_list_state_mut(current_tab);
        f.render_stateful_widget(list, list_area, current_list_state);
    }

    #[allow(dead_code)]
    fn render_enhanced_panel(
        &mut self,
        f: &mut Frame,
        area: Rect,
        panel: FocusedPanel,
        focused_panel: FocusedPanel,
    ) {
        let is_focused = focused_panel == panel;

        // Enhanced visual styling
        let (border_style, border_type, panel_bg) = if is_focused {
            (
                Style::default().fg(Color::Rgb(255, 215, 0)), // Gold for focused
                BorderType::Double,
                Some(Color::Rgb(10, 10, 30)), // Dark blue background
            )
        } else {
            (
                Style::default().fg(Color::Rgb(100, 150, 200)), // Light blue for unfocused
                BorderType::Rounded,
                None,
            )
        };

        let config_items = self.config_items.get(&panel).cloned().unwrap_or_default();

        let items: Vec<ListItem> = config_items
            .iter()
            .map(|item| {
                let value_style = match &item.data_type {
                    ConfigDataType::Integer { .. } => {
                        Style::default().fg(Color::Rgb(100, 255, 100))
                    } // Light green
                    ConfigDataType::Float { .. } => Style::default().fg(Color::Rgb(100, 255, 255)), // Light cyan
                    ConfigDataType::Boolean => Style::default().fg(Color::Rgb(255, 255, 100)), // Light yellow
                    ConfigDataType::Color => Style::default().fg(Color::Rgb(255, 150, 255)), // Light magenta
                    ConfigDataType::String => Style::default().fg(Color::White),
                    ConfigDataType::Keyword { .. } => {
                        Style::default().fg(Color::Rgb(255, 200, 100))
                    } // Light orange
                };

                let line = Line::from(vec![
                    Span::styled(
                        &item.key,
                        Style::default().fg(Color::Rgb(200, 200, 255)).bold(),
                    ),
                    Span::raw(": "),
                    Span::styled(&item.value, value_style.bold()),
                ]);

                ListItem::new(line)
            })
            .collect();

        // Panel title
        let title = match panel {
            FocusedPanel::General => "General Configuration",
            FocusedPanel::Input => "Input Configuration",
            FocusedPanel::Decoration => "Decoration Configuration",
            FocusedPanel::Animations => "Animations Configuration",
            FocusedPanel::Gestures => "Gestures Configuration",
            FocusedPanel::Binds => "Key Binds Configuration",
            FocusedPanel::WindowRules => "Window Rules Configuration",
            FocusedPanel::LayerRules => "Layer Rules Configuration",
            FocusedPanel::Misc => "Miscellaneous Configuration",
            FocusedPanel::Import => "Import Configuration",
            FocusedPanel::Export => "Export Configuration",
        };

        let mut block = Block::default()
            .title(title)
            .title_style(Style::default().fg(Color::White).bold())
            .borders(Borders::ALL)
            .border_style(border_style)
            .border_type(border_type);

        if let Some(bg) = panel_bg {
            block = block.style(Style::default().bg(bg));
        }

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(60, 60, 120))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let list_state = self.get_list_state_mut(panel);
        let selected_position = list_state.selected().unwrap_or(0);
        f.render_stateful_widget(list, area, list_state);

        // Enhanced scrollbar
        if area.height > 4 && !config_items.is_empty() {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(if is_focused { Some("▲") } else { Some("↑") })
                .end_symbol(if is_focused { Some("▼") } else { Some("↓") })
                .track_symbol(Some("│"))
                .thumb_symbol("█");

            let scrollbar_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };

            let mut scrollbar_state = ScrollbarState::default()
                .content_length(config_items.len())
                .position(selected_position);

            f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }

        // Show description at bottom of focused panel
        if is_focused && !config_items.is_empty() {
            if let Some(selected_item) = config_items.get(selected_position) {
                let desc_area = Rect {
                    x: area.x + 1,
                    y: area.y + area.height.saturating_sub(2),
                    width: area.width.saturating_sub(2),
                    height: 1,
                };

                let description = Paragraph::new(selected_item.description.as_str())
                    .style(Style::default().fg(Color::Rgb(180, 180, 180)).italic())
                    .wrap(Wrap { trim: true });

                f.render_widget(description, desc_area);
            }
        }
    }

    fn render_enhanced_footer(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Span::styled("Tab/→", self.theme.warning_style().bold()),
            Span::styled(" Next ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("Shift+Tab/←", self.theme.warning_style().bold()),
            Span::styled(" Previous ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("↑↓", self.theme.success_style().bold()),
            Span::styled(" Navigate ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("PgUp/PgDn", self.theme.success_style().bold()),
            Span::styled(" Page ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled(
                "Enter",
                Style::default().fg(self.theme.accent_primary).bold(),
            ),
            Span::styled(" Edit ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("S", self.theme.info_style().bold()),
            Span::styled(" Save ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("R", self.theme.info_style().bold()),
            Span::styled(" Reload ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("/", self.theme.warning_style().bold()),
            Span::styled(" Search ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("E", self.theme.success_style().bold()),
            Span::styled(" Export ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("M", self.theme.success_style().bold()),
            Span::styled(" Import ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("T", Style::default().fg(self.theme.accent_secondary).bold()),
            Span::styled(" Theme ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("F1/?", self.theme.info_style().bold()),
            Span::styled(" Help ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("L", Style::default().fg(self.theme.accent_secondary).bold()),
            Span::styled(" Live Preview ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled(
                "Ctrl+Z",
                Style::default().fg(self.theme.accent_secondary).bold(),
            ),
            Span::styled(" Undo ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled(
                "Ctrl+Y",
                Style::default().fg(self.theme.accent_secondary).bold(),
            ),
            Span::styled(" Redo ", Style::default().fg(self.theme.fg_muted)),
            Span::raw("• "),
            Span::styled("Q/Esc", self.theme.error_style().bold()),
            Span::styled(" Quit", Style::default().fg(self.theme.fg_muted)),
        ];

        let footer = Paragraph::new(Line::from(help_text))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.border_style(false))
                    .border_type(BorderType::Rounded)
                    .title(" Controls ")
                    .title_style(self.theme.info_style().bold()),
            );

        f.render_widget(footer, area);
    }

    fn render_popup(&self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(50, 25, area);

        let popup_content = vec![
            Line::from(vec![Span::styled(
                "ℹ️ Information",
                Style::default().fg(Color::Cyan).bold(),
            )]),
            Line::from(""),
            Line::from(self.popup_message.as_str()),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Yellow).bold()),
                Span::styled(" to continue", Style::default().fg(Color::Gray)),
            ]),
        ];

        let popup = Paragraph::new(popup_content)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .border_type(BorderType::Double)
                    .title(" Message ")
                    .title_style(Style::default().fg(Color::Cyan).bold()),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);
    }

    fn render_save_dialog(&self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(60, 30, area);

        let popup_content = vec![
            Line::from(vec![Span::styled(
                "💾 Save Configuration",
                Style::default().fg(Color::Green).bold(),
            )]),
            Line::from(""),
            Line::from("Save current configuration changes to file?"),
            Line::from(""),
            Line::from(vec![
                Span::styled("⚠️ Warning: ", Style::default().fg(Color::Yellow).bold()),
                Span::raw("This will overwrite your existing configuration"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Y", Style::default().fg(Color::Green).bold()),
                Span::styled(" - Yes, save  ", Style::default().fg(Color::Gray)),
                Span::styled("N", Style::default().fg(Color::Red).bold()),
                Span::styled(" - No, cancel", Style::default().fg(Color::Gray)),
            ]),
        ];

        let popup = Paragraph::new(popup_content)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(BorderType::Double)
                    .title(" Save Confirmation ")
                    .title_style(Style::default().fg(Color::Green).bold()),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);
    }

    fn render_nixos_export_dialog(&self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(90, 80, area);

        // Split the dialog into two sections: options and preview
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(popup_area);

        // Left side: Configuration type options
        let current_type = &self.nixos_export_config_type;
        let options_content = vec![
            Line::from(vec![Span::styled(
                "🏗️ NixOS Export Configuration",
                Style::default().fg(Color::Cyan).bold(),
            )]),
            Line::from(""),
            Line::from("Select NixOS configuration type:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("1", Style::default().fg(Color::Green).bold()),
                Span::raw(" - Home Manager "),
                if matches!(current_type, crate::nixos::NixConfigType::HomeManager) {
                    Span::styled("(selected)", Style::default().fg(Color::Green))
                } else {
                    Span::raw("")
                },
            ]),
            Line::from("    User-level configuration"),
            Line::from(""),
            Line::from(vec![
                Span::styled("2", Style::default().fg(Color::Green).bold()),
                Span::raw(" - System Configuration "),
                if matches!(current_type, crate::nixos::NixConfigType::SystemConfig) {
                    Span::styled("(selected)", Style::default().fg(Color::Green))
                } else {
                    Span::raw("")
                },
            ]),
            Line::from("    System-level Hyprland enabling"),
            Line::from(""),
            Line::from(vec![
                Span::styled("3", Style::default().fg(Color::Green).bold()),
                Span::raw(" - Flake Home Manager "),
                if matches!(current_type, crate::nixos::NixConfigType::FlakeHomeManager) {
                    Span::styled("(selected)", Style::default().fg(Color::Green))
                } else {
                    Span::raw("")
                },
            ]),
            Line::from("    Flake-based Home Manager"),
            Line::from(""),
            Line::from(vec![
                Span::styled("4", Style::default().fg(Color::Green).bold()),
                Span::raw(" - Flake System "),
                if matches!(current_type, crate::nixos::NixConfigType::FlakeSystem) {
                    Span::styled("(selected)", Style::default().fg(Color::Green))
                } else {
                    Span::raw("")
                },
            ]),
            Line::from("    System-level flake"),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green).bold()),
                Span::styled(" - Export  ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Red).bold()),
                Span::styled(" - Cancel", Style::default().fg(Color::Gray)),
            ]),
        ];

        let options_popup = Paragraph::new(options_content)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .border_type(BorderType::Double)
                    .title(" Configuration Type ")
                    .title_style(Style::default().fg(Color::Cyan).bold()),
            )
            .wrap(Wrap { trim: true });

        // Right side: Preview
        let preview_content = if let Some(preview) = &self.nixos_export_preview {
            preview.clone()
        } else {
            "Generating preview...".to_string()
        };

        let preview_popup = Paragraph::new(preview_content)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_type(BorderType::Rounded)
                    .title(" Preview ")
                    .title_style(Style::default().fg(Color::Yellow).bold()),
            )
            .wrap(Wrap { trim: true });

        // Render both sides
        f.render_widget(Clear, popup_area);
        f.render_widget(options_popup, chunks[0]);
        f.render_widget(preview_popup, chunks[1]);
    }

    fn render_reload_dialog(&self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(60, 30, area);

        let popup_content = vec![
            Line::from(vec![Span::styled(
                "🔄 Reload Configuration",
                Style::default().fg(Color::Blue).bold(),
            )]),
            Line::from(""),
            Line::from("Reload configuration from Hyprland?"),
            Line::from(""),
            Line::from(vec![
                Span::styled("⚠️ Warning: ", Style::default().fg(Color::Yellow).bold()),
                Span::raw("This will discard any unsaved changes"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Y", Style::default().fg(Color::Green).bold()),
                Span::styled(" - Yes, reload  ", Style::default().fg(Color::Gray)),
                Span::styled("N", Style::default().fg(Color::Red).bold()),
                Span::styled(" - No, cancel", Style::default().fg(Color::Gray)),
            ]),
        ];

        let popup = Paragraph::new(popup_content)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .border_type(BorderType::Double)
                    .title(" Reload Confirmation ")
                    .title_style(Style::default().fg(Color::Blue).bold()),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);
    }

    fn render_edit_popup(&self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(70, 40, area);

        // Get the item being edited
        let (panel, key) = if let Some((panel, key)) = &self.editing_item {
            (*panel, key.clone())
        } else {
            return;
        };

        let config_items = self.config_items.get(&panel).cloned().unwrap_or_default();
        let item = config_items.iter().find(|item| item.key == key);

        if let Some(item) = item {
            let mut popup_content = vec![
                Line::from(vec![Span::styled(
                    "✏️ Edit Configuration",
                    Style::default().fg(Color::Magenta).bold(),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Key: ", Style::default().fg(Color::Cyan).bold()),
                    Span::raw(&item.key),
                ]),
                Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(Color::Yellow).bold()),
                    Span::raw(&item.description),
                ]),
                Line::from(""),
            ];

            // Render different edit modes
            match &self.edit_mode {
                EditMode::Text {
                    current_value,
                    cursor_pos: _,
                } => {
                    popup_content.push(Line::from(vec![
                        Span::styled("Value: ", Style::default().fg(Color::Green).bold()),
                        Span::raw(current_value),
                        Span::styled("|", Style::default().fg(Color::White).bold()),
                    ]));
                }
                EditMode::Boolean { current_value } => {
                    popup_content.push(Line::from(vec![
                        Span::styled("Value: ", Style::default().fg(Color::Green).bold()),
                        Span::styled(
                            if *current_value { "true" } else { "false" },
                            Style::default()
                                .fg(if *current_value {
                                    Color::Green
                                } else {
                                    Color::Red
                                })
                                .bold(),
                        ),
                    ]));
                    popup_content.push(Line::from(""));
                    popup_content.push(Line::from("Press Space to toggle"));
                }
                EditMode::Select { options, selected } => {
                    popup_content.push(Line::from(vec![Span::styled(
                        "Options:",
                        Style::default().fg(Color::Green).bold(),
                    )]));
                    for (i, option) in options.iter().enumerate() {
                        let style = if i == *selected {
                            Style::default().fg(Color::Yellow).bold()
                        } else {
                            Style::default().fg(Color::Gray)
                        };
                        popup_content.push(Line::from(vec![
                            Span::raw("  "),
                            if i == *selected {
                                Span::styled("▶ ", Style::default().fg(Color::Yellow).bold())
                            } else {
                                Span::raw("  ")
                            },
                            Span::styled(option, style),
                        ]));
                    }
                }
                EditMode::Slider {
                    current_value,
                    min,
                    max,
                    ..
                } => {
                    let percentage = (current_value - min) / (max - min);
                    let bar_width = 40;
                    let filled = (percentage * bar_width as f32) as usize;

                    let mut bar = String::new();
                    bar.push('[');
                    for i in 0..bar_width {
                        if i < filled {
                            bar.push('█');
                        } else {
                            bar.push('░');
                        }
                    }
                    bar.push(']');

                    let current_value_str = current_value.to_string();
                    let min_str = format!("Min: {min}");
                    let max_str = format!("Max: {max}");

                    popup_content.push(Line::from(vec![
                        Span::styled("Value: ", Style::default().fg(Color::Green).bold()),
                        Span::styled(current_value_str, Style::default().fg(Color::Cyan).bold()),
                    ]));
                    popup_content.push(Line::from(""));
                    popup_content.push(Line::from(vec![Span::styled(
                        bar,
                        Style::default().fg(Color::Blue),
                    )]));
                    popup_content.push(Line::from(vec![
                        Span::styled(min_str, Style::default().fg(Color::Gray)),
                        Span::raw("  "),
                        Span::styled(max_str, Style::default().fg(Color::Gray)),
                    ]));
                }
                EditMode::Keybind {
                    modifiers,
                    key,
                    dispatcher,
                    args,
                    editing_field,
                } => {
                    popup_content.push(Line::from(vec![Span::styled(
                        "Keybind Editor",
                        Style::default().fg(Color::Magenta).bold(),
                    )]));
                    popup_content.push(Line::from(""));

                    // Show each field with highlighting for the currently editing field
                    let mod_style = if *editing_field == KeybindField::Modifiers {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let key_style = if *editing_field == KeybindField::Key {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let dispatcher_style = if *editing_field == KeybindField::Dispatcher {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let args_style = if *editing_field == KeybindField::Args {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let modifiers_text = modifiers.join(" + ");
                    popup_content.push(Line::from(vec![
                        Span::styled("Modifiers: ", Style::default().fg(Color::Cyan).bold()),
                        Span::styled(modifiers_text, mod_style),
                    ]));

                    popup_content.push(Line::from(vec![
                        Span::styled("Key: ", Style::default().fg(Color::Cyan).bold()),
                        Span::styled(key, key_style),
                    ]));

                    popup_content.push(Line::from(vec![
                        Span::styled("Action: ", Style::default().fg(Color::Cyan).bold()),
                        Span::styled(dispatcher, dispatcher_style),
                    ]));

                    popup_content.push(Line::from(vec![
                        Span::styled("Arguments: ", Style::default().fg(Color::Cyan).bold()),
                        Span::styled(args, args_style),
                    ]));

                    popup_content.push(Line::from(""));
                    popup_content.push(Line::from(vec![
                        Span::styled("Tab", Style::default().fg(Color::Yellow).bold()),
                        Span::styled(" - Next field  ", Style::default().fg(Color::Gray)),
                        Span::styled("Type", Style::default().fg(Color::Yellow).bold()),
                        Span::styled(" - Edit", Style::default().fg(Color::Gray)),
                    ]));
                }
                EditMode::Rule {
                    rule_type,
                    pattern,
                    action,
                    editing_field,
                } => {
                    popup_content.push(Line::from(vec![Span::styled(
                        "Rule Editor",
                        Style::default().fg(Color::Magenta).bold(),
                    )]));
                    popup_content.push(Line::from(""));

                    let rule_type_name = match rule_type {
                        RuleType::Window => "Window Rule",
                        RuleType::Layer => "Layer Rule",
                        RuleType::Workspace => "Workspace Rule",
                    };

                    popup_content.push(Line::from(vec![
                        Span::styled("Type: ", Style::default().fg(Color::Cyan).bold()),
                        Span::styled(rule_type_name, Style::default().fg(Color::White)),
                    ]));

                    let pattern_style = if *editing_field == RuleField::Pattern {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let action_style = if *editing_field == RuleField::Action {
                        Style::default().fg(Color::Yellow).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };

                    popup_content.push(Line::from(vec![
                        Span::styled("Pattern: ", Style::default().fg(Color::Cyan).bold()),
                        Span::styled(pattern, pattern_style),
                    ]));

                    popup_content.push(Line::from(vec![
                        Span::styled("Action: ", Style::default().fg(Color::Cyan).bold()),
                        Span::styled(action, action_style),
                    ]));

                    popup_content.push(Line::from(""));
                    popup_content.push(Line::from(vec![
                        Span::styled("Tab", Style::default().fg(Color::Yellow).bold()),
                        Span::styled(" - Switch fields  ", Style::default().fg(Color::Gray)),
                        Span::styled("Type", Style::default().fg(Color::Yellow).bold()),
                        Span::styled(" - Edit", Style::default().fg(Color::Gray)),
                    ]));
                }
                EditMode::None => {
                    popup_content.push(Line::from(vec![
                        Span::styled("Current Value: ", Style::default().fg(Color::Green).bold()),
                        Span::raw(&item.value),
                    ]));
                }
            }

            // Add suggestions if available
            if !item.suggestions.is_empty() {
                popup_content.push(Line::from(""));
                popup_content.push(Line::from(vec![Span::styled(
                    "Suggestions: ",
                    Style::default().fg(Color::Yellow).bold(),
                )]));
                let suggestions_text = item.suggestions.join(", ");
                popup_content.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        suggestions_text,
                        Style::default().fg(Color::Rgb(200, 200, 200)),
                    ),
                ]));
            }

            popup_content.push(Line::from(""));
            popup_content.push(Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green).bold()),
                Span::styled(" - Apply  ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Red).bold()),
                Span::styled(" - Cancel", Style::default().fg(Color::Gray)),
            ]));

            let popup = Paragraph::new(popup_content)
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Magenta))
                        .border_type(BorderType::Double)
                        .title(" Edit Value ")
                        .title_style(Style::default().fg(Color::Magenta).bold()),
                )
                .wrap(Wrap { trim: true });

            f.render_widget(Clear, popup_area);
            f.render_widget(popup, popup_area);
        }
    }

    // Helper function to create centered rectangles for popups
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    #[allow(dead_code)]
    fn get_static_panel_items(panel: FocusedPanel) -> Vec<ListItem<'static>> {
        match panel {
            FocusedPanel::General => vec![
                ListItem::new("gaps_in: 5"),
                ListItem::new("gaps_out: 20"),
                ListItem::new("border_size: 2"),
                ListItem::new("col.active_border: rgba(33ccffee)"),
                ListItem::new("col.inactive_border: rgba(595959aa)"),
                ListItem::new("resize_on_border: false"),
                ListItem::new("extend_border_grab_area: 15"),
                ListItem::new("hover_icon_on_border: true"),
            ],
            FocusedPanel::Input => vec![
                ListItem::new("kb_layout: us"),
                ListItem::new("kb_variant: "),
                ListItem::new("kb_model: "),
                ListItem::new("kb_options: "),
                ListItem::new("kb_rules: "),
                ListItem::new("follow_mouse: 1"),
                ListItem::new("mouse_refocus: true"),
                ListItem::new("sensitivity: 0.0"),
            ],
            FocusedPanel::Decoration => vec![
                ListItem::new("rounding: 10"),
                ListItem::new("blur.enabled: true"),
                ListItem::new("blur.size: 3"),
                ListItem::new("blur.passes: 1"),
                ListItem::new("drop_shadow: true"),
                ListItem::new("shadow_range: 4"),
                ListItem::new("shadow_render_power: 3"),
                ListItem::new("col.shadow: rgba(1a1a1aee)"),
            ],
            FocusedPanel::Animations => vec![
                ListItem::new("enabled: true"),
                ListItem::new("bezier: myBezier, 0.05, 0.9, 0.1, 1.05"),
                ListItem::new("animation: windows, 1, 7, myBezier"),
                ListItem::new("animation: windowsOut, 1, 7, default, popin 80%"),
                ListItem::new("animation: border, 1, 10, default"),
                ListItem::new("animation: borderangle, 1, 8, default"),
                ListItem::new("animation: fade, 1, 7, default"),
                ListItem::new("animation: workspaces, 1, 6, default"),
            ],
            FocusedPanel::Gestures => vec![
                ListItem::new("workspace_swipe: false"),
                ListItem::new("workspace_swipe_fingers: 3"),
                ListItem::new("workspace_swipe_distance: 300"),
                ListItem::new("workspace_swipe_invert: true"),
                ListItem::new("workspace_swipe_min_speed_to_force: 30"),
                ListItem::new("workspace_swipe_cancel_ratio: 0.5"),
                ListItem::new("workspace_swipe_create_new: true"),
                ListItem::new("workspace_swipe_forever: false"),
            ],
            FocusedPanel::Binds => vec![
                ListItem::new("$mainMod = SUPER"),
                ListItem::new("bind = $mainMod, Q, exec, kitty"),
                ListItem::new("bind = $mainMod, C, killactive,"),
                ListItem::new("bind = $mainMod, M, exit,"),
                ListItem::new("bind = $mainMod, E, exec, dolphin"),
                ListItem::new("bind = $mainMod, V, togglefloating,"),
                ListItem::new("bind = $mainMod, R, exec, wofi --show drun"),
                ListItem::new("bind = $mainMod, P, pseudo,"),
            ],
            FocusedPanel::WindowRules => vec![
                ListItem::new("windowrule = float, ^(kitty)$"),
                ListItem::new("windowrule = opacity 0.8 0.8, ^(Alacritty)$"),
                ListItem::new("windowrule = size 800 600, ^(pavucontrol)$"),
                ListItem::new("windowrule = center, ^(pavucontrol)$"),
                ListItem::new(
                    "windowrulev2 = float, class:^(firefox)$, title:^(Picture-in-Picture)$",
                ),
                ListItem::new(
                    "windowrulev2 = pin, class:^(firefox)$, title:^(Picture-in-Picture)$",
                ),
                ListItem::new("windowrulev2 = opacity 0.9 0.9, class:^(Code)$"),
                ListItem::new("windowrulev2 = workspace 2, class:^(firefox)$"),
            ],
            FocusedPanel::LayerRules => vec![
                ListItem::new("layerrule = blur, rofi"),
                ListItem::new("layerrule = ignorezero, rofi"),
                ListItem::new("layerrule = blur, notifications"),
                ListItem::new("layerrule = ignorezero, notifications"),
                ListItem::new("layerrule = blur, gtk-layer-shell"),
                ListItem::new("layerrule = ignorezero, gtk-layer-shell"),
                ListItem::new("layerrule = blur, launcher"),
                ListItem::new("layerrule = ignorezero, launcher"),
            ],
            FocusedPanel::Misc => vec![
                ListItem::new("disable_hyprland_logo: false"),
                ListItem::new("disable_splash_rendering: false"),
                ListItem::new("mouse_move_enables_dpms: true"),
                ListItem::new("key_press_enables_dpms: false"),
                ListItem::new("always_follow_on_dnd: true"),
                ListItem::new("layers_hog_keyboard_focus: true"),
                ListItem::new("animate_manual_resizes: false"),
                ListItem::new("animate_mouse_windowdragging: false"),
            ],
            FocusedPanel::Import => vec![
                ListItem::new("Select import source from the options above"),
                ListItem::new("Press Enter to start import wizard"),
                ListItem::new("Supports .conf, .json, .toml, .yaml formats"),
            ],
            FocusedPanel::Export => vec![
                ListItem::new("Select export format from the options above"),
                ListItem::new("Press Enter to start export wizard"),
                ListItem::new("Export to various formats for sharing"),
            ],
        }
    }

    #[allow(dead_code)]
    fn get_panel_items_count(&self, panel: FocusedPanel) -> usize {
        self.config_items
            .get(&panel)
            .map(|items| items.len())
            .unwrap_or(0)
    }

    #[allow(dead_code)]
    fn get_list_state(&self, panel: FocusedPanel) -> &ListState {
        match panel {
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

    fn get_list_state_mut(&mut self, panel: FocusedPanel) -> &mut ListState {
        match panel {
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

    pub fn scroll_up(&mut self) {
        let all_items = self
            .config_items
            .get(&self.current_tab)
            .cloned()
            .unwrap_or_default();
        if all_items.is_empty() {
            return;
        }

        // Get the current visible items (filtered and paginated)
        let filtered_items = self.filter_items_progressive(&all_items);
        let paginated_items = self.get_paginated_items(&filtered_items);

        if paginated_items.is_empty() {
            return;
        }

        let list_state = self.get_current_list_state();
        let selected = list_state.selected().unwrap_or(0);
        if selected > 0 {
            list_state.select(Some(selected - 1));
        } else {
            // Wrap to the last item on current page
            list_state.select(Some(paginated_items.len() - 1));
        }
    }

    pub fn scroll_down(&mut self) {
        let all_items = self
            .config_items
            .get(&self.current_tab)
            .cloned()
            .unwrap_or_default();
        if all_items.is_empty() {
            return;
        }

        // Get the current visible items (filtered and paginated)
        let filtered_items = self.filter_items_progressive(&all_items);
        let paginated_items = self.get_paginated_items(&filtered_items);

        if paginated_items.is_empty() {
            return;
        }

        let list_state = self.get_current_list_state();
        let selected = list_state.selected().unwrap_or(0);
        if selected < paginated_items.len() - 1 {
            list_state.select(Some(selected + 1));
        } else {
            // Wrap to the first item on current page
            list_state.select(Some(0));
        }
    }

    pub async fn start_editing(&mut self) -> Result<(), anyhow::Error> {
        // Get the currently selected item from current tab
        let config_items = self
            .config_items
            .get(&self.current_tab)
            .cloned()
            .unwrap_or_default();
        if config_items.is_empty() {
            return Ok(());
        }

        let list_state = self.get_current_list_state();
        let selected_index = list_state.selected().unwrap_or(0);

        if let Some(item) = config_items.get(selected_index) {
            self.editing_item = Some((self.current_tab, item.key.clone()));

            // Set edit mode based on data type and panel
            self.edit_mode =
                if self.current_tab == FocusedPanel::Binds && item.key.starts_with("bind_") {
                    // Special handling for keybinds
                    self.parse_keybind_for_editing(&item.value)
                } else if (self.current_tab == FocusedPanel::WindowRules
                    && item.key.starts_with("window_rule_"))
                    || (self.current_tab == FocusedPanel::LayerRules
                        && (item.key.starts_with("layer_rule_")
                            || item.key.starts_with("workspace_rule_")))
                {
                    // Special handling for rules
                    self.parse_rule_for_editing(&item.value, &self.current_tab)
                } else {
                    match &item.data_type {
                        ConfigDataType::Boolean => {
                            let current_value = item.value.to_lowercase() == "true";
                            EditMode::Boolean { current_value }
                        }
                        ConfigDataType::Integer { min, max } => {
                            if let (Some(min_val), Some(max_val)) = (min, max) {
                                let current_value =
                                    item.value.parse::<f32>().unwrap_or(*min_val as f32);
                                EditMode::Slider {
                                    current_value,
                                    min: *min_val as f32,
                                    max: *max_val as f32,
                                    step: 1.0,
                                }
                            } else {
                                EditMode::Text {
                                    current_value: item.value.clone(),
                                    cursor_pos: item.value.len(),
                                }
                            }
                        }
                        ConfigDataType::Float { min, max } => {
                            if let (Some(min_val), Some(max_val)) = (min, max) {
                                let current_value = item.value.parse::<f32>().unwrap_or(*min_val);
                                EditMode::Slider {
                                    current_value,
                                    min: *min_val,
                                    max: *max_val,
                                    step: 0.1,
                                }
                            } else {
                                EditMode::Text {
                                    current_value: item.value.clone(),
                                    cursor_pos: item.value.len(),
                                }
                            }
                        }
                        ConfigDataType::Keyword { options } => {
                            let selected = options
                                .iter()
                                .position(|opt| opt == &item.value)
                                .unwrap_or(0);
                            EditMode::Select {
                                options: options.clone(),
                                selected,
                            }
                        }
                        _ => EditMode::Text {
                            current_value: item.value.clone(),
                            cursor_pos: item.value.len(),
                        },
                    }
                };
        }

        Ok(())
    }

    fn parse_keybind_for_editing(&self, display_string: &str) -> EditMode {
        // Parse display string like "SUPER + q → exec [kitty]"
        if let Some((key_part, command_part)) = display_string.split_once(" → ") {
            let key_part = key_part.trim();
            let command_part = command_part.trim();

            // Parse modifiers and key
            let (modifiers, key) = if let Some((mods, k)) = key_part.rsplit_once(" + ") {
                (
                    mods.split(" + ").map(|s| s.to_string()).collect(),
                    k.to_string(),
                )
            } else {
                (vec![], key_part.to_string())
            };

            // Parse dispatcher and args
            let (dispatcher, args) = if let Some((disp, arg_part)) = command_part.split_once(' ') {
                let args = if arg_part.starts_with('[') && arg_part.ends_with(']') {
                    arg_part
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .to_string()
                } else {
                    arg_part.to_string()
                };
                (disp.to_string(), args)
            } else {
                (command_part.to_string(), String::new())
            };

            return EditMode::Keybind {
                modifiers,
                key,
                dispatcher,
                args,
                editing_field: KeybindField::Dispatcher, // Start with dispatcher
            };
        }

        // Fallback to text editing
        EditMode::Text {
            current_value: display_string.to_string(),
            cursor_pos: display_string.len(),
        }
    }

    #[allow(dead_code)]
    pub async fn apply_edit(&mut self) -> Result<(), anyhow::Error> {
        if let Some((panel, key)) = &self.editing_item.clone() {
            let new_value = match &self.edit_mode {
                EditMode::Text { current_value, .. } => current_value.clone(),
                EditMode::Boolean { current_value } => current_value.to_string(),
                EditMode::Select { options, selected } => {
                    options.get(*selected).cloned().unwrap_or_default()
                }
                EditMode::Slider { current_value, .. } => {
                    // Format as integer or float based on whether it has decimal places
                    if current_value.fract() == 0.0 {
                        (*current_value as i32).to_string()
                    } else {
                        format!("{current_value:.2}")
                    }
                }
                EditMode::Keybind {
                    modifiers,
                    key,
                    dispatcher,
                    args,
                    ..
                } => {
                    // Create display string for keybind
                    let mod_string = if modifiers.is_empty() {
                        String::new()
                    } else {
                        format!("{} + ", modifiers.join(" + "))
                    };

                    let args_string = if args.is_empty() {
                        String::new()
                    } else {
                        format!(" [{args}]")
                    };

                    format!("{mod_string}{key} → {dispatcher}{args_string}")
                }
                EditMode::Rule {
                    rule_type,
                    pattern,
                    action,
                    ..
                } => {
                    // Format rule for display and application
                    match rule_type {
                        RuleType::Window => format!("windowrule = {action}, {pattern}"),
                        RuleType::Layer => format!("layerrule = {action}, {pattern}"),
                        RuleType::Workspace => format!("workspace = {action}, {pattern}"),
                    }
                }
                EditMode::None => return Ok(()),
            };

            // Update the configuration item in UI
            if let Some(items) = self.config_items.get_mut(panel) {
                for item in items.iter_mut() {
                    if item.key == *key {
                        item.value = new_value.clone();
                        break;
                    }
                }
            }

            self.cancel_edit();
        }

        Ok(())
    }

    pub async fn apply_edit_with_hyprctl(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<(), anyhow::Error> {
        if let Some((panel, key)) = &self.editing_item.clone() {
            let new_value = match &self.edit_mode {
                EditMode::Text { current_value, .. } => current_value.clone(),
                EditMode::Boolean { current_value } => current_value.to_string(),
                EditMode::Select { options, selected } => {
                    options.get(*selected).cloned().unwrap_or_default()
                }
                EditMode::Slider { current_value, .. } => {
                    // Format as integer or float based on whether it has decimal places
                    if current_value.fract() == 0.0 {
                        (*current_value as i32).to_string()
                    } else {
                        format!("{current_value:.2}")
                    }
                }
                EditMode::Keybind {
                    modifiers,
                    key,
                    dispatcher,
                    args,
                    ..
                } => {
                    // Create display string for keybind
                    let mod_string = if modifiers.is_empty() {
                        String::new()
                    } else {
                        format!("{} + ", modifiers.join(" + "))
                    };

                    let args_string = if args.is_empty() {
                        String::new()
                    } else {
                        format!(" [{args}]")
                    };

                    format!("{mod_string}{key} → {dispatcher}{args_string}")
                }
                EditMode::Rule {
                    rule_type,
                    pattern,
                    action,
                    ..
                } => {
                    // Format rule for display and application
                    match rule_type {
                        RuleType::Window => format!("windowrule = {action}, {pattern}"),
                        RuleType::Layer => format!("layerrule = {action}, {pattern}"),
                        RuleType::Workspace => format!("workspace = {action}, {pattern}"),
                    }
                }
                EditMode::None => return Ok(()),
            };

            // Get the hyprctl key for this configuration option
            let hypr_key = self.get_hyprctl_key(panel, key);

            if let Some(hypr_key) = hypr_key {
                // Apply the change via hyprctl
                match hyprctl.set_option(&hypr_key, &new_value).await {
                    Ok(()) => {
                        // Successfully applied - update the UI
                        if let Some(items) = self.config_items.get_mut(panel) {
                            for item in items.iter_mut() {
                                if item.key == *key {
                                    item.value = new_value;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Failed to apply - show error but don't update UI
                        self.show_popup = true;
                        self.popup_message = format!("Failed to apply setting: {e}");
                        self.cancel_edit();
                        return Err(e);
                    }
                }
            } else {
                // No hyprctl mapping - just update UI (for items like binds/rules)
                if let Some(items) = self.config_items.get_mut(panel) {
                    for item in items.iter_mut() {
                        if item.key == *key {
                            item.value = new_value;
                            break;
                        }
                    }
                }
            }

            self.cancel_edit();
        }

        Ok(())
    }

    pub fn get_hyprctl_key(&self, panel: &FocusedPanel, key: &str) -> Option<String> {
        match panel {
            FocusedPanel::General => match key {
                "gaps_in" => Some("general:gaps_in".to_string()),
                "gaps_out" => Some("general:gaps_out".to_string()),
                "border_size" => Some("general:border_size".to_string()),
                "col.active_border" => Some("general:col.active_border".to_string()),
                "col.inactive_border" => Some("general:col.inactive_border".to_string()),
                _ => None,
            },
            FocusedPanel::Input => match key {
                "kb_layout" => Some("input:kb_layout".to_string()),
                "follow_mouse" => Some("input:follow_mouse".to_string()),
                "sensitivity" => Some("input:sensitivity".to_string()),
                _ => None,
            },
            FocusedPanel::Decoration => match key {
                "rounding" => Some("decoration:rounding".to_string()),
                "blur.enabled" => Some("decoration:blur:enabled".to_string()),
                "blur.size" => Some("decoration:blur:size".to_string()),
                _ => None,
            },
            FocusedPanel::Animations => match key {
                "animations.enabled" => Some("animations:enabled".to_string()),
                _ => None,
            },
            FocusedPanel::Gestures => match key {
                "gestures.workspace_swipe" => Some("gestures:workspace_swipe".to_string()),
                "gestures.workspace_swipe_fingers" => {
                    Some("gestures:workspace_swipe_fingers".to_string())
                }
                "gestures.workspace_swipe_distance" => {
                    Some("gestures:workspace_swipe_distance".to_string())
                }
                "gestures.workspace_swipe_invert" => {
                    Some("gestures:workspace_swipe_invert".to_string())
                }
                _ => None,
            },
            FocusedPanel::Misc => match key {
                "misc.disable_hyprland_logo" => Some("misc:disable_hyprland_logo".to_string()),
                "misc.disable_splash_rendering" => {
                    Some("misc:disable_splash_rendering".to_string())
                }
                "misc.mouse_move_enables_dpms" => Some("misc:mouse_move_enables_dpms".to_string()),
                "misc.vfr" => Some("misc:vfr".to_string()),
                "misc.vrr" => Some("misc:vrr".to_string()),
                _ => None,
            },
            // Binds, WindowRules, and LayerRules need different hyprctl commands
            _ => None,
        }
    }

    pub fn cancel_edit(&mut self) {
        self.edit_mode = EditMode::None;
        self.editing_item = None;
    }

    // Search functionality methods
    #[allow(dead_code)]
    pub fn start_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_cursor = 0;
    }

    #[allow(dead_code)]
    pub fn exit_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.search_cursor = 0;
    }

    #[allow(dead_code)]
    pub fn add_search_char(&mut self, c: char) {
        if self.search_mode {
            self.search_query.insert(self.search_cursor, c);
            self.search_cursor += 1;
        }
    }

    #[allow(dead_code)]
    pub fn remove_search_char(&mut self) {
        if self.search_mode && self.search_cursor > 0 {
            self.search_cursor -= 1;
            self.search_query.remove(self.search_cursor);
        }
    }

    pub fn move_search_cursor_left(&mut self) {
        if self.search_cursor > 0 {
            self.search_cursor -= 1;
        }
    }

    pub fn move_search_cursor_right(&mut self) {
        if self.search_cursor < self.search_query.len() {
            self.search_cursor += 1;
        }
    }

    // Debounced search methods
    pub fn add_search_char_debounced(&mut self, c: char) {
        if self.search_mode {
            // Update the pending search query immediately for visual feedback
            self.pending_search_query.insert(self.search_cursor, c);
            self.search_cursor += 1;

            // Update debounce timing
            self.last_search_input = std::time::Instant::now();
            self.debounced_search_active = true;
        }
    }

    pub fn remove_search_char_debounced(&mut self) {
        if self.search_mode && self.search_cursor > 0 {
            self.search_cursor -= 1;
            self.pending_search_query.remove(self.search_cursor);

            // Update debounce timing
            self.last_search_input = std::time::Instant::now();
            self.debounced_search_active = true;
        }
    }

    pub fn update_debounced_search(&mut self) -> bool {
        if !self.debounced_search_active {
            return false;
        }

        // Check if debounce delay has passed
        if self.last_search_input.elapsed() >= self.search_debounce_delay {
            // Apply the pending search query
            let query_changed = self.search_query != self.pending_search_query;
            self.search_query = self.pending_search_query.clone();
            self.debounced_search_active = false;

            return query_changed;
        }

        false
    }

    pub fn start_search_debounced(&mut self) {
        self.search_mode = true;
        self.pending_search_query = self.search_query.clone();
        self.search_cursor = self.search_query.len();
        self.debounced_search_active = false;
    }

    pub fn cancel_search_debounced(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.pending_search_query.clear();
        self.search_cursor = 0;
        self.debounced_search_active = false;
    }

    pub fn get_display_search_query(&self) -> &str {
        if self.debounced_search_active && self.search_mode {
            &self.pending_search_query
        } else {
            &self.search_query
        }
    }

    pub fn filter_items(&mut self, items: &[ConfigItem]) -> Vec<ConfigItem> {
        if self.search_query.is_empty() {
            return items.to_vec();
        }

        let query = self.search_query.to_lowercase();

        // Check cache first
        let cache_key = format!("{}:{}", self.current_tab as u8, query);
        if let Some(cached_results) = self.search_cache.get(&cache_key) {
            return cached_results.clone();
        }

        // Perform filtering
        let filtered_items: Vec<ConfigItem> = items
            .iter()
            .filter(|item| {
                item.key.to_lowercase().contains(&query)
                    || item.value.to_lowercase().contains(&query)
                    || item.description.to_lowercase().contains(&query)
            })
            .cloned()
            .collect();

        // Cache the results
        self.cache_search_results(cache_key, filtered_items.clone());

        filtered_items
    }

    fn cache_search_results(&mut self, cache_key: String, results: Vec<ConfigItem>) {
        // Implement LRU-like behavior by removing oldest entries when cache is full
        if self.search_cache.len() >= self.search_cache_max_size {
            // Remove a random entry (simple eviction strategy)
            if let Some(key_to_remove) = self.search_cache.keys().next().cloned() {
                self.search_cache.remove(&key_to_remove);
            }
        }

        self.search_cache.insert(cache_key, results);
    }

    pub fn clear_search_cache(&mut self) {
        self.search_cache.clear();
    }

    #[allow(dead_code)]
    pub fn invalidate_search_cache_for_panel(&mut self, panel: FocusedPanel) {
        let panel_prefix = format!("{}:", panel as u8);
        let keys_to_remove: Vec<String> = self
            .search_cache
            .keys()
            .filter(|key| key.starts_with(&panel_prefix))
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.search_cache.remove(&key);
        }
    }

    // Progressive search for very large datasets
    pub fn filter_items_progressive(&mut self, items: &[ConfigItem]) -> Vec<ConfigItem> {
        if self.search_query.is_empty() {
            return items.to_vec();
        }

        let query = self.search_query.to_lowercase();

        // Check cache first
        let cache_key = format!("{}:{}", self.current_tab as u8, query);
        if let Some(cached_results) = self.search_cache.get(&cache_key) {
            return cached_results.clone();
        }

        // Use progressive search for large datasets
        if items.len() > self.progressive_search_threshold {
            let mut filtered_items = Vec::new();

            // Process items in chunks to maintain responsiveness
            for chunk in items.chunks(self.progressive_search_chunk_size) {
                let chunk_results: Vec<ConfigItem> = chunk
                    .iter()
                    .filter(|item| {
                        item.key.to_lowercase().contains(&query)
                            || item.value.to_lowercase().contains(&query)
                            || item.description.to_lowercase().contains(&query)
                    })
                    .cloned()
                    .collect();

                filtered_items.extend(chunk_results);

                // Yield control periodically for UI responsiveness
                // Note: In a real implementation, you might want to add actual yielding
                // or async processing here for extremely large datasets
            }

            // Cache the results
            self.cache_search_results(cache_key, filtered_items.clone());

            filtered_items
        } else {
            // Use standard filtering for smaller datasets
            self.filter_items(items)
        }
    }

    // Lazy loading / pagination methods
    pub fn get_paginated_items(&self, items: &[ConfigItem]) -> Vec<ConfigItem> {
        let current_page = self
            .current_page
            .get(&self.current_tab)
            .copied()
            .unwrap_or(0);
        let start_idx = current_page * self.page_size;
        let end_idx = (start_idx + self.page_size).min(items.len());

        if start_idx >= items.len() {
            return Vec::new();
        }

        items[start_idx..end_idx].to_vec()
    }

    pub fn update_pagination(&mut self, panel: FocusedPanel, total_items: usize) {
        let total_pages = if total_items == 0 {
            1
        } else {
            total_items.div_ceil(self.page_size)
        };

        self.total_pages.insert(panel, total_pages);

        // Ensure current page is within bounds
        let current_page = self.current_page.get(&panel).copied().unwrap_or(0);
        if current_page >= total_pages {
            self.current_page
                .insert(panel, total_pages.saturating_sub(1));
        }
    }

    pub fn next_page(&mut self) {
        let current_page = self
            .current_page
            .get(&self.current_tab)
            .copied()
            .unwrap_or(0);
        let total_pages = self
            .total_pages
            .get(&self.current_tab)
            .copied()
            .unwrap_or(1);

        if current_page + 1 < total_pages {
            self.current_page.insert(self.current_tab, current_page + 1);
            // Reset list selection to top of new page
            self.get_list_state_mut(self.current_tab).select(Some(0));
        }
    }

    pub fn prev_page(&mut self) {
        let current_page = self
            .current_page
            .get(&self.current_tab)
            .copied()
            .unwrap_or(0);

        if current_page > 0 {
            self.current_page.insert(self.current_tab, current_page - 1);
            // Reset list selection to top of new page
            self.get_list_state_mut(self.current_tab).select(Some(0));
        }
    }

    pub fn get_pagination_info(&self) -> (usize, usize, usize) {
        let current_page = self
            .current_page
            .get(&self.current_tab)
            .copied()
            .unwrap_or(0);
        let total_pages = self
            .total_pages
            .get(&self.current_tab)
            .copied()
            .unwrap_or(1);
        let start_item = current_page * self.page_size + 1;
        (current_page + 1, total_pages, start_item)
    }

    pub fn update_all_pagination(&mut self) {
        // Update pagination for all panels based on their config items
        let panels = vec![
            FocusedPanel::General,
            FocusedPanel::Input,
            FocusedPanel::Decoration,
            FocusedPanel::Animations,
            FocusedPanel::Gestures,
            FocusedPanel::Binds,
            FocusedPanel::WindowRules,
            FocusedPanel::LayerRules,
            FocusedPanel::Misc,
        ];

        for panel in panels {
            let item_count = self
                .config_items
                .get(&panel)
                .map(|items| items.len())
                .unwrap_or(0);
            self.update_pagination(panel, item_count);
        }
    }

    // Virtualization methods
    pub fn get_visible_item_range(
        &self,
        available_height: usize,
        total_items: usize,
    ) -> (usize, usize) {
        if total_items == 0 || available_height == 0 {
            return (0, 0);
        }

        // Calculate how many items can fit in the available height
        let max_visible_items = (available_height.saturating_sub(2)) / self.item_height; // -2 for borders
        let max_visible_items = max_visible_items.max(1); // Ensure at least 1 item is visible

        // Get the currently selected item to determine scroll position
        let selected_index = self
            .get_list_state(self.current_tab)
            .selected()
            .unwrap_or(0);

        // Calculate the start index to keep the selected item visible
        let start_index = if selected_index < max_visible_items / 2 {
            0
        } else if selected_index + max_visible_items / 2 >= total_items {
            total_items.saturating_sub(max_visible_items)
        } else {
            selected_index.saturating_sub(max_visible_items / 2)
        };

        let end_index = (start_index + max_visible_items).min(total_items);

        (start_index, end_index)
    }

    pub fn get_virtualized_items(
        &self,
        items: &[ConfigItem],
        available_height: usize,
    ) -> (Vec<ConfigItem>, usize, usize) {
        // Disable virtualization for small item counts where it's not needed
        // This fixes the issue where users can't see all items in categories with reasonable counts
        if items.len() <= 100 {
            return (items.to_vec(), 0, items.len());
        }

        let (start_idx, end_idx) = self.get_visible_item_range(available_height, items.len());

        if start_idx >= items.len() {
            return (Vec::new(), 0, 0);
        }

        let visible_items = items[start_idx..end_idx].to_vec();
        (visible_items, start_idx, end_idx)
    }

    // Efficient ListItem creation with optimization
    pub fn create_optimized_list_items(
        items: &[ConfigItem],
        theme: &crate::theme::Theme,
    ) -> Vec<ListItem<'static>> {
        // Pre-allocate the vector with known capacity for better performance
        let mut list_items = Vec::with_capacity(items.len());

        for item in items {
            let value_style = theme.data_type_style(&item.data_type);

            let key_display = if item.key.len() > 25 {
                format!("{}...", &item.key[..22])
            } else {
                item.key.clone()
            };

            let value_display = if item.value.len() > 40 {
                format!("{}...", &item.value[..37])
            } else {
                item.value.clone()
            };

            // Create the ListItem directly without intermediate allocations
            let line = Line::from(vec![
                Span::styled(
                    format!("{key_display:<28}"),
                    Style::default().fg(Color::Rgb(200, 200, 255)).bold(),
                ),
                Span::raw("│ "),
                Span::styled(value_display, value_style.bold()),
            ]);

            let description_line = Line::from(vec![Span::styled(
                format!("  {}", item.description),
                Style::default().fg(Color::DarkGray).italic(),
            )]);

            list_items.push(ListItem::new(vec![line, description_line]));
        }

        // Explicitly shrink the vector if it has excess capacity
        list_items.shrink_to_fit();
        list_items
    }

    // Memory optimization for large collections
    pub fn optimize_memory_usage(&mut self) {
        // Periodically clean up config_items to free unused memory
        for (_, items) in self.config_items.iter_mut() {
            items.shrink_to_fit();
        }

        // Clear search cache to free memory and ensure fresh results
        self.clear_search_cache();

        // Increment cache generation to invalidate old cached data
        self.item_cache_generation = self.item_cache_generation.wrapping_add(1);
    }

    // Batch processing for very large datasets
    #[allow(dead_code)]
    pub fn process_items_in_batches<F>(
        &self,
        items: &[ConfigItem],
        batch_size: usize,
        mut processor: F,
    ) where
        F: FnMut(&[ConfigItem]),
    {
        for chunk in items.chunks(batch_size) {
            processor(chunk);
        }
    }

    fn render_search_bar(&self, f: &mut Frame, area: Rect) {
        let display_query = self.get_display_search_query();
        let search_text = if self.search_mode {
            if self.debounced_search_active {
                format!("Search: {display_query}⏳") // Show pending indicator
            } else {
                format!("Search: {display_query}")
            }
        } else {
            format!("Search: {display_query} (Press / to edit)")
        };

        let search_style = self.theme.search_style(self.search_mode);

        let search_paragraph = Paragraph::new(search_text)
            .style(search_style)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if self.search_mode {
                        self.theme.border_style(true)
                    } else {
                        self.theme.border_style(false)
                    })
                    .border_type(BorderType::Rounded)
                    .title(if !self.search_query.is_empty() {
                        " Search (filtered) ".to_string()
                    } else {
                        " Search ".to_string()
                    })
                    .title_style(search_style.bold()),
            );

        f.render_widget(search_paragraph, area);

        // Render cursor if in search mode
        if self.search_mode {
            let cursor_x = area.x + 1 + "Search: ".len() as u16 + self.search_cursor as u16;
            let cursor_y = area.y + 1;
            if cursor_x < area.x + area.width - 1 {
                f.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }

    // Theme management methods
    pub fn set_theme(&mut self, scheme: crate::theme::ColorScheme) {
        self.theme = crate::theme::Theme::from_scheme(scheme);
    }

    pub fn next_theme(&mut self) -> crate::theme::ColorScheme {
        let next_scheme = self.theme.scheme.next();
        self.theme = crate::theme::Theme::from_scheme(next_scheme.clone());
        next_scheme
    }

    #[allow(dead_code)]
    pub fn previous_theme(&mut self) -> crate::theme::ColorScheme {
        let prev_scheme = self.theme.scheme.previous();
        self.theme = crate::theme::Theme::from_scheme(prev_scheme.clone());
        prev_scheme
    }

    #[allow(dead_code)]
    pub fn get_current_theme(&self) -> &crate::theme::ColorScheme {
        &self.theme.scheme
    }

    // Help system methods
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.help_scroll = 0; // Reset scroll when toggling
    }

    #[allow(dead_code)]
    pub fn close_help(&mut self) {
        self.show_help = false;
        self.help_scroll = 0;
    }

    pub fn scroll_help_up(&mut self) {
        if self.help_scroll > 0 {
            self.help_scroll -= 1;
        }
    }

    pub fn scroll_help_down(&mut self) {
        // We'll limit this based on content length in the render method
        self.help_scroll += 1;
    }

    pub fn scroll_help_page_up(&mut self) {
        if self.help_scroll >= 10 {
            self.help_scroll -= 10;
        } else {
            self.help_scroll = 0;
        }
    }

    pub fn scroll_help_page_down(&mut self) {
        self.help_scroll += 10;
    }

    pub fn scroll_help_to_top(&mut self) {
        self.help_scroll = 0;
    }

    pub fn scroll_help_to_bottom(&mut self) {
        // Set to a large value; will be clamped in render
        self.help_scroll = 9999;
    }

    fn render_help_overlay(&self, f: &mut Frame, area: Rect) {
        let help_area = Self::centered_rect(90, 85, area);

        let help_content = vec![
            Line::from(vec![Span::styled(
                "📖 R-Hyprconfig Help System",
                Style::default().fg(self.theme.accent_primary).bold(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "🔍 Navigation & Search",
                Style::default().fg(self.theme.accent_secondary).bold(),
            )]),
            Line::from("  Tab/→              Next panel"),
            Line::from("  Shift+Tab/←        Previous panel"),
            Line::from("  ↑↓                 Navigate items"),
            Line::from("  PgUp/PgDn           Change page"),
            Line::from("  /                  Start search"),
            Line::from("  P                  Preview changes"),
            Line::from("  Esc                Exit search/dialogs"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "⚙️ Configuration",
                Style::default().fg(self.theme.accent_secondary).bold(),
            )]),
            Line::from("  Enter              Edit selected item"),
            Line::from("  P                  Preview setting changes"),
            Line::from("  S                  Save configuration"),
            Line::from("  R                  Reload configuration"),
            Line::from("  A                  Add new item"),
            Line::from("  D                  Delete selected item"),
            Line::from("  Ctrl+Z             Undo changes"),
            Line::from("  Ctrl+Y             Redo changes"),
            Line::from("  E                  Export configuration (TOML)"),
            Line::from("  M                  Import configuration"),
            Line::from(if self.nixos_env.is_nixos {
                "  N                  Export as NixOS configuration"
            } else {
                "  N                  Export as NixOS (requires NixOS)"
            }),
            Line::from("  B                  Batch configuration management"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "🎨 Interface",
                Style::default().fg(self.theme.accent_secondary).bold(),
            )]),
            Line::from("  T                  Switch theme"),
            Line::from("  F1                 Theme information"),
            Line::from("  ?/F1               Show this help"),
            Line::from("  E                  Show error details"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "📂 Configuration Panels",
                Style::default().fg(self.theme.accent_secondary).bold(),
            )]),
            Line::from("  General            Basic Hyprland settings"),
            Line::from("  Input              Keyboard & mouse configuration"),
            Line::from("  Decoration         Window borders & appearance"),
            Line::from("  Animations         Window & workspace animations"),
            Line::from("  Gestures           Touchpad gesture settings"),
            Line::from("  Binds              Keyboard shortcuts"),
            Line::from("  Win Rules          Window-specific rules"),
            Line::from("  Layer Rules        Layer-specific settings"),
            Line::from("  Misc               Miscellaneous options"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "🔧 Advanced Features",
                Style::default().fg(self.theme.accent_secondary).bold(),
            )]),
            Line::from("  • Automatic backup of configurations"),
            Line::from("  • Real-time validation with error detection"),
            Line::from("  • Search across all configuration options"),
            Line::from("  • Support for NixOS declarative configs"),
            Line::from("  • Theme persistence across sessions"),
            Line::from("  • High-performance handling of 500+ items"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "📋 Tips",
                Style::default().fg(self.theme.accent_secondary).bold(),
            )]),
            Line::from("  • Use search (/) to quickly find settings"),
            Line::from("  • Page navigation handles large configs smoothly"),
            Line::from("  • Validation prevents invalid configurations"),
            Line::from("  • All changes are backed up automatically"),
            Line::from("  • Theme changes are saved immediately"),
            Line::from("  • Use P to preview changes before applying"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "📖 Configuration Reference",
                Style::default().fg(self.theme.accent_warning).bold(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "General Settings:",
                Style::default().fg(Color::Cyan).bold(),
            )]),
            Line::from("  • gaps_in/out: Space between windows (pixels)"),
            Line::from("  • border_size: Window border thickness (1-20)"),
            Line::from("  • layout: Window layout algorithm (dwindle/master)"),
            Line::from("  • resize_on_border: Click border to resize (true/false)"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Input Settings:",
                Style::default().fg(Color::Cyan).bold(),
            )]),
            Line::from("  • kb_layout: Keyboard layout (us, de, fr, etc.)"),
            Line::from("  • sensitivity: Mouse sensitivity (-1.0 to 1.0)"),
            Line::from("  • repeat_rate/delay: Key repeat timing"),
            Line::from("  • follow_mouse: Focus follows mouse (0-3)"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Decoration Settings:",
                Style::default().fg(Color::Cyan).bold(),
            )]),
            Line::from("  • rounding: Corner radius for windows (0-20)"),
            Line::from("  • active_opacity: Opacity of focused windows (0.0-1.0)"),
            Line::from("  • inactive_opacity: Opacity of unfocused windows"),
            Line::from("  • drop_shadow: Enable window shadows (true/false)"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Animation Settings:",
                Style::default().fg(Color::Cyan).bold(),
            )]),
            Line::from("  • enabled: Enable animations (true/false)"),
            Line::from("  • bezier curves: Custom animation timing"),
            Line::from("  • windowsIn/Out: Window open/close animations"),
            Line::from("  • workspaces: Workspace switching animations"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "🚀 NixOS Integration",
                Style::default().fg(self.theme.accent_info).bold(),
            )]),
            Line::from(if self.nixos_env.is_nixos {
                format!(
                    "  Status: NixOS detected ({})",
                    self.nixos_env
                        .get_primary_config_location()
                        .map(|loc| loc.config_type.to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                )
            } else {
                "  Status: Traditional Linux system".to_string()
            }),
            Line::from(if self.nixos_env.is_nixos {
                "  • Import/export supports Nix expressions"
            } else {
                "  • Traditional config format supported"
            }),
            Line::from(if self.nixos_env.is_nixos {
                "  • Automatic config type detection"
            } else {
                "  • Ready for NixOS if you switch systems"
            }),
            Line::from(""),
            Line::from(vec![Span::styled(
                "⚠️ File Locations",
                Style::default()
                    .fg(self.theme.warning_style().fg.unwrap())
                    .bold(),
            )]),
            Line::from(
                if self.nixos_env.is_nixos && self.nixos_env.get_primary_config_location().is_some()
                {
                    format!(
                        "  NixOS Config: {}",
                        self.nixos_env
                            .get_primary_config_location()
                            .map(|loc| loc.path.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Not found".to_string())
                    )
                } else {
                    "  Config: ~/.config/hypr/hyprland.conf".to_string()
                },
            ),
            Line::from("  Backup: ~/.config/hypr/hyprland.conf.backup"),
            Line::from("  App config: ~/.config/r-hyprconfig/"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press Esc or ? to close help",
                Style::default().fg(self.theme.fg_muted).italic(),
            )]),
        ];

        // Calculate visible content based on scroll
        let content_height = help_area.height.saturating_sub(4) as usize; // Account for borders and padding
        let max_scroll = help_content.len().saturating_sub(content_height);
        let scroll = self.help_scroll.min(max_scroll);

        let total_lines = help_content.len();
        let visible_content: Vec<Line> = if total_lines > content_height {
            help_content
                .into_iter()
                .skip(scroll)
                .take(content_height)
                .collect()
        } else {
            help_content
        };

        let help_title = if max_scroll > 0 {
            format!(
                " Help - {} of {} lines (↑↓ to scroll) ",
                scroll + content_height.min(total_lines),
                total_lines
            )
        } else {
            " Help ".to_string()
        };

        let help_paragraph = Paragraph::new(visible_content)
            .block(
                Block::default()
                    .title(help_title)
                    .title_style(Style::default().fg(self.theme.accent_primary).bold())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.accent_primary))
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(self.theme.bg_primary)),
            )
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        // Clear the background
        f.render_widget(
            Block::default().style(Style::default().bg(Color::Black)),
            area,
        );

        // Render the help overlay
        f.render_widget(help_paragraph, help_area);
    }

    // Import/Export support methods
    pub fn update_config_item_from_import(&mut self, key: &str, value: &str) {
        // Find which panel this key belongs to and update it
        for (_panel, items) in self.config_items.iter_mut() {
            for item in items.iter_mut() {
                if item.key == key {
                    item.value = value.to_string();
                    return;
                }
            }
        }

        // If not found in existing items, try to determine the panel and add it
        let panel = self.determine_panel_for_key(key);
        let new_item = ConfigItem {
            key: key.to_string(),
            value: value.to_string(),
            data_type: ConfigDataType::String, // Default to string
            description: format!("Imported setting: {key}"),
            suggestions: Vec::new(),
        };

        self.config_items.entry(panel).or_default().push(new_item);
    }

    pub fn add_imported_keybind(&mut self, keybind: &str) {
        let new_item = ConfigItem {
            key: format!(
                "imported_bind_{}",
                self.config_items
                    .get(&FocusedPanel::Binds)
                    .map(|v| v.len())
                    .unwrap_or(0)
            ),
            value: keybind.to_string(),
            data_type: ConfigDataType::String,
            description: "Imported keybind".to_string(),
            suggestions: Vec::new(),
        };

        self.config_items
            .entry(FocusedPanel::Binds)
            .or_default()
            .push(new_item);
    }

    pub fn add_imported_window_rule(&mut self, rule: &str) {
        let new_item = ConfigItem {
            key: format!(
                "imported_windowrule_{}",
                self.config_items
                    .get(&FocusedPanel::WindowRules)
                    .map(|v| v.len())
                    .unwrap_or(0)
            ),
            value: rule.to_string(),
            data_type: ConfigDataType::String,
            description: "Imported window rule".to_string(),
            suggestions: Vec::new(),
        };

        self.config_items
            .entry(FocusedPanel::WindowRules)
            .or_default()
            .push(new_item);
    }

    pub fn add_imported_layer_rule(&mut self, rule: &str) {
        let new_item = ConfigItem {
            key: format!(
                "imported_layerrule_{}",
                self.config_items
                    .get(&FocusedPanel::LayerRules)
                    .map(|v| v.len())
                    .unwrap_or(0)
            ),
            value: rule.to_string(),
            data_type: ConfigDataType::String,
            description: "Imported layer rule".to_string(),
            suggestions: Vec::new(),
        };

        self.config_items
            .entry(FocusedPanel::LayerRules)
            .or_default()
            .push(new_item);
    }

    pub fn refresh_all_panels(&mut self) {
        // Update pagination for all panels
        self.update_all_pagination();

        // Clear search cache
        self.clear_search_cache();

        // Reset selections to top of each panel
        for panel in [
            FocusedPanel::General,
            FocusedPanel::Input,
            FocusedPanel::Decoration,
            FocusedPanel::Animations,
            FocusedPanel::Gestures,
            FocusedPanel::Binds,
            FocusedPanel::WindowRules,
            FocusedPanel::LayerRules,
            FocusedPanel::Misc,
        ] {
            self.get_list_state_mut(panel).select(Some(0));
        }
    }

    fn determine_panel_for_key(&self, key: &str) -> FocusedPanel {
        if key.starts_with("general") || key.contains("gaps") || key.contains("border") {
            FocusedPanel::General
        } else if key.starts_with("input") || key.contains("kb_") || key.contains("mouse") {
            FocusedPanel::Input
        } else if key.starts_with("decoration")
            || key.contains("rounding")
            || key.contains("shadow")
        {
            FocusedPanel::Decoration
        } else if key.starts_with("animations") || key.contains("animation") {
            FocusedPanel::Animations
        } else if key.starts_with("gestures") || key.contains("gesture") {
            FocusedPanel::Gestures
        } else if key.starts_with("misc") || key.contains("hyprland_logo") {
            FocusedPanel::Misc
        } else if key.contains("bind") {
            FocusedPanel::Binds
        } else if key.contains("windowrule") {
            FocusedPanel::WindowRules
        } else if key.contains("layerrule") {
            FocusedPanel::LayerRules
        } else {
            FocusedPanel::Misc // Default fallback
        }
    }

    fn render_batch_dialog(&self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(80, 70, area);

        match self.batch_dialog_mode {
            BatchDialogMode::ManageProfiles => {
                self.render_batch_manage_profiles(f, popup_area);
            }
            BatchDialogMode::SelectOperation => {
                self.render_batch_select_operation(f, popup_area);
            }
            BatchDialogMode::ExecuteOperation => {
                self.render_batch_execute_operation(f, popup_area);
            }
        }
    }

    fn render_batch_manage_profiles(&self, f: &mut Frame, area: Rect) {
        f.render_widget(Clear, area);

        let popup_content = vec![
            Line::from(vec![Span::styled(
                "🔧 Batch Configuration Management",
                Style::default().fg(Color::Cyan).bold(),
            )]),
            Line::from(""),
            Line::from("Manage configuration profiles:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::Yellow).bold()),
                Span::raw("Create new profile from current config"),
            ]),
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::Yellow).bold()),
                Span::raw("Select existing profile for operations"),
            ]),
            Line::from(vec![
                Span::styled("3. ", Style::default().fg(Color::Yellow).bold()),
                Span::raw("Delete profile"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Current profiles: ",
                    Style::default().fg(Color::Green).bold(),
                ),
                Span::raw("(implementation pending)"),
            ]),
            Line::from(""),
            Line::from("Press Esc to cancel"),
        ];

        let popup = Paragraph::new(popup_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Batch Management ")
                    .title_alignment(Alignment::Center),
            )
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        f.render_widget(popup, area);
    }

    fn render_batch_select_operation(&self, f: &mut Frame, area: Rect) {
        f.render_widget(Clear, area);

        let selected_profile = self.batch_selected_profile.as_deref().unwrap_or("None");

        let popup_content = vec![
            Line::from(vec![Span::styled(
                "🚀 Select Batch Operation",
                Style::default().fg(Color::Cyan).bold(),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Selected Profile: ",
                    Style::default().fg(Color::Green).bold(),
                ),
                Span::raw(selected_profile),
            ]),
            Line::from(""),
            Line::from("Choose operation type:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::Yellow).bold()),
                Span::raw("Apply - Apply profile settings to current config"),
            ]),
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::Yellow).bold()),
                Span::raw("Merge - Merge profile with current settings"),
            ]),
            Line::from(vec![
                Span::styled("3. ", Style::default().fg(Color::Yellow).bold()),
                Span::raw("Replace - Replace current config with profile"),
            ]),
            Line::from(vec![
                Span::styled("4. ", Style::default().fg(Color::Yellow).bold()),
                Span::raw("Backup - Create backup before applying"),
            ]),
            Line::from(""),
            Line::from("Press Esc to go back"),
        ];

        let popup = Paragraph::new(popup_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Operation Selection ")
                    .title_alignment(Alignment::Center),
            )
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        f.render_widget(popup, area);
    }

    fn render_batch_execute_operation(&self, f: &mut Frame, area: Rect) {
        f.render_widget(Clear, area);

        let selected_profile = self.batch_selected_profile.as_deref().unwrap_or("None");

        let operation_name = match self.batch_operation_type {
            crate::batch::BatchOperationType::Apply => "Apply",
            crate::batch::BatchOperationType::Merge => "Merge",
            crate::batch::BatchOperationType::Replace => "Replace",
            crate::batch::BatchOperationType::Backup => "Backup",
        };

        let popup_content = vec![
            Line::from(vec![Span::styled(
                "⚡ Execute Batch Operation",
                Style::default().fg(Color::Cyan).bold(),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Profile: ", Style::default().fg(Color::Green).bold()),
                Span::raw(selected_profile),
            ]),
            Line::from(vec![
                Span::styled("Operation: ", Style::default().fg(Color::Yellow).bold()),
                Span::raw(operation_name),
            ]),
            Line::from(""),
            Line::from("This will:"),
            Line::from(match self.batch_operation_type {
                crate::batch::BatchOperationType::Apply => {
                    "• Apply profile settings to your current Hyprland configuration"
                }
                crate::batch::BatchOperationType::Merge => {
                    "• Merge profile settings with your current configuration"
                }
                crate::batch::BatchOperationType::Replace => {
                    "• Replace your current configuration with the profile"
                }
                crate::batch::BatchOperationType::Backup => {
                    "• Create a backup of your current configuration"
                }
            }),
            Line::from("• Update the configuration file and reload Hyprland"),
            Line::from(""),
            Line::from(vec![
                Span::styled("⚠️  Warning: ", Style::default().fg(Color::Red).bold()),
                Span::raw("This will modify your Hyprland configuration!"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press Enter", Style::default().fg(Color::Green).bold()),
                Span::raw(" to execute or "),
                Span::styled("Esc", Style::default().fg(Color::Red).bold()),
                Span::raw(" to cancel"),
            ]),
        ];

        let popup = Paragraph::new(popup_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Confirm Operation ")
                    .title_alignment(Alignment::Center),
            )
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        f.render_widget(popup, area);
    }

    // Visual preview system methods
    pub fn show_setting_preview(&mut self, setting_name: String, before: String, after: String) {
        self.show_preview_dialog = true;
        self.preview_setting_name = setting_name;
        self.preview_before = Some(before);
        self.preview_after = Some(after);
        self.preview_scroll = 0;
    }

    pub fn close_preview_dialog(&mut self) {
        self.show_preview_dialog = false;
        self.preview_before = None;
        self.preview_after = None;
        self.preview_setting_name.clear();
        self.preview_scroll = 0;
    }

    pub fn scroll_preview_up(&mut self) {
        if self.preview_scroll > 0 {
            self.preview_scroll -= 1;
        }
    }

    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll += 1;
    }

    fn render_preview_dialog(&self, f: &mut Frame, area: Rect) {
        // Create popup area (80% of screen)
        let popup_area = Self::centered_rect(80, 70, area);

        // Clear the area
        f.render_widget(Clear, popup_area);

        // Split the popup into two columns for before/after comparison
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(popup_area);

        // Before panel (left side)
        let before_content = if let Some(before) = &self.preview_before {
            before.clone()
        } else {
            "No previous value".to_string()
        };

        let before_paragraph = Paragraph::new(before_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Red))
                    .title(" Before ")
                    .title_style(Style::default().fg(Color::Red).bold()),
            )
            .wrap(Wrap { trim: true })
            .scroll((self.preview_scroll as u16, 0));

        f.render_widget(before_paragraph, columns[0]);

        // After panel (right side)
        let after_content = if let Some(after) = &self.preview_after {
            after.clone()
        } else {
            "New value".to_string()
        };

        let after_paragraph = Paragraph::new(after_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Green))
                    .title(" After ")
                    .title_style(Style::default().fg(Color::Green).bold()),
            )
            .wrap(Wrap { trim: true })
            .scroll((self.preview_scroll as u16, 0));

        f.render_widget(after_paragraph, columns[1]);

        // Add a title bar with the setting name and instructions
        let title_area = Rect {
            x: popup_area.x,
            y: popup_area.y.saturating_sub(2),
            width: popup_area.width,
            height: 2,
        };

        let title_content = vec![
            Line::from(vec![
                Span::styled("Preview: ", Style::default().fg(Color::Cyan).bold()),
                Span::styled(
                    &self.preview_setting_name,
                    Style::default().fg(Color::White).bold(),
                ),
            ]),
            Line::from(vec![
                Span::raw("Use "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow).bold()),
                Span::raw(" to scroll, "),
                Span::styled("Enter", Style::default().fg(Color::Green).bold()),
                Span::raw(" to apply, "),
                Span::styled("Esc", Style::default().fg(Color::Red).bold()),
                Span::raw(" to cancel"),
            ]),
        ];

        let title_paragraph = Paragraph::new(title_content).alignment(Alignment::Center);

        f.render_widget(title_paragraph, title_area);
    }

    fn render_import_dialog(&self, f: &mut Frame, area: Rect) {
        let popup_area = self.center_rect(80, 70, area);
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" Import Configuration ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(self.theme.popup_style());

        f.render_widget(block, popup_area);

        let inner = popup_area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 2,
        });

        match self.import_export_mode {
            ImportExportMode::SelectSource => {
                let content = vec![
                    Line::from(vec![Span::styled(
                        "Select Import Source:",
                        self.theme.header_style().bold(),
                    )]),
                    Line::raw(""),
                    Line::from(vec![
                        Span::styled("1. ", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw("Local File - Import from a single configuration file"),
                    ]),
                    Line::from(vec![
                        Span::styled("2. ", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw("Local Folder - Scan and import from a directory"),
                    ]),
                    Line::from(vec![
                        Span::styled("3. ", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw("GitHub Repository - Import from a Git repository"),
                    ]),
                    Line::from(vec![
                        Span::styled("4. ", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw("URL Download - Import from a direct URL"),
                    ]),
                    Line::raw(""),
                    Line::from(vec![
                        Span::styled("Selected: ", Style::default().fg(self.theme.fg_secondary)),
                        Span::styled(
                            format!("{:?}", self.selected_import_source),
                            Style::default().fg(self.theme.accent_info).bold(),
                        ),
                    ]),
                ];

                let paragraph = Paragraph::new(content)
                    .style(Style::default().fg(self.theme.fg_primary))
                    .wrap(Wrap { trim: true });

                f.render_widget(paragraph, inner);
            }
            ImportExportMode::Preview => {
                if let Some(preview_text) = &self.import_preview {
                    let content: Vec<Line> = preview_text
                        .lines()
                        .skip(self.import_export_scroll)
                        .take(inner.height as usize - 3)
                        .map(|line| Line::raw(line))
                        .collect();

                    let paragraph = Paragraph::new(content)
                        .style(Style::default().fg(self.theme.fg_primary))
                        .wrap(Wrap { trim: true });

                    f.render_widget(paragraph, inner);

                    // Add scroll instructions
                    let instructions_area = Rect {
                        x: inner.x,
                        y: inner.bottom() - 2,
                        width: inner.width,
                        height: 2,
                    };

                    let instructions = Paragraph::new(vec![Line::from(vec![
                        Span::styled("↑↓", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw(" scroll, "),
                        Span::styled(
                            "Enter",
                            Style::default().fg(self.theme.accent_success).bold(),
                        ),
                        Span::raw(" to import, "),
                        Span::styled("Esc", Style::default().fg(self.theme.accent_warning).bold()),
                        Span::raw(" to go back"),
                    ])])
                    .style(Style::default().fg(self.theme.fg_secondary));

                    f.render_widget(instructions, instructions_area);
                }
            }
            ImportExportMode::Execute => {
                let content = vec![
                    Line::from(vec![Span::styled(
                        "Import Complete!",
                        Style::default().fg(self.theme.accent_success).bold(),
                    )]),
                    Line::raw(""),
                    Line::raw("The configuration has been imported successfully."),
                    Line::raw("Press Enter or Esc to close this dialog."),
                ];

                let paragraph = Paragraph::new(content)
                    .style(Style::default().fg(self.theme.fg_primary))
                    .wrap(Wrap { trim: true })
                    .alignment(Alignment::Center);

                f.render_widget(paragraph, inner);
            }
            _ => {}
        }
    }

    fn render_export_dialog(&self, f: &mut Frame, area: Rect) {
        let popup_area = self.center_rect(80, 70, area);
        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" Export Configuration ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(self.theme.popup_style());

        f.render_widget(block, popup_area);

        let inner = popup_area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 2,
        });

        match self.import_export_mode {
            ImportExportMode::SelectFormat => {
                let content = vec![
                    Line::from(vec![Span::styled(
                        "Select Export Format:",
                        Style::default().fg(self.theme.fg_primary).bold(),
                    )]),
                    Line::raw(""),
                    Line::from(vec![
                        Span::styled("1. ", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw("Hyprland Configuration (.conf) - Standard format"),
                    ]),
                    Line::from(vec![
                        Span::styled("2. ", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw("JSON (.json) - Structured data format"),
                    ]),
                    Line::from(vec![
                        Span::styled("3. ", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw("TOML (.toml) - Human-readable configuration"),
                    ]),
                    Line::from(vec![
                        Span::styled("4. ", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw("YAML (.yaml) - Clean, indented format"),
                    ]),
                    Line::from(vec![
                        Span::styled("5. ", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw("R-Hyprconfig (.rhypr) - Native format with full features"),
                    ]),
                    Line::from(vec![
                        Span::styled("6. ", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw("NixOS Module (.nix) - Declarative configuration"),
                    ]),
                    Line::raw(""),
                    Line::from(vec![
                        Span::styled("Selected: ", Style::default().fg(self.theme.fg_secondary)),
                        Span::styled(
                            format!("{:?}", self.selected_export_format),
                            Style::default().fg(self.theme.accent_info).bold(),
                        ),
                    ]),
                ];

                let paragraph = Paragraph::new(content)
                    .style(Style::default().fg(self.theme.fg_primary))
                    .wrap(Wrap { trim: true });

                f.render_widget(paragraph, inner);
            }
            ImportExportMode::Preview => {
                if let Some(preview_text) = &self.export_preview {
                    let content: Vec<Line> = preview_text
                        .lines()
                        .skip(self.import_export_scroll)
                        .take(inner.height as usize - 3)
                        .map(|line| Line::raw(line))
                        .collect();

                    let paragraph = Paragraph::new(content)
                        .style(Style::default().fg(self.theme.fg_primary))
                        .wrap(Wrap { trim: true });

                    f.render_widget(paragraph, inner);

                    // Add scroll instructions
                    let instructions_area = Rect {
                        x: inner.x,
                        y: inner.bottom() - 2,
                        width: inner.width,
                        height: 2,
                    };

                    let instructions = Paragraph::new(vec![Line::from(vec![
                        Span::styled("↑↓", Style::default().fg(self.theme.accent_primary).bold()),
                        Span::raw(" scroll, "),
                        Span::styled(
                            "Enter",
                            Style::default().fg(self.theme.accent_success).bold(),
                        ),
                        Span::raw(" to export, "),
                        Span::styled("Esc", Style::default().fg(self.theme.accent_warning).bold()),
                        Span::raw(" to go back"),
                    ])])
                    .style(Style::default().fg(self.theme.fg_secondary));

                    f.render_widget(instructions, instructions_area);
                }
            }
            ImportExportMode::Execute => {
                let content = vec![
                    Line::from(vec![Span::styled(
                        "Export Complete!",
                        Style::default().fg(self.theme.accent_success).bold(),
                    )]),
                    Line::raw(""),
                    Line::raw("The configuration has been exported successfully."),
                    Line::raw("Press Enter or Esc to close this dialog."),
                ];

                let paragraph = Paragraph::new(content)
                    .style(Style::default().fg(self.theme.fg_primary))
                    .wrap(Wrap { trim: true })
                    .alignment(Alignment::Center);

                f.render_widget(paragraph, inner);
            }
            _ => {}
        }
    }

    // Real-time preview functionality
    pub fn toggle_preview_mode(&mut self) {
        self.preview_mode = !self.preview_mode;
        if !self.preview_mode {
            // Clean up when disabling preview mode
            self.pending_preview_change = None;
            self.preview_original_value = None;
        }
    }

    pub fn is_preview_mode(&self) -> bool {
        self.preview_mode
    }

    pub async fn handle_preview_change(
        &mut self,
        key: &str,
        value: &str,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> anyhow::Result<()> {
        if !self.preview_mode {
            return Ok(());
        }

        // Get the hyprctl key for this configuration option
        let hypr_key = self.get_hyprctl_key(&self.current_tab, key);
        if hypr_key.is_none() {
            // No hyprctl mapping available for this setting
            return Ok(());
        }
        let hypr_key = hypr_key.unwrap();

        let now = std::time::Instant::now();

        // Store the original value if this is the first preview change
        if self.preview_original_value.is_none() {
            // Get current value from hyprctl using the proper hyprctl key
            match hyprctl.get_option(&hypr_key).await {
                Ok(current) => {
                    self.preview_original_value = Some(current);
                }
                Err(_) => {
                    // If we can't get the current value, store the value from UI
                    if let Some(item) = self.get_config_item(key) {
                        self.preview_original_value = Some(item.value.clone());
                    }
                }
            }
        }

        // Set up debounced preview change using the hyprctl key
        self.pending_preview_change = Some((hypr_key, value.to_string()));
        self.last_preview_time = now;

        Ok(())
    }

    pub async fn apply_pending_preview(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> anyhow::Result<()> {
        if !self.preview_mode {
            return Ok(());
        }

        let now = std::time::Instant::now();

        // Check if enough time has passed for debouncing
        if now.duration_since(self.last_preview_time) >= self.preview_debounce_delay {
            if let Some((key, value)) = &self.pending_preview_change {
                // Apply the preview change via hyprctl
                match hyprctl.set_option(key, value).await {
                    Ok(_) => {
                        // Success - clear pending change
                        self.pending_preview_change = None;
                    }
                    Err(e) => {
                        // Failed to apply - show error but don't clear pending change
                        self.show_popup = true;
                        self.popup_message = format!("Preview failed: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn cancel_preview(
        &mut self,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> anyhow::Result<()> {
        if let Some(original_value) = &self.preview_original_value {
            if let Some((key, _)) = &self.pending_preview_change {
                // Restore original value
                if let Err(e) = hyprctl.set_option(key, original_value).await {
                    self.show_popup = true;
                    self.popup_message = format!("Failed to restore original value: {}", e);
                }
            }
        }

        // Clean up preview state
        self.pending_preview_change = None;
        self.preview_original_value = None;

        Ok(())
    }

    pub fn has_pending_preview(&self) -> bool {
        self.pending_preview_change.is_some()
    }

    pub fn get_preview_status(&self) -> String {
        if !self.preview_mode {
            return "Preview: OFF".to_string();
        }

        if self.has_pending_preview() {
            let remaining = self
                .preview_debounce_delay
                .saturating_sub(self.last_preview_time.elapsed());
            if remaining > std::time::Duration::ZERO {
                return format!("Preview: Pending ({:.1}s)", remaining.as_secs_f32());
            } else {
                return "Preview: Applying...".to_string();
            }
        }

        "Preview: ON".to_string()
    }

    fn get_config_item(&self, key: &str) -> Option<&ConfigItem> {
        for items in self.config_items.values() {
            if let Some(item) = items.iter().find(|item| item.key == key) {
                return Some(item);
            }
        }
        None
    }

    fn center_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    pub fn start_add_keybind(&mut self) {
        // Start editing mode to add a new keybind
        let empty_item = ConfigItem {
            key: format!("new_keybind_{}", chrono::Utc::now().timestamp()),
            value: "SUPER, , exec, ".to_string(),
            description: "New keybinding".to_string(),
            data_type: ConfigDataType::String,
            suggestions: vec![
                "SUPER".to_string(),
                "ALT".to_string(),
                "CTRL".to_string(),
                "SHIFT".to_string(),
            ],
        };

        // Add the new item to the Binds panel
        self.config_items
            .entry(crate::app::FocusedPanel::Binds)
            .or_default()
            .push(empty_item);

        // Select the newly added item and start editing
        if let Some(items) = self.config_items.get(&crate::app::FocusedPanel::Binds) {
            self.binds_list_state.select(Some(items.len() - 1));
        }
        // Note: start_editing is async, but we can't await here since this method is not async
        // The edit will be started when the user presses Enter on the selected item
    }

    pub fn start_add_window_rule(&mut self) {
        // Start editing mode to add a new window rule
        let empty_item = ConfigItem {
            key: format!("new_window_rule_{}", chrono::Utc::now().timestamp()),
            value: "float, ^()$".to_string(),
            description: "New window rule".to_string(),
            data_type: ConfigDataType::String,
            suggestions: vec![
                "float".to_string(),
                "size".to_string(),
                "opacity".to_string(),
                "workspace".to_string(),
            ],
        };

        // Add the new item to the WindowRules panel
        self.config_items
            .entry(crate::app::FocusedPanel::WindowRules)
            .or_default()
            .push(empty_item);

        // Select the newly added item and start editing
        if let Some(items) = self
            .config_items
            .get(&crate::app::FocusedPanel::WindowRules)
        {
            self.window_rules_list_state.select(Some(items.len() - 1));
        }
        // Note: start_editing is async, but we can't await here since this method is not async
        // The edit will be started when the user presses Enter on the selected item
    }

    pub fn start_add_layer_rule(&mut self) {
        // Start editing mode to add a new layer rule
        let empty_item = ConfigItem {
            key: format!("new_layer_rule_{}", chrono::Utc::now().timestamp()),
            value: "blur, ".to_string(),
            description: "New layer rule".to_string(),
            data_type: ConfigDataType::String,
            suggestions: vec![
                "blur".to_string(),
                "ignorezero".to_string(),
                "ignorealpha".to_string(),
            ],
        };

        // Add the new item to the LayerRules panel
        self.config_items
            .entry(crate::app::FocusedPanel::LayerRules)
            .or_default()
            .push(empty_item);

        // Select the newly added item and start editing
        if let Some(items) = self.config_items.get(&crate::app::FocusedPanel::LayerRules) {
            self.layer_rules_list_state.select(Some(items.len() - 1));
        }
        // Note: start_editing is async, but we can't await here since this method is not async
        // The edit will be started when the user presses Enter on the selected item
    }

    pub fn get_selected_item(&self) -> Option<&ConfigItem> {
        // Get the currently selected item from the current panel
        let items = self.config_items.get(&self.current_tab)?;
        let selected_index = match self.current_tab {
            crate::app::FocusedPanel::General => self.general_list_state.selected()?,
            crate::app::FocusedPanel::Input => self.input_list_state.selected()?,
            crate::app::FocusedPanel::Decoration => self.decoration_list_state.selected()?,
            crate::app::FocusedPanel::Animations => self.animations_list_state.selected()?,
            crate::app::FocusedPanel::Gestures => self.gestures_list_state.selected()?,
            crate::app::FocusedPanel::Binds => self.binds_list_state.selected()?,
            crate::app::FocusedPanel::WindowRules => self.window_rules_list_state.selected()?,
            crate::app::FocusedPanel::LayerRules => self.layer_rules_list_state.selected()?,
            crate::app::FocusedPanel::Misc => self.misc_list_state.selected()?,
            _ => None?,
        };
        items.get(selected_index)
    }

    pub fn delete_item(&mut self, panel: &FocusedPanel, key: &str) -> bool {
        // Remove the item with the given key from the specified panel
        if let Some(items) = self.config_items.get_mut(panel) {
            if let Some(index) = items.iter().position(|item| item.key == key) {
                items.remove(index);

                // Adjust the selection if needed
                let list_state = match panel {
                    crate::app::FocusedPanel::General => &mut self.general_list_state,
                    crate::app::FocusedPanel::Input => &mut self.input_list_state,
                    crate::app::FocusedPanel::Decoration => &mut self.decoration_list_state,
                    crate::app::FocusedPanel::Animations => &mut self.animations_list_state,
                    crate::app::FocusedPanel::Gestures => &mut self.gestures_list_state,
                    crate::app::FocusedPanel::Binds => &mut self.binds_list_state,
                    crate::app::FocusedPanel::WindowRules => &mut self.window_rules_list_state,
                    crate::app::FocusedPanel::LayerRules => &mut self.layer_rules_list_state,
                    crate::app::FocusedPanel::Misc => &mut self.misc_list_state,
                    _ => return false,
                };

                if let Some(selected) = list_state.selected() {
                    if selected >= items.len() && !items.is_empty() {
                        list_state.select(Some(items.len() - 1));
                    } else if items.is_empty() {
                        list_state.select(None);
                    }
                }
                return true;
            }
        }
        false
    }
}
