use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Supported configuration formats for import/export
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigFormat {
    /// Standard Hyprland .conf format
    HyprlandConf,
    /// JSON format with structured data
    Json,
    /// TOML format for human-readable config
    Toml,
    /// NixOS Home Manager format
    NixHomeManager,
    /// NixOS System configuration format
    NixSystem,
    /// YAML format (alternative to JSON)
    Yaml,
    /// Custom r-hyprconfig format with metadata
    RHyprConfig,
}

impl ConfigFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFormat::HyprlandConf => "conf",
            ConfigFormat::Json => "json",
            ConfigFormat::Toml => "toml",
            ConfigFormat::NixHomeManager => "nix",
            ConfigFormat::NixSystem => "nix",
            ConfigFormat::Yaml => "yaml",
            ConfigFormat::RHyprConfig => "rhypr",
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ConfigFormat::HyprlandConf => "Hyprland Configuration",
            ConfigFormat::Json => "JSON Format",
            ConfigFormat::Toml => "TOML Format", 
            ConfigFormat::NixHomeManager => "NixOS Home Manager",
            ConfigFormat::NixSystem => "NixOS System Configuration",
            ConfigFormat::Yaml => "YAML Format",
            ConfigFormat::RHyprConfig => "r-hyprconfig Format",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "conf" => Some(ConfigFormat::HyprlandConf),
            "json" => Some(ConfigFormat::Json),
            "toml" => Some(ConfigFormat::Toml),
            "nix" => Some(ConfigFormat::NixHomeManager), // Default to Home Manager
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "rhypr" => Some(ConfigFormat::RHyprConfig),
            _ => None,
        }
    }

    /// Detect format from file path
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| Self::from_extension(ext))
    }

    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ConfigFormat::HyprlandConf => "text/plain",
            ConfigFormat::Json => "application/json",
            ConfigFormat::Toml => "application/toml",
            ConfigFormat::NixHomeManager | ConfigFormat::NixSystem => "text/x-nix",
            ConfigFormat::Yaml => "application/yaml",
            ConfigFormat::RHyprConfig => "application/x-rhyprconfig",
        }
    }
}

