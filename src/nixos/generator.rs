use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::hyprctl::HyprlandConfig;
use crate::platform::{ConfigPathManager, DistributionDetector, DistributionType};
use super::{NixConfigType, ConfigConverter};

// Allow dead code for NixOS generation functionality that will be used by TUI in future
#[allow(dead_code)]
/// Options for NixOS configuration generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixGenerationOptions {
    pub config_type: NixConfigType,
    pub output_path: Option<PathBuf>,
    pub module_name: String,
    pub enable_wayland: bool,
    pub enable_nvidia: bool,
    pub include_programs: bool,
    pub include_systemd_services: bool,
    pub custom_imports: Vec<String>,
}

impl Default for NixGenerationOptions {
    fn default() -> Self {
        Self {
            config_type: NixConfigType::HomeManager,
            output_path: None,
            module_name: "hyprland".to_string(),
            enable_wayland: true,
            enable_nvidia: false,
            include_programs: true,
            include_systemd_services: true,
            custom_imports: Vec::new(),
        }
    }
}

/// Enhanced NixOS configuration generator
pub struct NixConfigGenerator {
    converter: ConfigConverter,
}

impl NixConfigGenerator {
    pub fn new() -> Self {
        Self {
            converter: ConfigConverter::new(),
        }
    }

    /// Generate NixOS configuration from current system state
    pub async fn generate_from_system(
        &self,
        options: &NixGenerationOptions,
    ) -> Result<GeneratedNixConfig> {
        // Detect if we're on NixOS
        let distribution = DistributionDetector::detect()?;
        if distribution.distribution_type != DistributionType::NixOS {
            anyhow::bail!(
                "NixOS configuration generation is only supported on NixOS systems. Detected: {}",
                distribution.distribution_type
            );
        }

        // Get current hyprland configuration
        let hyprctl = crate::hyprctl::HyprCtl::new().await?;
        let current_config = self.extract_current_config(&hyprctl).await?;

        self.generate_config(&current_config, options)
    }

    /// Generate NixOS configuration from provided Hyprland config
    pub fn generate_from_config(
        &self,
        config: &HyprlandConfig,
        options: &NixGenerationOptions,
    ) -> Result<GeneratedNixConfig> {
        self.generate_config(config, options)
    }

    /// Generate configuration with advanced template support
    fn generate_config(
        &self,
        config: &HyprlandConfig,
        options: &NixGenerationOptions,
    ) -> Result<GeneratedNixConfig> {
        let settings = self.extract_settings(config);
        let keybinds = self.extract_keybinds(config);
        let window_rules = config.window_rules.clone();
        let layer_rules = config.layer_rules.clone();

        let nix_content = match &options.config_type {
            NixConfigType::HomeManager => {
                self.generate_enhanced_home_manager(&settings, &keybinds, &window_rules, &layer_rules, options)?
            }
            NixConfigType::SystemConfig => {
                self.generate_enhanced_system_config(&settings, &keybinds, &window_rules, &layer_rules, options)?
            }
            NixConfigType::FlakeHomeManager => {
                self.generate_enhanced_flake_home_manager(&settings, &keybinds, &window_rules, &layer_rules, options)?
            }
            NixConfigType::FlakeSystem => {
                self.generate_enhanced_flake_system(&settings, &keybinds, &window_rules, &layer_rules, options)?
            }
        };

        let output_path = self.determine_output_path(options)?;

        Ok(GeneratedNixConfig {
            content: nix_content,
            config_type: options.config_type.clone(),
            output_path,
            module_name: options.module_name.clone(),
            metadata: GenerationMetadata {
                generated_at: chrono::Utc::now(),
                source_distribution: DistributionDetector::detect()?.distribution_type,
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                options: options.clone(),
            },
        })
    }

