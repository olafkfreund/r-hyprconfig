// Command pattern for event handling
// Replaces large monolithic event handling functions with composable commands

use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::KeyCode;
use std::fmt::Debug;

use crate::{
    app::{App, AppState, FocusedPanel},
    state::StateManager,
    ui::EditMode,
};

/// Base trait for all commands
#[async_trait::async_trait]
pub trait Command: Debug + Send + Sync {
    /// Execute the command
    async fn execute(&self, app: &mut App, _context: &CommandContext) -> Result<CommandResult>;

    /// Check if this command can handle the given context
    fn can_handle(&self, _context: &CommandContext) -> bool;

    /// Get command priority (higher numbers execute first)
    fn priority(&self) -> u8 {
        0
    }

    /// Get command description for debugging
    fn description(&self) -> &'static str;
}

/// Result of command execution
#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    /// Command was executed successfully
    Handled,
    /// Command was not applicable, try next command
    NotHandled,
    /// Command was handled but should continue to next command
    Continue,
    /// Command was handled and no further commands should be processed
    Stop,
}

/// Context information for command execution
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub key: KeyCode,
    pub has_popup: bool,
    pub has_dialog: bool,
    pub in_search_mode: bool,
    pub in_edit_mode: bool,
    pub current_panel: FocusedPanel,
    pub show_help: bool,
    pub show_preview_dialog: bool,
    pub show_import_dialog: bool,
    pub show_export_dialog: bool,
    pub show_nixos_export_dialog: bool,
    pub show_batch_dialog: bool,
    pub show_save_dialog: bool,
    pub show_reload_dialog: bool,
}

impl CommandContext {
    /// Create context from current app state
    pub fn from_app(app: &App, key: KeyCode) -> Self {
        Self {
            key,
            has_popup: app.ui.show_popup,
            has_dialog: app.ui.show_import_dialog 
                || app.ui.show_export_dialog
                || app.ui.show_nixos_export_dialog
                || app.ui.show_batch_dialog
                || app.ui.show_save_dialog
                || app.ui.show_reload_dialog
                || app.ui.show_preview_dialog,
            in_search_mode: app.ui.search_mode,
            in_edit_mode: app.ui.edit_mode != EditMode::None,
            current_panel: app.focused_panel,
            show_help: app.ui.show_help,
            show_preview_dialog: app.ui.show_preview_dialog,
            show_import_dialog: app.ui.show_import_dialog,
            show_export_dialog: app.ui.show_export_dialog,
            show_nixos_export_dialog: app.ui.show_nixos_export_dialog,
            show_batch_dialog: app.ui.show_batch_dialog,
            show_save_dialog: app.ui.show_save_dialog,
            show_reload_dialog: app.ui.show_reload_dialog,
        }
    }

    /// Check if any modal is currently open
    pub fn has_modal_open(&self) -> bool {
        self.has_popup || self.has_dialog || self.show_help || self.in_edit_mode
    }
}

/// Command dispatcher that manages and executes commands
#[derive(Debug)]
pub struct CommandDispatcher {
    commands: Vec<Box<dyn Command>>,
}

impl CommandDispatcher {
    pub fn new() -> Self {
        let mut dispatcher = Self {
            commands: Vec::new(),
        };
        
        // Register all commands in priority order
        dispatcher.register_default_commands();
        dispatcher
    }

