use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::ui::ConfigItem;
use crate::app::FocusedPanel;

/// Represents a snapshot of the entire configuration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// All configuration items organized by panel
    pub config_items: HashMap<FocusedPanel, Vec<ConfigItem>>,
    /// Timestamp when the snapshot was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional description of what changed
    pub description: Option<String>,
}

impl ConfigSnapshot {
    pub fn new(config_items: HashMap<FocusedPanel, Vec<ConfigItem>>, description: Option<String>) -> Self {
        Self {
            config_items,
            timestamp: chrono::Utc::now(),
            description,
        }
    }

    /// Create a snapshot from the current UI state
    pub fn from_ui(ui: &crate::ui::UI, description: Option<String>) -> Self {
        Self::new(ui.config_items.clone(), description)
    }
}

/// Manages undo/redo operations for configuration changes
#[derive(Debug)]
pub struct UndoManager {
    /// Stack of previous states (undo stack)
    undo_stack: Vec<ConfigSnapshot>,
    /// Stack of undone states (redo stack)
    redo_stack: Vec<ConfigSnapshot>,
    /// Maximum number of undo operations to keep in memory
    max_history: usize,
    /// Current snapshot (if any)
    current_snapshot: Option<ConfigSnapshot>,
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new(50) // Default to 50 undo operations
    }
}

impl UndoManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
            current_snapshot: None,
        }
    }

    /// Take a snapshot of the current state before making changes
    pub fn take_snapshot(&mut self, snapshot: ConfigSnapshot) {
        // If we have a current snapshot, move it to undo stack
        if let Some(current) = self.current_snapshot.take() {
            self.undo_stack.push(current);
            
            // Limit undo stack size
            if self.undo_stack.len() > self.max_history {
                self.undo_stack.remove(0);
            }
        }

        // Clear redo stack when new changes are made
        self.redo_stack.clear();
        
        // Set new current snapshot
        self.current_snapshot = Some(snapshot);
    }

    /// Undo the last operation and return the previous state
    pub fn undo(&mut self) -> Option<ConfigSnapshot> {
        if let Some(current) = self.current_snapshot.take() {
            // Move current to redo stack
            self.redo_stack.push(current);
            
            // Pop from undo stack to become current
            if let Some(previous) = self.undo_stack.pop() {
                let snapshot = previous.clone();
                self.current_snapshot = Some(previous);
                return Some(snapshot);
            } else {
                // No undo available, restore current
                self.current_snapshot = self.redo_stack.pop();
            }
        }
        None
    }

    /// Redo the last undone operation and return the state
    pub fn redo(&mut self) -> Option<ConfigSnapshot> {
        if let Some(current) = self.current_snapshot.take() {
            // Move current to undo stack
            self.undo_stack.push(current);
        }

        // Pop from redo stack to become current
        if let Some(next) = self.redo_stack.pop() {
            let snapshot = next.clone();
            self.current_snapshot = Some(next);
            Some(snapshot)
        } else {
            // No redo available, restore current
            self.current_snapshot = self.undo_stack.pop();
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of available undo operations
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of available redo operations
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Get description of the last undo operation
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().and_then(|s| s.description.as_deref())
    }

    /// Get description of the last redo operation
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().and_then(|s| s.description.as_deref())
    }

    /// Clear all undo/redo history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_snapshot = None;
    }

    /// Get current snapshot (for debugging/inspection)
    pub fn current_snapshot(&self) -> Option<&ConfigSnapshot> {
        self.current_snapshot.as_ref()
    }

    /// Get undo stack history (for debugging/UI display)
    pub fn undo_history(&self) -> &[ConfigSnapshot] {
        &self.undo_stack
    }

    /// Get redo stack history (for debugging/UI display)
    pub fn redo_history(&self) -> &[ConfigSnapshot] {
        &self.redo_stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{ConfigDataType, ConfigItem};

    fn create_test_config_items() -> HashMap<FocusedPanel, Vec<ConfigItem>> {
        let mut items = HashMap::new();
        items.insert(
            FocusedPanel::General,
            vec![ConfigItem {
                key: "test_key".to_string(),
                value: "test_value".to_string(),
                description: "Test item".to_string(),
                data_type: ConfigDataType::String,
                suggestions: vec![],
            }],
        );
        items
    }

    #[test]
    fn test_undo_manager_creation() {
        let manager = UndoManager::new(10);
        assert_eq!(manager.max_history, 10);
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_take_snapshot() {
        let mut manager = UndoManager::new(10);
        let config_items = create_test_config_items();
        let snapshot = ConfigSnapshot::new(config_items, Some("Test snapshot".to_string()));

        manager.take_snapshot(snapshot);
        assert!(manager.current_snapshot.is_some());
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_undo_redo_cycle() {
        let mut manager = UndoManager::new(10);
        
        // Create first snapshot
        let config1 = create_test_config_items();
        let snapshot1 = ConfigSnapshot::new(config1, Some("First state".to_string()));
        manager.take_snapshot(snapshot1);

        // Create second snapshot
        let mut config2 = create_test_config_items();
        config2.get_mut(&FocusedPanel::General).unwrap()[0].value = "modified_value".to_string();
        let snapshot2 = ConfigSnapshot::new(config2, Some("Second state".to_string()));
        manager.take_snapshot(snapshot2);

        // Should be able to undo now
        assert!(manager.can_undo());
        assert!(!manager.can_redo());

        // Undo
        let undone = manager.undo().unwrap();
        assert_eq!(undone.description, Some("First state".to_string()));
        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        // Redo
        let redone = manager.redo().unwrap();
        assert_eq!(redone.description, Some("Second state".to_string()));
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_max_history_limit() {
        let mut manager = UndoManager::new(2);

        // Add 3 snapshots
        for i in 0..3 {
            let config = create_test_config_items();
            let snapshot = ConfigSnapshot::new(config, Some(format!("State {}", i)));
            manager.take_snapshot(snapshot);
        }

        // Should only keep 2 in history (current + 2 in undo stack)
        assert_eq!(manager.undo_count(), 2);
    }

    #[test]
    fn test_clear_redo_on_new_changes() {
        let mut manager = UndoManager::new(10);

        // Create initial state
        let config1 = create_test_config_items();
        let snapshot1 = ConfigSnapshot::new(config1, Some("State 1".to_string()));
        manager.take_snapshot(snapshot1);

        // Create second state
        let config2 = create_test_config_items();
        let snapshot2 = ConfigSnapshot::new(config2, Some("State 2".to_string()));
        manager.take_snapshot(snapshot2);

        // Undo to have redo available
        manager.undo();
        assert!(manager.can_redo());

        // Make new change - should clear redo stack
        let config3 = create_test_config_items();
        let snapshot3 = ConfigSnapshot::new(config3, Some("State 3".to_string()));
        manager.take_snapshot(snapshot3);

        assert!(!manager.can_redo());
    }
}