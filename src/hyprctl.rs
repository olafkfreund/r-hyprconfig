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

    pub async fn get_binds(&self) -> Result<Vec<String>> {
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
        let binds: Vec<String> = stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        Ok(binds)
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
}