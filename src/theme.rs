use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColorScheme {
    Default,
    Gruvbox,
    Nord,
    Catppuccin,
    Dracula,
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme::Gruvbox
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub scheme: ColorScheme,
    
    // Background colors
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,
    pub bg_selected: Color,
    pub bg_search: Color,
    
    // Foreground colors
    pub fg_primary: Color,
    pub fg_secondary: Color,
    pub fg_muted: Color,
    pub fg_bright: Color,
    
    // Accent colors
    pub accent_primary: Color,
    pub accent_secondary: Color,
    pub accent_success: Color,
    pub accent_warning: Color,
    pub accent_error: Color,
    pub accent_info: Color,
    
    // Border colors
    pub border_normal: Color,
    pub border_focused: Color,
    pub border_active: Color,
    
    // Data type colors
    pub type_integer: Color,
    pub type_float: Color,
    pub type_boolean: Color,
    pub type_string: Color,
    pub type_color: Color,
    pub type_keyword: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::gruvbox()
    }
}

impl Theme {
    pub fn gruvbox() -> Self {
        Self {
            name: "Gruvbox".to_string(),
            scheme: ColorScheme::Gruvbox,
            
            // Gruvbox background colors
            bg_primary: Color::Rgb(40, 40, 40),      // #282828 - dark0
            bg_secondary: Color::Rgb(60, 56, 54),    // #3c3836 - dark1
            bg_tertiary: Color::Rgb(80, 73, 69),     // #504945 - dark2
            bg_selected: Color::Rgb(102, 92, 84),    // #665c54 - dark3
            bg_search: Color::Rgb(50, 48, 47),       // #32302f - dark0_s
            
            // Gruvbox foreground colors
            fg_primary: Color::Rgb(235, 219, 178),   // #ebdbb2 - light1
            fg_secondary: Color::Rgb(213, 196, 161), // #d5c4a1 - light2
            fg_muted: Color::Rgb(189, 174, 147),     // #bdae93 - light3
            fg_bright: Color::Rgb(251, 241, 199),    // #fbf1c7 - light0
            
            // Gruvbox accent colors
            accent_primary: Color::Rgb(254, 128, 25),    // #fe8019 - orange
            accent_secondary: Color::Rgb(250, 189, 47),  // #fabd2f - yellow
            accent_success: Color::Rgb(184, 187, 38),    // #b8bb26 - green
            accent_warning: Color::Rgb(250, 189, 47),    // #fabd2f - yellow
            accent_error: Color::Rgb(251, 73, 52),       // #fb4934 - red
            accent_info: Color::Rgb(131, 165, 152),      // #83a598 - blue
            
            // Gruvbox border colors
            border_normal: Color::Rgb(124, 111, 100),    // #7c6f64 - gray
            border_focused: Color::Rgb(254, 128, 25),    // #fe8019 - orange
            border_active: Color::Rgb(250, 189, 47),     // #fabd2f - yellow
            
            // Gruvbox data type colors
            type_integer: Color::Rgb(184, 187, 38),      // #b8bb26 - green
            type_float: Color::Rgb(131, 165, 152),       // #83a598 - blue
            type_boolean: Color::Rgb(250, 189, 47),      // #fabd2f - yellow
            type_string: Color::Rgb(235, 219, 178),      // #ebdbb2 - light1
            type_color: Color::Rgb(211, 134, 155),       // #d3869b - purple
            type_keyword: Color::Rgb(254, 128, 25),      // #fe8019 - orange
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            scheme: ColorScheme::Nord,
            
            // Nord background colors
            bg_primary: Color::Rgb(46, 52, 64),      // #2e3440 - polar night 0
            bg_secondary: Color::Rgb(59, 66, 82),    // #3b4252 - polar night 1
            bg_tertiary: Color::Rgb(67, 76, 94),     // #434c5e - polar night 2
            bg_selected: Color::Rgb(76, 86, 106),    // #4c566a - polar night 3
            bg_search: Color::Rgb(46, 52, 64),       // #2e3440
            
            // Nord foreground colors
            fg_primary: Color::Rgb(236, 239, 244),   // #eceff4 - snow storm 2
            fg_secondary: Color::Rgb(229, 233, 240), // #e5e9f0 - snow storm 1
            fg_muted: Color::Rgb(216, 222, 233),     // #d8dee9 - snow storm 0
            fg_bright: Color::Rgb(236, 239, 244),    // #eceff4
            
            // Nord accent colors
            accent_primary: Color::Rgb(136, 192, 208),   // #88c0d0 - frost 1
            accent_secondary: Color::Rgb(129, 161, 193), // #81a1c1 - frost 2
            accent_success: Color::Rgb(163, 190, 140),   // #a3be8c - aurora green
            accent_warning: Color::Rgb(235, 203, 139),   // #ebcb8b - aurora yellow
            accent_error: Color::Rgb(191, 97, 106),      // #bf616a - aurora red
            accent_info: Color::Rgb(94, 129, 172),       // #5e81ac - frost 3
            
            // Nord border colors
            border_normal: Color::Rgb(76, 86, 106),      // #4c566a
            border_focused: Color::Rgb(136, 192, 208),   // #88c0d0
            border_active: Color::Rgb(129, 161, 193),    // #81a1c1
            
            // Nord data type colors
            type_integer: Color::Rgb(163, 190, 140),     // #a3be8c - green
            type_float: Color::Rgb(136, 192, 208),       // #88c0d0 - frost 1
            type_boolean: Color::Rgb(235, 203, 139),     // #ebcb8b - yellow
            type_string: Color::Rgb(236, 239, 244),      // #eceff4 - white
            type_color: Color::Rgb(180, 142, 173),       // #b48ead - aurora purple
            type_keyword: Color::Rgb(208, 135, 112),     // #d08770 - aurora orange
        }
    }