    /// Register a command
    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
        // Sort by priority (highest first)
        self.commands.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Execute the first applicable command
    pub async fn dispatch(app: &mut App, key: KeyCode) -> Result<CommandResult> {
        // Create a simple hardcoded dispatch to avoid borrow checker issues
        // This can be optimized later with a different pattern
        
        let context = CommandContext::from_app(app, key);
        
        // High priority commands first
        if context.has_popup {
            return app.handle_popup_key(key).await.map(|_| CommandResult::Handled);
        }
        
        if context.show_help {
            return app.handle_help_key(key).await.map(|_| CommandResult::Handled);
        }
        
        if context.show_import_dialog {
            return app.handle_import_dialog_key(key).await.map(|_| CommandResult::Handled);
        }
        
        if context.show_export_dialog {
            return app.handle_export_dialog_key(key).await.map(|_| CommandResult::Handled);
        }
        
        if context.show_nixos_export_dialog {
            return app.handle_nixos_export_dialog_key(key).await.map(|_| CommandResult::Handled);
        }
        
        if context.show_batch_dialog {
            return app.handle_batch_dialog_key(key).await.map(|_| CommandResult::Handled);
        }
        
        if context.show_save_dialog {
            return app.handle_save_dialog_key(key).await.map(|_| CommandResult::Handled);
        }
        
        if context.show_reload_dialog {
            return app.handle_reload_dialog_key(key).await.map(|_| CommandResult::Handled);
        }
        
        if context.show_preview_dialog {
            return app.handle_preview_dialog_key(key).await.map(|_| CommandResult::Handled);
        }
        
        if context.in_search_mode {
            return app.handle_search_key(key).await.map(|_| CommandResult::Handled);
        }
        
        if context.in_edit_mode {
            return app.handle_edit_key(key).await.map(|_| CommandResult::Handled);
        }
        
        // Handle quit
        if matches!(key, KeyCode::Char('q') | KeyCode::Esc) && !context.has_modal_open() {
            app.state = AppState::Quitting;
            return Ok(CommandResult::Handled);
        }
        
        // Handle navigation
        match key {
            KeyCode::Tab | KeyCode::Right => {
                app.ui.next_tab();
                app.focused_panel = app.ui.current_tab;
                return Ok(CommandResult::Handled);
            }
            KeyCode::BackTab | KeyCode::Left => {
                app.ui.previous_tab();
                app.focused_panel = app.ui.current_tab;
                return Ok(CommandResult::Handled);
            }
            KeyCode::Up => {
                app.ui.scroll_up();
                if app.ui.is_preview_mode() {
                    if let Some(item) = app.ui.get_selected_item() {
                        let item_key = item.key.clone();
                        let item_value = item.value.clone();
                        if let Err(e) = app.ui.handle_preview_change(&item_key, &item_value, &app.hyprctl).await {
                            eprintln!("Preview error: {}", e);
                        }
                    }
                }
                return Ok(CommandResult::Handled);
            }
            KeyCode::Down => {
                app.ui.scroll_down();
                if app.ui.is_preview_mode() {
                    if let Some(item) = app.ui.get_selected_item() {
                        let item_key = item.key.clone();
                        let item_value = item.value.clone();
                        if let Err(e) = app.ui.handle_preview_change(&item_key, &item_value, &app.hyprctl).await {
                            eprintln!("Preview error: {}", e);
                        }
                    }
                }
                return Ok(CommandResult::Handled);
            }
            KeyCode::PageUp => {
                app.ui.prev_page();
                return Ok(CommandResult::Handled);
            }
            KeyCode::PageDown => {
                app.ui.next_page();
                return Ok(CommandResult::Handled);
            }
            KeyCode::Home => {
                let list_state = app.ui.get_current_list_state();
                list_state.select(Some(0));
                return Ok(CommandResult::Handled);
            }
            KeyCode::End => {
                let list_state = app.ui.get_current_list_state();
                list_state.select(Some(999));
                return Ok(CommandResult::Handled);
            }
            KeyCode::Enter => {
                if let Some(item) = app.ui.get_selected_item() {
                    app.take_config_snapshot(&format!("Edit {}", item.key));
                    app.ui.start_editing().await?;
                }
                return Ok(CommandResult::Handled);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                app.ui.show_save_dialog = true;
                return Ok(CommandResult::Handled);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                app.ui.show_reload_dialog = true;
                return Ok(CommandResult::Handled);
            }
            KeyCode::Char('/') => {
                app.ui.start_search_debounced();
                return Ok(CommandResult::Handled);
            }
            KeyCode::Char('?') => {
                app.ui.toggle_help();
                return Ok(CommandResult::Handled);
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                app.show_enhanced_preview().await;
                return Ok(CommandResult::Handled);
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                app.show_batch_dialog().await;
                return Ok(CommandResult::Handled);
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                app.show_add_item_dialog().await;
                return Ok(CommandResult::Handled);
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                app.show_export_dialog().await;
                return Ok(CommandResult::Handled);
            }
            _ => {}
        }
        
        Ok(CommandResult::NotHandled)
    }

