use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

use crate::{config::Config, hyprctl::HyprCtl, ui::UI};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Running,
    Quitting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusedPanel {
    General,
    Input,
    Decoration,
    Animations,
    Gestures,
    Binds,
    WindowRules,
    LayerRules,
    Misc,
}

impl FocusedPanel {
    pub fn next(self) -> Self {
        match self {
            FocusedPanel::General => FocusedPanel::Input,
            FocusedPanel::Input => FocusedPanel::Decoration,
            FocusedPanel::Decoration => FocusedPanel::Animations,
            FocusedPanel::Animations => FocusedPanel::Gestures,
            FocusedPanel::Gestures => FocusedPanel::Binds,
            FocusedPanel::Binds => FocusedPanel::WindowRules,
            FocusedPanel::WindowRules => FocusedPanel::LayerRules,
            FocusedPanel::LayerRules => FocusedPanel::Misc,
            FocusedPanel::Misc => FocusedPanel::General,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            FocusedPanel::General => FocusedPanel::Misc,
            FocusedPanel::Input => FocusedPanel::General,
            FocusedPanel::Decoration => FocusedPanel::Input,
            FocusedPanel::Animations => FocusedPanel::Decoration,
            FocusedPanel::Gestures => FocusedPanel::Animations,
            FocusedPanel::Binds => FocusedPanel::Gestures,
            FocusedPanel::WindowRules => FocusedPanel::Binds,
            FocusedPanel::LayerRules => FocusedPanel::WindowRules,
            FocusedPanel::Misc => FocusedPanel::LayerRules,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            FocusedPanel::General => "General",
            FocusedPanel::Input => "Input",
            FocusedPanel::Decoration => "Decoration",
            FocusedPanel::Animations => "Animations",
            FocusedPanel::Gestures => "Gestures",
            FocusedPanel::Binds => "Binds",
            FocusedPanel::WindowRules => "Window Rules",
            FocusedPanel::LayerRules => "Layer Rules",
            FocusedPanel::Misc => "Misc",
        }
    }
}

pub struct App {
    pub state: AppState,
    pub debug: bool,
    pub focused_panel: FocusedPanel,
    pub config: Config,
    pub hyprctl: HyprCtl,
    pub ui: UI,
    pub last_tick: Instant,
    pub tick_rate: Duration,
}

impl App {
    pub async fn test_save_functionality(&mut self) -> Result<()> {
        eprintln!("=== Testing Save Functionality ===");

        // Collect all data that would be saved
        let config_changes = self.ui.collect_all_config_changes();
        let keybinds = self.ui.collect_keybinds();
        let window_rules = self.ui.collect_window_rules();
        let layer_rules = self.ui.collect_layer_rules();

        eprintln!("Config changes: {}", config_changes.len());
        eprintln!("Keybinds: {}", keybinds.len());
        eprintln!("Window rules: {}", window_rules.len());
        eprintln!("Layer rules: {}", layer_rules.len());

        // Test the save without actually writing to avoid modifying user's config
        eprintln!("=== Save functionality test complete ===");
        Ok(())
    }

