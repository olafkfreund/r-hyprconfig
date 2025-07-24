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
    pub async fn new(debug: bool) -> Result<Self> {
        let config = Config::load().await?;
        let hyprctl = HyprCtl::new().await?;
        let mut ui = UI::new();
        
        // Load current configuration from hyprctl
        if let Err(e) = ui.load_current_config(&hyprctl).await {
            eprintln!("Warning: Failed to load current Hyprland configuration: {}", e);
            eprintln!("Using default values. Make sure Hyprland is running.");
        }

        Ok(Self {
            state: AppState::Running,
            debug,
            focused_panel: FocusedPanel::General,
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
        if self.ui.show_save_dialog {
            return self.handle_save_dialog_key(key).await;
        }
        
        if self.ui.show_reload_dialog {
            return self.handle_reload_dialog_key(key).await;
        }
        
        if self.ui.show_popup {
            return self.handle_popup_key(key).await;
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
                self.focused_panel = self.focused_panel.next();
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.focused_panel = self.focused_panel.previous();
            }
            KeyCode::Up => {
                self.ui.scroll_up(self.focused_panel);
            }
            KeyCode::Down => {
                self.ui.scroll_down(self.focused_panel);
            }
            KeyCode::Enter => {
                self.ui.start_editing(self.focused_panel).await?;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.ui.show_reload_dialog = true;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.ui.show_save_dialog = true;
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
            EditMode::Text { current_value, cursor_pos } => {
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
            EditMode::Slider { current_value, min, max, step } => {
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
            EditMode::None => {
                // This shouldn't happen, but handle it gracefully
                self.ui.cancel_edit();
            }
        }
        Ok(())
    }

    async fn tick(&mut self) {
        // Update any time-based UI elements here
    }

    async fn reload_config(&mut self) -> Result<()> {
        self.config = Config::load().await?;
        
        // Also reload current configuration from hyprctl
        if let Err(e) = self.ui.load_current_config(&self.hyprctl).await {
            eprintln!("Warning: Failed to reload Hyprland configuration: {}", e);
        }
        
        Ok(())
    }

    async fn save_config(&mut self) -> Result<()> {
        self.config.save().await?;
        Ok(())
    }
}