pub mod detection;
pub mod parser;
pub mod converter;
pub mod config_discovery;

pub use converter::*;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NixConfigType {
    SystemConfig,      // /etc/nixos/configuration.nix
    HomeManager,       // ~/.config/nixpkgs/home.nix
    FlakeHomeManager,  // flake-based home manager
    FlakeSystem,       // flake-based system config
}

impl std::fmt::Display for NixConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NixConfigType::SystemConfig => write!(f, "System Configuration"),
            NixConfigType::HomeManager => write!(f, "Home Manager"),
            NixConfigType::FlakeHomeManager => write!(f, "Flake-based Home Manager"),
            NixConfigType::FlakeSystem => write!(f, "Flake-based System"),
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct NixOSEnvironment {
    pub is_nixos: bool,
    #[allow(dead_code)]
    pub nix_store_path: Option<PathBuf>,
    #[allow(dead_code)]
    pub nixos_version: Option<String>,
    #[allow(dead_code)]
    pub has_home_manager: bool,
    pub config_locations: Vec<NixConfigLocation>,
}

#[derive(Debug, Clone)]
pub struct NixConfigLocation {
    pub config_type: NixConfigType,
    pub path: PathBuf,
    pub exists: bool,
    pub has_hyprland_config: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NixOSConfig {
    pub config_type: NixConfigType,
    pub file_path: PathBuf,
    pub hyprland_settings: HashMap<String, String>,
    pub keybinds: Vec<String>,
    pub window_rules: Vec<String>,
    pub layer_rules: Vec<String>,
}


impl NixOSEnvironment {
    pub fn detect() -> Self {
        detection::detect_nixos_environment()
    }
    
    pub fn get_primary_config_location(&self) -> Option<&NixConfigLocation> {
        // Prefer locations with existing Hyprland config
        self.config_locations
            .iter()
            .find(|loc| loc.has_hyprland_config)
            .or_else(|| {
                // Fallback to first existing config file
                self.config_locations
                    .iter()
                    .find(|loc| loc.exists)
            })
    }
    
    #[allow(dead_code)]
    pub fn supports_hyprland(&self) -> bool {
        self.is_nixos && !self.config_locations.is_empty()
    }
}