    pub async fn new(debug: bool) -> Result<Self> {
        let config = Config::load().await?;

        // Try to initialize hyprctl, but don't fail if it's not available
        let hyprctl = match HyprCtl::new().await {
            Ok(hyprctl) => hyprctl,
            Err(e) => {
                eprintln!("Warning: Failed to initialize hyprctl: {e}");
                eprintln!("Will try to load configuration from config file instead.");
                // Create a dummy HyprCtl that will fail gracefully
                HyprCtl::new_disconnected()
            }
        };

        let mut ui = UI::new();

        // Set theme from config
        ui.set_theme(config.theme.clone());

        // Load current configuration from hyprctl or config file
        if let Err(e) = ui.load_current_config(&hyprctl).await {
            eprintln!("Warning: Failed to load current configuration: {e}");
            eprintln!("Using default placeholder values.");
        }

        Ok(Self {
            state: AppState::Running,
            debug,
            focused_panel: ui.current_tab,
            config,
            hyprctl,
            ui,
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(250),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            let app_state = (self.focused_panel, self.debug);
            terminal.draw(|f| self.ui.render(f, app_state))?;

            let timeout = self.tick_rate.saturating_sub(self.last_tick.elapsed());
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key.code).await?;
                    }
                }
            }

            if self.last_tick.elapsed() >= self.tick_rate {
                self.tick().await;
                self.last_tick = Instant::now();
            }

            if self.state == AppState::Quitting {
                break;
            }
        }
        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyCode) -> Result<()> {
        // Handle popup states first
        if self.ui.show_nixos_export_dialog {
            return self.handle_nixos_export_dialog_key(key).await;
        }
        
        if self.ui.show_save_dialog {
            return self.handle_save_dialog_key(key).await;
        }

        if self.ui.show_reload_dialog {
            return self.handle_reload_dialog_key(key).await;
        }

        if self.ui.show_popup {
            return self.handle_popup_key(key).await;
        }

        if self.ui.show_help {
            return self.handle_help_key(key).await;
        }

        if self.ui.search_mode {
            return self.handle_search_key(key).await;
        }

        if self.ui.edit_mode != crate::ui::EditMode::None {
            return self.handle_edit_key(key).await;
        }

        // Handle normal navigation
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = AppState::Quitting;
            }
            KeyCode::Tab | KeyCode::Right => {
                self.ui.next_tab();
                self.focused_panel = self.ui.current_tab;
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.ui.previous_tab();
                self.focused_panel = self.ui.current_tab;
            }
            KeyCode::Up => {
                self.ui.scroll_up();
            }
            KeyCode::Down => {
                self.ui.scroll_down();
            }
            KeyCode::PageUp => {
                self.ui.prev_page();
            }
            KeyCode::PageDown => {
                self.ui.next_page();
            }
            KeyCode::Enter => {
                self.ui.start_editing().await?;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.ui.show_reload_dialog = true;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.ui.show_save_dialog = true;
            }
            // Additional functionality to be re-implemented
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // TODO: Add item functionality
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // TODO: Delete item functionality
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                // TODO: Insert/Add item functionality
            }
            KeyCode::Char('/') => {
                self.ui.start_search_debounced();
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                let new_theme = self.ui.next_theme();
                self.config.theme = new_theme;
                // Save theme to config file
                if let Err(e) = self.config.save().await {
                    eprintln!("Warning: Failed to save theme to config: {e}");
                }
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Theme changed to: {}", self.config.theme);
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.export_config().await;
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.import_config().await;
            }
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.ui.toggle_help();
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                if self.config.nixos_mode {
                    self.show_nixos_export_dialog().await;
                } else {
                    self.ui.show_popup = true;
                    self.ui.popup_message = "NixOS export is only available on NixOS systems".to_string();
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_save_dialog_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.ui.show_save_dialog = false;
                self.save_config().await?;
                self.ui.show_popup = true;
                self.ui.popup_message = "Configuration saved successfully!".to_string();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.ui.show_save_dialog = false;
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn handle_nixos_export_dialog_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('1') => {
                self.ui.nixos_export_config_type = crate::nixos::NixConfigType::HomeManager;
                self.update_nixos_export_preview().await;
            }
            KeyCode::Char('2') => {
                self.ui.nixos_export_config_type = crate::nixos::NixConfigType::SystemConfig;
                self.update_nixos_export_preview().await;
            }
            KeyCode::Char('3') => {
                self.ui.nixos_export_config_type = crate::nixos::NixConfigType::FlakeHomeManager;
                self.update_nixos_export_preview().await;
            }
            KeyCode::Char('4') => {
                self.ui.nixos_export_config_type = crate::nixos::NixConfigType::FlakeSystem;
                self.update_nixos_export_preview().await;
            }
            KeyCode::Enter => {
                // Export with selected config type
                self.ui.show_nixos_export_dialog = false;
                self.export_nixos_config().await;
            }
            KeyCode::Esc => {
                self.ui.show_nixos_export_dialog = false;
                self.ui.nixos_export_preview = None;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_reload_dialog_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.ui.show_reload_dialog = false;
                self.reload_config().await?;
                self.ui.show_popup = true;
                self.ui.popup_message = "Configuration reloaded successfully!".to_string();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.ui.show_reload_dialog = false;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_popup_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter | KeyCode::Esc => {
                self.ui.show_popup = false;
                self.ui.popup_message.clear();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_edit_key(&mut self, key: KeyCode) -> Result<()> {
        use crate::ui::EditMode;

        match &mut self.ui.edit_mode {
            EditMode::Text {
                current_value,
                cursor_pos,
            } => {
                match key {
                    KeyCode::Enter => {
                        match self.ui.apply_edit_with_hyprctl(&self.hyprctl).await {
                            Ok(()) => {
                                self.ui.show_popup = true;
                                self.ui.popup_message = "Value updated successfully!".to_string();
                            }
                            Err(_) => {
                                // Error message already set in apply_edit_with_hyprctl
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.ui.cancel_edit();
                    }
                    KeyCode::Char(c) => {
                        current_value.insert(*cursor_pos, c);
                        *cursor_pos += 1;
                    }
                    KeyCode::Backspace => {
                        if *cursor_pos > 0 {
                            *cursor_pos -= 1;
                            current_value.remove(*cursor_pos);
                        }
                    }
                    KeyCode::Left => {
                        if *cursor_pos > 0 {
                            *cursor_pos -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if *cursor_pos < current_value.len() {
                            *cursor_pos += 1;
                        }
                    }
                    KeyCode::Home => {
                        *cursor_pos = 0;
                    }
                    KeyCode::End => {
                        *cursor_pos = current_value.len();
                    }
                    _ => {}
                }
            }
            EditMode::Boolean { current_value } => {
                match key {
                    KeyCode::Enter => {
                        match self.ui.apply_edit_with_hyprctl(&self.hyprctl).await {
                            Ok(()) => {
                                self.ui.show_popup = true;
                                self.ui.popup_message = "Value updated successfully!".to_string();
                            }
                            Err(_) => {
                                // Error message already set in apply_edit_with_hyprctl
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.ui.cancel_edit();
                    }
                    KeyCode::Char(' ') => {
                        *current_value = !*current_value;
                    }
                    _ => {}
                }
            }
            EditMode::Select { options, selected } => {
                match key {
                    KeyCode::Enter => {
                        match self.ui.apply_edit_with_hyprctl(&self.hyprctl).await {
                            Ok(()) => {
                                self.ui.show_popup = true;
                                self.ui.popup_message = "Value updated successfully!".to_string();
                            }
                            Err(_) => {
                                // Error message already set in apply_edit_with_hyprctl
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.ui.cancel_edit();
                    }
                    KeyCode::Up => {
                        if *selected > 0 {
                            *selected -= 1;
                        } else {
                            *selected = options.len() - 1;
                        }
                    }
                    KeyCode::Down => {
                        if *selected < options.len() - 1 {
                            *selected += 1;
                        } else {
                            *selected = 0;
                        }
                    }
                    _ => {}
                }
            }
            EditMode::Slider {
                current_value,
                min,
                max,
                step,
            } => {
                match key {
                    KeyCode::Enter => {
                        match self.ui.apply_edit_with_hyprctl(&self.hyprctl).await {
                            Ok(()) => {
                                self.ui.show_popup = true;
                                self.ui.popup_message = "Value updated successfully!".to_string();
                            }
                            Err(_) => {
                                // Error message already set in apply_edit_with_hyprctl
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.ui.cancel_edit();
                    }
                    KeyCode::Left => {
                        *current_value = (*current_value - *step).max(*min);
                    }
                    KeyCode::Right => {
                        *current_value = (*current_value + *step).min(*max);
                    }
                    KeyCode::Home => {
                        *current_value = *min;
                    }
                    KeyCode::End => {
                        *current_value = *max;
                    }
                    _ => {}
                }
            }
            EditMode::Keybind {
                modifiers,
                key: key_field,
                dispatcher,
                args,
                editing_field,
            } => {
                match key {
                    KeyCode::Enter => {
                        match self.ui.apply_edit_with_hyprctl(&self.hyprctl).await {
                            Ok(()) => {
                                self.ui.show_popup = true;
                                self.ui.popup_message = "Keybind updated successfully!".to_string();
                            }
                            Err(_) => {
                                // Error message already set in apply_edit_with_hyprctl
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.ui.cancel_edit();
                    }
                    KeyCode::Tab => {
                        // Cycle through editing fields
                        *editing_field = match editing_field {
                            crate::ui::KeybindField::Modifiers => crate::ui::KeybindField::Key,
                            crate::ui::KeybindField::Key => crate::ui::KeybindField::Dispatcher,
                            crate::ui::KeybindField::Dispatcher => crate::ui::KeybindField::Args,
                            crate::ui::KeybindField::Args => crate::ui::KeybindField::Modifiers,
                        };
                    }
                    KeyCode::Char(c) => {
                        // Add character to the currently editing field
                        match editing_field {
                            crate::ui::KeybindField::Key => {
                                *key_field = c.to_string();
                            }
                            crate::ui::KeybindField::Dispatcher => {
                                dispatcher.push(c);
                            }
                            crate::ui::KeybindField::Args => {
                                args.push(c);
                            }
                            crate::ui::KeybindField::Modifiers => {
                                // Handle modifier addition
                                let mod_string = match c {
                                    's' => "SUPER",
                                    'a' => "ALT",
                                    'c' => "CTRL",
                                    'h' => "SHIFT",
                                    _ => return Ok(()),
                                };

                                if !modifiers.contains(&mod_string.to_string()) {
                                    modifiers.push(mod_string.to_string());
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        // Remove characters from the currently editing field
                        match editing_field {
                            crate::ui::KeybindField::Key => {
                                key_field.clear();
                            }
                            crate::ui::KeybindField::Dispatcher => {
                                dispatcher.pop();
                            }
                            crate::ui::KeybindField::Args => {
                                args.pop();
                            }
                            crate::ui::KeybindField::Modifiers => {
                                modifiers.pop();
                            }
                        }
                    }
                    _ => {}
                }
            }
            EditMode::Rule {
                rule_type: _,
                pattern,
                action,
                editing_field,
            } => {
                match key {
                    KeyCode::Enter => {
                        match self.ui.apply_edit_with_hyprctl(&self.hyprctl).await {
                            Ok(()) => {
                                self.ui.show_popup = true;
                                self.ui.popup_message = "Rule updated successfully!".to_string();
                            }
                            Err(_) => {
                                // Error message already set in apply_edit_with_hyprctl
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.ui.cancel_edit();
                    }
                    KeyCode::Tab => {
                        // Cycle between pattern and action editing
                        *editing_field = match editing_field {
                            crate::ui::RuleField::Pattern => crate::ui::RuleField::Action,
                            crate::ui::RuleField::Action => crate::ui::RuleField::Pattern,
                        };
                    }
                    KeyCode::Char(c) => {
                        // Add character to the currently editing field
                        match editing_field {
                            crate::ui::RuleField::Pattern => {
                                pattern.push(c);
                            }
                            crate::ui::RuleField::Action => {
                                action.push(c);
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        // Remove characters from the currently editing field
                        match editing_field {
                            crate::ui::RuleField::Pattern => {
                                pattern.pop();
                            }
                            crate::ui::RuleField::Action => {
                                action.pop();
                            }
                        }
                    }
                    _ => {}
                }
            }
            // TODO: Re-implement AddingItem edit mode
            EditMode::None => {
                // This shouldn't happen, but handle it gracefully
                self.ui.cancel_edit();
            }
        }
        Ok(())
    }

    async fn handle_help_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?') => {
                self.ui.toggle_help();
            }
            KeyCode::Up => {
                self.ui.scroll_help_up();
            }
            KeyCode::Down => {
                self.ui.scroll_help_down();
            }
            KeyCode::PageUp => {
                self.ui.scroll_help_page_up();
            }
            KeyCode::PageDown => {
                self.ui.scroll_help_page_down();
            }
            KeyCode::Home => {
                self.ui.scroll_help_to_top();
            }
            KeyCode::End => {
                self.ui.scroll_help_to_bottom();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_search_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                self.ui.cancel_search_debounced();
            }
            KeyCode::Enter => {
                self.ui.cancel_search_debounced();
            }
            KeyCode::Char(c) => {
                self.ui.add_search_char_debounced(c);
            }
            KeyCode::Backspace => {
                self.ui.remove_search_char_debounced();
            }
            KeyCode::Left => {
                self.ui.move_search_cursor_left();
            }
            KeyCode::Right => {
                self.ui.move_search_cursor_right();
            }
            KeyCode::Home => {
                self.ui.search_cursor = 0;
            }
            KeyCode::End => {
                self.ui.search_cursor = self.ui.search_query.len();
            }
            _ => {}
        }
        Ok(())
    }

    async fn tick(&mut self) {
        // Update debounced search if delay has passed
        if self.ui.update_debounced_search() {
            // Search query was updated, pagination may need to be recalculated
            // This will be handled automatically in the next render cycle
        }
    }

    async fn reload_config(&mut self) -> Result<()> {
        // Reload the application's own config
        self.config = Config::load().await?;

        // If Hyprland is running, try to reload its configuration first
        if self.hyprctl.is_hyprland_running().await {
            match self.hyprctl.reload_config().await {
                Ok(()) => {
                    eprintln!("Hyprland configuration reloaded from file");
                }
                Err(e) => {
                    eprintln!("Warning: Failed to reload Hyprland configuration: {e}");
                }
            }
        }

        // Reload current configuration into UI (from hyprctl or config file)
        if let Err(e) = self.ui.load_current_config(&self.hyprctl).await {
            eprintln!("Warning: Failed to reload UI configuration: {e}");
        }

        Ok(())
    }

    async fn save_config(&mut self) -> Result<()> {
        // TODO: Re-implement validation before saving

        // Save the application's own config
        self.config.save().await?;

        // Collect all configuration changes from the UI
        let config_changes = self.ui.collect_all_config_changes();
        let keybinds = self.ui.collect_keybinds();
        let window_rules = self.ui.collect_window_rules();
        let layer_rules = self.ui.collect_layer_rules();

        // Check if we have any changes to save
        let has_changes = !config_changes.is_empty()
            || !keybinds.is_empty()
            || !window_rules.is_empty()
            || !layer_rules.is_empty();

        if has_changes {
            // Save changes to the actual Hyprland config file
            self.config
                .save_hyprland_config_with_rules(
                    &config_changes,
                    &keybinds,
                    &window_rules,
                    &layer_rules,
                )
                .await?;

            eprintln!(
                "Saved {} config options, {} keybinds, {} window rules, {} layer rules",
                config_changes.len(),
                keybinds.len(),
                window_rules.len(),
                layer_rules.len()
            );

            // If Hyprland is running, try to reload the configuration
            if self.hyprctl.is_hyprland_running().await {
                match self.hyprctl.reload_config().await {
                    Ok(()) => {
                        eprintln!("Hyprland configuration reloaded successfully");
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to reload Hyprland configuration: {e}");
                        eprintln!("Changes saved to config file but may require manual restart");
                    }
                }
            } else {
                eprintln!("Hyprland not running - changes saved to config file");
            }
        } else {
            eprintln!("No configuration changes to save");
        }

        Ok(())
    }

    async fn export_config(&mut self) {
        match self.export_config_to_file().await {
            Ok(path) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Configuration exported to: {path}");
            }
            Err(e) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Export failed: {e}");
            }
        }
    }

    async fn import_config(&mut self) {
        match self.import_config_from_file().await {
            Ok(imported_count) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Imported {imported_count} configuration items successfully!");
            }
            Err(e) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Import failed: {e}");
            }
        }
    }

    async fn export_config_to_file(&mut self) -> Result<String> {
        use std::fs;
        use chrono::Utc;

        // Create export directory if it doesn't exist
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        let export_dir = config_dir.join("r-hyprconfig").join("exports");
        fs::create_dir_all(&export_dir)?;

        // Generate timestamp for unique filename
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("hyprconfig_export_{timestamp}.toml");
        let export_path = export_dir.join(&filename);

        // Collect all configuration data
        let config_changes = self.ui.collect_all_config_changes();
        let keybinds = self.ui.collect_keybinds();
        let window_rules = self.ui.collect_window_rules();
        let layer_rules = self.ui.collect_layer_rules();

        // Create export data structure
        let export_data = toml::Table::from_iter([
            ("metadata".to_string(), toml::Value::Table(toml::Table::from_iter([
                ("export_date".to_string(), toml::Value::String(Utc::now().to_rfc3339())),
                ("version".to_string(), toml::Value::String("1.0".to_string())),
                ("theme".to_string(), toml::Value::String(self.config.theme.to_string())),
            ]))),
            ("config_options".to_string(), toml::Value::Table(
                config_changes.into_iter()
                    .map(|(k, v)| (k, toml::Value::String(v)))
                    .collect()
            )),
            ("keybinds".to_string(), toml::Value::Array(
                keybinds.into_iter()
                    .map(toml::Value::String)
                    .collect()
            )),
            ("window_rules".to_string(), toml::Value::Array(
                window_rules.into_iter()
                    .map(toml::Value::String)
                    .collect()
            )),
            ("layer_rules".to_string(), toml::Value::Array(
                layer_rules.into_iter()
                    .map(toml::Value::String)
                    .collect()
            )),
        ]);

        // Write to file
        let toml_content = toml::to_string_pretty(&export_data)?;
        fs::write(&export_path, toml_content)?;

        Ok(export_path.to_string_lossy().to_string())
    }

    async fn import_config_from_file(&mut self) -> Result<usize> {
        use std::fs;

        // Look for the most recent export file
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        let export_dir = config_dir.join("r-hyprconfig").join("exports");

        if !export_dir.exists() {
            return Err(anyhow::anyhow!("No export directory found. Please export a configuration first."));
        }

        // Find the most recent export file
        let mut export_files: Vec<_> = fs::read_dir(&export_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "toml")
                    .unwrap_or(false)
            })
            .collect();

        if export_files.is_empty() {
            return Err(anyhow::anyhow!("No export files found in {:?}", export_dir));
        }

        // Sort by modification time (most recent first)
        export_files.sort_by_key(|entry| {
            entry.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH)
        });
        export_files.reverse();

        let latest_export = &export_files[0];
        let export_path = latest_export.path();

        // Read and parse the export file
        let content = fs::read_to_string(&export_path)?;
        let export_data: toml::Table = toml::from_str(&content)?;

        let mut imported_count = 0;

        // Import theme if present
        if let Some(metadata) = export_data.get("metadata").and_then(|v| v.as_table()) {
            if let Some(theme_str) = metadata.get("theme").and_then(|v| v.as_str()) {
                if let Ok(theme) = theme_str.parse::<crate::theme::ColorScheme>() {
                    self.config.theme = theme.clone();
                    self.ui.set_theme(theme);
                    let _ = self.config.save().await;
                }
            }
        }

        // Import configuration options
        if let Some(config_options) = export_data.get("config_options").and_then(|v| v.as_table()) {
            for (key, value) in config_options {
                if let Some(value_str) = value.as_str() {
                    // Update the UI config items
                    self.ui.update_config_item_from_import(key, value_str);
                    imported_count += 1;
                }
            }
        }

        // Import keybinds
        if let Some(keybinds) = export_data.get("keybinds").and_then(|v| v.as_array()) {
            for keybind in keybinds {
                if let Some(kb_str) = keybind.as_str() {
                    self.ui.add_imported_keybind(kb_str);
                    imported_count += 1;
                }
            }
        }

        // Import window rules
        if let Some(rules) = export_data.get("window_rules").and_then(|v| v.as_array()) {
            for rule in rules {
                if let Some(rule_str) = rule.as_str() {
                    self.ui.add_imported_window_rule(rule_str);
                    imported_count += 1;
                }
            }
        }

        // Import layer rules
        if let Some(rules) = export_data.get("layer_rules").and_then(|v| v.as_array()) {
            for rule in rules {
                if let Some(rule_str) = rule.as_str() {
                    self.ui.add_imported_layer_rule(rule_str);
                    imported_count += 1;
                }
            }
        }

        // Refresh the UI to show imported data
        self.ui.refresh_all_panels();

        Ok(imported_count)
    }
    
    async fn show_nixos_export_dialog(&mut self) {
        // Initialize the dialog with current config type or default
        if let Some(nixos_config_type) = &self.config.nixos_config_type {
            self.ui.nixos_export_config_type = nixos_config_type.clone();
        }
        
        // Generate preview
        self.update_nixos_export_preview().await;
        
        self.ui.show_nixos_export_dialog = true;
    }
    
    async fn update_nixos_export_preview(&mut self) {
        use crate::nixos::ConfigConverter;
        
        let config_changes = self.ui.collect_all_config_changes();
        let keybinds = self.ui.collect_keybinds();
        let window_rules = self.ui.collect_window_rules();
        let layer_rules = self.ui.collect_layer_rules();
        
        let converter = ConfigConverter::new();
        match converter.traditional_to_nixos(
            &config_changes,
            &keybinds,
            &window_rules,
            &layer_rules,
            self.ui.nixos_export_config_type.clone(),
        ) {
            Ok(config) => {
                // Truncate preview if too long
                let preview = if config.len() > 2000 {
                    format!("{}...\n\n[Output truncated - {} more characters]", 
                           &config[..2000], config.len() - 2000)
                } else {
                    config
                };
                self.ui.nixos_export_preview = Some(preview);
            }
            Err(e) => {
                self.ui.nixos_export_preview = Some(format!("Error generating preview: {e}"));
            }
        }
    }

    async fn export_nixos_config(&mut self) {
        match self.export_nixos_config_to_file().await {
            Ok(path) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("NixOS configuration exported to: {path}");
            }
            Err(e) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("NixOS export failed: {e}");
            }
        }
    }
    
    async fn export_nixos_config_to_file(&mut self) -> Result<String> {
        use crate::nixos::ConfigConverter;
        use std::fs;
        use chrono::Utc;
        
        // Create export directory if it doesn't exist
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        let export_dir = config_dir.join("r-hyprconfig").join("nixos-exports");
        fs::create_dir_all(&export_dir)?;
        
        // Generate timestamp for unique filename
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("hyprland_nixos_export_{timestamp}.nix");
        let export_path = export_dir.join(&filename);
        
        // Collect all configuration data
        let config_changes = self.ui.collect_all_config_changes();
        let keybinds = self.ui.collect_keybinds();
        let window_rules = self.ui.collect_window_rules();
        let layer_rules = self.ui.collect_layer_rules();
        
        // Use the selected config type from the dialog
        let target_type = self.ui.nixos_export_config_type.clone();
        
        // Convert to NixOS format
        let converter = ConfigConverter::new();
        let nixos_config = converter.traditional_to_nixos(
            &config_changes,
            &keybinds,
            &window_rules,
            &layer_rules,
            target_type,
        )?;
        
        // Write to file
        fs::write(&export_path, nixos_config)?;
        
        Ok(export_path.to_string_lossy().to_string())
    }
}