    /// Generate enhanced Home Manager configuration
    fn generate_enhanced_home_manager(
        &self,
        settings: &HashMap<String, String>,
        keybinds: &[String],
        window_rules: &[String],
        layer_rules: &[String],
        options: &NixGenerationOptions,
    ) -> Result<String> {
        let mut nix_config = String::new();

        // File header with metadata
        nix_config.push_str(&self.generate_header(&options.module_name)?);

        // Imports section
        nix_config.push_str("{ config, lib, pkgs, ... }:\n\n");
        nix_config.push_str("{\n");

        if !options.custom_imports.is_empty() {
            nix_config.push_str("  imports = [\n");
            for import in &options.custom_imports {
                nix_config.push_str(&format!("    {}\n", import));
            }
            nix_config.push_str("  ];\n\n");
        }

        // Wayland environment
        if options.enable_wayland {
            nix_config.push_str(&self.generate_wayland_config()?);
        }

        // Programs section
        if options.include_programs {
            nix_config.push_str(&self.generate_programs_config(options)?);
        }

        // Main Hyprland configuration
        nix_config.push_str("  wayland.windowManager.hyprland = {\n");
        nix_config.push_str("    enable = true;\n");
        
        if options.enable_nvidia {
            nix_config.push_str("    enableNvidiaPatches = true;\n");
        }

        nix_config.push_str("    settings = {\n");
        
        // General settings
        nix_config.push_str(&self.generate_nix_settings(settings)?);
        
        // Keybinds
        if !keybinds.is_empty() {
            nix_config.push_str("      bind = [\n");
            for keybind in keybinds {
                nix_config.push_str(&format!("        \"{}\"\n", self.escape_nix_string(keybind)));
            }
            nix_config.push_str("      ];\n");
        }

        // Window rules
        if !window_rules.is_empty() {
            nix_config.push_str("      windowrule = [\n");
            for rule in window_rules {
                nix_config.push_str(&format!("        \"{}\"\n", self.escape_nix_string(rule)));
            }
            nix_config.push_str("      ];\n");
        }

        // Layer rules
        if !layer_rules.is_empty() {
            nix_config.push_str("      layerrule = [\n");
            for rule in layer_rules {
                nix_config.push_str(&format!("        \"{}\"\n", self.escape_nix_string(rule)));
            }
            nix_config.push_str("      ];\n");
        }

        nix_config.push_str("    };\n");
        nix_config.push_str("  };\n");

        // Systemd services
        if options.include_systemd_services {
            nix_config.push_str(&self.generate_systemd_services()?);
        }

        nix_config.push_str("}\n");

        Ok(nix_config)
    }

    /// Generate enhanced system configuration
    fn generate_enhanced_system_config(
        &self,
        _settings: &HashMap<String, String>,
        _keybinds: &[String],
        _window_rules: &[String],
        _layer_rules: &[String],
        options: &NixGenerationOptions,
    ) -> Result<String> {
        let mut nix_config = String::new();

        nix_config.push_str(&self.generate_header(&options.module_name)?);
        nix_config.push_str("{ config, lib, pkgs, ... }:\n\n");
        nix_config.push_str("{\n");

        // System-level Hyprland configuration
        nix_config.push_str("  programs.hyprland = {\n");
        nix_config.push_str("    enable = true;\n");
        
        if options.enable_nvidia {
            nix_config.push_str("    enableNvidiaPatches = true;\n");
        }
        
        nix_config.push_str("  };\n\n");

        // Environment variables
        nix_config.push_str("  environment.sessionVariables = {\n");
        if options.enable_wayland {
            nix_config.push_str("    NIXOS_OZONE_WL = \"1\";\n");
            nix_config.push_str("    MOZ_ENABLE_WAYLAND = \"1\";\n");
        }
        if options.enable_nvidia {
            nix_config.push_str("    LIBVA_DRIVER_NAME = \"nvidia\";\n");
            nix_config.push_str("    GBM_BACKEND = \"nvidia-drm\";\n");
            nix_config.push_str("    __GLX_VENDOR_LIBRARY_NAME = \"nvidia\";\n");
        }
        nix_config.push_str("  };\n\n");

        // Hardware configuration
        if options.enable_nvidia {
            nix_config.push_str(&self.generate_nvidia_config()?);
        }

        // Default configuration hint
        nix_config.push_str("  # Note: User-specific Hyprland configuration should be managed through Home Manager\n");
        nix_config.push_str("  # This system configuration only enables Hyprland system-wide\n");

        nix_config.push_str("}\n");

        Ok(nix_config)
    }