    /// Register all default commands
    fn register_default_commands(&mut self) {
        // High priority - dialogs and modals
        self.register(Box::new(QuitCommand));
        self.register(Box::new(PopupCommand));
        self.register(Box::new(HelpCommand));
        self.register(Box::new(ImportDialogCommand));
        self.register(Box::new(ExportDialogCommand));
        self.register(Box::new(NixOSExportDialogCommand));
        self.register(Box::new(BatchDialogCommand));
        self.register(Box::new(SaveDialogCommand));
        self.register(Box::new(ReloadDialogCommand));
        self.register(Box::new(PreviewDialogCommand));
        self.register(Box::new(SearchCommand));
        self.register(Box::new(EditCommand));
        
        // Medium priority - navigation and actions
        self.register(Box::new(TabNavigationCommand));
        self.register(Box::new(VerticalNavigationCommand));
        self.register(Box::new(ActionCommand));
        
        // Low priority - fallback
        self.register(Box::new(FallbackCommand));
    }
}

impl Default for CommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ================================
// SPECIFIC COMMAND IMPLEMENTATIONS
// ================================

/// Quit command (Esc or 'q')
#[derive(Debug)]
pub struct QuitCommand;

#[async_trait::async_trait]
impl Command for QuitCommand {
    async fn execute(&self, app: &mut App, _context: &CommandContext) -> Result<CommandResult> {
        app.state = AppState::Quitting;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        matches!(context.key, KeyCode::Char('q') | KeyCode::Esc) && !context.has_modal_open()
    }

    fn priority(&self) -> u8 { 100 }
    fn description(&self) -> &'static str { "Quit application" }
}

/// Popup handling command
#[derive(Debug)]
pub struct PopupCommand;

#[async_trait::async_trait]
impl Command for PopupCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_popup_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.has_popup
    }

    fn priority(&self) -> u8 { 90 }
    fn description(&self) -> &'static str { "Handle popup interactions" }
}

/// Help system command
#[derive(Debug)]
pub struct HelpCommand;

#[async_trait::async_trait]
impl Command for HelpCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_help_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.show_help
    }

    fn priority(&self) -> u8 { 85 }
    fn description(&self) -> &'static str { "Handle help navigation" }
}

/// Import dialog command
#[derive(Debug)]
pub struct ImportDialogCommand;

#[async_trait::async_trait]
impl Command for ImportDialogCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_import_dialog_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.show_import_dialog
    }

    fn priority(&self) -> u8 { 80 }
    fn description(&self) -> &'static str { "Handle import dialog" }
}

/// Export dialog command
#[derive(Debug)]
pub struct ExportDialogCommand;

#[async_trait::async_trait]
impl Command for ExportDialogCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_export_dialog_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.show_export_dialog
    }

    fn priority(&self) -> u8 { 80 }
    fn description(&self) -> &'static str { "Handle export dialog" }
}

/// NixOS export dialog command
#[derive(Debug)]
pub struct NixOSExportDialogCommand;

#[async_trait::async_trait]
impl Command for NixOSExportDialogCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_nixos_export_dialog_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.show_nixos_export_dialog
    }

    fn priority(&self) -> u8 { 80 }
    fn description(&self) -> &'static str { "Handle NixOS export dialog" }
}

/// Batch dialog command
#[derive(Debug)]
pub struct BatchDialogCommand;

#[async_trait::async_trait]
impl Command for BatchDialogCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_batch_dialog_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.show_batch_dialog
    }

    fn priority(&self) -> u8 { 80 }
    fn description(&self) -> &'static str { "Handle batch operations dialog" }
}

/// Save dialog command  
#[derive(Debug)]
pub struct SaveDialogCommand;

#[async_trait::async_trait]
impl Command for SaveDialogCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_save_dialog_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.show_save_dialog
    }

    fn priority(&self) -> u8 { 80 }
    fn description(&self) -> &'static str { "Handle save dialog" }
}

/// Reload dialog command
#[derive(Debug)]
pub struct ReloadDialogCommand;