    pub fn catppuccin() -> Self {
        Self {
            name: "Catppuccin".to_string(),
            scheme: ColorScheme::Catppuccin,
            
            // Catppuccin Mocha background colors
            bg_primary: Color::Rgb(30, 30, 46),      // #1e1e2e - base
            bg_secondary: Color::Rgb(49, 50, 68),    // #313244 - surface0
            bg_tertiary: Color::Rgb(69, 71, 90),     // #45475a - surface1
            bg_selected: Color::Rgb(88, 91, 112),    // #585b70 - surface2
            bg_search: Color::Rgb(24, 24, 37),       // #181825 - crust
            
            // Catppuccin foreground colors
            fg_primary: Color::Rgb(205, 214, 244),   // #cdd6f4 - text
            fg_secondary: Color::Rgb(186, 194, 222), // #bac2de - subtext1
            fg_muted: Color::Rgb(166, 173, 200),     // #a6adc8 - subtext0
            fg_bright: Color::Rgb(205, 214, 244),    // #cdd6f4 - text
            
            // Catppuccin accent colors
            accent_primary: Color::Rgb(137, 180, 250),   // #89b4fa - blue
            accent_secondary: Color::Rgb(203, 166, 247), // #cba6f7 - mauve
            accent_success: Color::Rgb(166, 227, 161),   // #a6e3a1 - green
            accent_warning: Color::Rgb(249, 226, 175),   // #f9e2af - yellow
            accent_error: Color::Rgb(243, 139, 168),     // #f38ba8 - red
            accent_info: Color::Rgb(116, 199, 236),      // #74c7ec - sapphire
            
            // Catppuccin border colors
            border_normal: Color::Rgb(88, 91, 112),      // #585b70 - surface2
            border_focused: Color::Rgb(137, 180, 250),   // #89b4fa - blue
            border_active: Color::Rgb(203, 166, 247),    // #cba6f7 - mauve
            
            // Catppuccin data type colors
            type_integer: Color::Rgb(166, 227, 161),     // #a6e3a1 - green
            type_float: Color::Rgb(116, 199, 236),       // #74c7ec - sapphire
            type_boolean: Color::Rgb(249, 226, 175),     // #f9e2af - yellow
            type_string: Color::Rgb(205, 214, 244),      // #cdd6f4 - text
            type_color: Color::Rgb(245, 194, 231),       // #f5c2e7 - pink
            type_keyword: Color::Rgb(250, 179, 135),     // #fab387 - peach
        }
    }