    /// Generate flake-based home manager configuration
    fn generate_enhanced_flake_home_manager(
        &self,
        settings: &HashMap<String, String>,
        keybinds: &[String],
        window_rules: &[String],
        layer_rules: &[String],
        options: &NixGenerationOptions,
    ) -> Result<String> {
        let mut nix_config = String::new();

        nix_config.push_str(&self.generate_header(&options.module_name)?);
        nix_config.push_str("{\n");
        nix_config.push_str("  inputs = {\n");
        nix_config.push_str("    nixpkgs.url = \"github:NixOS/nixpkgs/nixos-unstable\";\n");
        nix_config.push_str("    home-manager = {\n");
        nix_config.push_str("      url = \"github:nix-community/home-manager\";\n");
        nix_config.push_str("      inputs.nixpkgs.follows = \"nixpkgs\";\n");
        nix_config.push_str("    };\n");
        nix_config.push_str("    hyprland.url = \"github:hyprwm/Hyprland\";\n");
        nix_config.push_str("  };\n\n");

        nix_config.push_str("  outputs = { nixpkgs, home-manager, hyprland, ... }:\n");
        nix_config.push_str("    let\n");
        nix_config.push_str("      system = \"x86_64-linux\";\n");
        nix_config.push_str("      pkgs = nixpkgs.legacyPackages.${system};\n");
        nix_config.push_str("    in {\n");
        nix_config.push_str("      homeConfigurations.\"$USER\" = home-manager.lib.homeManagerConfiguration {\n");
        nix_config.push_str("        inherit pkgs;\n");
        nix_config.push_str("        modules = [\n");
        nix_config.push_str("          hyprland.homeManagerModules.default\n");
        nix_config.push_str("          {\n");

        // Include the home manager configuration inline
        let home_config = self.generate_enhanced_home_manager(settings, keybinds, window_rules, layer_rules, options)?;
        // Remove the outer wrapper and add proper indentation
        let lines: Vec<&str> = home_config.lines().collect();
        for line in lines.iter().skip(3).take(lines.len() - 4) { // Skip header and wrapper
            nix_config.push_str("            ");
            nix_config.push_str(line);
            nix_config.push('\n');
        }

        nix_config.push_str("          }\n");
        nix_config.push_str("        ];\n");
        nix_config.push_str("      };\n");
        nix_config.push_str("    };\n");
        nix_config.push_str("}\n");

        Ok(nix_config)
    }

    /// Generate flake-based system configuration
    fn generate_enhanced_flake_system(
        &self,
        settings: &HashMap<String, String>,
        keybinds: &[String],
        window_rules: &[String],
        layer_rules: &[String],
        options: &NixGenerationOptions,
    ) -> Result<String> {
        let mut nix_config = String::new();

        nix_config.push_str(&self.generate_header(&options.module_name)?);
        nix_config.push_str("{\n");
        nix_config.push_str("  inputs = {\n");
        nix_config.push_str("    nixpkgs.url = \"github:NixOS/nixpkgs/nixos-unstable\";\n");
        nix_config.push_str("    hyprland.url = \"github:hyprwm/Hyprland\";\n");
        nix_config.push_str("  };\n\n");

        nix_config.push_str("  outputs = { nixpkgs, hyprland, ... }:\n");
        nix_config.push_str("    {\n");
        nix_config.push_str("      nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {\n");
        nix_config.push_str("        system = \"x86_64-linux\";\n");
        nix_config.push_str("        modules = [\n");
        nix_config.push_str("          hyprland.nixosModules.default\n");
        nix_config.push_str("          {\n");

        // Include the system configuration inline
        let system_config = self.generate_enhanced_system_config(settings, keybinds, window_rules, layer_rules, options)?;
        let lines: Vec<&str> = system_config.lines().collect();
        for line in lines.iter().skip(3).take(lines.len() - 4) {
            nix_config.push_str("            ");
            nix_config.push_str(line);
            nix_config.push('\n');
        }

        nix_config.push_str("          }\n");
        nix_config.push_str("        ];\n");
        nix_config.push_str("      };\n");
        nix_config.push_str("    };\n");
        nix_config.push_str("}\n");

        Ok(nix_config)
    }