#[async_trait::async_trait]
impl Command for ReloadDialogCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_reload_dialog_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.show_reload_dialog
    }

    fn priority(&self) -> u8 { 80 }
    fn description(&self) -> &'static str { "Handle reload dialog" }
}

/// Preview dialog command
#[derive(Debug)]
pub struct PreviewDialogCommand;

#[async_trait::async_trait]
impl Command for PreviewDialogCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_preview_dialog_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.show_preview_dialog
    }

    fn priority(&self) -> u8 { 75 }
    fn description(&self) -> &'static str { "Handle preview dialog" }
}

/// Search command
#[derive(Debug)]
pub struct SearchCommand;

#[async_trait::async_trait]
impl Command for SearchCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_search_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.in_search_mode
    }

    fn priority(&self) -> u8 { 70 }
    fn description(&self) -> &'static str { "Handle search input" }
}

/// Edit command
#[derive(Debug)]
pub struct EditCommand;

#[async_trait::async_trait]
impl Command for EditCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        app.handle_edit_key(context.key).await?;
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        context.in_edit_mode
    }

    fn priority(&self) -> u8 { 65 }
    fn description(&self) -> &'static str { "Handle editing input" }
}

/// Tab navigation command
#[derive(Debug)]
pub struct TabNavigationCommand;

#[async_trait::async_trait]
impl Command for TabNavigationCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        match context.key {
            KeyCode::Tab | KeyCode::Right => {
                app.ui.next_tab();
                app.focused_panel = app.ui.current_tab;
            }
            KeyCode::BackTab | KeyCode::Left => {
                app.ui.previous_tab();
                app.focused_panel = app.ui.current_tab;
            }
            _ => return Ok(CommandResult::NotHandled),
        }
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        matches!(context.key, KeyCode::Tab | KeyCode::BackTab | KeyCode::Left | KeyCode::Right)
            && !context.has_modal_open()
    }

    fn priority(&self) -> u8 { 50 }
    fn description(&self) -> &'static str { "Handle tab navigation" }
}

/// Vertical navigation command
#[derive(Debug)]
pub struct VerticalNavigationCommand;

#[async_trait::async_trait]
impl Command for VerticalNavigationCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        match context.key {
            KeyCode::Up => {
                app.ui.scroll_up();
                
                // Trigger live preview if enabled
                if app.ui.is_preview_mode() {
                    if let Some(item) = app.ui.get_selected_item() {
                        let item_key = item.key.clone();
                        let item_value = item.value.clone();
                        if let Err(e) = app.ui.handle_preview_change(&item_key, &item_value, &app.hyprctl).await {
                            eprintln!("Preview error: {}", e);
                        }
                    }
                }
            }
            KeyCode::Down => {
                app.ui.scroll_down();
                
                // Trigger live preview if enabled
                if app.ui.is_preview_mode() {
                    if let Some(item) = app.ui.get_selected_item() {
                        let item_key = item.key.clone();
                        let item_value = item.value.clone();
                        if let Err(e) = app.ui.handle_preview_change(&item_key, &item_value, &app.hyprctl).await {
                            eprintln!("Preview error: {}", e);
                        }
                    }
                }
            }
            KeyCode::PageUp => {
                app.ui.prev_page();
            }
            KeyCode::PageDown => {
                app.ui.next_page();
            }
            KeyCode::Home => {
                // Go to first item
                let list_state = app.ui.get_current_list_state();
                list_state.select(Some(0));
            }
            KeyCode::End => {
                // Go to last item - simplified for now
                // This would need proper implementation with item counting
                // For now, just select a large number to go to end
                let list_state = app.ui.get_current_list_state();
                list_state.select(Some(999));
            }
            _ => return Ok(CommandResult::NotHandled),
        }
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        matches!(
            context.key,
            KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End
        ) && !context.has_modal_open()
    }

    fn priority(&self) -> u8 { 45 }
    fn description(&self) -> &'static str { "Handle vertical navigation" }
}

/// Action command for various app actions
#[derive(Debug)]
pub struct ActionCommand;