    pub fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            scheme: ColorScheme::Dracula,
            
            // Dracula background colors
            bg_primary: Color::Rgb(40, 42, 54),      // #282a36 - background
            bg_secondary: Color::Rgb(68, 71, 90),    // #44475a - current line
            bg_tertiary: Color::Rgb(98, 114, 164),   // #6272a4 - comment
            bg_selected: Color::Rgb(68, 71, 90),     // #44475a - selection
            bg_search: Color::Rgb(40, 42, 54),       // #282a36
            
            // Dracula foreground colors
            fg_primary: Color::Rgb(248, 248, 242),   // #f8f8f2 - foreground
            fg_secondary: Color::Rgb(248, 248, 242), // #f8f8f2
            fg_muted: Color::Rgb(98, 114, 164),      // #6272a4 - comment
            fg_bright: Color::Rgb(255, 255, 255),    // #ffffff
            
            // Dracula accent colors
            accent_primary: Color::Rgb(139, 233, 253),   // #8be9fd - cyan
            accent_secondary: Color::Rgb(255, 121, 198), // #ff79c6 - pink
            accent_success: Color::Rgb(80, 250, 123),    // #50fa7b - green
            accent_warning: Color::Rgb(241, 250, 140),   // #f1fa8c - yellow
            accent_error: Color::Rgb(255, 85, 85),       // #ff5555 - red
            accent_info: Color::Rgb(98, 114, 164),       // #6272a4 - comment
            
            // Dracula border colors
            border_normal: Color::Rgb(98, 114, 164),     // #6272a4 - comment
            border_focused: Color::Rgb(139, 233, 253),   // #8be9fd - cyan
            border_active: Color::Rgb(255, 121, 198),    // #ff79c6 - pink
            
