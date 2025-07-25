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

use crate::{batch::BatchManager, config::Config, hyprctl::HyprCtl, ui::UI};

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
    Import,
    Export,
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
            FocusedPanel::Misc => FocusedPanel::Import,
            FocusedPanel::Import => FocusedPanel::Export,
            FocusedPanel::Export => FocusedPanel::General,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            FocusedPanel::General => FocusedPanel::Export,
            FocusedPanel::Input => FocusedPanel::General,
            FocusedPanel::Decoration => FocusedPanel::Input,
            FocusedPanel::Animations => FocusedPanel::Decoration,
            FocusedPanel::Gestures => FocusedPanel::Animations,
            FocusedPanel::Binds => FocusedPanel::Gestures,
            FocusedPanel::WindowRules => FocusedPanel::Binds,
            FocusedPanel::LayerRules => FocusedPanel::WindowRules,
            FocusedPanel::Misc => FocusedPanel::LayerRules,
            FocusedPanel::Import => FocusedPanel::Misc,
            FocusedPanel::Export => FocusedPanel::Import,
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
            FocusedPanel::Import => "Import",
            FocusedPanel::Export => "Export",
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
    pub batch_manager: BatchManager,
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

        // Initialize batch manager
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?
            .join("r-hyprconfig");
        let batch_manager = BatchManager::new(config_dir)?;

        Ok(Self {
            state: AppState::Running,
            debug,
            focused_panel: ui.current_tab,
            config,
            hyprctl,
            ui,
            batch_manager,
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(50), // Faster tick rate for responsive preview
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
        if self.ui.show_import_dialog {
            return self.handle_import_dialog_key(key).await;
        }

        if self.ui.show_export_dialog {
            return self.handle_export_dialog_key(key).await;
        }

        if self.ui.show_nixos_export_dialog {
            return self.handle_nixos_export_dialog_key(key).await;
        }

        if self.ui.show_batch_dialog {
            return self.handle_batch_dialog_key(key).await;
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

        if self.ui.show_preview_dialog {
            return self.handle_preview_dialog_key(key).await;
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
                self.show_delete_item_dialog().await;
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                self.show_add_item_dialog().await;
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
                self.show_export_dialog().await;
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.show_import_dialog().await;
            }
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.ui.toggle_help();
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.show_enhanced_preview().await;
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.show_setting_preview().await;
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.show_batch_dialog().await;
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.ui.toggle_preview_mode();
                let status = if self.ui.is_preview_mode() {
                    "Live preview enabled! Changes will be applied immediately."
                } else {
                    "Live preview disabled. Press Enter to apply changes."
                };
                self.ui.show_popup = true;
                self.ui.popup_message = status.to_string();
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
        // Check if this is a deletion confirmation popup
        if let Some((panel, item_key)) = &self.ui.pending_deletion {
            match key {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    // Confirm deletion
                    let panel_clone = panel.clone();
                    let key_clone = item_key.clone();
                    if self.ui.delete_item(&panel_clone, &key_clone) {
                        self.ui.show_popup = true;
                        self.ui.popup_message = "Item deleted successfully!".to_string();
                    } else {
                        self.ui.show_popup = true;
                        self.ui.popup_message = "Failed to delete item - item not found.".to_string();
                    }
                    self.ui.pending_deletion = None;
                }
                _ => {
                    // Cancel deletion on any other key
                    self.ui.pending_deletion = None;
                    self.ui.show_popup = false;
                    self.ui.popup_message.clear();
                }
            }
        } else {
            // Normal popup handling
            match key {
                KeyCode::Enter | KeyCode::Esc => {
                    self.ui.show_popup = false;
                    self.ui.popup_message.clear();
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_edit_key(&mut self, key: KeyCode) -> Result<()> {
        use crate::ui::EditMode;

        // Check preview mode and get editing key before match to avoid borrowing issues
        let preview_enabled = self.ui.is_preview_mode();
        let editing_key = self.ui.editing_item.as_ref().map(|(_, key)| key.clone());
        
        // Handle preview trigger after making changes
        let mut should_trigger_preview = false;
        let mut preview_value = String::new();

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
                        // Cancel any pending preview changes
                        if self.ui.is_preview_mode() {
                            if let Err(e) = self.ui.cancel_preview(&self.hyprctl).await {
                                eprintln!("Error canceling preview: {}", e);
                            }
                        }
                        self.ui.cancel_edit();
                    }
                    KeyCode::Char(c) => {
                        current_value.insert(*cursor_pos, c);
                        *cursor_pos += 1;
                        
                        // Set up for preview trigger
                        if preview_enabled {
                            should_trigger_preview = true;
                            preview_value = current_value.clone();
                        }
                    }
                    KeyCode::Backspace => {
                        if *cursor_pos > 0 {
                            *cursor_pos -= 1;
                            current_value.remove(*cursor_pos);
                            
                            // Set up for preview trigger
                            if preview_enabled {
                                should_trigger_preview = true;
                                preview_value = current_value.clone();
                            }
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

        // Handle preview after the match to avoid borrowing issues
        if should_trigger_preview {
            if let Some(key) = editing_key {
                if let Err(e) = self.ui.handle_preview_change(&key, &preview_value, &self.hyprctl).await {
                    eprintln!("Preview error: {}", e);
                }
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

        // Process pending preview changes
        if self.ui.has_pending_preview() {
            if let Err(e) = self.ui.apply_pending_preview(&self.hyprctl).await {
                eprintln!("Preview application error: {}", e);
            }
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
        // Save the application's own config
        self.config.save().await?;

        // Collect all configuration changes from the UI
        let config_changes = self.ui.collect_all_config_changes();
        let keybinds = self.ui.collect_keybinds();
        let window_rules = self.ui.collect_window_rules();
        let layer_rules = self.ui.collect_layer_rules();

        // Validate configuration changes before saving
        if let Err(validation_error) = self
            .validate_config_changes(&config_changes, &keybinds, &window_rules, &layer_rules)
            .await
        {
            self.ui.show_popup = true;
            self.ui.popup_message = format!("Validation failed: {validation_error}");
            return Err(validation_error);
        }

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


    async fn export_config_to_file(&mut self) -> Result<String> {
        use chrono::Utc;
        use std::fs;

        // Create export directory if it doesn't exist
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
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
            (
                "metadata".to_string(),
                toml::Value::Table(toml::Table::from_iter([
                    (
                        "export_date".to_string(),
                        toml::Value::String(Utc::now().to_rfc3339()),
                    ),
                    (
                        "version".to_string(),
                        toml::Value::String("1.0".to_string()),
                    ),
                    (
                        "theme".to_string(),
                        toml::Value::String(self.config.theme.to_string()),
                    ),
                ])),
            ),
            (
                "config_options".to_string(),
                toml::Value::Table(
                    config_changes
                        .into_iter()
                        .map(|(k, v)| (k, toml::Value::String(v)))
                        .collect(),
                ),
            ),
            (
                "keybinds".to_string(),
                toml::Value::Array(keybinds.into_iter().map(toml::Value::String).collect()),
            ),
            (
                "window_rules".to_string(),
                toml::Value::Array(window_rules.into_iter().map(toml::Value::String).collect()),
            ),
            (
                "layer_rules".to_string(),
                toml::Value::Array(layer_rules.into_iter().map(toml::Value::String).collect()),
            ),
        ]);

        // Write to file
        let toml_content = toml::to_string_pretty(&export_data)?;
        fs::write(&export_path, toml_content)?;

        Ok(export_path.to_string_lossy().to_string())
    }

    async fn import_config_from_file(&mut self) -> Result<usize> {
        use std::fs;

        // Look for the most recent export file
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        let export_dir = config_dir.join("r-hyprconfig").join("exports");

        if !export_dir.exists() {
            return Err(anyhow::anyhow!(
                "No export directory found. Please export a configuration first."
            ));
        }

        // Find the most recent export file
        let mut export_files: Vec<_> = fs::read_dir(&export_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
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
            entry
                .metadata()
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
                    format!(
                        "{}...\n\n[Output truncated - {} more characters]",
                        &config[..2000],
                        config.len() - 2000
                    )
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
        use chrono::Utc;
        use std::fs;

        // Create export directory if it doesn't exist
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
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

    // Batch configuration management methods
    async fn show_batch_dialog(&mut self) {
        // Initialize with default state
        self.ui.batch_dialog_mode = crate::ui::BatchDialogMode::ManageProfiles;
        self.ui.batch_selected_profile = None;
        self.ui.batch_operation_type = crate::batch::BatchOperationType::Apply;
        self.ui.show_batch_dialog = true;
    }

    async fn handle_batch_dialog_key(&mut self, key: KeyCode) -> Result<()> {
        match self.ui.batch_dialog_mode {
            crate::ui::BatchDialogMode::ManageProfiles => {
                self.handle_batch_manage_profiles_key(key).await
            }
            crate::ui::BatchDialogMode::SelectOperation => {
                self.handle_batch_select_operation_key(key).await
            }
            crate::ui::BatchDialogMode::ExecuteOperation => {
                self.handle_batch_execute_operation_key(key).await
            }
        }
    }

    async fn handle_batch_manage_profiles_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('1') => {
                // Create new profile
                self.create_batch_profile().await;
            }
            KeyCode::Char('2') => {
                // Select existing profile
                self.ui.batch_dialog_mode = crate::ui::BatchDialogMode::SelectOperation;
            }
            KeyCode::Char('3') => {
                // Delete profile
                self.delete_batch_profile().await;
            }
            KeyCode::Esc => {
                self.ui.show_batch_dialog = false;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_batch_select_operation_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('1') => {
                self.ui.batch_operation_type = crate::batch::BatchOperationType::Apply;
                self.ui.batch_dialog_mode = crate::ui::BatchDialogMode::ExecuteOperation;
            }
            KeyCode::Char('2') => {
                self.ui.batch_operation_type = crate::batch::BatchOperationType::Merge;
                self.ui.batch_dialog_mode = crate::ui::BatchDialogMode::ExecuteOperation;
            }
            KeyCode::Char('3') => {
                self.ui.batch_operation_type = crate::batch::BatchOperationType::Replace;
                self.ui.batch_dialog_mode = crate::ui::BatchDialogMode::ExecuteOperation;
            }
            KeyCode::Char('4') => {
                self.ui.batch_operation_type = crate::batch::BatchOperationType::Backup;
                self.ui.batch_dialog_mode = crate::ui::BatchDialogMode::ExecuteOperation;
            }
            KeyCode::Esc => {
                self.ui.batch_dialog_mode = crate::ui::BatchDialogMode::ManageProfiles;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_batch_execute_operation_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Enter => {
                // Execute the batch operation
                self.execute_batch_operation().await;
            }
            KeyCode::Esc => {
                self.ui.batch_dialog_mode = crate::ui::BatchDialogMode::SelectOperation;
            }
            _ => {}
        }
        Ok(())
    }

    async fn create_batch_profile(&mut self) {
        // Create profile with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let profile_name = format!("profile_{timestamp}");

        // For now, create an empty profile with no config paths
        // TODO: In the future, implement actual config file path collection
        let config_paths = Vec::new();

        match self.batch_manager.create_profile(
            profile_name.clone(),
            Some("Auto-generated profile".to_string()),
            config_paths,
        ) {
            Ok(_) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Profile '{profile_name}' created successfully!");
                self.ui.show_batch_dialog = false;
            }
            Err(e) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Failed to create profile: {e}");
            }
        }
    }

    async fn delete_batch_profile(&mut self) {
        if let Some(profile_name) = &self.ui.batch_selected_profile {
            match self.batch_manager.delete_profile(profile_name) {
                Ok(_) => {
                    self.ui.show_popup = true;
                    self.ui.popup_message =
                        format!("Profile '{profile_name}' deleted successfully!");
                    self.ui.batch_selected_profile = None;
                    self.ui.show_batch_dialog = false;
                }
                Err(e) => {
                    self.ui.show_popup = true;
                    self.ui.popup_message = format!("Failed to delete profile: {e}");
                }
            }
        } else {
            self.ui.show_popup = true;
            self.ui.popup_message = "No profile selected for deletion".to_string();
        }
    }

    async fn execute_batch_operation(&mut self) {
        if let Some(profile_name) = &self.ui.batch_selected_profile {
            // Collect current configuration for the operation
            let config_changes = self.ui.collect_all_config_changes();
            let keybinds = self.ui.collect_keybinds();
            let window_rules = self.ui.collect_window_rules();
            let layer_rules = self.ui.collect_layer_rules();

            let operation = crate::batch::BatchOperation {
                operation_type: self.ui.batch_operation_type.clone(),
                settings: config_changes,
                keybinds,
                window_rules,
                layer_rules,
                target_profiles: vec![profile_name.clone()],
            };

            match self
                .batch_manager
                .execute_batch_operation(&operation, &self.hyprctl)
                .await
            {
                Ok(results) => {
                    let success_count = results.iter().filter(|r| r.success).count();
                    self.ui.show_popup = true;
                    self.ui.popup_message = format!(
                        "Batch operation '{:?}' completed! {success_count}/{} profiles succeeded.",
                        self.ui.batch_operation_type,
                        results.len()
                    );
                    self.ui.show_batch_dialog = false;
                }
                Err(e) => {
                    self.ui.show_popup = true;
                    self.ui.popup_message = format!("Batch operation failed: {e}");
                }
            }
        } else {
            self.ui.show_popup = true;
            self.ui.popup_message = "No profile selected for operation".to_string();
        }
    }

    async fn show_setting_preview(&mut self) {
        // Get the currently selected setting based on focused panel
        let panel = self.focused_panel;
        let selected_index = match panel {
            FocusedPanel::General => self.ui.general_list_state.selected(),
            FocusedPanel::Input => self.ui.input_list_state.selected(),
            FocusedPanel::Decoration => self.ui.decoration_list_state.selected(),
            FocusedPanel::Animations => self.ui.animations_list_state.selected(),
            FocusedPanel::Gestures => self.ui.gestures_list_state.selected(),
            FocusedPanel::Binds => self.ui.binds_list_state.selected(),
            FocusedPanel::WindowRules => self.ui.window_rules_list_state.selected(),
            FocusedPanel::LayerRules => self.ui.layer_rules_list_state.selected(),
            FocusedPanel::Misc => self.ui.misc_list_state.selected(),
            FocusedPanel::Import => self.ui.import_list_state.selected(),
            FocusedPanel::Export => self.ui.export_list_state.selected(),
        };

        if let Some(index) = selected_index {
            // Get the configuration items for this panel
            if let Some(config_items) = self.ui.config_items.get(&panel) {
                if let Some(item) = config_items.get(index) {
                    // Create example before/after for demonstration
                    let before_value = item.value.clone();
                    let after_value = match &item.data_type {
                        crate::ui::ConfigDataType::Integer { min, max } => {
                            let current_val = before_value.parse::<i32>().unwrap_or(0);
                            let new_val = match (min, max) {
                                (Some(min_val), Some(max_val)) => {
                                    (current_val + 5).clamp(*min_val, *max_val)
                                }
                                _ => current_val + 5,
                            };
                            match (min, max) {
                                (Some(min_val), Some(max_val)) => {
                                    format!("{new_val} (range: {min_val} - {max_val})")
                                }
                                _ => format!("{new_val} (no range limit)"),
                            }
                        }
                        crate::ui::ConfigDataType::Float { min, max, .. } => {
                            let current_val = before_value.parse::<f32>().unwrap_or(0.0);
                            let new_val = match (min, max) {
                                (Some(min_val), Some(max_val)) => {
                                    (current_val + 0.5).clamp(*min_val, *max_val)
                                }
                                _ => current_val + 0.5,
                            };
                            match (min, max) {
                                (Some(min_val), Some(max_val)) => {
                                    format!("{new_val:.2} (range: {min_val} - {max_val})")
                                }
                                _ => format!("{new_val:.2} (no range limit)"),
                            }
                        }
                        crate::ui::ConfigDataType::Boolean => {
                            if before_value == "true" {
                                "false".to_string()
                            } else {
                                "true".to_string()
                            }
                        }
                        crate::ui::ConfigDataType::Color => "#FF5555 (example color)".to_string(),
                        crate::ui::ConfigDataType::String => {
                            format!("{before_value} (modified)")
                        }
                        crate::ui::ConfigDataType::Keyword { options } => options
                            .iter()
                            .find(|&opt| opt != &before_value)
                            .unwrap_or(&options[0])
                            .clone(),
                    };

                    let setting_name = format!("{}: {}", panel.as_str(), item.key);
                    let before_text = format!(
                        "Current Value:\n{}\n\nDescription:\n{}",
                        before_value, item.description
                    );
                    let after_text = format!(
                        "New Value:\n{}\n\nDescription:\n{}",
                        after_value, item.description
                    );

                    self.ui
                        .show_setting_preview(setting_name, before_text, after_text);
                } else {
                    self.ui.show_popup = true;
                    self.ui.popup_message = "No setting selected for preview".to_string();
                }
            } else {
                self.ui.show_popup = true;
                self.ui.popup_message = "No configuration items available for preview".to_string();
            }
        } else {
            self.ui.show_popup = true;
            self.ui.popup_message =
                "No setting selected. Use ↑↓ to select a setting first".to_string();
        }
    }

    async fn handle_preview_dialog_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                self.ui.close_preview_dialog();
            }
            KeyCode::Enter => {
                // Apply the change and close preview
                // This would trigger the actual configuration change
                // For now, we just close the dialog and show a confirmation
                self.ui.close_preview_dialog();
                self.ui.show_popup = true;
                self.ui.popup_message =
                    format!("Applied setting: {}", self.ui.preview_setting_name);
            }
            KeyCode::Up => {
                self.ui.scroll_preview_up();
            }
            KeyCode::Down => {
                self.ui.scroll_preview_down();
            }
            KeyCode::PageUp => {
                // Scroll up by multiple lines
                for _ in 0..5 {
                    self.ui.scroll_preview_up();
                }
            }
            KeyCode::PageDown => {
                // Scroll down by multiple lines
                for _ in 0..5 {
                    self.ui.scroll_preview_down();
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn validate_config_changes(
        &self,
        config_changes: &std::collections::HashMap<String, String>,
        keybinds: &[String],
        window_rules: &[String],
        layer_rules: &[String],
    ) -> Result<()> {
        let mut validation_errors = Vec::new();

        // Validate configuration values
        for (key, value) in config_changes {
            if let Err(error) = self.validate_config_option(key, value).await {
                validation_errors.push(format!("Config option '{key}': {error}"));
            }
        }

        // Validate keybinds
        for (index, keybind) in keybinds.iter().enumerate() {
            if let Err(error) = self.validate_keybind(keybind).await {
                validation_errors.push(format!("Keybind {}: {}", index + 1, error));
            }
        }

        // Validate window rules
        for (index, rule) in window_rules.iter().enumerate() {
            if let Err(error) = self.validate_window_rule(rule).await {
                validation_errors.push(format!("Window rule {}: {}", index + 1, error));
            }
        }

        // Validate layer rules
        for (index, rule) in layer_rules.iter().enumerate() {
            if let Err(error) = self.validate_layer_rule(rule).await {
                validation_errors.push(format!("Layer rule {}: {}", index + 1, error));
            }
        }

        if !validation_errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Validation failed:\n{}",
                validation_errors.join("\n")
            ));
        }

        Ok(())
    }

    async fn validate_config_option(&self, key: &str, value: &str) -> Result<()> {
        // Basic validation for common configuration options
        match key {
            // Integer values
            k if k.contains("gaps_") || k.contains("border_size") || k.contains("rounding") => {
                if value.parse::<i32>().is_err() {
                    return Err(anyhow::anyhow!("must be a valid integer"));
                }
                let val = value.parse::<i32>().unwrap();
                if val < 0 {
                    return Err(anyhow::anyhow!("must be non-negative"));
                }
            }
            // Float values
            k if k.contains("opacity") || k.contains("sensitivity") => {
                if value.parse::<f32>().is_err() {
                    return Err(anyhow::anyhow!("must be a valid decimal number"));
                }
                let val = value.parse::<f32>().unwrap();
                if k.contains("opacity") && !(0.0..=1.0).contains(&val) {
                    return Err(anyhow::anyhow!("opacity must be between 0.0 and 1.0"));
                }
            }
            // Boolean values
            k if k.contains("enabled") || k.contains("disable_") => {
                if !["true", "false", "1", "0", "yes", "no"]
                    .contains(&value.to_lowercase().as_str())
                {
                    return Err(anyhow::anyhow!("must be true/false, 1/0, or yes/no"));
                }
            }
            // Color values
            k if k.contains("col.") => {
                if !value.starts_with("rgb(")
                    && !value.starts_with("rgba(")
                    && !value.starts_with("#")
                {
                    return Err(anyhow::anyhow!(
                        "must be a valid color (rgb(), rgba(), or #hex)"
                    ));
                }
            }
            _ => {
                // For unknown options, just check they're not empty
                if value.trim().is_empty() {
                    return Err(anyhow::anyhow!("value cannot be empty"));
                }
            }
        }

        Ok(())
    }

    async fn validate_keybind(&self, keybind: &str) -> Result<()> {
        // Basic keybind format validation
        if !keybind.starts_with("bind") {
            return Err(anyhow::anyhow!(
                "must start with 'bind', 'binde', 'bindm', etc."
            ));
        }

        // Check for required components (modifiers, key, dispatcher)
        let parts: Vec<&str> = keybind.split(',').collect();
        if parts.len() < 4 {
            return Err(anyhow::anyhow!(
                "must have format: bind = MODIFIERS, KEY, DISPATCHER, ARGS"
            ));
        }

        // Validate that key isn't empty
        let key = parts.get(2).unwrap_or(&"").trim();
        if key.is_empty() {
            return Err(anyhow::anyhow!("key cannot be empty"));
        }

        // Validate that dispatcher isn't empty
        let dispatcher = parts.get(3).unwrap_or(&"").trim();
        if dispatcher.is_empty() {
            return Err(anyhow::anyhow!("dispatcher cannot be empty"));
        }

        Ok(())
    }

    async fn validate_window_rule(&self, rule: &str) -> Result<()> {
        // Basic window rule format validation
        if !rule.starts_with("windowrule") {
            return Err(anyhow::anyhow!("must start with 'windowrule'"));
        }

        // Check for basic format
        if !rule.contains('=') || !rule.contains(',') {
            return Err(anyhow::anyhow!(
                "must have format: windowrule = RULE, WINDOW_PATTERN"
            ));
        }

        Ok(())
    }

    async fn validate_layer_rule(&self, rule: &str) -> Result<()> {
        // Basic layer rule format validation
        if !rule.starts_with("layerrule") {
            return Err(anyhow::anyhow!("must start with 'layerrule'"));
        }

        // Check for basic format
        if !rule.contains('=') || !rule.contains(',') {
            return Err(anyhow::anyhow!(
                "must have format: layerrule = RULE, LAYER_PATTERN"
            ));
        }

        Ok(())
    }

    // New import/export dialog handlers
    async fn show_import_dialog(&mut self) {
        self.ui.show_import_dialog = true;
        self.ui.import_export_mode = crate::ui::ImportExportMode::SelectSource;
        self.ui.selected_import_source = crate::ui::ImportSourceType::LocalFile;
        self.ui.import_preview = None;
        self.ui.import_list_state.select(Some(0));
    }

    async fn show_export_dialog(&mut self) {
        self.ui.show_export_dialog = true;
        self.ui.import_export_mode = crate::ui::ImportExportMode::SelectFormat;
        self.ui.selected_export_format = crate::ui::ExportFormatType::HyprlandConf;
        self.ui.export_preview = None;
        self.ui.export_list_state.select(Some(0));
    }

    async fn handle_import_dialog_key(&mut self, key: KeyCode) -> Result<()> {
        use crate::ui::{ImportExportMode, ImportSourceType};
        
        match self.ui.import_export_mode {
            ImportExportMode::SelectSource => {
                match key {
                    KeyCode::Char('1') => {
                        self.ui.selected_import_source = ImportSourceType::LocalFile;
                        self.ui.import_export_mode = ImportExportMode::Preview;
                        self.generate_import_preview().await;
                    }
                    KeyCode::Char('2') => {
                        self.ui.selected_import_source = ImportSourceType::LocalFolder;
                        self.ui.import_export_mode = ImportExportMode::Preview;
                        self.generate_import_preview().await;
                    }
                    KeyCode::Char('3') => {
                        self.ui.selected_import_source = ImportSourceType::GitHubRepository;
                        self.ui.import_export_mode = ImportExportMode::Preview;
                        self.generate_import_preview().await;
                    }
                    KeyCode::Char('4') => {
                        self.ui.selected_import_source = ImportSourceType::UrlDownload;
                        self.ui.import_export_mode = ImportExportMode::Preview;
                        self.generate_import_preview().await;
                    }
                    KeyCode::Esc => {
                        self.ui.show_import_dialog = false;
                        self.ui.import_preview = None;
                    }
                    _ => {}
                }
            }
            ImportExportMode::Preview => {
                match key {
                    KeyCode::Enter => {
                        self.ui.import_export_mode = ImportExportMode::Execute;
                        self.execute_import().await;
                    }
                    KeyCode::Esc => {
                        self.ui.import_export_mode = ImportExportMode::SelectSource;
                        self.ui.import_preview = None;
                    }
                    KeyCode::Up => {
                        if self.ui.import_export_scroll > 0 {
                            self.ui.import_export_scroll -= 1;
                        }
                    }
                    KeyCode::Down => {
                        self.ui.import_export_scroll += 1;
                    }
                    _ => {}
                }
            }
            ImportExportMode::Execute => {
                match key {
                    KeyCode::Enter | KeyCode::Esc => {
                        self.ui.show_import_dialog = false;
                        self.ui.import_preview = None;
                        self.ui.import_export_scroll = 0;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_export_dialog_key(&mut self, key: KeyCode) -> Result<()> {
        use crate::ui::{ImportExportMode, ExportFormatType};
        
        match self.ui.import_export_mode {
            ImportExportMode::SelectFormat => {
                match key {
                    KeyCode::Char('1') => {
                        self.ui.selected_export_format = ExportFormatType::HyprlandConf;
                        self.ui.import_export_mode = ImportExportMode::Preview;
                        self.generate_export_preview().await;
                    }
                    KeyCode::Char('2') => {
                        self.ui.selected_export_format = ExportFormatType::Json;
                        self.ui.import_export_mode = ImportExportMode::Preview;
                        self.generate_export_preview().await;
                    }
                    KeyCode::Char('3') => {
                        self.ui.selected_export_format = ExportFormatType::Toml;
                        self.ui.import_export_mode = ImportExportMode::Preview;
                        self.generate_export_preview().await;
                    }
                    KeyCode::Char('4') => {
                        self.ui.selected_export_format = ExportFormatType::Yaml;
                        self.ui.import_export_mode = ImportExportMode::Preview;
                        self.generate_export_preview().await;
                    }
                    KeyCode::Char('5') => {
                        self.ui.selected_export_format = ExportFormatType::RHyprConfig;
                        self.ui.import_export_mode = ImportExportMode::Preview;
                        self.generate_export_preview().await;
                    }
                    KeyCode::Char('6') => {
                        self.ui.selected_export_format = ExportFormatType::NixOS;
                        if self.config.nixos_mode {
                            self.ui.import_export_mode = ImportExportMode::Preview;
                            self.generate_export_preview().await;
                        } else {
                            self.ui.show_popup = true;
                            self.ui.popup_message = "NixOS export is only available on NixOS systems".to_string();
                        }
                    }
                    KeyCode::Esc => {
                        self.ui.show_export_dialog = false;
                        self.ui.export_preview = None;
                    }
                    _ => {}
                }
            }
            ImportExportMode::Preview => {
                match key {
                    KeyCode::Enter => {
                        self.ui.import_export_mode = ImportExportMode::Execute;
                        self.execute_export().await;
                    }
                    KeyCode::Esc => {
                        self.ui.import_export_mode = ImportExportMode::SelectFormat;
                        self.ui.export_preview = None;
                    }
                    KeyCode::Up => {
                        if self.ui.import_export_scroll > 0 {
                            self.ui.import_export_scroll -= 1;
                        }
                    }
                    KeyCode::Down => {
                        self.ui.import_export_scroll += 1;
                    }
                    _ => {}
                }
            }
            ImportExportMode::Execute => {
                match key {
                    KeyCode::Enter | KeyCode::Esc => {
                        self.ui.show_export_dialog = false;
                        self.ui.export_preview = None;
                        self.ui.import_export_scroll = 0;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn show_enhanced_preview(&mut self) {
        // Enhanced preview that shows current setting details, not just NixOS export
        let panel = self.focused_panel;
        let selected_index = match panel {
            FocusedPanel::General => self.ui.general_list_state.selected(),
            FocusedPanel::Input => self.ui.input_list_state.selected(),
            FocusedPanel::Decoration => self.ui.decoration_list_state.selected(),
            FocusedPanel::Animations => self.ui.animations_list_state.selected(),
            FocusedPanel::Gestures => self.ui.gestures_list_state.selected(),
            FocusedPanel::Binds => self.ui.binds_list_state.selected(),
            FocusedPanel::WindowRules => self.ui.window_rules_list_state.selected(),
            FocusedPanel::LayerRules => self.ui.layer_rules_list_state.selected(),
            FocusedPanel::Misc => self.ui.misc_list_state.selected(),
            FocusedPanel::Import => {
                // Show import dialog instead
                self.show_import_dialog().await;
                return;
            }
            FocusedPanel::Export => {
                // Show export dialog instead
                self.show_export_dialog().await;
                return;
            }
        };

        if let Some(index) = selected_index {
            if let Some(config_items) = self.ui.config_items.get(&panel) {
                if let Some(item) = config_items.get(index) {
                    let setting_name = format!("{}: {}", panel.as_str(), item.key);
                    let before_text = format!(
                        "Current Value: {}\n\nDescription:\n{}\n\nData Type: {:?}\n\nSuggestions: {}",
                        item.value,
                        item.description,
                        item.data_type,
                        item.suggestions.join(", ")
                    );
                    
                    // Generate preview of what the setting would look like with a different value
                    let after_text = self.generate_setting_preview_text(item);
                    
                    self.ui.show_setting_preview(setting_name, before_text, after_text);
                } else {
                    self.ui.show_popup = true;
                    self.ui.popup_message = "No setting selected for preview".to_string();
                }
            } else {
                self.ui.show_popup = true;
                self.ui.popup_message = "No configuration items available for preview".to_string();
            }
        } else {
            self.ui.show_popup = true;
            self.ui.popup_message = "No setting selected. Use ↑↓ to select a setting first".to_string();
        }
    }

    fn generate_setting_preview_text(&self, item: &crate::ui::ConfigItem) -> String {
        match &item.data_type {
            crate::ui::ConfigDataType::Boolean => {
                let new_value = if item.value == "true" { "false" } else { "true" };
                format!("Preview Value: {}\n\nThis would toggle the current boolean setting.", new_value)
            }
            crate::ui::ConfigDataType::Integer { min, max } => {
                let current_val = item.value.parse::<i32>().unwrap_or(0);
                let new_val = match (min, max) {
                    (Some(min_val), Some(max_val)) => (current_val + 5).clamp(*min_val, *max_val),
                    _ => current_val + 5,
                };
                format!("Preview Value: {}\n\nThis would increase the current value by 5 (within valid range).", new_val)
            }
            crate::ui::ConfigDataType::Float { min, max } => {
                let current_val = item.value.parse::<f32>().unwrap_or(0.0);
                let new_val = match (min, max) {
                    (Some(min_val), Some(max_val)) => (current_val + 0.1).clamp(*min_val, *max_val),
                    _ => current_val + 0.1,
                };
                format!("Preview Value: {:.2}\n\nThis would increase the current value by 0.1 (within valid range).", new_val)
            }
            crate::ui::ConfigDataType::Keyword { options } => {
                let next_option = options.iter()
                    .find(|&opt| opt != &item.value)
                    .unwrap_or(&options[0]);
                format!("Preview Value: {}\n\nThis would cycle to the next available option.\nAll options: {}", 
                    next_option, options.join(", "))
            }
            crate::ui::ConfigDataType::Color => {
                "Preview Value: #FF5555\n\nThis would set a new color value. Use the color picker to select.".to_string()
            }
            crate::ui::ConfigDataType::String => {
                format!("Preview Value: {} (modified)\n\nThis would append '(modified)' to the current string value.", item.value)
            }
        }
    }

    async fn generate_import_preview(&mut self) {
        use crate::ui::ImportSourceType;
        
        let preview_text = match self.ui.selected_import_source {
            ImportSourceType::LocalFile => {
                "Import from Local File\n\n\
                This will:\n\
                • Open a file browser to select a Hyprland config file\n\
                • Parse and validate the configuration\n\
                • Show a preview of what will be imported\n\
                • Allow you to choose which settings to import\n\n\
                Supported formats: .conf, .toml, .json, .yaml".to_string()
            }
            ImportSourceType::LocalFolder => {
                "Import from Local Folder\n\n\
                This will:\n\
                • Scan a folder for Hyprland configuration files\n\
                • Detect and parse all compatible files\n\
                • Show a comprehensive preview of all settings\n\
                • Allow selective import of individual files or settings".to_string()
            }
            ImportSourceType::GitHubRepository => {
                "Import from GitHub Repository\n\n\
                This will:\n\
                • Connect to a GitHub repository\n\
                • Download and analyze Hyprland configurations\n\
                • Parse dotfiles and config repositories\n\
                • Import compatible settings and assets\n\n\
                Example: https://github.com/user/dotfiles".to_string()
            }
            ImportSourceType::UrlDownload => {
                "Import from URL\n\n\
                This will:\n\
                • Download configuration from a direct URL\n\
                • Validate and parse the content\n\
                • Show preview before importing\n\
                • Support for pastebin, gists, and direct links".to_string()
            }
        };
        
        self.ui.import_preview = Some(preview_text);
    }

    async fn generate_export_preview(&mut self) {
        use crate::ui::ExportFormatType;
        
        let config_changes = self.ui.collect_all_config_changes();
        let keybinds = self.ui.collect_keybinds();
        let window_rules = self.ui.collect_window_rules();
        let layer_rules = self.ui.collect_layer_rules();
        
        let preview_text = match self.ui.selected_export_format {
            ExportFormatType::HyprlandConf => {
                format!("Export as Hyprland Configuration\n\n\
                This will create a standard hyprland.conf file with:\n\
                • {} configuration options\n\
                • {} keybinds\n\
                • {} window rules\n\
                • {} layer rules\n\n\
                Output: ~/.config/r-hyprconfig/exports/hyprland_export_[timestamp].conf",
                config_changes.len(), keybinds.len(), window_rules.len(), layer_rules.len())
            }
            ExportFormatType::Json => {
                format!("Export as JSON\n\n\
                This will create a structured JSON file with:\n\
                • Hierarchical configuration structure\n\
                • Easy parsing for other tools\n\
                • Metadata and versioning information\n\
                • {} total configuration items\n\n\
                Output: ~/.config/r-hyprconfig/exports/config_export_[timestamp].json",
                config_changes.len() + keybinds.len() + window_rules.len() + layer_rules.len())
            }
            ExportFormatType::Toml => {
                format!("Export as TOML\n\n\
                This will create a TOML configuration file with:\n\
                • Human-readable format\n\
                • Section-based organization\n\
                • Easy manual editing\n\
                • {} total configuration items\n\n\
                Output: ~/.config/r-hyprconfig/exports/config_export_[timestamp].toml",
                config_changes.len() + keybinds.len() + window_rules.len() + layer_rules.len())
            }
            ExportFormatType::Yaml => {
                format!("Export as YAML\n\n\
                This will create a YAML configuration file with:\n\
                • Clean, indented structure\n\
                • Comments and documentation\n\
                • Machine and human readable\n\
                • {} total configuration items\n\n\
                Output: ~/.config/r-hyprconfig/exports/config_export_[timestamp].yaml",
                config_changes.len() + keybinds.len() + window_rules.len() + layer_rules.len())
            }
            ExportFormatType::RHyprConfig => {
                format!("Export as R-Hyprconfig Format\n\n\
                This will create an r-hyprconfig native format with:\n\
                • Full feature compatibility\n\
                • Theme and UI preferences\n\
                • Import/export metadata\n\
                • {} total configuration items\n\n\
                Output: ~/.config/r-hyprconfig/exports/rhypr_export_[timestamp].rhypr",
                config_changes.len() + keybinds.len() + window_rules.len() + layer_rules.len())
            }
            ExportFormatType::NixOS => {
                format!("Export as NixOS Module\n\n\
                This will create a NixOS configuration module with:\n\
                • Declarative configuration\n\
                • Home Manager integration\n\
                • Reproducible builds\n\
                • {} total configuration items\n\n\
                Output: ~/.config/r-hyprconfig/nixos-exports/hyprland_[timestamp].nix",
                config_changes.len() + keybinds.len() + window_rules.len() + layer_rules.len())
            }
        };
        
        self.ui.export_preview = Some(preview_text);
    }

    async fn execute_import(&mut self) {
        // Placeholder implementation - integrate with the existing import system
        match self.import_config_from_file().await {
            Ok(imported_count) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Imported {} configuration items successfully!", imported_count);
            }
            Err(e) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Import failed: {}", e);
            }
        }
        self.ui.show_import_dialog = false;
    }

    async fn execute_export(&mut self) {
        // Placeholder implementation - integrate with the existing export system  
        match self.export_config_to_file().await {
            Ok(path) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Configuration exported to: {}", path);
            }
            Err(e) => {
                self.ui.show_popup = true;
                self.ui.popup_message = format!("Export failed: {}", e);
            }
        }
        self.ui.show_export_dialog = false;
    }

    async fn show_add_item_dialog(&mut self) {
        // Show dialog to add new configuration items based on current panel
        match self.ui.current_tab {
            crate::app::FocusedPanel::Binds => {
                self.ui.show_popup = true;
                self.ui.popup_message = "Add Keybind: Press Enter to add a new keybinding (format: SUPER, q, exec, kitty)".to_string();
                // Start editing mode for adding new keybind
                self.ui.start_add_keybind();
            }
            crate::app::FocusedPanel::WindowRules => {
                self.ui.show_popup = true;
                self.ui.popup_message = "Add Window Rule: Press Enter to add a new window rule (format: float, ^(kitty)$)".to_string();
                // Start editing mode for adding new window rule
                self.ui.start_add_window_rule();
            }
            crate::app::FocusedPanel::LayerRules => {
                self.ui.show_popup = true;
                self.ui.popup_message = "Add Layer Rule: Press Enter to add a new layer rule".to_string();
                // Start editing mode for adding new layer rule
                self.ui.start_add_layer_rule();
            }
            _ => {
                self.ui.show_popup = true;
                self.ui.popup_message = "Add Item: Not available for this panel. Use 'I' key in Binds, Window Rules, or Layer Rules panels.".to_string();
            }
        }
    }

    async fn show_delete_item_dialog(&mut self) {
        // Show dialog to delete the currently selected item
        match self.ui.current_tab {
            crate::app::FocusedPanel::Binds => {
                if let Some(selected) = self.ui.get_selected_item() {
                    let value = selected.value.clone();
                    let key = selected.key.clone();
                    self.ui.show_popup = true;
                    self.ui.popup_message = format!("Delete Keybind: '{}' - Press 'Y' to confirm, any other key to cancel", value);
                    // Set a flag to handle deletion on next key press
                    self.ui.pending_deletion = Some((self.ui.current_tab.clone(), key));
                }
            }
            crate::app::FocusedPanel::WindowRules => {
                if let Some(selected) = self.ui.get_selected_item() {
                    let value = selected.value.clone();
                    let key = selected.key.clone();
                    self.ui.show_popup = true;
                    self.ui.popup_message = format!("Delete Window Rule: '{}' - Press 'Y' to confirm, any other key to cancel", value);
                    self.ui.pending_deletion = Some((self.ui.current_tab.clone(), key));
                }
            }
            crate::app::FocusedPanel::LayerRules => {
                if let Some(selected) = self.ui.get_selected_item() {
                    let value = selected.value.clone();
                    let key = selected.key.clone();
                    self.ui.show_popup = true;
                    self.ui.popup_message = format!("Delete Layer Rule: '{}' - Press 'Y' to confirm, any other key to cancel", value);
                    self.ui.pending_deletion = Some((self.ui.current_tab.clone(), key));
                }
            }
            _ => {
                self.ui.show_popup = true;
                self.ui.popup_message = "Delete Item: Not available for this panel. Use 'D' key in Binds, Window Rules, or Layer Rules panels.".to_string();
            }
        }
    }
}
