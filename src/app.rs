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
        let ui = UI::new();

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
                self.ui.toggle_selection(self.focused_panel).await?;
            }
            KeyCode::Char('r') => {
                self.reload_config().await?;
            }
            KeyCode::Char('s') => {
                self.save_config().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn tick(&mut self) {
        // Update any time-based UI elements here
    }

    async fn reload_config(&mut self) -> Result<()> {
        self.config = Config::load().await?;
        Ok(())
    }

    async fn save_config(&mut self) -> Result<()> {
        self.config.save().await?;
        Ok(())
    }
}