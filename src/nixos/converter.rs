// Configuration format converter module
// Converts between traditional Hyprland config and NixOS Nix expressions

use std::collections::HashMap;
use super::{NixOSConfig, NixConfigType};

pub struct ConfigConverter;

impl ConfigConverter {
    pub fn new() -> Self {
        Self
    }
    
    // Convert traditional hyprland.conf format to NixOS format
    pub fn traditional_to_nixos(
        &self,
        traditional_config: &HashMap<String, String>,
        keybinds: &[String],
        window_rules: &[String],
        layer_rules: &[String],
        target_type: NixConfigType,
    ) -> Result<String, anyhow::Error> {
        match target_type {
            NixConfigType::HomeManager => {
                self.generate_home_manager_config(traditional_config, keybinds, window_rules, layer_rules)
            }
            NixConfigType::SystemConfig => {
                self.generate_system_config(traditional_config, keybinds, window_rules, layer_rules)
            }
            NixConfigType::FlakeHomeManager => {
                self.generate_flake_home_manager_config(traditional_config, keybinds, window_rules, layer_rules)
            }
            NixConfigType::FlakeSystem => {
                self.generate_flake_system_config(traditional_config, keybinds, window_rules, layer_rules)
            }
        }
    }
    
    // Convert NixOS config back to traditional format
    #[allow(dead_code, clippy::type_complexity)]
    pub fn nixos_to_traditional(
        &self,
        nixos_config: &NixOSConfig,
    ) -> Result<(HashMap<String, String>, Vec<String>, Vec<String>, Vec<String>), anyhow::Error> {
        let mut traditional_config = HashMap::new();
        let mut keybinds = Vec::new();
        let mut window_rules = Vec::new();
        let mut layer_rules = Vec::new();
        
        // Convert NixOS settings back to traditional format
        for (key, value) in &nixos_config.hyprland_settings {
            traditional_config.insert(key.clone(), value.clone());
        }
        
        // Convert keybinds
        for keybind in &nixos_config.keybinds {
            keybinds.push(self.nix_keybind_to_traditional(keybind));
        }
        
        // Convert window rules  
        for rule in &nixos_config.window_rules {
            window_rules.push(self.nix_rule_to_traditional(rule, "windowrule"));
        }
        
        // Convert layer rules
        for rule in &nixos_config.layer_rules {
            layer_rules.push(self.nix_rule_to_traditional(rule, "layerrule"));
        }
        
        Ok((traditional_config, keybinds, window_rules, layer_rules))
    }
    
    // Generate Nix configuration template
    #[allow(dead_code)]
    pub fn generate_template(&self, config_type: NixConfigType) -> String {
        match config_type {
            NixConfigType::HomeManager => self.generate_home_manager_template(),
            NixConfigType::SystemConfig => self.generate_system_config_template(),
            NixConfigType::FlakeHomeManager => self.generate_flake_home_manager_template(),
            NixConfigType::FlakeSystem => self.generate_flake_system_template(),
        }
    }
    
