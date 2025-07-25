#![allow(dead_code)]

use super::{NixConfigLocation, NixConfigType, NixOSConfig};
use std::collections::HashMap;
use std::fs;

pub fn load_nixos_config(location: &NixConfigLocation) -> Result<NixOSConfig, anyhow::Error> {
    let content = fs::read_to_string(&location.path)?;

    match location.config_type {
        NixConfigType::SystemConfig | NixConfigType::HomeManager => {
            parse_traditional_nix_config(&content, location)
        }
        NixConfigType::FlakeHomeManager | NixConfigType::FlakeSystem => {
            parse_flake_config(&content, location)
        }
    }
}

fn parse_traditional_nix_config(
    content: &str,
    location: &NixConfigLocation,
) -> Result<NixOSConfig, anyhow::Error> {
    let mut config = NixOSConfig {
        config_type: location.config_type.clone(),
        file_path: location.path.clone(),
        hyprland_settings: HashMap::new(),
        keybinds: Vec::new(),
        window_rules: Vec::new(),
        layer_rules: Vec::new(),
    };

    // Parse Hyprland settings from Nix configuration
    if let Some(hyprland_section) = extract_hyprland_section(content) {
        parse_hyprland_settings(&hyprland_section, &mut config)?;
    }

    Ok(config)
}

fn parse_flake_config(
    content: &str,
    location: &NixConfigLocation,
) -> Result<NixOSConfig, anyhow::Error> {
    let mut config = NixOSConfig {
        config_type: location.config_type.clone(),
        file_path: location.path.clone(),
        hyprland_settings: HashMap::new(),
        keybinds: Vec::new(),
        window_rules: Vec::new(),
        layer_rules: Vec::new(),
    };

    // For flakes, we need to look in the outputs section
    if let Some(outputs_section) = extract_flake_outputs(content) {
        if let Some(hyprland_section) = extract_hyprland_section(&outputs_section) {
            parse_hyprland_settings(&hyprland_section, &mut config)?;
        }
    }

    Ok(config)
}

fn extract_hyprland_section(content: &str) -> Option<String> {
    // Look for different Hyprland configuration patterns
    let patterns = [
        "wayland.windowManager.hyprland",
        "programs.hyprland",
        "services.hyprland",
    ];

    for pattern in &patterns {
        if let Some(section) = extract_nix_section(content, pattern) {
            return Some(section);
        }
    }

    None
}

fn extract_nix_section(content: &str, section_name: &str) -> Option<String> {
    // Find the start of the section
    let start_pattern = format!("{section_name} =");
    if let Some(start_pos) = content.find(&start_pattern) {
        // Find the opening brace
        if let Some(brace_pos) = content[start_pos..].find('{') {
            let absolute_brace_pos = start_pos + brace_pos;

            // Find the matching closing brace
            if let Some(end_pos) = find_matching_brace(&content[absolute_brace_pos..]) {
                let section_content =
                    &content[absolute_brace_pos..absolute_brace_pos + end_pos + 1];
                return Some(section_content.to_string());
            }
        }
    }

    None
}