#[async_trait::async_trait]
impl Command for ActionCommand {
    async fn execute(&self, app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        match context.key {
            KeyCode::Enter => {
                // Take snapshot before starting to edit
                if let Some(item) = app.ui.get_selected_item() {
                    app.take_config_snapshot(&format!("Edit {}", item.key));
                    app.ui.start_editing().await?;
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                app.ui.show_save_dialog = true;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                app.ui.show_reload_dialog = true;
            }
            KeyCode::Char('/') => {
                app.ui.start_search_debounced();
            }
            KeyCode::Char('?') => {
                app.ui.toggle_help();
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                app.show_enhanced_preview().await;
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                app.show_batch_dialog().await;
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                app.show_add_item_dialog().await;
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                app.show_export_dialog().await;
            }
            _ => return Ok(CommandResult::NotHandled),
        }
        Ok(CommandResult::Handled)
    }

    fn can_handle(&self, context: &CommandContext) -> bool {
        matches!(
            context.key,
            KeyCode::Enter
                | KeyCode::Char('s') | KeyCode::Char('S')
                | KeyCode::Char('r') | KeyCode::Char('R')
                | KeyCode::Char('/') 
                | KeyCode::Char('?')
                | KeyCode::Char('n') | KeyCode::Char('N')
                | KeyCode::Char('b') | KeyCode::Char('B')
                | KeyCode::Char('i') | KeyCode::Char('I')
                | KeyCode::Char('e') | KeyCode::Char('E')
        ) && !context.has_modal_open()
    }

    fn priority(&self) -> u8 { 40 }
    fn description(&self) -> &'static str { "Handle application actions" }
}

/// Fallback command for unhandled keys
#[derive(Debug)]
pub struct FallbackCommand;

#[async_trait::async_trait]
impl Command for FallbackCommand {
    async fn execute(&self, _app: &mut App, context: &CommandContext) -> Result<CommandResult> {
        // Log unhandled keys if in debug mode
        #[cfg(debug_assertions)]
        eprintln!("Unhandled key: {:?}", context.key);
        
        Ok(CommandResult::NotHandled)
    }

    fn can_handle(&self, __context: &CommandContext) -> bool {
        true // Always can handle (fallback)
    }

    fn priority(&self) -> u8 { 0 }
    fn description(&self) -> &'static str { "Fallback for unhandled keys" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[tokio::test]
    async fn test_command_dispatcher_creation() {
        let dispatcher = CommandDispatcher::new();
        assert!(!dispatcher.commands.is_empty());
        
        // Commands should be sorted by priority
        let priorities: Vec<u8> = dispatcher.commands.iter().map(|c| c.priority()).collect();
        let mut sorted_priorities = priorities.clone();
        sorted_priorities.sort_by(|a, b| b.cmp(a));
        assert_eq!(priorities, sorted_priorities);
    }

    #[tokio::test]
    async fn test_quit_command() {
        let quit_command = QuitCommand;
        let context = CommandContext {
            key: KeyCode::Char('q'),
            has_popup: false,
            has_dialog: false,
            in_search_mode: false,
            in_edit_mode: false,
            current_panel: FocusedPanel::General,
            show_help: false,
            show_preview_dialog: false,
            show_import_dialog: false,
            show_export_dialog: false,
            show_nixos_export_dialog: false,
            show_batch_dialog: false,
            show_save_dialog: false,
            show_reload_dialog: false,
        };

        assert!(quit_command.can_handle(&context));
        assert_eq!(quit_command.priority(), 100);
        assert_eq!(quit_command.description(), "Quit application");
    }

    #[test]
    fn test_command_context_modal_detection() {
        let mut context = CommandContext {
            key: KeyCode::Char('q'),
            has_popup: false,
            has_dialog: false,
            in_search_mode: false,
            in_edit_mode: false,
            current_panel: FocusedPanel::General,
            show_help: false,
            show_preview_dialog: false,
            show_import_dialog: false,
            show_export_dialog: false,
            show_nixos_export_dialog: false,
            show_batch_dialog: false,
            show_save_dialog: false,
            show_reload_dialog: false,
        };

        assert!(!context.has_modal_open());

        context.has_popup = true;
        assert!(context.has_modal_open());

        context.has_popup = false;
        context.in_edit_mode = true;
        assert!(context.has_modal_open());
    }
}