    fn generate_home_manager_config(
        &self,
        traditional_config: &HashMap<String, String>,
        keybinds: &[String],
        window_rules: &[String],
        layer_rules: &[String],
    ) -> Result<String, anyhow::Error> {
        let mut nix_config = String::new();
        
        nix_config.push_str("{ config, pkgs, ... }: {\n");
        nix_config.push_str("  wayland.windowManager.hyprland = {\n");
        nix_config.push_str("    enable = true;\n");
        nix_config.push_str("    settings = {\n");
        
        // Convert traditional config sections to Nix attribute sets
        let config_sections = self.group_config_by_section(traditional_config);
        
        for (section, options) in config_sections {
            if !options.is_empty() {
                nix_config.push_str(&format!("      {section} = {{\n"));
                for (key, value) in options {
                    let nix_value = self.convert_value_to_nix(&value);
                    nix_config.push_str(&format!("        {key} = {nix_value};\n"));
                }
                nix_config.push_str("      };\n");
            }
        }
        
        // Add keybinds
        if !keybinds.is_empty() {
            nix_config.push_str("      bind = [\n");
            for keybind in keybinds {
                let clean_bind = self.convert_keybind_to_nix(keybind);
                nix_config.push_str(&format!("        \"{clean_bind}\"\n"));
            }
            nix_config.push_str("      ];\n");
        }
        
        // Add window rules
        if !window_rules.is_empty() {
            nix_config.push_str("      windowrule = [\n");
            for rule in window_rules {
                let clean_rule = self.convert_rule_to_nix(rule);
                nix_config.push_str(&format!("        \"{clean_rule}\"\n"));
            }
            nix_config.push_str("      ];\n");
        }
        
        // Add layer rules
        if !layer_rules.is_empty() {
            nix_config.push_str("      layerrule = [\n");
            for rule in layer_rules {
                let clean_rule = self.convert_rule_to_nix(rule);
                nix_config.push_str(&format!("        \"{clean_rule}\"\n"));
            }
            nix_config.push_str("      ];\n");
        }
        
        nix_config.push_str("    };\n");
        nix_config.push_str("  };\n");
        nix_config.push_str("}\n");
        
        Ok(nix_config)
    }