fn find_matching_brace(content: &str) -> Option<usize> {
    let mut brace_count = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in content.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => brace_count += 1,
            '}' if !in_string => {
                brace_count -= 1;
                if brace_count == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

fn extract_flake_outputs(content: &str) -> Option<String> {
    // Look for the outputs section in a flake
    if let Some(outputs_section) = extract_nix_section(content, "outputs") {
        return Some(outputs_section);
    }

    // Alternative: look for specific output attributes
    let output_patterns = [
        "nixosConfigurations",
        "homeConfigurations",
        "homeManagerConfigurations",
    ];

    for pattern in &output_patterns {
        if let Some(section) = extract_nix_section(content, pattern) {
            return Some(section);
        }
    }

    None
}

fn parse_hyprland_settings(content: &str, config: &mut NixOSConfig) -> Result<(), anyhow::Error> {
    // Parse various Hyprland configuration patterns

    // Parse settings attribute set
    if let Some(settings_section) = extract_nix_section(content, "settings") {
        parse_settings_attribute_set(&settings_section, config)?;
    }

    // Parse individual configuration items
    parse_individual_settings(content, config)?;

    Ok(())
}

fn parse_settings_attribute_set(
    content: &str,
    config: &mut NixOSConfig,
) -> Result<(), anyhow::Error> {
    // Parse nested attribute sets like:
    // settings = {
    //   general = {
    //     gaps_in = 5;
    //   };
    //   bind = [ "SUPER, Q, exec, kitty" ];
    // };

    // This is a simplified parser - for production, we'd want to use a proper Nix parser
    let lines: Vec<&str> = content.lines().collect();
    let mut current_section = String::new();

    for line in lines {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Detect section headers
        if trimmed.ends_with(" = {") {
            current_section = trimmed.replace(" = {", "");
            continue;
        }

        // Parse key-value pairs
        if let Some(eq_pos) = trimmed.find('=') {
            let key_part = trimmed[..eq_pos].trim();
            let value_part = trimmed[eq_pos + 1..].trim().trim_end_matches(';');

            // Build full key name
            let full_key = if current_section.is_empty() {
                key_part.to_string()
            } else {
                format!("{current_section}:{key_part}")
            };

            // Handle different value types
            if key_part == "bind" || key_part == "windowrule" || key_part == "layerrule" {
                parse_array_value(value_part, key_part, config)?;
            } else {
                // Regular setting
                let clean_value = clean_nix_value(value_part);
                config.hyprland_settings.insert(full_key, clean_value);
            }
        }

        // Reset section on closing brace
        if trimmed == "}" {
            current_section.clear();
        }
    }

    Ok(())
}

fn parse_individual_settings(content: &str, config: &mut NixOSConfig) -> Result<(), anyhow::Error> {
    // Parse settings that are defined individually like:
    // extraConfig = ''
    //   bind = SUPER, Q, exec, kitty
    // '';

    if let Some(extra_config) = extract_extra_config(content) {
        // Parse as traditional hyprland.conf format
        parse_traditional_hyprland_config(&extra_config, config)?;
    }

    Ok(())
}

fn extract_extra_config(content: &str) -> Option<String> {
    // Look for extraConfig sections with multi-line strings
    if let Some(start) = content.find("extraConfig = ''") {
        if let Some(end) = content[start..].find("'';") {
            let config_content = &content[start + 16..start + end]; // Skip "extraConfig = ''"
            return Some(config_content.to_string());
        }
    }

    None
}

fn parse_traditional_hyprland_config(
    content: &str,
    config: &mut NixOSConfig,
) -> Result<(), anyhow::Error> {
    // Parse traditional hyprland.conf format within extraConfig
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("bind ") {
            config.keybinds.push(trimmed.to_string());
        } else if trimmed.starts_with("windowrule ") {
            config.window_rules.push(trimmed.to_string());
        } else if trimmed.starts_with("layerrule ") {
            config.layer_rules.push(trimmed.to_string());
        } else if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let value = trimmed[eq_pos + 1..].trim().to_string();
            config.hyprland_settings.insert(key, value);
        }
    }

    Ok(())
}

fn parse_array_value(
    value: &str,
    key: &str,
    config: &mut NixOSConfig,
) -> Result<(), anyhow::Error> {
    // Parse Nix array syntax: [ "item1" "item2" ]
    let cleaned = value.trim_start_matches('[').trim_end_matches(']').trim();

    // Split by quoted strings
    let items = extract_quoted_items(cleaned);

    match key {
        "bind" => config.keybinds.extend(items),
        "windowrule" => config.window_rules.extend(items),
        "layerrule" => config.layer_rules.extend(items),
        _ => {}
    }

    Ok(())
}

fn extract_quoted_items(content: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current_item = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;

    for ch in content.chars() {
        if escape_next {
            current_item.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => escape_next = true,
            '"' => {
                if in_quotes {
                    // End of quoted string
                    items.push(current_item.trim().to_string());
                    current_item.clear();
                }
                in_quotes = !in_quotes;
            }
            _ if in_quotes => current_item.push(ch),
            _ => {} // Ignore whitespace and other characters outside quotes
        }
    }

    items
}

fn clean_nix_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hyprland_section() {
        let config = r#"
        { config, pkgs, ... }: {
          wayland.windowManager.hyprland = {
            enable = true;
            settings = {
              general = {
                gaps_in = 5;
              };
            };
          };
        }
        "#;

        let section = extract_hyprland_section(config);
        assert!(section.is_some());
        assert!(section.unwrap().contains("enable = true"));
    }

    #[test]
    fn test_find_matching_brace() {
        let content = "{ inner { nested } more }";
        let end_pos = find_matching_brace(content);
        assert_eq!(end_pos, Some(content.len() - 1));
    }

    #[test]
    fn test_extract_quoted_items() {
        let content = r#""SUPER, Q, exec, kitty" "SUPER, W, killactive""#;
        let items = extract_quoted_items(content);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], "SUPER, Q, exec, kitty");
        assert_eq!(items[1], "SUPER, W, killactive");
    }
}