/// Structured configuration data for import/export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredConfig {
    pub metadata: ConfigMetadata,
    pub general: GeneralSettings,
    pub input: InputSettings,
    pub decoration: DecorationSettings,
    pub animations: AnimationSettings,
    pub gestures: GestureSettings,
    pub keybinds: Vec<KeybindEntry>,
    pub window_rules: Vec<WindowRuleEntry>,
    pub layer_rules: Vec<LayerRuleEntry>,
    pub misc: MiscSettings,
    pub custom_settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub source_url: Option<String>,
    pub source_format: ConfigFormat,
    pub r_hyprconfig_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub gaps_in: Option<i32>,
    pub gaps_out: Option<i32>,
    pub border_size: Option<i32>,
    pub col_active_border: Option<String>,
    pub col_inactive_border: Option<String>,
    pub resize_on_border: Option<bool>,
    pub extend_border_grab_area: Option<i32>,
    pub hover_icon_on_border: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSettings {
    pub kb_layout: Option<String>,
    pub kb_variant: Option<String>,
    pub kb_model: Option<String>,
    pub kb_options: Option<String>,
    pub kb_rules: Option<String>,
    pub follow_mouse: Option<i32>,
    pub mouse_refocus: Option<bool>,
    pub sensitivity: Option<f32>,
    pub accel_profile: Option<String>,
    pub natural_scroll: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecorationSettings {
    pub rounding: Option<i32>,
    pub blur_enabled: Option<bool>,
    pub blur_size: Option<i32>,
    pub blur_passes: Option<i32>,
    pub drop_shadow: Option<bool>,
    pub shadow_range: Option<i32>,
    pub shadow_render_power: Option<i32>,
    pub col_shadow: Option<String>,
    pub dim_inactive: Option<bool>,
    pub dim_strength: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSettings {
    pub enabled: Option<bool>,
    pub beziers: Vec<String>,
    pub animations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureSettings {
    pub workspace_swipe: Option<bool>,
    pub workspace_swipe_fingers: Option<i32>,
    pub workspace_swipe_distance: Option<i32>,
    pub workspace_swipe_invert: Option<bool>,
    pub workspace_swipe_min_speed_to_force: Option<i32>,
    pub workspace_swipe_cancel_ratio: Option<f32>,
    pub workspace_swipe_create_new: Option<bool>,
    pub workspace_swipe_forever: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiscSettings {
    pub disable_hyprland_logo: Option<bool>,
    pub disable_splash_rendering: Option<bool>,
    pub mouse_move_enables_dpms: Option<bool>,
    pub key_press_enables_dpms: Option<bool>,
    pub always_follow_on_dnd: Option<bool>,
    pub layers_hog_keyboard_focus: Option<bool>,
    pub animate_manual_resizes: Option<bool>,
    pub animate_mouse_windowdragging: Option<bool>,
    pub disable_autoreload: Option<bool>,
    pub enable_swallow: Option<bool>,
    pub swallow_regex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindEntry {
    pub bind_type: String,
    pub modifiers: Vec<String>,
    pub key: String,
    pub dispatcher: String,
    pub args: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowRuleEntry {
    pub rule: String,
    pub window_identifier: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerRuleEntry {
    pub rule: String,
    pub layer: String,
    pub description: Option<String>,
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            name: "Untitled Configuration".to_string(),
            description: None,
            author: None,
            version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            source_url: None,
            source_format: ConfigFormat::HyprlandConf,
            r_hyprconfig_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            gaps_in: None,
            gaps_out: None,
            border_size: None,
            col_active_border: None,
            col_inactive_border: None,
            resize_on_border: None,
            extend_border_grab_area: None,
            hover_icon_on_border: None,
        }
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            kb_layout: None,
            kb_variant: None,
            kb_model: None,
            kb_options: None,
            kb_rules: None,
            follow_mouse: None,
            mouse_refocus: None,
            sensitivity: None,
            accel_profile: None,
            natural_scroll: None,
        }
    }
}

impl Default for DecorationSettings {
    fn default() -> Self {
        Self {
            rounding: None,
            blur_enabled: None,
            blur_size: None,
            blur_passes: None,
            drop_shadow: None,
            shadow_range: None,
            shadow_render_power: None,
            col_shadow: None,
            dim_inactive: None,
            dim_strength: None,
        }
    }
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            enabled: None,
            beziers: Vec::new(),
            animations: Vec::new(),
        }
    }
}

impl Default for GestureSettings {
    fn default() -> Self {
        Self {
            workspace_swipe: None,
            workspace_swipe_fingers: None,
            workspace_swipe_distance: None,
            workspace_swipe_invert: None,
            workspace_swipe_min_speed_to_force: None,
            workspace_swipe_cancel_ratio: None,
            workspace_swipe_create_new: None,
            workspace_swipe_forever: None,
        }
    }
}

impl Default for MiscSettings {
    fn default() -> Self {
        Self {
            disable_hyprland_logo: None,
            disable_splash_rendering: None,
            mouse_move_enables_dpms: None,
            key_press_enables_dpms: None,
            always_follow_on_dnd: None,
            layers_hog_keyboard_focus: None,
            animate_manual_resizes: None,
            animate_mouse_windowdragging: None,
            disable_autoreload: None,
            enable_swallow: None,
            swallow_regex: None,
        }
    }
}

impl StructuredConfig {
    /// Create a new structured config with metadata
    pub fn new(name: &str) -> Self {
        Self {
            metadata: ConfigMetadata {
                name: name.to_string(),
                ..Default::default()
            },
            general: Default::default(),
            input: Default::default(),
            decoration: Default::default(),
            animations: Default::default(),
            gestures: Default::default(),
            keybinds: Vec::new(),
            window_rules: Vec::new(),
            layer_rules: Vec::new(),
            misc: Default::default(),
            custom_settings: HashMap::new(),
        }
    }

    /// Add a tag to the configuration
    pub fn add_tag(&mut self, tag: &str) {
        if !self.metadata.tags.contains(&tag.to_string()) {
            self.metadata.tags.push(tag.to_string());
        }
    }

    /// Set the source URL for this configuration
    pub fn set_source_url(&mut self, url: &str) {
        self.metadata.source_url = Some(url.to_string());
    }

    /// Update the modification timestamp
    pub fn touch(&mut self) {
        self.metadata.updated_at = chrono::Utc::now();
    }

    /// Validate that the configuration has required fields
    pub fn validate(&self) -> Result<()> {
        if self.metadata.name.trim().is_empty() {
            anyhow::bail!("Configuration name cannot be empty");
        }

        // Validate keybinds
        for (i, keybind) in self.keybinds.iter().enumerate() {
            if keybind.key.is_empty() {
                anyhow::bail!("Keybind {} has empty key", i);
            }
            if keybind.dispatcher.is_empty() {
                anyhow::bail!("Keybind {} has empty dispatcher", i);
            }
        }

        // Validate window rules
        for (i, rule) in self.window_rules.iter().enumerate() {
            if rule.rule.is_empty() {
                anyhow::bail!("Window rule {} has empty rule", i);
            }
            if rule.window_identifier.is_empty() {
                anyhow::bail!("Window rule {} has empty window identifier", i);
            }
        }

        // Validate layer rules
        for (i, rule) in self.layer_rules.iter().enumerate() {
            if rule.rule.is_empty() {
                anyhow::bail!("Layer rule {} has empty rule", i);
            }
            if rule.layer.is_empty() {
                anyhow::bail!("Layer rule {} has empty layer", i);
            }
        }

        Ok(())
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "{} - {} keybinds, {} window rules, {} layer rules",
            self.metadata.name,
            self.keybinds.len(),
            self.window_rules.len(),
            self.layer_rules.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_format_extension() {
        assert_eq!(ConfigFormat::HyprlandConf.extension(), "conf");
        assert_eq!(ConfigFormat::Json.extension(), "json");
        assert_eq!(ConfigFormat::Toml.extension(), "toml");
        assert_eq!(ConfigFormat::NixHomeManager.extension(), "nix");
        assert_eq!(ConfigFormat::Yaml.extension(), "yaml");
        assert_eq!(ConfigFormat::RHyprConfig.extension(), "rhypr");
    }

    #[test]
    fn test_config_format_from_extension() {
        assert_eq!(ConfigFormat::from_extension("conf"), Some(ConfigFormat::HyprlandConf));
        assert_eq!(ConfigFormat::from_extension("json"), Some(ConfigFormat::Json));
        assert_eq!(ConfigFormat::from_extension("TOML"), Some(ConfigFormat::Toml));
        assert_eq!(ConfigFormat::from_extension("nix"), Some(ConfigFormat::NixHomeManager));
        assert_eq!(ConfigFormat::from_extension("yaml"), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::from_extension("yml"), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::from_extension("rhypr"), Some(ConfigFormat::RHyprConfig));
        assert_eq!(ConfigFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_config_format_from_path() {
        let path = PathBuf::from("config.conf");
        assert_eq!(ConfigFormat::from_path(&path), Some(ConfigFormat::HyprlandConf));
        
        let path = PathBuf::from("/home/user/.config/hypr/hyprland.conf");
        assert_eq!(ConfigFormat::from_path(&path), Some(ConfigFormat::HyprlandConf));
        
        let path = PathBuf::from("config.json");
        assert_eq!(ConfigFormat::from_path(&path), Some(ConfigFormat::Json));
        
        let path = PathBuf::from("no_extension");
        assert_eq!(ConfigFormat::from_path(&path), None);
    }

    #[test]
    fn test_structured_config_new() {
        let config = StructuredConfig::new("Test Config");
        assert_eq!(config.metadata.name, "Test Config");
        assert_eq!(config.keybinds.len(), 0);
        assert_eq!(config.window_rules.len(), 0);
        assert_eq!(config.layer_rules.len(), 0);
    }

    #[test]
    fn test_structured_config_add_tag() {
        let mut config = StructuredConfig::new("Test");
        config.add_tag("gaming");
        config.add_tag("minimal");
        config.add_tag("gaming"); // Duplicate should be ignored
        
        assert_eq!(config.metadata.tags.len(), 2);
        assert!(config.metadata.tags.contains(&"gaming".to_string()));
        assert!(config.metadata.tags.contains(&"minimal".to_string()));
    }

    #[test]
    fn test_structured_config_validation() -> Result<()> {
        let mut config = StructuredConfig::new("Valid Config");
        
        // Valid config should pass
        config.validate()?;
        
        // Empty name should fail
        config.metadata.name = "".to_string();
        assert!(config.validate().is_err());
        
        // Fix name
        config.metadata.name = "Valid Config".to_string();
        
        // Invalid keybind should fail
        config.keybinds.push(KeybindEntry {
            bind_type: "bind".to_string(),
            modifiers: vec!["SUPER".to_string()],
            key: "".to_string(), // Empty key
            dispatcher: "exec".to_string(),
            args: Some("kitty".to_string()),
            description: None,
        });
        assert!(config.validate().is_err());
        
        Ok(())
    }

    #[test]
    fn test_structured_config_summary() {
        let mut config = StructuredConfig::new("Test Config");
        
        config.keybinds.push(KeybindEntry {
            bind_type: "bind".to_string(),
            modifiers: vec!["SUPER".to_string()],
            key: "Return".to_string(),
            dispatcher: "exec".to_string(),
            args: Some("kitty".to_string()),
            description: None,
        });
        
        config.window_rules.push(WindowRuleEntry {
            rule: "float".to_string(),
            window_identifier: "^(kitty)$".to_string(),
            description: None,
        });
        
        let summary = config.summary();
        assert!(summary.contains("Test Config"));
        assert!(summary.contains("1 keybinds"));
        assert!(summary.contains("1 window rules"));
        assert!(summary.contains("0 layer rules"));
    }

    #[test]
    fn test_config_metadata_default() {
        let metadata = ConfigMetadata::default();
        assert_eq!(metadata.name, "Untitled Configuration");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.source_format, ConfigFormat::HyprlandConf);
        assert_eq!(metadata.r_hyprconfig_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_config_format_mime_type() {
        assert_eq!(ConfigFormat::HyprlandConf.mime_type(), "text/plain");
        assert_eq!(ConfigFormat::Json.mime_type(), "application/json");
        assert_eq!(ConfigFormat::Toml.mime_type(), "application/toml");
        assert_eq!(ConfigFormat::NixHomeManager.mime_type(), "text/x-nix");
        assert_eq!(ConfigFormat::Yaml.mime_type(), "application/yaml");
        assert_eq!(ConfigFormat::RHyprConfig.mime_type(), "application/x-rhyprconfig");
    }

    #[test]
    fn test_config_format_description() {
        assert_eq!(ConfigFormat::HyprlandConf.description(), "Hyprland Configuration");
        assert_eq!(ConfigFormat::Json.description(), "JSON Format");
        assert_eq!(ConfigFormat::NixHomeManager.description(), "NixOS Home Manager");
    }
}