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

    fn initialize_config_items(&mut self) {
        
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

    pub fn render(&mut self, f: &mut Frame, app_state: (FocusedPanel, bool)) {
        let size = f.area();
        let (focused_panel, debug) = app_state;

        // Create main layout with better proportions
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Header - taller for better presence  
                Constraint::Min(0),    // Main content
                Constraint::Length(4), // Footer - taller for more info
            ])
            .margin(1) // Add margin around entire UI
            .split(size);

        // Render enhanced header
        self.render_enhanced_header(f, main_chunks[0], debug);

        // Create main content layout - use 2x2 grid for better use of space
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Left half
                Constraint::Percentage(50), // Right half
            ])
            .margin(1)
            .split(main_chunks[1]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25), // Larger panels - 4 per side
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .margin(1)
            .split(content_chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .margin(1)
            .split(content_chunks[1]);

        // Render enhanced configuration panels with better arrangement
        self.render_enhanced_panel(f, left_chunks[0], FocusedPanel::General, focused_panel);
        self.render_enhanced_panel(f, left_chunks[1], FocusedPanel::Input, focused_panel);
        self.render_enhanced_panel(f, left_chunks[2], FocusedPanel::Decoration, focused_panel);
        self.render_enhanced_panel(f, left_chunks[3], FocusedPanel::Animations, focused_panel);

        self.render_enhanced_panel(f, right_chunks[0], FocusedPanel::Gestures, focused_panel);
        self.render_enhanced_panel(f, right_chunks[1], FocusedPanel::Binds, focused_panel);
        self.render_enhanced_panel(f, right_chunks[2], FocusedPanel::WindowRules, focused_panel);
        self.render_enhanced_panel(f, right_chunks[3], FocusedPanel::Misc, focused_panel);

        // Render enhanced footer
        self.render_enhanced_footer(f, main_chunks[2]);

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

    pub fn scroll_up(&mut self, panel: FocusedPanel) {
        let items_count = self.get_panel_items_count(panel);
        
        if items_count == 0 {
            return;
        }

        let list_state = self.get_list_state_mut(panel);
        let selected = list_state.selected().unwrap_or(0);
        if selected > 0 {
            list_state.select(Some(selected - 1));
        } else {
            list_state.select(Some(items_count - 1));
        }
    }

    pub fn scroll_down(&mut self, panel: FocusedPanel) {
        let items_count = self.get_panel_items_count(panel);
        
        if items_count == 0 {
            return;
        }

        let list_state = self.get_list_state_mut(panel);
        let selected = list_state.selected().unwrap_or(0);
        if selected < items_count - 1 {
            list_state.select(Some(selected + 1));
        } else {
            list_state.select(Some(0));
        }
    }

    pub async fn start_editing(&mut self, panel: FocusedPanel) -> Result<(), anyhow::Error> {
        // Get the currently selected item
        let config_items = self.config_items.get(&panel).cloned().unwrap_or_default();
        if config_items.is_empty() {
            return Ok(());
        }

        let list_state = self.get_list_state_mut(panel);
        let selected_index = list_state.selected().unwrap_or(0);
        
        if let Some(item) = config_items.get(selected_index) {
            self.editing_item = Some((panel, item.key.clone()));
            
            // Set edit mode based on data type
            self.edit_mode = match &item.data_type {
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
            };
        }
        
        Ok(())
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
                EditMode::None => return Ok(()),
            };

            // Update the configuration item
            if let Some(items) = self.config_items.get_mut(panel) {
                for item in items.iter_mut() {
                    if item.key == *key {
                        item.value = new_value;
                        break;
                    }
                }
            }

            // TODO: Here we would call hyprctl to apply the change in real-time
            // For now, we just update the UI
            
            self.cancel_edit();
        }
        
        Ok(())
    }

    pub fn cancel_edit(&mut self) {
        self.edit_mode = EditMode::None;
        self.editing_item = None;
    }
}