use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyprlandConfig {
    pub general: GeneralConfig,
    pub input: InputConfig,
    pub decoration: DecorationConfig,
    pub animations: AnimationsConfig,
    pub gestures: GesturesConfig,
    pub binds: Vec<String>,
    pub window_rules: Vec<String>,
    pub layer_rules: Vec<String>,
    pub misc: MiscConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub gaps_in: i32,
    pub gaps_out: i32,
    pub border_size: i32,
    pub col_active_border: String,
    pub col_inactive_border: String,
    pub resize_on_border: bool,
    pub extend_border_grab_area: i32,
    pub hover_icon_on_border: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub kb_layout: String,
    pub kb_variant: String,
    pub kb_model: String,
    pub kb_options: String,
    pub kb_rules: String,
    pub follow_mouse: i32,
    pub mouse_refocus: bool,
    pub sensitivity: f32,
    pub accel_profile: String,
    pub natural_scroll: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecorationConfig {
    pub rounding: i32,
    pub blur_enabled: bool,
    pub blur_size: i32,
    pub blur_passes: i32,
    pub drop_shadow: bool,
    pub shadow_range: i32,
    pub shadow_render_power: i32,
    pub col_shadow: String,
    pub dim_inactive: bool,
    pub dim_strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationsConfig {
    pub enabled: bool,
    pub beziers: Vec<String>,
    pub animations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GesturesConfig {
    pub workspace_swipe: bool,
    pub workspace_swipe_fingers: i32,
    pub workspace_swipe_distance: i32,
    pub workspace_swipe_invert: bool,
    pub workspace_swipe_min_speed_to_force: i32,
    pub workspace_swipe_cancel_ratio: f32,
    pub workspace_swipe_create_new: bool,
    pub workspace_swipe_forever: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiscConfig {
    pub disable_hyprland_logo: bool,
    pub disable_splash_rendering: bool,
    pub mouse_move_enables_dpms: bool,
    pub key_press_enables_dpms: bool,
    pub always_follow_on_dnd: bool,
    pub layers_hog_keyboard_focus: bool,
    pub animate_manual_resizes: bool,
    pub animate_mouse_windowdragging: bool,
    pub disable_autoreload: bool,
    pub enable_swallow: bool,
    pub swallow_regex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyprlandKeybind {
    pub modifiers: Vec<String>,  // SUPER, ALT, CTRL, SHIFT
    pub key: String,             // q, Return, space, etc.
    pub dispatcher: String,      // exec, killactive, togglefloating, etc.
    pub args: Option<String>,    // Command arguments
    pub bind_type: String,       // bind, bindm, bindr, etc.
}

impl HyprlandKeybind {
    pub fn new(modifiers: Vec<String>, key: String, dispatcher: String, args: Option<String>) -> Self {
        Self {
            modifiers,
            key,
            dispatcher,
            args,
            bind_type: "bind".to_string(),
        }
    }

    pub fn to_hyprland_config(&self) -> String {
        let mod_string = if self.modifiers.is_empty() {
            String::new()
        } else {
            format!("{}, ", self.modifiers.join(" + "))
        };
        
        let args_string = if let Some(ref args) = self.args {
            format!(", {}", args)
        } else {
            String::new()
        };
        
        format!("{} = {}{}, {}{}", 
            self.bind_type, 
            mod_string, 
            self.key, 
            self.dispatcher,
            args_string
        )
    }

    pub fn display_string(&self) -> String {
        let mod_string = if self.modifiers.is_empty() {
            String::new()
        } else {
            format!("{} + ", self.modifiers.join(" + "))
        };
        
        let args_string = if let Some(ref args) = self.args {
            format!(" [{}]", args)
        } else {
            String::new()
        };
        
        format!("{}{} → {}{}", mod_string, self.key, self.dispatcher, args_string)
    }
}

pub struct HyprCtl {
    socket_path: Option<String>,
}

impl HyprCtl {
    pub async fn new() -> Result<Self> {
        let mut hyprctl = Self { socket_path: None };
        
        // Try to detect Hyprland socket
        hyprctl.detect_socket().await?;
        
        Ok(hyprctl)
    }

    async fn detect_socket(&mut self) -> Result<()> {
        // Try to get Hyprland instance signature
        let output = AsyncCommand::new("hyprctl")
            .arg("getoption")
            .arg("general:border_size")
            .output()
            .await;

        match output {
            Ok(_) => {
                // hyprctl is available and working
                Ok(())
            }
            Err(e) => {
                anyhow::bail!("Hyprland is not running or hyprctl is not available: {}", e);
            }
        }
    }

    pub async fn get_option(&self, option: &str) -> Result<String> {
        let output = AsyncCommand::new("hyprctl")
            .arg("getoption")
            .arg(option)
            .output()
            .await
            .context("Failed to execute hyprctl getoption")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl getoption failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    pub async fn set_option(&self, option: &str, value: &str) -> Result<()> {
        let output = AsyncCommand::new("hyprctl")
            .arg("keyword")
            .arg(format!("{}:{}", option, value))
            .output()
            .await
            .context("Failed to execute hyprctl keyword")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl keyword failed: {}", stderr);
        }

        Ok(())
    }

    pub async fn get_all_options(&self) -> Result<HashMap<String, String>> {
        let mut options = HashMap::new();
        
        // Get general options
        let general_options = vec![
            "general:gaps_in",
            "general:gaps_out",
            "general:border_size",
            "general:col.active_border",
            "general:col.inactive_border",
            "general:resize_on_border",
            "general:extend_border_grab_area",
            "general:hover_icon_on_border",
        ];

        for option in general_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {}: {}", option, e);
                }
            }
        }

        // Get input options
        let input_options = vec![
            "input:kb_layout",
            "input:kb_variant",
            "input:kb_model",
            "input:kb_options",
            "input:kb_rules",
            "input:follow_mouse",
            "input:mouse_refocus",
            "input:sensitivity",
            "input:accel_profile",
            "input:natural_scroll",
        ];

        for option in input_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {}: {}", option, e);
                }
            }
        }

        // Get decoration options
        let decoration_options = vec![
            "decoration:rounding",
            "decoration:blur:enabled",
            "decoration:blur:size",
            "decoration:blur:passes",
            "decoration:drop_shadow",
            "decoration:shadow_range",
            "decoration:shadow_render_power",
            "decoration:col.shadow",
            "decoration:dim_inactive",
            "decoration:dim_strength",
        ];

        for option in decoration_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {}: {}", option, e);
                }
            }
        }

        // Get animations options
        let animation_options = vec![
            "animations:enabled",
        ];

        for option in animation_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {}: {}", option, e);
                }
            }
        }

        // Get gestures options
        let gesture_options = vec![
            "gestures:workspace_swipe",
            "gestures:workspace_swipe_fingers",
            "gestures:workspace_swipe_distance",
            "gestures:workspace_swipe_invert",
            "gestures:workspace_swipe_min_speed_to_force",
            "gestures:workspace_swipe_cancel_ratio",
            "gestures:workspace_swipe_create_new",
            "gestures:workspace_swipe_forever",
        ];

        for option in gesture_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {}: {}", option, e);
                }
            }
        }

        // Get misc options
        let misc_options = vec![
            "misc:disable_hyprland_logo",
            "misc:disable_splash_rendering",
            "misc:mouse_move_enables_dpms",
            "misc:key_press_enables_dpms",
            "misc:always_follow_on_dnd",
            "misc:layers_hog_keyboard_focus",
            "misc:animate_manual_resizes",
            "misc:animate_mouse_windowdragging",
            "misc:disable_autoreload",
            "misc:enable_swallow",
            "misc:swallow_regex",
        ];

        for option in misc_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {}: {}", option, e);
                }
            }
        }

        Ok(options)
    }

    pub async fn get_binds(&self) -> Result<Vec<HyprlandKeybind>> {
        let output = AsyncCommand::new("hyprctl")
            .arg("binds")
            .output()
            .await
            .context("Failed to execute hyprctl binds")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl binds failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut binds = Vec::new();
        
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            // Parse hyprctl binds output format
            // Example: "bind: SUPER + Q -> exec [kitty]"
            if let Some(keybind) = Self::parse_bind_line(line) {
                binds.push(keybind);
            }
        }

        Ok(binds)
    }

    fn parse_bind_line(line: &str) -> Option<HyprlandKeybind> {
        // hyprctl binds output format: "modmask,key -> dispatcher [arg]"
        // Example: "64,q -> exec [kitty]"
        
        if let Some((key_part, command_part)) = line.split_once(" -> ") {
            let key_part = key_part.trim();
            let command_part = command_part.trim();
            
            // Parse modifiers and key
            let (modifiers, key) = if let Some((mods, k)) = key_part.split_once(',') {
                (Self::parse_modifiers(mods), k.to_string())
            } else {
                (vec![], key_part.to_string())
            };
            
            // Parse command and args
            let (dispatcher, args) = if let Some((disp, arg_part)) = command_part.split_once(' ') {
                let args = if arg_part.starts_with('[') && arg_part.ends_with(']') {
                    arg_part.trim_start_matches('[').trim_end_matches(']').to_string()
                } else {
                    arg_part.to_string()
                };
                (disp.to_string(), Some(args))
            } else {
                (command_part.to_string(), None)
            };
            
            return Some(HyprlandKeybind {
                modifiers,
                key,
                dispatcher,
                args,
                bind_type: "bind".to_string(), // Default, could be enhanced
            });
        }
        
        None
    }
    
    fn parse_modifiers(mod_mask: &str) -> Vec<String> {
        // Convert numeric modifier mask to readable modifiers
        if let Ok(mask) = mod_mask.parse::<u32>() {
            let mut mods = Vec::new();
            if mask & 64 != 0 { mods.push("SUPER".to_string()); }  // Mod4
            if mask & 8 != 0 { mods.push("ALT".to_string()); }     // Mod1
            if mask & 4 != 0 { mods.push("CTRL".to_string()); }    // Control
            if mask & 1 != 0 { mods.push("SHIFT".to_string()); }   // Shift
            mods
        } else {
            vec![]
        }
    }

    pub async fn add_keybind(&self, bind: &HyprlandKeybind) -> Result<()> {
        let bind_command = bind.to_hyprland_config();
        self.dispatch(&format!("keyword {}", bind_command)).await
    }

    pub async fn remove_keybind(&self, modifiers: &[String], key: &str) -> Result<()> {
        let mod_string = if modifiers.is_empty() {
            String::new()
        } else {
            format!("{}_", modifiers.join("_"))
        };
        
        let unbind_command = format!("unbind {}{}", mod_string, key);
        self.dispatch(&unbind_command).await
    }

    pub async fn reload_config(&self) -> Result<()> {
        let output = AsyncCommand::new("hyprctl")
            .arg("reload")
            .output()
            .await
            .context("Failed to execute hyprctl reload")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl reload failed: {}", stderr);
        }

        Ok(())
    }

    pub async fn dispatch(&self, command: &str) -> Result<()> {
        let output = AsyncCommand::new("hyprctl")
            .arg("dispatch")
            .arg(command)
            .output()
            .await
            .context("Failed to execute hyprctl dispatch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl dispatch failed: {}", stderr);
        }

        Ok(())
    }

    pub async fn get_version(&self) -> Result<String> {
        let output = AsyncCommand::new("hyprctl")
            .arg("version")
            .output()
            .await
            .context("Failed to execute hyprctl version")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl version failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    pub async fn is_hyprland_running(&self) -> bool {
        AsyncCommand::new("hyprctl")
            .arg("version")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub async fn get_workspace_rules(&self) -> Result<Vec<String>> {
        let output = AsyncCommand::new("hyprctl")
            .arg("workspacerules")
            .output()
            .await
            .context("Failed to execute hyprctl workspacerules")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl workspacerules failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let rules: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(rules)
    }

    pub async fn get_window_rules(&self) -> Result<Vec<String>> {
        // Window rules are typically parsed from the config file or through clients command
        // For now, we'll return a placeholder that shows current window classes
        let output = AsyncCommand::new("hyprctl")
            .arg("clients")
            .arg("-j")
            .output()
            .await
            .context("Failed to execute hyprctl clients")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl clients failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse JSON to extract window classes for rule suggestions
        let mut window_classes = std::collections::HashSet::new();
        
        // Try to parse as JSON, but fallback to text parsing if it fails
        if let Ok(clients) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(clients_array) = clients.as_array() {
                for client in clients_array {
                    if let Some(class) = client.get("class").and_then(|c| c.as_str()) {
                        if !class.is_empty() {
                            window_classes.insert(format!("class:^({})$", class));
                        }
                    }
                    if let Some(title) = client.get("title").and_then(|t| t.as_str()) {
                        if !title.is_empty() && title.len() > 3 {
                            window_classes.insert(format!("title:^({})$", title));
                        }
                    }
                }
            }
        }

        // Convert to vector and add some common window rule examples
        let mut rules: Vec<String> = window_classes.into_iter().collect();
        rules.sort();
        
        // Add some example rules if no windows are found
        if rules.is_empty() {
            rules = vec![
                "windowrule = float, ^(kitty)$".to_string(),
                "windowrule = size 800 600, ^(floating-app)$".to_string(),
                "windowrule = workspace 2, ^(firefox)$".to_string(),
                "windowrule = opacity 0.8, ^(terminal)$".to_string(),
            ];
        }

        Ok(rules)
    }

    pub async fn get_layer_rules(&self) -> Result<Vec<String>> {
        let output = AsyncCommand::new("hyprctl")
            .arg("layers")
            .output()
            .await
            .context("Failed to execute hyprctl layers")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl layers failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse layer information and create layer rule suggestions
        let mut layer_names = std::collections::HashSet::new();
        
        for line in stdout.lines() {
            let line = line.trim();
            if line.contains("namespace:") {
                if let Some(namespace) = line.split("namespace: ").nth(1) {
                    let namespace = namespace.split_whitespace().next().unwrap_or(namespace);
                    layer_names.insert(namespace.to_string());
                }
            }
        }

        // Convert to layer rule suggestions
        let mut rules: Vec<String> = layer_names
            .into_iter()
            .map(|name| format!("layerrule = blur, {}", name))
            .collect();
        
        rules.sort();
        
        // Add common layer rule examples if no layers found
        if rules.is_empty() {
            rules = vec![
                "layerrule = blur, waybar".to_string(),
                "layerrule = ignorezero, waybar".to_string(),
                "layerrule = noanim, wallpaper".to_string(),
                "layerrule = blur, notifications".to_string(),
            ];
        }

        Ok(rules)
    }

    pub async fn add_window_rule(&self, rule: &str) -> Result<()> {
        let command = format!("keyword windowrule {}", rule);
        self.dispatch(&command).await
    }

    pub async fn add_layer_rule(&self, rule: &str) -> Result<()> {
        let command = format!("keyword layerrule {}", rule);
        self.dispatch(&command).await
    }

    pub async fn add_workspace_rule(&self, rule: &str) -> Result<()> {
        let command = format!("keyword workspace {}", rule);
        self.dispatch(&command).await
    }
}