            // Dracula data type colors
            type_integer: Color::Rgb(80, 250, 123),      // #50fa7b - green
            type_float: Color::Rgb(139, 233, 253),       // #8be9fd - cyan
            type_boolean: Color::Rgb(241, 250, 140),     // #f1fa8c - yellow
            type_string: Color::Rgb(248, 248, 242),      // #f8f8f2 - foreground
            type_color: Color::Rgb(255, 121, 198),       // #ff79c6 - pink
            type_keyword: Color::Rgb(255, 184, 108),     // #ffb86c - orange
        }
    }

    pub fn default_theme() -> Self {
        Self {
            name: "Default".to_string(),
            scheme: ColorScheme::Default,
            
            // Default TUI colors
            bg_primary: Color::Black,
            bg_secondary: Color::Rgb(20, 20, 20),
            bg_tertiary: Color::Rgb(40, 40, 40),
            bg_selected: Color::Rgb(60, 60, 60),
            bg_search: Color::Rgb(30, 30, 60),
            
            fg_primary: Color::White,
            fg_secondary: Color::Gray,
            fg_muted: Color::Rgb(128, 128, 128),
            fg_bright: Color::Rgb(255, 255, 255),
            
            accent_primary: Color::Cyan,
            accent_secondary: Color::Yellow,
            accent_success: Color::Green,
            accent_warning: Color::Yellow,
            accent_error: Color::Red,
            accent_info: Color::Blue,
            
            border_normal: Color::Gray,
            border_focused: Color::Cyan,
            border_active: Color::Yellow,
            
            type_integer: Color::Green,
            type_float: Color::Cyan,
            type_boolean: Color::Yellow,
            type_string: Color::White,
            type_color: Color::Magenta,
            type_keyword: Color::Rgb(255, 165, 0),
        }
    }

    pub fn from_scheme(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Default => Self::default_theme(),
            ColorScheme::Gruvbox => Self::gruvbox(),
            ColorScheme::Nord => Self::nord(),
            ColorScheme::Catppuccin => Self::catppuccin(),
            ColorScheme::Dracula => Self::dracula(),
        }
    }

    // Helper methods for creating styled components
    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.accent_primary)
            .bg(self.bg_primary)
    }

    pub fn tab_style(&self, focused: bool) -> Style {
        if focused {
            Style::default()
                .fg(self.bg_primary)
                .bg(self.accent_primary)
        } else {
            Style::default()
                .fg(self.fg_secondary)
                .bg(self.bg_secondary)
        }
    }

    pub fn list_style(&self) -> Style {
        Style::default()
            .fg(self.fg_primary)
            .bg(self.bg_primary)
    }

    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.fg_bright)
            .bg(self.bg_selected)
    }

    pub fn border_style(&self, focused: bool) -> Style {
        Style::default().fg(if focused {
            self.border_focused
        } else {
            self.border_normal
        })
    }

    pub fn search_style(&self, active: bool) -> Style {
        if active {
            Style::default()
                .fg(self.accent_warning)
                .bg(self.bg_search)
        } else {
            Style::default()
                .fg(self.fg_muted)
                .bg(self.bg_secondary)
        }
    }

    pub fn data_type_style(&self, data_type: &crate::ui::ConfigDataType) -> Style {
        let color = match data_type {
            crate::ui::ConfigDataType::Integer { .. } => self.type_integer,
            crate::ui::ConfigDataType::Float { .. } => self.type_float,
            crate::ui::ConfigDataType::Boolean => self.type_boolean,
            crate::ui::ConfigDataType::String => self.type_string,
            crate::ui::ConfigDataType::Color => self.type_color,
            crate::ui::ConfigDataType::Keyword { .. } => self.type_keyword,
        };
        Style::default().fg(color)
    }

    pub fn footer_style(&self) -> Style {
        Style::default()
            .fg(self.fg_secondary)
            .bg(self.bg_primary)
    }

    pub fn popup_style(&self) -> Style {
        Style::default()
            .fg(self.fg_primary)
            .bg(self.bg_secondary)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.accent_success)
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.accent_warning)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.accent_error)
    }

    pub fn info_style(&self) -> Style {
        Style::default().fg(self.accent_info)
    }
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorScheme::Default => write!(f, "Default"),
            ColorScheme::Gruvbox => write!(f, "Gruvbox"),
            ColorScheme::Nord => write!(f, "Nord"),
            ColorScheme::Catppuccin => write!(f, "Catppuccin"),
            ColorScheme::Dracula => write!(f, "Dracula"),
        }
    }
}

impl ColorScheme {
    pub fn all() -> Vec<ColorScheme> {
        vec![
            ColorScheme::Gruvbox,
            ColorScheme::Nord,
            ColorScheme::Catppuccin,
            ColorScheme::Dracula,
            ColorScheme::Default,
        ]
    }

    pub fn next(&self) -> Self {
        match self {
            ColorScheme::Default => ColorScheme::Gruvbox,
            ColorScheme::Gruvbox => ColorScheme::Nord,
            ColorScheme::Nord => ColorScheme::Catppuccin,
            ColorScheme::Catppuccin => ColorScheme::Dracula,
            ColorScheme::Dracula => ColorScheme::Default,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            ColorScheme::Default => ColorScheme::Dracula,
            ColorScheme::Gruvbox => ColorScheme::Default,
            ColorScheme::Nord => ColorScheme::Gruvbox,
            ColorScheme::Catppuccin => ColorScheme::Nord,
            ColorScheme::Dracula => ColorScheme::Catppuccin,
        }
    }
}