    // Helper methods

    fn generate_header(&self, module_name: &str) -> Result<String> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        Ok(format!(
            "# {module_name} - Generated by r-hyprconfig\n# Generated on: {timestamp}\n# Generator version: {}\n\n",
            env!("CARGO_PKG_VERSION")
        ))
    }

    fn generate_wayland_config(&self) -> Result<String> {
        Ok(r#"  # Wayland environment configuration
  home.sessionVariables = {
    NIXOS_OZONE_WL = "1";
    MOZ_ENABLE_WAYLAND = "1";
    XDG_CURRENT_DESKTOP = "Hyprland";
    XDG_SESSION_TYPE = "wayland";
    XDG_SESSION_DESKTOP = "Hyprland";
  };

"#.to_string())
    }

    fn generate_programs_config(&self, options: &NixGenerationOptions) -> Result<String> {
        let mut config = String::new();
        
        config.push_str("  # Essential Wayland programs\n");
        config.push_str("  programs = {\n");
        config.push_str("    waybar.enable = true;\n");
        config.push_str("    rofi.enable = true;\n");
        config.push_str("    kitty.enable = true;\n");
        
        if options.enable_nvidia {
            config.push_str("    # NVIDIA-specific program configurations\n");
        }
        
        config.push_str("  };\n\n");
        
        Ok(config)
    }

    fn generate_nix_settings(&self, settings: &HashMap<String, String>) -> Result<String> {
        let mut nix_settings = String::new();

        // Group settings by category
        let categories = [
            ("general", "General settings"),
            ("input", "Input configuration"), 
            ("decoration", "Visual decoration"),
            ("animations", "Animation settings"),
            ("gestures", "Gesture configuration"),
            ("misc", "Miscellaneous settings"),
        ];

        for (category, description) in &categories {
            let category_settings: HashMap<String, String> = settings
                .iter()
                .filter(|(key, _)| key.starts_with(&format!("{}:", category)))
                .map(|(key, value)| (key.strip_prefix(&format!("{}:", category)).unwrap().to_string(), value.clone()))
                .collect();

            if !category_settings.is_empty() {
                nix_settings.push_str(&format!("      # {}\n", description));
                nix_settings.push_str(&format!("      {} = {{\n", category));
                
                for (key, value) in &category_settings {
                    let nix_value = self.convert_value_to_nix(value)?;
                    nix_settings.push_str(&format!("        {} = {};\n", key, nix_value));
                }
                
                nix_settings.push_str("      };\n");
            }
        }

        Ok(nix_settings)
    }

    fn generate_systemd_services(&self) -> Result<String> {
        Ok(r#"
  # Systemd user services for Hyprland
  systemd.user.services = {
    hyprland-autostart = {
      description = "Hyprland autostart applications";
      partOf = [ "hyprland-session.target" ];
      after = [ "hyprland-session.target" ];
      wantedBy = [ "hyprland-session.target" ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
      };
    };
  };

"#.to_string())
    }

    fn generate_nvidia_config(&self) -> Result<String> {
        Ok(r#"  # NVIDIA configuration for Hyprland
  hardware.nvidia = {
    modesetting.enable = true;
    powerManagement.enable = false;
    powerManagement.finegrained = false;
    open = false;
    nvidiaSettings = true;
  };

  hardware.opengl = {
    enable = true;
    driSupport = true;
    driSupport32Bit = true;
  };

"#.to_string())
    }

    fn convert_value_to_nix(&self, value: &str) -> Result<String> {
        // Handle different value types for Nix
        if value == "true" || value == "false" || value.parse::<i32>().is_ok() || value.parse::<f32>().is_ok() {
            Ok(value.to_string())
        } else if value.starts_with("rgba(") || value.starts_with("rgb(") {
            Ok(format!("\"{}\"", value))
        } else {
            // String value
            Ok(format!("\"{}\"", self.escape_nix_string(value)))
        }
    }

    fn escape_nix_string(&self, value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\t', "\\t")
    }

    fn determine_output_path(&self, options: &NixGenerationOptions) -> Result<PathBuf> {
        if let Some(ref path) = options.output_path {
            return Ok(path.clone());
        }

        let paths = ConfigPathManager::get_paths()?;
        let filename = match &options.config_type {
            NixConfigType::HomeManager => format!("{}-home.nix", options.module_name),
            NixConfigType::SystemConfig => format!("{}-system.nix", options.module_name),
            NixConfigType::FlakeHomeManager => "flake-home.nix".to_string(),
            NixConfigType::FlakeSystem => "flake.nix".to_string(),
        };

        Ok(paths.exports_dir.join(filename))
    }

    async fn extract_current_config(&self, hyprctl: &crate::hyprctl::HyprCtl) -> Result<HyprlandConfig> {
        let options = hyprctl.get_all_options().await?;
        let binds = hyprctl.get_binds().await?;
        let window_rules = hyprctl.get_window_rules().await?;
        let layer_rules = hyprctl.get_layer_rules().await?;

        // Convert options to structured config
        Ok(HyprlandConfig {
            general: crate::hyprctl::GeneralConfig {
                gaps_in: options.get("general:gaps_in").and_then(|v| v.parse().ok()).unwrap_or(5),
                gaps_out: options.get("general:gaps_out").and_then(|v| v.parse().ok()).unwrap_or(20),
                border_size: options.get("general:border_size").and_then(|v| v.parse().ok()).unwrap_or(2),
                col_active_border: options.get("general:col.active_border").cloned().unwrap_or_else(|| "rgba(33ccffee)".to_string()),
                col_inactive_border: options.get("general:col.inactive_border").cloned().unwrap_or_else(|| "rgba(595959aa)".to_string()),
                resize_on_border: options.get("general:resize_on_border").map(|v| v == "true").unwrap_or(false),
                extend_border_grab_area: options.get("general:extend_border_grab_area").and_then(|v| v.parse().ok()).unwrap_or(15),
                hover_icon_on_border: options.get("general:hover_icon_on_border").map(|v| v == "true").unwrap_or(true),
            },
            input: Default::default(),
            decoration: Default::default(),
            animations: Default::default(),
            gestures: Default::default(),
            binds: binds.iter().map(|b| b.display_string()).collect(),
            window_rules,
            layer_rules,
            misc: Default::default(),
        })
    }

    fn extract_settings(&self, config: &HyprlandConfig) -> HashMap<String, String> {
        let mut settings = HashMap::new();

        // General settings
        settings.insert("general:gaps_in".to_string(), config.general.gaps_in.to_string());
        settings.insert("general:gaps_out".to_string(), config.general.gaps_out.to_string());
        settings.insert("general:border_size".to_string(), config.general.border_size.to_string());
        settings.insert("general:col.active_border".to_string(), config.general.col_active_border.clone());
        settings.insert("general:col.inactive_border".to_string(), config.general.col_inactive_border.clone());

        // Add other categories as needed...

        settings
    }

    fn extract_keybinds(&self, config: &HyprlandConfig) -> Vec<String> {
        config.binds.clone()
    }
}

/// Result of NixOS configuration generation
#[derive(Debug, Clone)]
pub struct GeneratedNixConfig {
    pub content: String,
    pub config_type: NixConfigType,
    pub output_path: PathBuf,
    pub module_name: String,
    pub metadata: GenerationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub source_distribution: DistributionType,
    pub generator_version: String,
    pub options: NixGenerationOptions,
}

impl GeneratedNixConfig {
    /// Save the generated configuration to disk
    pub fn save(&self) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        std::fs::write(&self.output_path, &self.content)
            .with_context(|| format!("Failed to write config to: {:?}", self.output_path))?;

        Ok(())
    }

    /// Get a preview of the generated content (first 20 lines)
    pub fn preview(&self) -> String {
        self.content
            .lines()
            .take(20)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_nix_generation_options_default() {
        let options = NixGenerationOptions::default();
        assert_eq!(options.config_type, NixConfigType::HomeManager);
        assert_eq!(options.module_name, "hyprland");
        assert!(options.enable_wayland);
        assert!(!options.enable_nvidia);
    }

    #[test]
    fn test_nix_config_generator_creation() {
        let _generator = NixConfigGenerator::new();
        // Test that generator can be created without errors
    }

    #[test]
    fn test_convert_value_to_nix() -> Result<()> {
        let generator = NixConfigGenerator::new();
        
        assert_eq!(generator.convert_value_to_nix("true")?, "true");
        assert_eq!(generator.convert_value_to_nix("false")?, "false");
        assert_eq!(generator.convert_value_to_nix("42")?, "42");
        assert_eq!(generator.convert_value_to_nix("3.14")?, "3.14");
        assert_eq!(generator.convert_value_to_nix("rgba(255,255,255,1)")?, "\"rgba(255,255,255,1)\"");
        assert_eq!(generator.convert_value_to_nix("kitty")?, "\"kitty\"");
        
        Ok(())
    }

    #[test]
    fn test_escape_nix_string() {
        let generator = NixConfigGenerator::new();
        
        assert_eq!(generator.escape_nix_string("simple"), "simple");
        assert_eq!(generator.escape_nix_string("with\"quote"), "with\\\"quote");
        assert_eq!(generator.escape_nix_string("with\\backslash"), "with\\\\backslash");
        assert_eq!(generator.escape_nix_string("with\nnewline"), "with\\nnewline");
    }

    #[test]
    fn test_generate_header() -> Result<()> {
        let generator = NixConfigGenerator::new();
        let header = generator.generate_header("test-module")?;
        
        assert!(header.contains("test-module"));
        assert!(header.contains("Generated by r-hyprconfig"));
        assert!(header.contains("Generated on:"));
        
        Ok(())
    }

    #[test]
    fn test_determine_output_path() -> Result<()> {
        let generator = NixConfigGenerator::new();
        let temp_dir = TempDir::new()?;
        
        let options = NixGenerationOptions {
            config_type: NixConfigType::HomeManager,
            output_path: Some(temp_dir.path().join("custom.nix")),
            module_name: "test".to_string(),
            ..Default::default()
        };
        
        let path = generator.determine_output_path(&options)?;
        assert_eq!(path, temp_dir.path().join("custom.nix"));
        
        Ok(())
    }

    #[test]
    fn test_generated_nix_config_save() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("test.nix");
        
        let config = GeneratedNixConfig {
            content: "# Test content\n{ config, lib, pkgs, ... }: {\n  # Test\n}\n".to_string(),
            config_type: NixConfigType::HomeManager,
            output_path: output_path.clone(),
            module_name: "test".to_string(),
            metadata: GenerationMetadata {
                generated_at: chrono::Utc::now(),
                source_distribution: DistributionType::NixOS,
                generator_version: "1.0.0".to_string(),
                options: NixGenerationOptions::default(),
            },
        };
        
        config.save()?;
        
        assert!(output_path.exists());
        let content = std::fs::read_to_string(&output_path)?;
        assert!(content.contains("Test content"));
        
        Ok(())
    }

    #[test]
    fn test_generated_nix_config_preview() {
        let config = GeneratedNixConfig {
            content: (0..30).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"),
            config_type: NixConfigType::HomeManager,
            output_path: PathBuf::from("/tmp/test.nix"),
            module_name: "test".to_string(),
            metadata: GenerationMetadata {
                generated_at: chrono::Utc::now(),
                source_distribution: DistributionType::NixOS,
                generator_version: "1.0.0".to_string(),
                options: NixGenerationOptions::default(),
            },
        };
        
        let preview = config.preview();
        let lines: Vec<&str> = preview.lines().collect();
        assert_eq!(lines.len(), 20);
        assert_eq!(lines[0], "Line 0");
        assert_eq!(lines[19], "Line 19");
    }
}