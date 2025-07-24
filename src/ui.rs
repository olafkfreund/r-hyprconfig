use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::app::FocusedPanel;

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

        ui
    }

    pub fn render(&mut self, f: &mut Frame, app_state: (FocusedPanel, bool)) {
        let size = f.area();

        // Create main layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(size);

        let (focused_panel, debug) = app_state;
        
        // Render header
        self.render_header(f, main_chunks[0], debug);

        // Create grid layout for configuration panels
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(main_chunks[1]);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(content_chunks[0]);

        let middle_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(content_chunks[1]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(content_chunks[2]);

        // Render configuration panels
        self.render_panel(f, left_chunks[0], FocusedPanel::General, focused_panel);
        self.render_panel(f, left_chunks[1], FocusedPanel::Input, focused_panel);
        self.render_panel(f, left_chunks[2], FocusedPanel::Decoration, focused_panel);

        self.render_panel(f, middle_chunks[0], FocusedPanel::Animations, focused_panel);
        self.render_panel(f, middle_chunks[1], FocusedPanel::Gestures, focused_panel);
        self.render_panel(f, middle_chunks[2], FocusedPanel::Binds, focused_panel);

        self.render_panel(f, right_chunks[0], FocusedPanel::WindowRules, focused_panel);
        self.render_panel(f, right_chunks[1], FocusedPanel::LayerRules, focused_panel);
        self.render_panel(f, right_chunks[2], FocusedPanel::Misc, focused_panel);

        // Render footer
        self.render_footer(f, main_chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect, debug: bool) {
        let title = if debug {
            "R-Hyprconfig - Debug Mode"
        } else {
            "R-Hyprconfig - Hyprland Configuration Manager"
        };

        let header = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
                    .border_type(BorderType::Rounded),
            );

        f.render_widget(header, area);
    }

    fn render_panel(&mut self, f: &mut Frame, area: Rect, panel: FocusedPanel, focused_panel: FocusedPanel) {
        let is_focused = focused_panel == panel;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let block = Block::default()
            .title(panel.as_str())
            .borders(Borders::ALL)
            .border_style(border_style)
            .border_type(BorderType::Rounded);

        // Create items without borrowing self
        let items = Self::get_static_panel_items(panel);
        let items_len = items.len();
        
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let list_state = self.get_list_state_mut(panel);
        let selected_position = list_state.selected().unwrap_or(0);
        f.render_stateful_widget(list, area, list_state);

        // Render scrollbar if needed
        if area.height > 3 {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let scrollbar_area = Rect {
                x: area.x + area.width - 1,
                y: area.y + 1,
                width: 1,
                height: area.height - 2,
            };

            let mut scrollbar_state = ScrollbarState::default()
                .content_length(items_len)
                .position(selected_position);

            f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Span::styled("Tab/→", Style::default().fg(Color::Yellow)),
            Span::raw(": Next panel | "),
            Span::styled("Shift+Tab/←", Style::default().fg(Color::Yellow)),
            Span::raw(": Previous panel | "),
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate | "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Select | "),
            Span::styled("R", Style::default().fg(Color::Yellow)),
            Span::raw(": Reload | "),
            Span::styled("S", Style::default().fg(Color::Yellow)),
            Span::raw(": Save | "),
            Span::styled("Q/Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ];

        let footer = Paragraph::new(Line::from(help_text))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
                    .border_type(BorderType::Rounded),
            );

        f.render_widget(footer, area);
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
        Self::get_static_panel_items(panel).len()
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

    pub async fn toggle_selection(&mut self, _panel: FocusedPanel) -> Result<(), anyhow::Error> {
        // TODO: Implement configuration value editing
        // This will be used to modify the selected configuration option
        Ok(())
    }
}