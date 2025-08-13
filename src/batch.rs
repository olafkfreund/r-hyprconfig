// Batch configuration management module
// Allows applying settings to multiple configurations simultaneously

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::file_io::{FileOperations, FileUtils};
use crate::hyprctl::HyprCtl;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProfile {
    pub name: String,
    pub description: Option<String>,
    pub config_paths: Vec<PathBuf>,
    pub created_at: std::time::SystemTime,
    pub last_modified: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    pub operation_type: BatchOperationType,
    pub settings: HashMap<String, String>,
    pub keybinds: Vec<String>,
    pub window_rules: Vec<String>,
    pub layer_rules: Vec<String>,
    pub target_profiles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchOperationType {
    Apply,   // Apply settings to all targets
    Merge,   // Merge settings with existing configs
    Replace, // Replace specific sections
    Backup,  // Create backups of all targets
}

#[derive(Debug, Clone)]
pub struct BatchExecutionResult {
    #[allow(dead_code)]
    pub profile_name: String,
    pub success: bool,
    #[allow(dead_code)]
    pub error_message: Option<String>,
    #[allow(dead_code)]
    pub changes_applied: usize,
}

pub struct BatchManager {
    profiles: HashMap<String, BatchProfile>,
    config_dir: PathBuf,
}

impl BatchManager {
    pub async fn new(config_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            profiles: HashMap::new(),
            config_dir,
        };

        // Load existing profiles
        manager.load_profiles().await?;

        Ok(manager)
    }

    pub async fn create_profile(
        &mut self,
        name: String,
        description: Option<String>,
        config_paths: Vec<PathBuf>,
    ) -> Result<()> {
        if self.profiles.contains_key(&name) {
            return Err(anyhow!("Profile '{}' already exists", name));
        }

        // Validate that all config paths exist
        for path in &config_paths {
            if !path.exists() {
                return Err(anyhow!(
                    "Configuration file does not exist: {}",
                    path.display()
                ));
            }
        }

        let now = std::time::SystemTime::now();
        let profile = BatchProfile {
            name: name.clone(),
            description,
            config_paths,
            created_at: now.into(),
            last_modified: now.into(),
        };

        self.profiles.insert(name, profile);
        self.save_profiles().await?;

        Ok(())
    }

    pub async fn delete_profile(&mut self, name: &str) -> Result<()> {
        if !self.profiles.contains_key(name) {
            return Err(anyhow!("Profile '{}' does not exist", name));
        }

        self.profiles.remove(name);
        self.save_profiles().await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn list_profiles(&self) -> Vec<&BatchProfile> {
        self.profiles.values().collect()
    }

    #[allow(dead_code)]
    pub fn get_profile(&self, name: &str) -> Option<&BatchProfile> {
        self.profiles.get(name)
    }

    pub async fn execute_batch_operation(
        &self,
        operation: &BatchOperation,
        hyprctl: &HyprCtl,
    ) -> Result<Vec<BatchExecutionResult>> {
        let mut results = Vec::new();

        for profile_name in &operation.target_profiles {
            let result = self
                .execute_single_profile_operation(profile_name, operation, hyprctl)
                .await;

            results.push(result);
        }

        Ok(results)
    }

    async fn execute_single_profile_operation(
        &self,
        profile_name: &str,
        operation: &BatchOperation,
        hyprctl: &HyprCtl,
    ) -> BatchExecutionResult {
        let profile = match self.profiles.get(profile_name) {
            Some(profile) => profile,
            None => {
                return BatchExecutionResult {
                    profile_name: profile_name.to_string(),
                    success: false,
                    error_message: Some(format!("Profile '{profile_name}' not found")),
                    changes_applied: 0,
                };
            }
        };

        let mut changes_applied = 0;
        let mut errors = Vec::new();

        for config_path in &profile.config_paths {
            match self
                .apply_operation_to_config(config_path, operation, hyprctl)
                .await
            {
                Ok(changes) => changes_applied += changes,
                Err(e) => errors.push(format!("{}:{}", config_path.display(), e)),
            }
        }

        BatchExecutionResult {
            profile_name: profile_name.to_string(),
            success: errors.is_empty(),
            error_message: if errors.is_empty() {
                None
            } else {
                Some(errors.join(", "))
            },
            changes_applied,
        }
    }

    async fn apply_operation_to_config(
        &self,
        config_path: &Path,
        operation: &BatchOperation,
        hyprctl: &HyprCtl,
    ) -> Result<usize> {
        let mut changes_applied = 0;

        match operation.operation_type {
            BatchOperationType::Backup => {
                self.create_backup(config_path).await?;
                changes_applied = 1;
            }
            BatchOperationType::Apply | BatchOperationType::Merge | BatchOperationType::Replace => {
                // Load existing config
                let mut config = Config::load_from_file(config_path)?;

                // Apply settings based on operation type
                match operation.operation_type {
                    BatchOperationType::Apply | BatchOperationType::Replace => {
                        // Replace or apply settings directly
                        for (key, value) in &operation.settings {
                            config.set_value(key, value)?;
                            changes_applied += 1;
                        }

                        // Apply keybinds
                        if !operation.keybinds.is_empty() {
                            config.clear_keybinds();
                            for keybind in &operation.keybinds {
                                config.add_keybind(keybind.clone())?;
                                changes_applied += 1;
                            }
                        }

                        // Apply window rules
                        if !operation.window_rules.is_empty() {
                            config.clear_window_rules();
                            for rule in &operation.window_rules {
                                config.add_window_rule(rule.clone())?;
                                changes_applied += 1;
                            }
                        }

                        // Apply layer rules
                        if !operation.layer_rules.is_empty() {
                            config.clear_layer_rules();
                            for rule in &operation.layer_rules {
                                config.add_layer_rule(rule.clone())?;
                                changes_applied += 1;
                            }
                        }
                    }
                    BatchOperationType::Merge => {
                        // Merge settings without replacing existing ones
                        for (key, value) in &operation.settings {
                            if !config.has_value(key) {
                                config.set_value(key, value)?;
                                changes_applied += 1;
                            }
                        }

                        // Merge keybinds (add new ones, keep existing)
                        for keybind in &operation.keybinds {
                            if !config.has_keybind(keybind) {
                                config.add_keybind(keybind.clone())?;
                                changes_applied += 1;
                            }
                        }

                        // Similar for rules...
                    }
                    _ => unreachable!(),
                }

                // Save the modified config
                config.save_to_file(config_path)?;

                // Apply to running Hyprland if this is the active config
                if self.is_active_config(config_path)? {
                    self.apply_to_hyprland(&config, hyprctl).await?;
                }
            }
        }

        Ok(changes_applied)
    }

    async fn create_backup(&self, config_path: &Path) -> Result<()> {
        let file_ops = FileOperations::new();
        file_ops.create_backup(config_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create backup: {}", e))?;
        Ok(())
    }

    fn is_active_config(&self, config_path: &Path) -> Result<bool> {
        // Check if this is the currently active Hyprland config
        // This is a simplified check - in practice, you'd want to compare
        // with the actual Hyprland config path
        let hypr_config = dirs::home_dir()
            .ok_or_else(|| anyhow!("Cannot determine home directory"))?
            .join(".config/hypr/hyprland.conf");

        Ok(config_path == hypr_config)
    }

    async fn apply_to_hyprland(&self, _config: &Config, hyprctl: &HyprCtl) -> Result<()> {
        // Apply configuration changes to running Hyprland
        // This would use the existing HyprCtl integration

        // For now, just reload the config
        hyprctl.reload_config().await?;
        Ok(())
    }

    async fn load_profiles(&mut self) -> Result<()> {
        let profiles_file = self.config_dir.join("batch_profiles.json");

        if profiles_file.exists() {
            let content = FileUtils::resilient_read(&profiles_file)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read profiles file: {}", e))?;
            self.profiles = serde_json::from_str(&content)?;
        }

        Ok(())
    }

    async fn save_profiles(&self) -> Result<()> {
        // Ensure config directory exists
        FileUtils::ensure_directory(&self.config_dir)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create config directory: {}", e))?;

        let profiles_file = self.config_dir.join("batch_profiles.json");
        let content = serde_json::to_string_pretty(&self.profiles)?;
        
        FileUtils::safe_write(&profiles_file, &content)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write profiles file: {}", e))?;

        Ok(())
    }

    // Utility methods for creating common batch operations
    #[allow(dead_code)]
    pub fn create_apply_settings_operation(
        settings: HashMap<String, String>,
        target_profiles: Vec<String>,
    ) -> BatchOperation {
        BatchOperation {
            operation_type: BatchOperationType::Apply,
            settings,
            keybinds: Vec::new(),
            window_rules: Vec::new(),
            layer_rules: Vec::new(),
            target_profiles,
        }
    }

    #[allow(dead_code)]
    pub fn create_backup_operation(target_profiles: Vec<String>) -> BatchOperation {
        BatchOperation {
            operation_type: BatchOperationType::Backup,
            settings: HashMap::new(),
            keybinds: Vec::new(),
            window_rules: Vec::new(),
            layer_rules: Vec::new(),
            target_profiles,
        }
    }

    #[allow(dead_code)]
    pub fn create_full_config_operation(
        settings: HashMap<String, String>,
        keybinds: Vec<String>,
        window_rules: Vec<String>,
        layer_rules: Vec<String>,
        target_profiles: Vec<String>,
        merge: bool,
    ) -> BatchOperation {
        BatchOperation {
            operation_type: if merge {
                BatchOperationType::Merge
            } else {
                BatchOperationType::Apply
            },
            settings,
            keybinds,
            window_rules,
            layer_rules,
            target_profiles,
        }
    }
}

// Helper trait to extend Config with batch-related methods
pub trait ConfigBatchExt {
    fn set_value(&mut self, key: &str, value: &str) -> Result<()>;
    fn has_value(&self, key: &str) -> bool;
    fn clear_keybinds(&mut self);
    fn add_keybind(&mut self, keybind: String) -> Result<()>;
    fn has_keybind(&self, keybind: &str) -> bool;
    fn clear_window_rules(&mut self);
    fn add_window_rule(&mut self, rule: String) -> Result<()>;
    fn clear_layer_rules(&mut self);
    fn add_layer_rule(&mut self, rule: String) -> Result<()>;
    fn load_from_file(path: &Path) -> Result<Config>;
    fn save_to_file(&self, path: &Path) -> Result<()>;
}

impl ConfigBatchExt for Config {
    fn set_value(&mut self, _key: &str, _value: &str) -> Result<()> {
        // Implementation would depend on your Config structure
        // For now, this is a placeholder
        Ok(())
    }

    fn has_value(&self, _key: &str) -> bool {
        // Check if config already has this value
        false
    }

    fn clear_keybinds(&mut self) {
        // Clear all keybinds
    }

    fn add_keybind(&mut self, _keybind: String) -> Result<()> {
        // Add a keybind
        Ok(())
    }

    fn has_keybind(&self, _keybind: &str) -> bool {
        // Check if keybind exists
        false
    }

    fn clear_window_rules(&mut self) {
        // Clear window rules
    }

    fn add_window_rule(&mut self, _rule: String) -> Result<()> {
        // Add window rule
        Ok(())
    }

    fn clear_layer_rules(&mut self) {
        // Clear layer rules
    }

    fn add_layer_rule(&mut self, _rule: String) -> Result<()> {
        // Add layer rule
        Ok(())
    }

    fn load_from_file(_path: &Path) -> Result<Config> {
        // TODO: Parse hyprland.conf file
        Ok(Config::default())
    }

    fn save_to_file(&self, _path: &Path) -> Result<()> {
        // Save config to file
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_batch_profile_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = BatchManager::new(temp_dir.path().to_path_buf()).await.unwrap();

        let config_paths = vec![
            temp_dir.path().join("config1.conf"),
            temp_dir.path().join("config2.conf"),
        ];

        // Create dummy config files
        for path in &config_paths {
            std::fs::write(path, "# Test config").unwrap();
        }

        let result = manager.create_profile(
            "test_profile".to_string(),
            Some("Test profile description".to_string()),
            config_paths,
        ).await;

        assert!(result.is_ok());
        assert!(manager.get_profile("test_profile").is_some());
    }

    #[test]
    fn test_batch_operation_creation() {
        let mut settings = HashMap::new();
        settings.insert("general:gaps_in".to_string(), "10".to_string());

        let operation = BatchManager::create_apply_settings_operation(
            settings,
            vec!["profile1".to_string(), "profile2".to_string()],
        );

        assert_eq!(operation.operation_type, BatchOperationType::Apply);
        assert_eq!(operation.target_profiles.len(), 2);
        assert_eq!(operation.settings.len(), 1);
    }
}
