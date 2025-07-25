use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::hyprctl::HyprlandConfig;
use crate::platform::ConfigPathManager;
use super::formats::{ConfigFormat, StructuredConfig, ConfigMetadata, GeneralSettings, InputSettings, DecorationSettings, AnimationSettings, GestureSettings, MiscSettings, KeybindEntry, WindowRuleEntry, LayerRuleEntry};

/// Configuration exporter that can save configurations in various formats
pub struct ConfigExporter {
    output_dir: PathBuf,
}

impl ConfigExporter {
    /// Create a new exporter with default output directory
    pub fn new() -> Result<Self> {
        let paths = ConfigPathManager::get_paths()?;
        Ok(Self {
            output_dir: paths.exports_dir,
        })
    }

    /// Create a new exporter with custom output directory
    pub fn with_output_dir(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    /// Export current system configuration
    pub async fn export_current_config(
        &self,
        name: &str,
        format: ConfigFormat,
        hyprctl: &crate::hyprctl::HyprCtl,
    ) -> Result<PathBuf> {
        let config = self.extract_current_config(hyprctl).await?;
        self.export_structured_config(&config, name, format)
    }

    /// Export a provided Hyprland configuration
    pub fn export_hyprland_config(
        &self,
        config: &HyprlandConfig,
        name: &str,
        format: ConfigFormat,
    ) -> Result<PathBuf> {
        let structured = self.convert_to_structured(config, name)?;
        self.export_structured_config(&structured, name, format)
    }

    /// Export structured configuration to specified format
    pub fn export_structured_config(
        &self,
        config: &StructuredConfig,
        name: &str,
        format: ConfigFormat,
    ) -> Result<PathBuf> {
        // Ensure output directory exists
        std::fs::create_dir_all(&self.output_dir)
            .with_context(|| format!("Failed to create output directory: {:?}", self.output_dir))?;

        let filename = format!("{}.{}", name, format.extension());
        let output_path = self.output_dir.join(filename);

        let content = match format {
            ConfigFormat::HyprlandConf => self.to_hyprland_conf(config)?,
            ConfigFormat::Json => self.to_json(config)?,
            ConfigFormat::Toml => self.to_toml(config)?,
            ConfigFormat::Yaml => self.to_yaml(config)?,
            ConfigFormat::RHyprConfig => self.to_rhypr_config(config)?,
            ConfigFormat::NixHomeManager => self.to_nix_home_manager(config)?,
            ConfigFormat::NixSystem => self.to_nix_system(config)?,
        };

        std::fs::write(&output_path, content)
            .with_context(|| format!("Failed to write config to: {:?}", output_path))?;

        Ok(output_path)
    }

    /// Convert Hyprland config to structured format
    fn convert_to_structured(&self, config: &HyprlandConfig, name: &str) -> Result<StructuredConfig> {
        let mut structured = StructuredConfig::new(name);
        
        // General settings
        structured.general = GeneralSettings {
            gaps_in: Some(config.general.gaps_in),
            gaps_out: Some(config.general.gaps_out),
            border_size: Some(config.general.border_size),
            col_active_border: Some(config.general.col_active_border.clone()),
            col_inactive_border: Some(config.general.col_inactive_border.clone()),
            resize_on_border: Some(config.general.resize_on_border),
            extend_border_grab_area: Some(config.general.extend_border_grab_area),
            hover_icon_on_border: Some(config.general.hover_icon_on_border),
        };

        // Input settings
        structured.input = InputSettings {
            kb_layout: Some(config.input.kb_layout.clone()),
            kb_variant: Some(config.input.kb_variant.clone()),
            kb_model: Some(config.input.kb_model.clone()),
            kb_options: Some(config.input.kb_options.clone()),
            kb_rules: Some(config.input.kb_rules.clone()),
            follow_mouse: Some(config.input.follow_mouse),
            mouse_refocus: Some(config.input.mouse_refocus),
            sensitivity: Some(config.input.sensitivity),
            accel_profile: Some(config.input.accel_profile.clone()),
            natural_scroll: Some(config.input.natural_scroll),
        };

        // Decoration settings
        structured.decoration = DecorationSettings {
            rounding: Some(config.decoration.rounding),
            blur_enabled: Some(config.decoration.blur_enabled),
            blur_size: Some(config.decoration.blur_size),
            blur_passes: Some(config.decoration.blur_passes),
            drop_shadow: Some(config.decoration.drop_shadow),
            shadow_range: Some(config.decoration.shadow_range),
            shadow_render_power: Some(config.decoration.shadow_render_power),
            col_shadow: Some(config.decoration.col_shadow.clone()),
            dim_inactive: Some(config.decoration.dim_inactive),
            dim_strength: Some(config.decoration.dim_strength),
        };

        // Animation settings
        structured.animations = AnimationSettings {
            enabled: Some(config.animations.enabled),
            beziers: config.animations.beziers.clone(),
            animations: config.animations.animations.clone(),
        };

        // Gesture settings
        structured.gestures = GestureSettings {
            workspace_swipe: Some(config.gestures.workspace_swipe),
            workspace_swipe_fingers: Some(config.gestures.workspace_swipe_fingers),
            workspace_swipe_distance: Some(config.gestures.workspace_swipe_distance),
            workspace_swipe_invert: Some(config.gestures.workspace_swipe_invert),
            workspace_swipe_min_speed_to_force: Some(config.gestures.workspace_swipe_min_speed_to_force),
            workspace_swipe_cancel_ratio: Some(config.gestures.workspace_swipe_cancel_ratio),
            workspace_swipe_create_new: Some(config.gestures.workspace_swipe_create_new),
            workspace_swipe_forever: Some(config.gestures.workspace_swipe_forever),
        };

        // Misc settings
        structured.misc = MiscSettings {
            disable_hyprland_logo: Some(config.misc.disable_hyprland_logo),
            disable_splash_rendering: Some(config.misc.disable_splash_rendering),
            mouse_move_enables_dpms: Some(config.misc.mouse_move_enables_dpms),
            key_press_enables_dpms: Some(config.misc.key_press_enables_dpms),
            always_follow_on_dnd: Some(config.misc.always_follow_on_dnd),
            layers_hog_keyboard_focus: Some(config.misc.layers_hog_keyboard_focus),
            animate_manual_resizes: Some(config.misc.animate_manual_resizes),
            animate_mouse_windowdragging: Some(config.misc.animate_mouse_windowdragging),
            disable_autoreload: Some(config.misc.disable_autoreload),
            enable_swallow: Some(config.misc.enable_swallow),
            swallow_regex: Some(config.misc.swallow_regex.clone()),
        };

        // Convert keybinds
        for bind in &config.binds {
            // Parse the bind string to extract components
            // This is a simplified parser - in reality you'd want more robust parsing
            let parts: Vec<&str> = bind.split("->").collect();
            if parts.len() == 2 {
                let key_part = parts[0].trim();
                let command_part = parts[1].trim();
                
                structured.keybinds.push(KeybindEntry {
                    bind_type: "bind".to_string(),
                    modifiers: vec![], // Would need proper parsing
                    key: key_part.to_string(),
                    dispatcher: command_part.to_string(),
                    args: None,
                    description: None,
                });
            }
        }

        // Convert window rules
        for rule in &config.window_rules {
            let parts: Vec<&str> = rule.splitn(2, ',').collect();
            if parts.len() == 2 {
                structured.window_rules.push(WindowRuleEntry {
                    rule: parts[0].trim().to_string(),
                    window_identifier: parts[1].trim().to_string(),
                    description: None,
                });
            }
        }

        // Convert layer rules
        for rule in &config.layer_rules {
            let parts: Vec<&str> = rule.splitn(2, ',').collect();
            if parts.len() == 2 {
                structured.layer_rules.push(LayerRuleEntry {
                    rule: parts[0].trim().to_string(),
                    layer: parts[1].trim().to_string(),
                    description: None,
                });
            }
        }

        structured.touch();
        Ok(structured)
    }

    /// Extract current configuration from hyprctl
    async fn extract_current_config(&self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<StructuredConfig> {
        let options = hyprctl.get_all_options().await?;
        let binds = hyprctl.get_binds().await?;
        let window_rules = hyprctl.get_window_rules().await?;
        let layer_rules = hyprctl.get_layer_rules().await?;

        let mut config = StructuredConfig::new("Current Configuration");

        // Extract general settings
        config.general.gaps_in = options.get("general:gaps_in").and_then(|v| v.parse().ok());
        config.general.gaps_out = options.get("general:gaps_out").and_then(|v| v.parse().ok());
        config.general.border_size = options.get("general:border_size").and_then(|v| v.parse().ok());
        config.general.col_active_border = options.get("general:col.active_border").cloned();
        config.general.col_inactive_border = options.get("general:col.inactive_border").cloned();

        // Extract keybinds
        for bind in &binds {
            config.keybinds.push(KeybindEntry {
                bind_type: bind.bind_type.clone(),
                modifiers: bind.modifiers.clone(),
                key: bind.key.clone(),
                dispatcher: bind.dispatcher.clone(),
                args: bind.args.clone(),
                description: None,
            });
        }

        // Extract window rules
        for rule in &window_rules {
            let parts: Vec<&str> = rule.splitn(2, ',').collect();
            if parts.len() == 2 {
                config.window_rules.push(WindowRuleEntry {
                    rule: parts[0].trim().to_string(),
                    window_identifier: parts[1].trim().to_string(),
                    description: None,
                });
            }
        }

        // Extract layer rules
        for rule in &layer_rules {
            let parts: Vec<&str> = rule.splitn(2, ',').collect();
            if parts.len() == 2 {
                config.layer_rules.push(LayerRuleEntry {
                    rule: parts[0].trim().to_string(),
                    layer: parts[1].trim().to_string(),
                    description: None,
                });
            }
        }

        config.touch();
        Ok(config)
    }

    // Format conversion methods

    fn to_hyprland_conf(&self, config: &StructuredConfig) -> Result<String> {
        let mut conf = String::new();
        
        // Header
        conf.push_str(&format!("# {} - Generated by r-hyprconfig\n", config.metadata.name));
        conf.push_str(&format!("# Generated on: {}\n", config.metadata.updated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        conf.push_str("\n");

        // General settings
        if let Some(gaps_in) = config.general.gaps_in {
            conf.push_str(&format!("general:gaps_in = {}\n", gaps_in));
        }
        if let Some(gaps_out) = config.general.gaps_out {
            conf.push_str(&format!("general:gaps_out = {}\n", gaps_out));
        }
        if let Some(border_size) = config.general.border_size {
            conf.push_str(&format!("general:border_size = {}\n", border_size));
        }
        if let Some(ref color) = config.general.col_active_border {
            conf.push_str(&format!("general:col.active_border = {}\n", color));
        }
        if let Some(ref color) = config.general.col_inactive_border {
            conf.push_str(&format!("general:col.inactive_border = {}\n", color));
        }

        // Input settings
        conf.push_str("\n# Input configuration\n");
        if let Some(ref layout) = config.input.kb_layout {
            conf.push_str(&format!("input:kb_layout = {}\n", layout));
        }
        if let Some(follow_mouse) = config.input.follow_mouse {
            conf.push_str(&format!("input:follow_mouse = {}\n", follow_mouse));
        }
        if let Some(sensitivity) = config.input.sensitivity {
            conf.push_str(&format!("input:sensitivity = {}\n", sensitivity));
        }

        // Decoration settings
        conf.push_str("\n# Decoration\n");
        if let Some(rounding) = config.decoration.rounding {
            conf.push_str(&format!("decoration:rounding = {}\n", rounding));
        }
        if let Some(blur_enabled) = config.decoration.blur_enabled {
            conf.push_str(&format!("decoration:blur:enabled = {}\n", blur_enabled));
        }

        // Animations
        if let Some(enabled) = config.animations.enabled {
            conf.push_str(&format!("\nanimations:enabled = {}\n", enabled));
        }

        // Keybinds
        if !config.keybinds.is_empty() {
            conf.push_str("\n# Keybinds\n");
            for keybind in &config.keybinds {
                let modifiers = if keybind.modifiers.is_empty() {
                    String::new()
                } else {
                    format!("{}, ", keybind.modifiers.join(" + "))
                };
                
                let args = if let Some(ref args) = keybind.args {
                    format!(", {}", args)
                } else {
                    String::new()
                };

                conf.push_str(&format!(
                    "{} = {}{}, {}{}\n",
                    keybind.bind_type, modifiers, keybind.key, keybind.dispatcher, args
                ));
            }
        }

        // Window rules
        if !config.window_rules.is_empty() {
            conf.push_str("\n# Window rules\n");
            for rule in &config.window_rules {
                conf.push_str(&format!("windowrule = {}, {}\n", rule.rule, rule.window_identifier));
            }
        }

        // Layer rules
        if !config.layer_rules.is_empty() {
            conf.push_str("\n# Layer rules\n");
            for rule in &config.layer_rules {
                conf.push_str(&format!("layerrule = {}, {}\n", rule.rule, rule.layer));
            }
        }

        Ok(conf)
    }

    fn to_json(&self, config: &StructuredConfig) -> Result<String> {
        serde_json::to_string_pretty(config)
            .context("Failed to serialize config to JSON")
    }

    fn to_toml(&self, config: &StructuredConfig) -> Result<String> {
        toml::to_string_pretty(config)
            .context("Failed to serialize config to TOML")
    }

    fn to_yaml(&self, config: &StructuredConfig) -> Result<String> {
        serde_yaml::to_string(config)
            .context("Failed to serialize config to YAML")
    }

    fn to_rhypr_config(&self, config: &StructuredConfig) -> Result<String> {
        // Custom format with metadata and structured content
        let mut content = String::new();
        
        // Metadata header
        content.push_str("# r-hyprconfig format\n");
        content.push_str(&format!("# Name: {}\n", config.metadata.name));
        if let Some(ref desc) = config.metadata.description {
            content.push_str(&format!("# Description: {}\n", desc));
        }
        if let Some(ref author) = config.metadata.author {
            content.push_str(&format!("# Author: {}\n", author));
        }
        content.push_str(&format!("# Version: {}\n", config.metadata.version));
        content.push_str(&format!("# Created: {}\n", config.metadata.created_at.format("%Y-%m-%d %H:%M:%S UTC")));
        content.push_str(&format!("# Updated: {}\n", config.metadata.updated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        
        if !config.metadata.tags.is_empty() {
            content.push_str(&format!("# Tags: {}\n", config.metadata.tags.join(", ")));
        }
        
        content.push_str("\n");
        
        // JSON content
        content.push_str(&self.to_json(config)?);
        
        Ok(content)
    }

    fn to_nix_home_manager(&self, config: &StructuredConfig) -> Result<String> {
        use crate::nixos::{NixConfigGenerator, NixGenerationOptions, NixConfigType};
        
        let generator = NixConfigGenerator::new();
        let options = NixGenerationOptions {
            config_type: NixConfigType::HomeManager,
            module_name: config.metadata.name.clone(),
            ..Default::default()
        };

        // Convert structured config to HyprlandConfig for NixOS generator
        let hyprland_config = self.structured_to_hyprland_config(config)?;
        let generated = generator.generate_from_config(&hyprland_config, &options)?;
        
        Ok(generated.content)
    }

    fn to_nix_system(&self, config: &StructuredConfig) -> Result<String> {
        use crate::nixos::{NixConfigGenerator, NixGenerationOptions, NixConfigType};
        
        let generator = NixConfigGenerator::new();
        let options = NixGenerationOptions {
            config_type: NixConfigType::SystemConfig,
            module_name: config.metadata.name.clone(),
            ..Default::default()
        };

        let hyprland_config = self.structured_to_hyprland_config(config)?;
        let generated = generator.generate_from_config(&hyprland_config, &options)?;
        
        Ok(generated.content)
    }

    fn structured_to_hyprland_config(&self, config: &StructuredConfig) -> Result<HyprlandConfig> {
        // Convert structured config back to HyprlandConfig for NixOS generator
        let general = crate::hyprctl::GeneralConfig {
            gaps_in: config.general.gaps_in.unwrap_or(5),
            gaps_out: config.general.gaps_out.unwrap_or(20),
            border_size: config.general.border_size.unwrap_or(2),
            col_active_border: config.general.col_active_border.clone().unwrap_or_else(|| "rgba(33ccffee)".to_string()),
            col_inactive_border: config.general.col_inactive_border.clone().unwrap_or_else(|| "rgba(595959aa)".to_string()),
            resize_on_border: config.general.resize_on_border.unwrap_or(false),
            extend_border_grab_area: config.general.extend_border_grab_area.unwrap_or(15),
            hover_icon_on_border: config.general.hover_icon_on_border.unwrap_or(true),
        };

        let binds = config.keybinds.iter().map(|kb| {
            format!("{} {} -> {}", kb.modifiers.join(" + "), kb.key, kb.dispatcher)
        }).collect();

        let window_rules = config.window_rules.iter().map(|wr| {
            format!("{}, {}", wr.rule, wr.window_identifier)
        }).collect();

        let layer_rules = config.layer_rules.iter().map(|lr| {
            format!("{}, {}", lr.rule, lr.layer)
        }).collect();

        Ok(HyprlandConfig {
            general,
            input: Default::default(),
            decoration: Default::default(),
            animations: Default::default(),
            gestures: Default::default(),
            binds,
            window_rules,
            layer_rules,
            misc: Default::default(),
        })
    }
}

impl Default for ConfigExporter {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self::with_output_dir(PathBuf::from("./exports")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_exporter_new() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let exporter = ConfigExporter::with_output_dir(temp_dir.path().to_path_buf());
        assert_eq!(exporter.output_dir, temp_dir.path());
        Ok(())
    }

    #[test] 
    fn test_export_structured_config_json() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let exporter = ConfigExporter::with_output_dir(temp_dir.path().to_path_buf());
        
        let config = StructuredConfig::new("Test Config");
        let output_path = exporter.export_structured_config(&config, "test", ConfigFormat::Json)?;
        
        assert!(output_path.exists());
        assert_eq!(output_path.extension().unwrap(), "json");
        
        let content = std::fs::read_to_string(&output_path)?;
        assert!(content.contains("Test Config"));
        
        Ok(())
    }

    #[test]
    fn test_export_structured_config_toml() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let exporter = ConfigExporter::with_output_dir(temp_dir.path().to_path_buf());
        
        let config = StructuredConfig::new("Test Config");
        let output_path = exporter.export_structured_config(&config, "test", ConfigFormat::Toml)?;
        
        assert!(output_path.exists());
        assert_eq!(output_path.extension().unwrap(), "toml");
        
        Ok(())
    }

    #[test]
    fn test_to_hyprland_conf() -> Result<()> {
        let exporter = ConfigExporter::with_output_dir(PathBuf::from("/tmp"));
        let mut config = StructuredConfig::new("Test Config");
        
        // Add some test settings
        config.general.gaps_in = Some(10);
        config.general.gaps_out = Some(20);
        config.input.kb_layout = Some("us".to_string());
        
        // Add a keybind
        config.keybinds.push(KeybindEntry {
            bind_type: "bind".to_string(),
            modifiers: vec!["SUPER".to_string()],
            key: "Return".to_string(),
            dispatcher: "exec".to_string(),
            args: Some("kitty".to_string()),
            description: None,
        });
        
        let conf = exporter.to_hyprland_conf(&config)?;
        
        assert!(conf.contains("Test Config"));
        assert!(conf.contains("general:gaps_in = 10"));
        assert!(conf.contains("general:gaps_out = 20"));
        assert!(conf.contains("input:kb_layout = us"));
        assert!(conf.contains("bind = SUPER, Return, exec, kitty"));
        
        Ok(())
    }

    #[test]
    fn test_to_json() -> Result<()> {
        let exporter = ConfigExporter::with_output_dir(PathBuf::from("/tmp"));
        let config = StructuredConfig::new("Test Config");
        
        let json = exporter.to_json(&config)?;
        assert!(json.contains("Test Config"));
        assert!(json.contains("\"metadata\""));
        
        Ok(())
    }

    #[test]
    fn test_to_rhypr_config() -> Result<()> {
        let exporter = ConfigExporter::with_output_dir(PathBuf::from("/tmp"));
        let mut config = StructuredConfig::new("Test Config");
        config.metadata.description = Some("A test configuration".to_string());
        config.add_tag("test");
        
        let rhypr = exporter.to_rhypr_config(&config)?;
        assert!(rhypr.contains("# r-hyprconfig format"));
        assert!(rhypr.contains("# Name: Test Config"));
        assert!(rhypr.contains("# Description: A test configuration"));
        assert!(rhypr.contains("# Tags: test"));
        assert!(rhypr.contains("\"metadata\""));
        
        Ok(())
    }
}