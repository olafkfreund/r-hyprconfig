use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hyprland_config_path: PathBuf,
    pub backup_enabled: bool,
    pub auto_save: bool,
    pub nixos_mode: bool,
    pub current_values: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hyprland_config_path: Self::default_hyprland_config_path(),
            backup_enabled: true,
            auto_save: false,
            nixos_mode: Self::detect_nixos(),
            current_values: HashMap::new(),
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            let content = async_fs::read_to_string(&config_path)
                .await
                .context("Failed to read config file")?;
            
            let mut config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            
            // Ensure hyprland config path exists
            config.validate_hyprland_config_path().await?;
            
            Ok(config)
        } else {
            let config = Self::default();
            config.save().await?;
            Ok(config)
        }
    }

    pub async fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        async_fs::write(&config_path, content)
            .await
            .context("Failed to write config file")?;

        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("r-hyprconfig");
        
        Ok(config_dir.join("config.toml"))
    }

    fn default_hyprland_config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("hypr").join("hyprland.conf")
        } else {
            PathBuf::from("~/.config/hypr/hyprland.conf")
        }
    }

    fn detect_nixos() -> bool {
        // Check if we're running on NixOS
        Path::new("/etc/NIXOS").exists() || 
        std::env::var("NIX_STORE").is_ok() ||
        which::which("nixos-rebuild").is_ok()
    }

    async fn validate_hyprland_config_path(&mut self) -> Result<()> {
        if !self.hyprland_config_path.exists() {
            // Try to find hyprland.conf in common locations
            let possible_paths = vec![
                dirs::config_dir().map(|d| d.join("hypr").join("hyprland.conf")),
                Some(PathBuf::from("/etc/hypr/hyprland.conf")),
                dirs::home_dir().map(|d| d.join(".config").join("hypr").join("hyprland.conf")),
            ];

            for path in possible_paths.into_iter().flatten() {
                if path.exists() {
                    self.hyprland_config_path = path;
                    return Ok(());
                }
            }

            // If no config found, create a basic one
            self.create_default_hyprland_config().await?;
        }
        Ok(())
    }

    async fn create_default_hyprland_config(&self) -> Result<()> {
        if let Some(parent) = self.hyprland_config_path.parent() {
            async_fs::create_dir_all(parent)
                .await
                .context("Failed to create hyprland config directory")?;
        }

        let default_config = include_str!("../templates/default_hyprland.conf");
        async_fs::write(&self.hyprland_config_path, default_config)
            .await
            .context("Failed to create default hyprland config")?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn backup_config(&self) -> Result<PathBuf> {
        if !self.backup_enabled {
            return Ok(self.hyprland_config_path.clone());
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = self.hyprland_config_path
            .with_extension(format!("conf.backup.{}", timestamp));

        async_fs::copy(&self.hyprland_config_path, &backup_path)
            .await
            .context("Failed to create backup")?;

        Ok(backup_path)
    }

    #[allow(dead_code)]
    pub async fn save_hyprland_config(&self, options: &HashMap<String, String>) -> Result<()> {
        if self.nixos_mode {
            return self.save_nixos_config(options).await;
        }

        // Backup current config
        let _backup_path = self.backup_config().await?;

        // Read current config
        let current_content = async_fs::read_to_string(&self.hyprland_config_path)
            .await
            .unwrap_or_else(|_| String::new());

        // Parse and update config
        let updated_content = self.update_config_content(&current_content, options)?;

        // Write updated config
        async_fs::write(&self.hyprland_config_path, updated_content)
            .await
            .context("Failed to write hyprland config")?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn save_nixos_config(&self, _options: &HashMap<String, String>) -> Result<()> {
        // For NixOS, we can't directly modify the config file
        // Instead, we'll save the configuration to a separate file
        // that can be imported or referenced in the NixOS configuration
        
        let nixos_config_path = self.hyprland_config_path
            .parent()
            .unwrap_or(Path::new("/tmp"))
            .join("r-hyprconfig-generated.conf");

        let content = self.generate_nixos_config_content(_options)?;
        
        async_fs::write(&nixos_config_path, content)
            .await
            .context("Failed to write NixOS compatible config")?;

        println!("NixOS mode: Configuration saved to {:?}", nixos_config_path);
        println!("Please import this file in your NixOS configuration or copy the settings manually.");

        Ok(())
    }

    #[allow(dead_code)]
    fn update_config_content(&self, content: &str, options: &HashMap<String, String>) -> Result<String> {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut updated_options = HashMap::new();

        // Update existing options - collect indices first to avoid borrow checker issues
        let mut line_updates = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            // Check if this line contains a configuration option we want to update
            for (option, value) in options {
                let option_prefix = if option.contains(':') {
                    option.split_once(':').unwrap().1
                } else {
                    option.as_str()
                };

                if trimmed.starts_with(&format!("{} =", option_prefix)) ||
                   trimmed.starts_with(&format!("{}=", option_prefix)) {
                    line_updates.push((i, format!("    {} = {}", option_prefix, value)));
                    updated_options.insert(option.clone(), value.clone());
                    break;
                }
            }
        }
        
        // Apply the collected updates
        for (i, new_line) in line_updates {
            lines[i] = new_line;
        }

        // Add options that weren't found in the existing config  
        for (option, value) in options {
            if !updated_options.contains_key(option) {
                let _section = if let Some((section, option_name)) = option.split_once(':') {
                    // Find or create section
                    let section_header = format!("{} {{", section);
                    let mut section_found = false;
                    let mut section_end = lines.len();

                    for (i, line) in lines.iter().enumerate() {
                        if line.trim().starts_with(&section_header) {
                            section_found = true;
                            // Find the end of this section
                            for (j, inner_line) in lines.iter().enumerate().skip(i + 1) {
                                if inner_line.trim() == "}" {
                                    section_end = j;
                                    break;
                                }
                            }
                            break;
                        }
                    }

                    if section_found {
                        lines.insert(section_end, format!("    {} = {}", option_name, value));
                    } else {
                        // Create new section
                        lines.push(String::new());
                        lines.push(format!("{} {{", section));
                        lines.push(format!("    {} = {}", option_name, value));
                        lines.push("}".to_string());
                    }
                    section
                } else {
                    // Global option
                    lines.push(format!("{} = {}", option, value));
                    "global"
                };
                
                updated_options.insert(option.clone(), value.clone());
            }
        }

        Ok(lines.join("\n"))
    }

    #[allow(dead_code)]
    fn generate_nixos_config_content(&self, options: &HashMap<String, String>) -> Result<String> {
        let mut content = String::new();
        content.push_str("# Generated by r-hyprconfig for NixOS\n");
        content.push_str("# Import this in your NixOS configuration or copy settings manually\n\n");

        // Group options by section
        let mut sections: HashMap<String, Vec<(String, String)>> = HashMap::new();
        
        for (option, value) in options {
            if let Some((section, option_name)) = option.split_once(':') {
                sections.entry(section.to_string())
                    .or_default()
                    .push((option_name.to_string(), value.clone()));
            } else {
                sections.entry("general".to_string())
                    .or_default()
                    .push((option.clone(), value.clone()));
            }
        }

        // Generate NixOS-style configuration
        for (section, options) in sections {
            content.push_str(&format!("{} = {{\n", section));
            for (option, value) in options {
                content.push_str(&format!("  {} = {};\n", option, value));
            }
            content.push_str("};\n\n");
        }

        Ok(content)
    }

    #[allow(dead_code)]
    pub fn set_current_value(&mut self, key: String, value: String) {
        self.current_values.insert(key, value);
    }

    #[allow(dead_code)]
    pub fn get_current_value(&self, key: &str) -> Option<&String> {
        self.current_values.get(key)
    }

    #[allow(dead_code)]
    pub fn is_nixos_mode(&self) -> bool {
        self.nixos_mode
    }

    #[allow(dead_code)]
    pub fn set_nixos_mode(&mut self, enabled: bool) {
        self.nixos_mode = enabled;
    }

    pub async fn parse_hyprland_config(&self) -> Result<HyprlandConfigFile> {
        let content = async_fs::read_to_string(&self.hyprland_config_path)
            .await
            .context("Failed to read Hyprland config file")?;
        
        Ok(HyprlandConfigFile::parse(&content)?)
    }
}

#[derive(Debug, Clone)]
pub struct HyprlandConfigFile {
    pub keybinds: Vec<ParsedKeybind>,
    pub window_rules: Vec<String>,
    pub layer_rules: Vec<String>,
    pub workspace_rules: Vec<String>,
    #[allow(dead_code)]
    pub options: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ParsedKeybind {
    #[allow(dead_code)]
    pub bind_type: String,  // bind, bindm, binde, etc.
    pub modifiers: String,
    pub key: String,
    pub dispatcher: String,
    pub args: String,
    #[allow(dead_code)]
    pub original_line: String,
}

impl HyprlandConfigFile {
    pub fn parse(content: &str) -> Result<Self> {
        let mut keybinds = Vec::new();
        let mut window_rules = Vec::new();
        let mut layer_rules = Vec::new();
        let mut workspace_rules = Vec::new();
        let mut options = HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse keybinds
            if line.starts_with("bind") {
                if let Some(keybind) = Self::parse_keybind_line(line) {
                    keybinds.push(keybind);
                }
            }
            // Parse window rules
            else if line.starts_with("windowrule") {
                window_rules.push(line.to_string());
            }
            // Parse layer rules
            else if line.starts_with("layerrule") {
                layer_rules.push(line.to_string());
            }
            // Parse blur layer rules (legacy format)
            else if line.starts_with("blurls") {
                // Convert blurls to layerrule format
                if let Some(layer_name) = line.strip_prefix("blurls=") {
                    layer_rules.push(format!("layerrule = blur, {}", layer_name));
                }
            }
            // Parse workspace rules
            else if line.starts_with("workspace") {
                workspace_rules.push(line.to_string());
            }
            // Parse configuration options in sections
            else if line.contains('=') && !line.contains(' ') {
                // Simple key=value pairs
                if let Some((key, value)) = line.split_once('=') {
                    options.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        Ok(Self {
            keybinds,
            window_rules,
            layer_rules,
            workspace_rules,
            options,
        })
    }
    
    fn parse_keybind_line(line: &str) -> Option<ParsedKeybind> {
        // Parse different bind formats:
        // bind = SUPER, N, exec, swaync-client -t -sw
        // bindm = $mainMod, mouse:272, movewindow
        // binde = $mainMod, l, resizeactive, 30 0
        
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let bind_type = parts[0].trim().to_string();
        let bind_content = parts[1].trim();
        
        // Split by commas, but be careful about commas in arguments
        let mut bind_parts = Vec::new();
        let mut current_part = String::new();
        let mut paren_depth = 0;
        
        for ch in bind_content.chars() {
            match ch {
                ',' if paren_depth == 0 => {
                    bind_parts.push(current_part.trim().to_string());
                    current_part.clear();
                }
                '(' | '[' => {
                    paren_depth += 1;
                    current_part.push(ch);
                }
                ')' | ']' => {
                    paren_depth -= 1;
                    current_part.push(ch);
                }
                _ => current_part.push(ch),
            }
        }
        if !current_part.trim().is_empty() {
            bind_parts.push(current_part.trim().to_string());
        }
        
        if bind_parts.len() >= 3 {
            let modifiers = bind_parts[0].clone();
            let key = bind_parts[1].clone();
            let dispatcher = bind_parts[2].clone();
            let args = if bind_parts.len() > 3 {
                bind_parts[3..].join(", ")
            } else {
                String::new()
            };
            
            Some(ParsedKeybind {
                bind_type,
                modifiers,
                key,
                dispatcher,
                args,
                original_line: line.to_string(),
            })
        } else {
            None
        }
    }
}