    #[allow(dead_code)]
    fn generate_home_manager_template(&self) -> String {
        r#"{ config, pkgs, ... }: {
  wayland.windowManager.hyprland = {
    enable = true;
    settings = {
      general = {
        # gaps_in = 5;
        # gaps_out = 10;
      };
      decoration = {
        # rounding = 8;
      };
      bind = [
        # "SUPER, Q, exec, kitty"
        # "SUPER, W, killactive"
      ];
      windowrule = [
        # "float, ^(kitty)$"
      ];
    };
  };
}"#.to_string()
    }
    
    #[allow(dead_code)]
    fn generate_system_config_template(&self) -> String {
        r#"{ config, pkgs, ... }: {
  programs.hyprland = {
    enable = true;
    # Additional system-level Hyprland configuration
  };
}"#.to_string()
    }
    
    #[allow(dead_code)]
    fn generate_flake_home_manager_template(&self) -> String {
        r#"{
  description = "Home Manager flake with Hyprland";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    hyprland.url = "github:hyprwm/Hyprland";
  };
  
  outputs = { nixpkgs, home-manager, hyprland, ... }: {
    homeConfigurations."user" = home-manager.lib.homeManagerConfiguration {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      modules = [
        hyprland.homeManagerModules.default
        {
          wayland.windowManager.hyprland = {
            enable = true;
            settings = {
              # Your Hyprland configuration here
            };
          };
        }
      ];
    };
  };
}"#.to_string()
    }
    
    #[allow(dead_code)]
    fn generate_flake_system_template(&self) -> String {
        r#"{
  description = "NixOS flake with Hyprland";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    hyprland.url = "github:hyprwm/Hyprland";
  };
  
  outputs = { nixpkgs, hyprland, ... }: {
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      modules = [
        hyprland.nixosModules.default
        {
          programs.hyprland.enable = true;
          # Additional system configuration
        }
      ];
    };
  };
}"#.to_string()
    }
    
    // Helper methods for conversion
    fn group_config_by_section(&self, config: &HashMap<String, String>) -> HashMap<String, Vec<(String, String)>> {
        let mut sections: HashMap<String, Vec<(String, String)>> = HashMap::new();
        
        for (key, value) in config {
            let (section, option) = if key.contains(':') {
                let parts: Vec<&str> = key.splitn(2, ':').collect();
                (parts[0].to_string(), parts[1].to_string())
            } else {
                // If no section prefix, assume it's general
                ("general".to_string(), key.clone())
            };
            
            sections.entry(section).or_default().push((option, value.clone()));
        }
        
        sections
    }
    
    fn convert_value_to_nix(&self, value: &str) -> String {
        // Handle different value types for Nix
        if value == "true" || value == "false" {
            // Boolean values don't need quotes
            value.to_string()
        } else if value.parse::<i32>().is_ok() || value.parse::<f32>().is_ok() {
            // Numeric values don't need quotes
            value.to_string()
        } else if value.starts_with("rgb(") || value.starts_with("rgba(") {
            // Color values - keep as string but format nicely
            format!("\"{value}\"")
        } else {
            // String values need quotes
            format!("\"{}\"", self.escape_nix_string(value))
        }
    }
    
    fn convert_keybind_to_nix(&self, keybind: &str) -> String {
        // Convert from "bind = SUPER, Q, exec, kitty" to "SUPER, Q, exec, kitty"
        if keybind.starts_with("bind = ") {
            keybind.strip_prefix("bind = ").unwrap_or(keybind).to_string()
        } else if keybind.contains(" → ") {
            // Convert from display format "SUPER + Q → exec [kitty]" to config format
            self.display_keybind_to_config(keybind)
        } else {
            keybind.to_string()
        }
    }
    
    fn convert_rule_to_nix(&self, rule: &str) -> String {
        // Convert from "windowrule = float, ^(kitty)$" to "float, ^(kitty)$"
        if rule.starts_with("windowrule = ") {
            rule.strip_prefix("windowrule = ").unwrap_or(rule).to_string()
        } else if rule.starts_with("layerrule = ") {
            rule.strip_prefix("layerrule = ").unwrap_or(rule).to_string()
        } else {
            rule.to_string()
        }
    }
    
    fn display_keybind_to_config(&self, display_keybind: &str) -> String {
        // Convert "SUPER + Q → exec [kitty]" to "SUPER, Q, exec, kitty"
        let parts: Vec<&str> = display_keybind.split(" → ").collect();
        if parts.len() == 2 {
            let modifiers_and_key = parts[0].replace(" + ", ", ");
            let action_part = parts[1];
            
            // Remove brackets from individual arguments like "exec [kitty]" -> "exec kitty"
            let cleaned_action = action_part.replace(['[', ']'], "");
            let action_and_args = cleaned_action.replace(' ', ", ");
            format!("{modifiers_and_key}, {action_and_args}")
        } else {
            display_keybind.to_string()
        }
    }
    
    fn escape_nix_string(&self, s: &str) -> String {
        // Escape special characters for Nix strings
        s.replace('\\', "\\\\")
         .replace('"', "\\\"")
         .replace('\n', "\\n")
         .replace('\t', "\\t")
    }
    
    // Generate other config types
    fn generate_system_config(
        &self,
        _traditional_config: &HashMap<String, String>,
        _keybinds: &[String],
        _window_rules: &[String],
        _layer_rules: &[String],
    ) -> Result<String, anyhow::Error> {
        // System config typically just enables Hyprland, user config goes in Home Manager
        Ok(r#"{ config, pkgs, ... }: {
  programs.hyprland = {
    enable = true;
    # User configuration should be in Home Manager
  };
}"#.to_string())
    }
    
    fn generate_flake_home_manager_config(
        &self,
        traditional_config: &HashMap<String, String>,
        keybinds: &[String],
        window_rules: &[String],
        layer_rules: &[String],
    ) -> Result<String, anyhow::Error> {
        let home_manager_config = self.generate_home_manager_config(
            traditional_config, keybinds, window_rules, layer_rules
        )?;
        
        // Wrap in flake structure
        let flake_config = format!(r#"{{
  description = "Home Manager flake with Hyprland configuration";
  
  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {{
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
    hyprland.url = "github:hyprwm/Hyprland";
  }};
  
  outputs = {{ nixpkgs, home-manager, hyprland, ... }}: {{
    homeConfigurations."${{USER}}" = home-manager.lib.homeManagerConfiguration {{
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      modules = [
        hyprland.homeManagerModules.default
        ({home_manager_config})
      ];
    }};
  }};
}}"#);
        
        Ok(flake_config)
    }
    
    fn generate_flake_system_config(
        &self,
        _traditional_config: &HashMap<String, String>,
        _keybinds: &[String],
        _window_rules: &[String],
        _layer_rules: &[String],
    ) -> Result<String, anyhow::Error> {
        Ok(r#"{
  description = "NixOS flake with Hyprland";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    hyprland.url = "github:hyprwm/Hyprland";
  };
  
  outputs = { nixpkgs, hyprland, ... }: {
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      modules = [
        hyprland.nixosModules.default
        {
          programs.hyprland.enable = true;
          # User configuration should be in Home Manager
        }
      ];
    };
  };
}"#.to_string())
    }
    
    // Helper methods for reverse conversion (NixOS → Traditional)
    #[allow(dead_code)]
    fn nix_keybind_to_traditional(&self, keybind: &str) -> String {
        // Convert from "SUPER, Q, exec, kitty" to "bind = SUPER, Q, exec, kitty"
        format!("bind = {keybind}")
    }
    
    #[allow(dead_code)]
    fn nix_rule_to_traditional(&self, rule: &str, rule_type: &str) -> String {
        // Convert from "float, ^(kitty)$" to "windowrule = float, ^(kitty)$"
        format!("{rule_type} = {rule}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_traditional_to_nixos_conversion() {
        let converter = ConfigConverter::new();
        
        // Test data
        let mut traditional_config = HashMap::new();
        traditional_config.insert("general:gaps_in".to_string(), "5".to_string());
        traditional_config.insert("general:gaps_out".to_string(), "10".to_string());
        traditional_config.insert("decoration:rounding".to_string(), "8".to_string());
        
        let keybinds = vec![
            "SUPER, Q, exec, kitty".to_string(),
            "SUPER, W, killactive".to_string(),
        ];
        
        let window_rules = vec![
            "float, ^(kitty)$".to_string(),
        ];
        
        let layer_rules = vec![
            "blur, waybar".to_string(),
        ];
        
        // Convert to NixOS Home Manager format
        let result = converter.traditional_to_nixos(
            &traditional_config,
            &keybinds,
            &window_rules,
            &layer_rules,
            NixConfigType::HomeManager,
        );
        
        assert!(result.is_ok());
        let nix_config = result.unwrap();
        
        // Verify the structure
        assert!(nix_config.contains("wayland.windowManager.hyprland"));
        assert!(nix_config.contains("enable = true"));
        assert!(nix_config.contains("settings = {"));
        assert!(nix_config.contains("general = {"));
        assert!(nix_config.contains("gaps_in = 5"));
        assert!(nix_config.contains("gaps_out = 10"));
        assert!(nix_config.contains("decoration = {"));
        assert!(nix_config.contains("rounding = 8"));
        assert!(nix_config.contains("bind = ["));
        assert!(nix_config.contains("\"SUPER, Q, exec, kitty\""));
        assert!(nix_config.contains("windowrule = ["));
        assert!(nix_config.contains("\"float, ^(kitty)$\""));
        assert!(nix_config.contains("layerrule = ["));
        assert!(nix_config.contains("\"blur, waybar\""));
    }
    
    #[test]
    fn test_value_conversion() {
        let converter = ConfigConverter::new();
        
        // Test different value types
        assert_eq!(converter.convert_value_to_nix("true"), "true");
        assert_eq!(converter.convert_value_to_nix("false"), "false");
        assert_eq!(converter.convert_value_to_nix("42"), "42");
        assert_eq!(converter.convert_value_to_nix("3.14"), "3.14");
        assert_eq!(converter.convert_value_to_nix("rgb(255, 0, 0)"), "\"rgb(255, 0, 0)\"");
        assert_eq!(converter.convert_value_to_nix("hello world"), "\"hello world\"");
    }
    
    #[test]
    fn test_keybind_conversion() {
        let converter = ConfigConverter::new();
        
        // Test different keybind formats
        assert_eq!(
            converter.convert_keybind_to_nix("bind = SUPER, Q, exec, kitty"),
            "SUPER, Q, exec, kitty"
        );
        
        assert_eq!(
            converter.convert_keybind_to_nix("SUPER, W, killactive"),
            "SUPER, W, killactive"
        );
        
        assert_eq!(
            converter.display_keybind_to_config("SUPER + Q → exec [kitty]"),
            "SUPER, Q, exec, kitty"
        );
    }
}