use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, 
        Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::app::FocusedPanel;

#[derive(Debug, Clone, PartialEq)]
pub enum EditMode {
    None,
    Text { current_value: String, cursor_pos: usize },
    Slider { current_value: f32, min: f32, max: f32, step: f32 },
    Select { options: Vec<String>, selected: usize },
    Boolean { current_value: bool },
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

#[derive(Debug, Clone)]
pub struct ConfigItem {
    pub key: String,
    pub value: String,
    pub description: String,
    pub data_type: ConfigDataType,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
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
    pub config_items: std::collections::HashMap<FocusedPanel, Vec<ConfigItem>>,
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
            config_items: std::collections::HashMap::new(),
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

        // Initialize config items with enhanced data
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
                data_type: ConfigDataType::Integer { min: Some(0), max: Some(50) },
                suggestions: vec!["0".to_string(), "5".to_string(), "10".to_string(), "15".to_string()],
            },
            ConfigItem {
                key: "gaps_out".to_string(),
                value: "20".to_string(),
                description: "Outer gaps between windows and monitor edges".to_string(),
                data_type: ConfigDataType::Integer { min: Some(0), max: Some(100) },
                suggestions: vec!["10".to_string(), "20".to_string(), "30".to_string()],
            },
            ConfigItem {
                key: "border_size".to_string(),
                value: "2".to_string(),
                description: "Border width in pixels".to_string(),
                data_type: ConfigDataType::Integer { min: Some(0), max: Some(20) },
                suggestions: vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()],
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
                    options: vec!["us".to_string(), "uk".to_string(), "de".to_string(), "fr".to_string(), "es".to_string()] 
                },
                suggestions: vec!["us".to_string(), "uk".to_string(), "de".to_string()],
            },
            ConfigItem {
                key: "follow_mouse".to_string(),
                value: "1".to_string(),
                description: "Follow mouse focus behavior".to_string(),
                data_type: ConfigDataType::Keyword { 
                    options: vec!["0".to_string(), "1".to_string(), "2".to_string(), "3".to_string()] 
                },
                suggestions: vec!["0".to_string(), "1".to_string(), "2".to_string()],
            },
            ConfigItem {
                key: "sensitivity".to_string(),
                value: "0.0".to_string(),
                description: "Mouse sensitivity (-1.0 to 1.0)".to_string(),
                data_type: ConfigDataType::Float { min: Some(-1.0), max: Some(1.0) },
                suggestions: vec!["-0.5".to_string(), "0.0".to_string(), "0.5".to_string()],
            },
        ];

        // Decoration items
        let decoration_items = vec![
            ConfigItem {
                key: "rounding".to_string(),
                value: "10".to_string(),
                description: "Window corner rounding in pixels".to_string(),
                data_type: ConfigDataType::Integer { min: Some(0), max: Some(50) },
                suggestions: vec!["0".to_string(), "5".to_string(), "10".to_string(), "15".to_string()],
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
                data_type: ConfigDataType::Integer { min: Some(1), max: Some(20) },
                suggestions: vec!["1".to_string(), "3".to_string(), "5".to_string(), "8".to_string()],
            },
        ];

        self.config_items.insert(FocusedPanel::General, general_items);
        self.config_items.insert(FocusedPanel::Input, input_items);
        self.config_items.insert(FocusedPanel::Decoration, decoration_items);
        
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
                data_type: ConfigDataType::Integer { min: Some(3), max: Some(5) },
                suggestions: vec!["3".to_string(), "4".to_string(), "5".to_string()],
            },
            ConfigItem {
                key: "gestures.workspace_swipe_distance".to_string(),
                value: "300".to_string(),
                description: "Distance in pixels to trigger swipe".to_string(),
                data_type: ConfigDataType::Integer { min: Some(100), max: Some(1000) },
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
                    options: vec!["SUPER".to_string(), "ALT".to_string(), "CTRL".to_string(), "SHIFT".to_string()] 
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
                    options: vec!["0".to_string(), "1".to_string(), "2".to_string()] 
                },
                suggestions: vec!["0".to_string(), "1".to_string(), "2".to_string()],
            },
        ];

        // Insert all configuration items
        self.config_items.insert(FocusedPanel::Animations, animations_items);
        self.config_items.insert(FocusedPanel::Gestures, gestures_items);
        self.config_items.insert(FocusedPanel::Binds, binds_items);
        self.config_items.insert(FocusedPanel::WindowRules, window_rules_items);
        self.config_items.insert(FocusedPanel::LayerRules, layer_rules_items);
        self.config_items.insert(FocusedPanel::Misc, misc_items);
    }

    pub async fn load_current_config(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
        // Load ALL configuration options from hyprctl  
        match hyprctl.get_all_options().await {
            Ok(all_options) => {
                self.populate_config_from_options(all_options);
            }
            Err(e) => {
                eprintln!("Warning: Failed to load all options from hyprctl: {}", e);
                // Fall back to loading individual sections
                self.load_general_config(hyprctl).await?;
                self.load_input_config(hyprctl).await?;
                self.load_decoration_config(hyprctl).await?;
                self.load_animations_config(hyprctl).await?;
                self.load_gestures_config(hyprctl).await?;
                self.load_misc_config(hyprctl).await?;
            }
        }

        // These need special hyprctl commands - always load them
        self.load_binds_config(hyprctl).await?;
        self.load_window_rules_config(hyprctl).await?;
        self.load_layer_rules_config(hyprctl).await?;
        
        Ok(())
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
            self.config_items.insert(FocusedPanel::General, general_items);
        }
        if !input_items.is_empty() {
            self.config_items.insert(FocusedPanel::Input, input_items);
        }
        if !decoration_items.is_empty() {
            self.config_items.insert(FocusedPanel::Decoration, decoration_items);
        }
        if !animation_items.is_empty() {
            self.config_items.insert(FocusedPanel::Animations, animation_items);
        }
        if !gesture_items.is_empty() {
            self.config_items.insert(FocusedPanel::Gestures, gesture_items);
        }
        if !misc_items.is_empty() {
            self.config_items.insert(FocusedPanel::Misc, misc_items);
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
            
            _ => format!("Configuration option: {}", key),
        }
    }

    fn infer_data_type(&self, key: &str, value: &str) -> ConfigDataType {
        // Infer data type based on key patterns and value content
        match key {
            // Color options
            k if k.contains("col.") || k.contains("color") => ConfigDataType::Color,
            
            // Boolean options
            k if k.contains("enable") || k.contains("disable") || 
                 k.ends_with("_on_border") || k.contains("natural_scroll") ||
                 k.contains("mouse_refocus") || k.contains("resize_on_border") => {
                ConfigDataType::Boolean
            }
            
            // Float options  
            "input:sensitivity" | "decoration:dim_strength" | 
            "gestures:workspace_swipe_cancel_ratio" => {
                ConfigDataType::Float { min: Some(0.0), max: Some(10.0) }
            }
            
            // Integer options with ranges
            "general:gaps_in" | "general:gaps_out" => {
                ConfigDataType::Integer { min: Some(0), max: Some(100) }
            }
            "general:border_size" => {
                ConfigDataType::Integer { min: Some(0), max: Some(20) }
            }
            "decoration:rounding" => {
                ConfigDataType::Integer { min: Some(0), max: Some(50) }
            }
            
            k if k.contains("size") || k.contains("range") || k.contains("passes") ||
                 k.contains("fingers") || k.contains("distance") => {
                ConfigDataType::Integer { min: Some(0), max: Some(100) }
            }
            
            // Try to infer from value
            _ => {
                let trimmed = value.trim();
                if trimmed == "true" || trimmed == "false" || trimmed == "1" || trimmed == "0" {
                    ConfigDataType::Boolean
                } else if trimmed.parse::<i32>().is_ok() {
                    ConfigDataType::Integer { min: None, max: None }
                } else if trimmed.parse::<f32>().is_ok() {
                    ConfigDataType::Float { min: None, max: None }
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
            "input:kb_layout" => vec!["us".to_string(), "de".to_string(), "fr".to_string(), "uk".to_string()],
            "general:col.active_border" => vec![
                "0xffff0000".to_string(), 
                "0xff00ff00".to_string(), 
                "0xff0000ff".to_string(),
                "0xffffffff".to_string()
            ],
            "general:col.inactive_border" => vec![
                "0x66333333".to_string(), 
                "0x66666666".to_string(), 
                "0x66999999".to_string()
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

    async fn load_general_config(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
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
                        eprintln!("Warning: Failed to get {}: {}", hypr_key, e);
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_input_config(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
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
                        eprintln!("Warning: Failed to get {}: {}", hypr_key, e);
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_decoration_config(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
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
                        eprintln!("Warning: Failed to get {}: {}", hypr_key, e);
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_animations_config(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
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
                        eprintln!("Warning: Failed to get {}: {}", hypr_key, e);
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_gestures_config(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
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
                        eprintln!("Warning: Failed to get {}: {}", hypr_key, e);
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_misc_config(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
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
                        eprintln!("Warning: Failed to get {}: {}", hypr_key, e);
                    }
                }
            }
        }
        Ok(())
    }

    async fn load_binds_config(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
        match hyprctl.get_binds().await {
            Ok(keybinds) => {
                let mut bind_items = Vec::new();
                
                for (i, keybind) in keybinds.iter().enumerate() {
                    // Create a unique key for each keybind
                    let key = format!("bind_{}", i);
                    
                    bind_items.push(ConfigItem {
                        key: key.clone(),
                        value: keybind.display_string(),
                        description: format!("Keybind: {} {}", 
                            keybind.dispatcher,
                            keybind.args.as_deref().unwrap_or("")
                        ),
                        data_type: ConfigDataType::String,
                        suggestions: self.get_keybind_suggestions(&keybind.dispatcher),
                    });
                }
                
                // If we got keybinds, replace the default ones
                if !bind_items.is_empty() {
                    self.config_items.insert(FocusedPanel::Binds, bind_items);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to load keybinds: {}", e);
                // Create informative placeholder when Hyprland is not available
                let placeholder_binds = vec![
                    ConfigItem {
                        key: "bind_example_1".to_string(),
                        value: "SUPER + Q → exec [kitty]".to_string(),
                        description: "Example: Open terminal with Super+Q".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: vec!["exec".to_string(), "killactive".to_string(), "togglefloating".to_string()],
                    },
                    ConfigItem {
                        key: "bind_example_2".to_string(),
                        value: "SUPER + C → killactive".to_string(),
                        description: "Example: Close window with Super+C".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: vec!["killactive".to_string(), "exec".to_string(), "workspace".to_string()],
                    },
                    ConfigItem {
                        key: "hyprland_not_running".to_string(),
                        value: "⚠️  Hyprland not detected".to_string(),
                        description: "Start Hyprland to see actual keybinds".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: vec![],
                    },
                ];
                self.config_items.insert(FocusedPanel::Binds, placeholder_binds);
            }
        }
        
        Ok(())
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
                "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(),
                "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(),
                "9".to_string(), "10".to_string(),
            ],
            "movetoworkspace" => vec![
                "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(),
                "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(),
                "9".to_string(), "10".to_string(),
            ],
            "movefocus" => vec![
                "l".to_string(), "r".to_string(), "u".to_string(), "d".to_string(),
                "left".to_string(), "right".to_string(), "up".to_string(), "down".to_string(),
            ],
            "movewindow" => vec![
                "l".to_string(), "r".to_string(), "u".to_string(), "d".to_string(),
                "left".to_string(), "right".to_string(), "up".to_string(), "down".to_string(),
            ],
            "resizeactive" => vec![
                "10 0".to_string(), "-10 0".to_string(), "0 10".to_string(), "0 -10".to_string(),
                "50 0".to_string(), "-50 0".to_string(), "0 50".to_string(), "0 -50".to_string(),
            ],
            _ => vec![],
        }
    }

    async fn load_window_rules_config(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
        match hyprctl.get_window_rules().await {
            Ok(window_rules) => {
                let mut rule_items = Vec::new();
                
                for (i, rule) in window_rules.iter().enumerate() {
                    let key = format!("window_rule_{}", i);
                    
                    // Parse rule to extract description
                    let description = if rule.contains("windowrule") {
                        let parts: Vec<&str> = rule.splitn(3, " = ").collect();
                        if parts.len() >= 2 {
                            format!("Window rule: {}", parts[1])
                        } else {
                            "Window rule configuration".to_string()
                        }
                    } else {
                        format!("Window pattern: {}", rule)
                    };
                    
                    rule_items.push(ConfigItem {
                        key: key.clone(),
                        value: rule.clone(),
                        description,
                        data_type: ConfigDataType::String,
                        suggestions: self.get_window_rule_suggestions(),
                    });
                }
                
                // Replace the default window rules with actual ones
                if !rule_items.is_empty() {
                    self.config_items.insert(FocusedPanel::WindowRules, rule_items);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to load window rules: {}", e);
                // Create informative placeholders when Hyprland is not available
                let placeholder_rules = vec![
                    ConfigItem {
                        key: "window_rule_example_1".to_string(),
                        value: "windowrule = float, ^(kitty)$".to_string(),
                        description: "Example: Float kitty terminal windows".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: self.get_window_rule_suggestions(),
                    },
                    ConfigItem {
                        key: "window_rule_example_2".to_string(),
                        value: "windowrule = workspace 2, ^(firefox)$".to_string(),
                        description: "Example: Send Firefox to workspace 2".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: self.get_window_rule_suggestions(),
                    },
                    ConfigItem {
                        key: "hyprland_not_running".to_string(),
                        value: "⚠️  Hyprland not detected".to_string(),
                        description: "Start Hyprland to see actual window rules".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: vec![],
                    },
                ];
                self.config_items.insert(FocusedPanel::WindowRules, placeholder_rules);
            }
        }
        Ok(())
    }

    async fn load_layer_rules_config(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
        match hyprctl.get_layer_rules().await {
            Ok(layer_rules) => {
                let mut rule_items = Vec::new();
                
                for (i, rule) in layer_rules.iter().enumerate() {
                    let key = format!("layer_rule_{}", i);
                    
                    // Parse rule to extract description
                    let description = if rule.contains("layerrule") {
                        let parts: Vec<&str> = rule.splitn(3, " = ").collect();
                        if parts.len() >= 2 {
                            format!("Layer rule: {}", parts[1])
                        } else {
                            "Layer rule configuration".to_string()
                        }
                    } else {
                        format!("Layer configuration: {}", rule)
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
                            let key = format!("workspace_rule_{}", i);
                            
                            rule_items.push(ConfigItem {
                                key: key.clone(),
                                value: rule.clone(),
                                description: format!("Workspace rule: {}", rule),
                                data_type: ConfigDataType::String,
                                suggestions: self.get_workspace_rule_suggestions(),
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load workspace rules: {}", e);
                    }
                }

                // Replace the default layer rules with actual ones (including workspace rules)
                if !rule_items.is_empty() {
                    self.config_items.insert(FocusedPanel::LayerRules, rule_items);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to load layer rules: {}", e);
                // Create informative placeholders when Hyprland is not available
                let placeholder_layer_rules = vec![
                    ConfigItem {
                        key: "layer_rule_example_1".to_string(),
                        value: "layerrule = blur, waybar".to_string(),
                        description: "Example: Apply blur effect to waybar".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: self.get_layer_rule_suggestions(),
                    },
                    ConfigItem {
                        key: "layer_rule_example_2".to_string(),
                        value: "layerrule = ignorezero, notifications".to_string(),
                        description: "Example: Ignore zero alpha pixels in notifications".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: self.get_layer_rule_suggestions(),
                    },
                    ConfigItem {
                        key: "workspace_rule_example_1".to_string(),
                        value: "workspace = 1, monitor:DP-1".to_string(),
                        description: "Example: Assign workspace 1 to DP-1 monitor".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: self.get_workspace_rule_suggestions(),
                    },
                    ConfigItem {
                        key: "hyprland_not_running".to_string(),
                        value: "⚠️  Hyprland not detected".to_string(),
                        description: "Start Hyprland to see actual layer and workspace rules".to_string(),
                        data_type: ConfigDataType::String,
                        suggestions: vec![],
                    },
                ];
                self.config_items.insert(FocusedPanel::LayerRules, placeholder_layer_rules);
            }
        }
        Ok(())
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
                Constraint::Length(4),  // Header
                Constraint::Length(3),  // Tab bar
                Constraint::Min(0),     // Main content
                Constraint::Length(3),  // Footer
            ])
            .margin(1)
            .split(size);

        // Render enhanced header
        self.render_enhanced_header(f, main_chunks[0], debug);

        // Render tab bar
        self.render_tab_bar(f, main_chunks[1]);

        // Render current tab content
        self.render_current_tab(f, main_chunks[2]);

        // Render enhanced footer
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

        if self.edit_mode != EditMode::None {
            self.render_edit_popup(f, size);
        }
    }

    fn render_enhanced_header(&self, f: &mut Frame, area: Rect, debug: bool) {
        let _title_text = if debug {
            "🦀 R-Hyprconfig - Debug Mode 🔧"
        } else {
            "🎨 R-Hyprconfig - Hyprland Configuration Manager ⚡"
        };

        // Create gradient-like effect with different colors
        let title_spans = vec![
            Span::styled("🎨 R-Hyprconfig", Style::default().fg(Color::Rgb(0, 255, 255)).bold()),
            Span::raw(" - "),
            Span::styled("Hyprland Configuration Manager", Style::default().fg(Color::Rgb(255, 165, 0)).bold()),
            Span::raw(" ⚡"),
        ];

        let header_content = vec![
            Line::from(title_spans),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Yellow).bold()),
                Span::styled(" to edit • ", Style::default().fg(Color::Gray)),
                Span::styled("Tab", Style::default().fg(Color::Yellow).bold()),
                Span::styled(" to switch tabs • ", Style::default().fg(Color::Gray)),
                Span::styled("↑↓", Style::default().fg(Color::Cyan).bold()),
                Span::styled(" to navigate • ", Style::default().fg(Color::Gray)),
                Span::styled("S", Style::default().fg(Color::Green).bold()),
                Span::styled(" to save • ", Style::default().fg(Color::Gray)),
                Span::styled("R", Style::default().fg(Color::Blue).bold()),
                Span::styled(" to reload", Style::default().fg(Color::Gray)),
            ]),
        ];

        let header = Paragraph::new(header_content)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(100, 200, 255)))
                    .border_type(BorderType::Double)
                    .title(" Hyprland TUI ")
                    .title_style(Style::default().fg(Color::Cyan).bold())
            );

        f.render_widget(header, area);
    }

    fn render_tab_bar(&self, f: &mut Frame, area: Rect) {
        let tabs = vec![
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

        let tab_spans: Vec<Span> = tabs.iter().enumerate().map(|(i, &panel)| {
            let tab_name = match panel {
                FocusedPanel::General => "🏠 General",
                FocusedPanel::Input => "⌨️ Input",
                FocusedPanel::Decoration => "✨ Decoration",
                FocusedPanel::Animations => "🎬 Animations",
                FocusedPanel::Gestures => "👆 Gestures",
                FocusedPanel::Binds => "🔗 Binds",
                FocusedPanel::WindowRules => "📏 Win Rules",
                FocusedPanel::LayerRules => "📐 Layer Rules",
                FocusedPanel::Misc => "⚙️ Misc",
            };

            let style = if panel == self.current_tab {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray).bold()
            } else {
                Style::default().fg(Color::Gray)
            };

            let mut result = vec![Span::styled(tab_name, style)];
            if i < tabs.len() - 1 {
                result.push(Span::raw(" │ "));
            }
            result
        }).flatten().collect();

        let tabs_paragraph = Paragraph::new(Line::from(tab_spans))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
            );

        f.render_widget(tabs_paragraph, area);
    }

    fn render_current_tab(&mut self, f: &mut Frame, area: Rect) {
        // Get config items for current tab
        let config_items = self.config_items.get(&self.current_tab).cloned().unwrap_or_default();
        
        // Create list items with enhanced formatting
        let items: Vec<ListItem> = config_items.iter().map(|item| {
            let value_style = match &item.data_type {
                ConfigDataType::Integer { .. } => Style::default().fg(Color::Rgb(100, 255, 100)), // Light green
                ConfigDataType::Float { .. } => Style::default().fg(Color::Rgb(100, 255, 255)), // Light cyan  
                ConfigDataType::Boolean => Style::default().fg(Color::Rgb(255, 255, 100)), // Light yellow
                ConfigDataType::Color => Style::default().fg(Color::Rgb(255, 150, 255)), // Light magenta
                ConfigDataType::String => Style::default().fg(Color::White),
                ConfigDataType::Keyword { .. } => Style::default().fg(Color::Rgb(255, 200, 100)), // Light orange
            };

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

            let line = Line::from(vec![
                Span::styled(format!("{:<28}", key_display), Style::default().fg(Color::Rgb(200, 200, 255)).bold()),
                Span::raw("│ "),
                Span::styled(value_display, value_style.bold()),
            ]);

            ListItem::new(vec![
                line,
                Line::from(vec![
                    Span::styled(format!("  {}", item.description), Style::default().fg(Color::DarkGray).italic()),
                ]),
            ])
        }).collect();

        // Panel title
        let title = match self.current_tab {
            FocusedPanel::General => "🏠 General Configuration",
            FocusedPanel::Input => "⌨️ Input Configuration",
            FocusedPanel::Decoration => "✨ Decoration Configuration",
            FocusedPanel::Animations => "🎬 Animation Configuration",
            FocusedPanel::Gestures => "👆 Gesture Configuration",
            FocusedPanel::Binds => "🔗 Key Bindings Configuration",
            FocusedPanel::WindowRules => "📏 Window Rules Configuration",
            FocusedPanel::LayerRules => "📐 Layer Rules Configuration",
            FocusedPanel::Misc => "⚙️ Miscellaneous Configuration",
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .title_style(Style::default().fg(Color::Cyan).bold())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .border_type(BorderType::Rounded)
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(60, 60, 120))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let current_list_state = self.get_current_list_state();
        f.render_stateful_widget(list, area, current_list_state);
    }

    fn render_enhanced_panel(&mut self, f: &mut Frame, area: Rect, panel: FocusedPanel, focused_panel: FocusedPanel) {
        let is_focused = focused_panel == panel;
        
        // Enhanced visual styling
        let (border_style, border_type, panel_bg) = if is_focused {
            (
                Style::default().fg(Color::Rgb(255, 215, 0)), // Gold for focused
                BorderType::Double,
                Some(Color::Rgb(10, 10, 30)) // Dark blue background
            )
        } else {
            (
                Style::default().fg(Color::Rgb(100, 150, 200)), // Light blue for unfocused
                BorderType::Rounded,
                None
            )
        };

        // Get enhanced config items
        let config_items = self.config_items.get(&panel).cloned().unwrap_or_default();
        
        // Create enhanced list items with better formatting
        let items: Vec<ListItem> = config_items.iter().map(|item| {
            let value_style = match &item.data_type {
                ConfigDataType::Integer { .. } => Style::default().fg(Color::Rgb(100, 255, 100)), // Light green
                ConfigDataType::Float { .. } => Style::default().fg(Color::Rgb(100, 255, 255)), // Light cyan
                ConfigDataType::Boolean => Style::default().fg(Color::Rgb(255, 255, 100)), // Light yellow
                ConfigDataType::Color => Style::default().fg(Color::Rgb(255, 150, 255)), // Light magenta
                ConfigDataType::String => Style::default().fg(Color::White),
                ConfigDataType::Keyword { .. } => Style::default().fg(Color::Rgb(255, 200, 100)), // Light orange
            };

            let line = Line::from(vec![
                Span::styled(&item.key, Style::default().fg(Color::Rgb(200, 200, 255)).bold()),
                Span::raw(": "),
                Span::styled(&item.value, value_style.bold()),
            ]);

            ListItem::new(line)
        }).collect();

        // Panel title with emoji
        let title = match panel {
            FocusedPanel::General => "🏠 General",
            FocusedPanel::Input => "⌨️  Input", 
            FocusedPanel::Decoration => "✨ Decoration",
            FocusedPanel::Animations => "🎬 Animations",
            FocusedPanel::Gestures => "👆 Gestures",
            FocusedPanel::Binds => "🔗 Key Binds",
            FocusedPanel::WindowRules => "📏 Window Rules",
            FocusedPanel::LayerRules => "📐 Layer Rules",
            FocusedPanel::Misc => "⚙️  Misc",
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
            Span::styled("Tab/→", Style::default().fg(Color::Rgb(255, 215, 0)).bold()),
            Span::styled(" Next ", Style::default().fg(Color::Gray)),
            Span::raw("• "),
            Span::styled("Shift+Tab/←", Style::default().fg(Color::Rgb(255, 215, 0)).bold()),
            Span::styled(" Previous ", Style::default().fg(Color::Gray)),
            Span::raw("• "),
            Span::styled("↑↓", Style::default().fg(Color::Rgb(100, 255, 100)).bold()),
            Span::styled(" Navigate ", Style::default().fg(Color::Gray)),
            Span::raw("• "),
            Span::styled("Enter", Style::default().fg(Color::Rgb(255, 100, 255)).bold()),
            Span::styled(" Edit ", Style::default().fg(Color::Gray)),
            Span::raw("• "),
            Span::styled("S", Style::default().fg(Color::Rgb(100, 255, 255)).bold()),
            Span::styled(" Save ", Style::default().fg(Color::Gray)),
            Span::raw("• "),
            Span::styled("R", Style::default().fg(Color::Rgb(255, 165, 0)).bold()),
            Span::styled(" Reload ", Style::default().fg(Color::Gray)),
            Span::raw("• "),
            Span::styled("Q/Esc", Style::default().fg(Color::Rgb(255, 100, 100)).bold()),
            Span::styled(" Quit", Style::default().fg(Color::Gray)),
        ];

        let footer = Paragraph::new(Line::from(help_text))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(150, 150, 200)))
                    .border_type(BorderType::Rounded)
                    .title(" Controls ")
                    .title_style(Style::default().fg(Color::Cyan).bold()),
            );

        f.render_widget(footer, area);
    }

    fn render_popup(&self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(50, 25, area);
        
        let popup_content = vec![
            Line::from(vec![
                Span::styled("ℹ️ Information", Style::default().fg(Color::Cyan).bold()),
            ]),
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
                    .title_style(Style::default().fg(Color::Cyan).bold())
            )
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);
    }

    fn render_save_dialog(&self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(60, 30, area);
        
        let popup_content = vec![
            Line::from(vec![
                Span::styled("💾 Save Configuration", Style::default().fg(Color::Green).bold()),
            ]),
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
                    .title_style(Style::default().fg(Color::Green).bold())
            )
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);
    }

    fn render_reload_dialog(&self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(60, 30, area);
        
        let popup_content = vec![
            Line::from(vec![
                Span::styled("🔄 Reload Configuration", Style::default().fg(Color::Blue).bold()),
            ]),
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
                    .title_style(Style::default().fg(Color::Blue).bold())
            )
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);
    }

    fn render_edit_popup(&self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(70, 40, area);
        
        // Get the item being edited
        let (panel, key) = if let Some((panel, key)) = &self.editing_item {
            (panel.clone(), key.clone())
        } else {
            return;
        };

        let config_items = self.config_items.get(&panel).cloned().unwrap_or_default();
        let item = config_items.iter().find(|item| item.key == key);
        
        if let Some(item) = item {
            let mut popup_content = vec![
                Line::from(vec![
                    Span::styled("✏️ Edit Configuration", Style::default().fg(Color::Magenta).bold()),
                ]),
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
                EditMode::Text { current_value, cursor_pos } => {
                    popup_content.push(Line::from(vec![
                        Span::styled("Value: ", Style::default().fg(Color::Green).bold()),
                        Span::raw(current_value),
                        if *cursor_pos < current_value.len() {
                            Span::styled("|", Style::default().fg(Color::White).bold())
                        } else {
                            Span::styled("|", Style::default().fg(Color::White).bold())
                        },
                    ]));
                }
                EditMode::Boolean { current_value } => {
                    popup_content.push(Line::from(vec![
                        Span::styled("Value: ", Style::default().fg(Color::Green).bold()),
                        Span::styled(
                            if *current_value { "true" } else { "false" },
                            Style::default().fg(if *current_value { Color::Green } else { Color::Red }).bold()
                        ),
                    ]));
                    popup_content.push(Line::from(""));
                    popup_content.push(Line::from("Press Space to toggle"));
                }
                EditMode::Select { options, selected } => {
                    popup_content.push(Line::from(vec![
                        Span::styled("Options:", Style::default().fg(Color::Green).bold()),
                    ]));
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
                EditMode::Slider { current_value, min, max, .. } => {
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
                    let min_str = format!("Min: {}", min);
                    let max_str = format!("Max: {}", max);
                    
                    popup_content.push(Line::from(vec![
                        Span::styled("Value: ", Style::default().fg(Color::Green).bold()),
                        Span::styled(current_value_str, Style::default().fg(Color::Cyan).bold()),
                    ]));
                    popup_content.push(Line::from(""));
                    popup_content.push(Line::from(vec![
                        Span::styled(bar, Style::default().fg(Color::Blue)),
                    ]));
                    popup_content.push(Line::from(vec![
                        Span::styled(min_str, Style::default().fg(Color::Gray)),
                        Span::raw("  "),
                        Span::styled(max_str, Style::default().fg(Color::Gray)),
                    ]));
                }
                EditMode::Keybind { modifiers, key, dispatcher, args, editing_field } => {
                    popup_content.push(Line::from(vec![
                        Span::styled("Keybind Editor", Style::default().fg(Color::Magenta).bold()),
                    ]));
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
                EditMode::Rule { rule_type, pattern, action, editing_field } => {
                    popup_content.push(Line::from(vec![
                        Span::styled("Rule Editor", Style::default().fg(Color::Magenta).bold()),
                    ]));
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
                popup_content.push(Line::from(vec![
                    Span::styled("Suggestions: ", Style::default().fg(Color::Yellow).bold()),
                ]));
                let suggestions_text = item.suggestions.join(", ");
                popup_content.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(suggestions_text, Style::default().fg(Color::Rgb(200, 200, 200))),
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
                        .title_style(Style::default().fg(Color::Magenta).bold())
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
                ListItem::new("windowrulev2 = float, class:^(firefox)$, title:^(Picture-in-Picture)$"),
                ListItem::new("windowrulev2 = pin, class:^(firefox)$, title:^(Picture-in-Picture)$"),
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
        }
    }

    fn get_panel_items_count(&self, panel: FocusedPanel) -> usize {
        self.config_items.get(&panel).map(|items| items.len()).unwrap_or(0)
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
        }
    }

    pub fn scroll_up(&mut self) {
        let items_count = self.get_panel_items_count(self.current_tab);
        
        if items_count == 0 {
            return;
        }

        let list_state = self.get_current_list_state();
        let selected = list_state.selected().unwrap_or(0);
        if selected > 0 {
            list_state.select(Some(selected - 1));
        } else {
            list_state.select(Some(items_count - 1));
        }
    }

    pub fn scroll_down(&mut self) {
        let items_count = self.get_panel_items_count(self.current_tab);
        
        if items_count == 0 {
            return;
        }

        let list_state = self.get_current_list_state();
        let selected = list_state.selected().unwrap_or(0);
        if selected < items_count - 1 {
            list_state.select(Some(selected + 1));
        } else {
            list_state.select(Some(0));
        }
    }

    pub async fn start_editing(&mut self) -> Result<(), anyhow::Error> {
        // Get the currently selected item from current tab
        let config_items = self.config_items.get(&self.current_tab).cloned().unwrap_or_default();
        if config_items.is_empty() {
            return Ok(());
        }

        let list_state = self.get_current_list_state();
        let selected_index = list_state.selected().unwrap_or(0);
        
        if let Some(item) = config_items.get(selected_index) {
            self.editing_item = Some((self.current_tab, item.key.clone()));
            
            // Set edit mode based on data type and panel
            self.edit_mode = if self.current_tab == FocusedPanel::Binds && item.key.starts_with("bind_") {
                // Special handling for keybinds
                self.parse_keybind_for_editing(&item.value)
            } else if (self.current_tab == FocusedPanel::WindowRules && item.key.starts_with("window_rule_")) ||
                      (self.current_tab == FocusedPanel::LayerRules && (item.key.starts_with("layer_rule_") || item.key.starts_with("workspace_rule_"))) {
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
                            let current_value = item.value.parse::<f32>().unwrap_or(*min_val as f32);
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
                        let selected = options.iter().position(|opt| opt == &item.value).unwrap_or(0);
                        EditMode::Select {
                            options: options.clone(),
                            selected,
                        }
                    }
                    _ => {
                        EditMode::Text {
                            current_value: item.value.clone(),
                            cursor_pos: item.value.len(),
                        }
                    }
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
                (mods.split(" + ").map(|s| s.to_string()).collect(), k.to_string())
            } else {
                (vec![], key_part.to_string())
            };
            
            // Parse dispatcher and args
            let (dispatcher, args) = if let Some((disp, arg_part)) = command_part.split_once(' ') {
                let args = if arg_part.starts_with('[') && arg_part.ends_with(']') {
                    arg_part.trim_start_matches('[').trim_end_matches(']').to_string()
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
                        format!("{:.2}", current_value)
                    }
                }
                EditMode::Keybind { modifiers, key, dispatcher, args, .. } => {
                    // Create display string for keybind
                    let mod_string = if modifiers.is_empty() {
                        String::new()
                    } else {
                        format!("{} + ", modifiers.join(" + "))
                    };
                    
                    let args_string = if args.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", args)
                    };
                    
                    format!("{}{} → {}{}", mod_string, key, dispatcher, args_string)
                }
                EditMode::Rule { rule_type, pattern, action, .. } => {
                    // Format rule for display and application
                    match rule_type {
                        RuleType::Window => format!("windowrule = {}, {}", action, pattern),
                        RuleType::Layer => format!("layerrule = {}, {}", action, pattern),
                        RuleType::Workspace => format!("workspace = {}, {}", action, pattern),
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

    pub async fn apply_edit_with_hyprctl(&mut self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<(), anyhow::Error> {
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
                        format!("{:.2}", current_value)
                    }
                }
                EditMode::Keybind { modifiers, key, dispatcher, args, .. } => {
                    // Create display string for keybind
                    let mod_string = if modifiers.is_empty() {
                        String::new()
                    } else {
                        format!("{} + ", modifiers.join(" + "))
                    };
                    
                    let args_string = if args.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", args)
                    };
                    
                    format!("{}{} → {}{}", mod_string, key, dispatcher, args_string)
                }
                EditMode::Rule { rule_type, pattern, action, .. } => {
                    // Format rule for display and application
                    match rule_type {
                        RuleType::Window => format!("windowrule = {}, {}", action, pattern),
                        RuleType::Layer => format!("layerrule = {}, {}", action, pattern),
                        RuleType::Workspace => format!("workspace = {}, {}", action, pattern),
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
                        self.popup_message = format!("Failed to apply setting: {}", e);
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

    fn get_hyprctl_key(&self, panel: &FocusedPanel, key: &str) -> Option<String> {
        match panel {
            FocusedPanel::General => {
                match key {
                    "gaps_in" => Some("general:gaps_in".to_string()),
                    "gaps_out" => Some("general:gaps_out".to_string()),
                    "border_size" => Some("general:border_size".to_string()),
                    "col.active_border" => Some("general:col.active_border".to_string()),
                    "col.inactive_border" => Some("general:col.inactive_border".to_string()),
                    _ => None,
                }
            }
            FocusedPanel::Input => {
                match key {
                    "kb_layout" => Some("input:kb_layout".to_string()),
                    "follow_mouse" => Some("input:follow_mouse".to_string()),
                    "sensitivity" => Some("input:sensitivity".to_string()),
                    _ => None,
                }
            }
            FocusedPanel::Decoration => {
                match key {
                    "rounding" => Some("decoration:rounding".to_string()),
                    "blur.enabled" => Some("decoration:blur:enabled".to_string()),
                    "blur.size" => Some("decoration:blur:size".to_string()),
                    _ => None,
                }
            }
            FocusedPanel::Animations => {
                match key {
                    "animations.enabled" => Some("animations:enabled".to_string()),
                    _ => None,
                }
            }
            FocusedPanel::Gestures => {
                match key {
                    "gestures.workspace_swipe" => Some("gestures:workspace_swipe".to_string()),
                    "gestures.workspace_swipe_fingers" => Some("gestures:workspace_swipe_fingers".to_string()),
                    "gestures.workspace_swipe_distance" => Some("gestures:workspace_swipe_distance".to_string()),
                    "gestures.workspace_swipe_invert" => Some("gestures:workspace_swipe_invert".to_string()),
                    _ => None,
                }
            }
            FocusedPanel::Misc => {
                match key {
                    "misc.disable_hyprland_logo" => Some("misc:disable_hyprland_logo".to_string()),
                    "misc.disable_splash_rendering" => Some("misc:disable_splash_rendering".to_string()),
                    "misc.mouse_move_enables_dpms" => Some("misc:mouse_move_enables_dpms".to_string()),
                    "misc.vfr" => Some("misc:vfr".to_string()),
                    "misc.vrr" => Some("misc:vrr".to_string()),
                    _ => None,
                }
            }
            // Binds, WindowRules, and LayerRules need different hyprctl commands
            _ => None,
        }
    }

    pub fn cancel_edit(&mut self) {
        self.edit_mode = EditMode::None;
        self.editing_item = None;
    }
}