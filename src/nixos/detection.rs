use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use super::{NixOSEnvironment, NixConfigLocation, NixConfigType};

pub fn detect_nixos_environment() -> NixOSEnvironment {
    let is_nixos = is_nixos_system();
    
    if is_nixos {
        NixOSEnvironment {
            is_nixos,
            nix_store_path: detect_nix_store(),
            nixos_version: detect_nixos_version(),
            has_home_manager: detect_home_manager(),
            config_locations: discover_config_locations(),
        }
    } else {
        NixOSEnvironment {
            is_nixos,
            ..Default::default()
        }
    }
}

fn is_nixos_system() -> bool {
    // Method 1: Check for /etc/NIXOS file (most reliable)
    if Path::new("/etc/NIXOS").exists() {
        return true;
    }
    
    // Method 2: Check for NIX_STORE environment variable
    if std::env::var("NIX_STORE").is_ok() {
        return true;
    }
    
    // Method 3: Check if nixos-rebuild command exists
    if Command::new("which")
        .arg("nixos-rebuild")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return true;
    }
    
    // Method 4: Check /nix/store directory
    if Path::new("/nix/store").exists() {
        return true;
    }
    
    // Method 5: Check for nix-env command (indicates Nix is installed)
    if Command::new("which")
        .arg("nix-env")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return true;
    }
    
    false
}

fn detect_nix_store() -> Option<PathBuf> {
    // Check environment variable first
    if let Ok(store_path) = std::env::var("NIX_STORE") {
        let path = PathBuf::from(store_path);
        if path.exists() {
            return Some(path);
        }
    }
    
    // Default location
    let default_store = PathBuf::from("/nix/store");
    if default_store.exists() {
        Some(default_store)
    } else {
        None
    }
}

fn detect_nixos_version() -> Option<String> {
    // Try to read NixOS version from /etc/nixos-version
    if let Ok(version) = fs::read_to_string("/etc/nixos-version") {
        return Some(version.trim().to_string());
    }
    
    // Try nixos-version command
    if let Ok(output) = Command::new("nixos-version").output() {
        if output.status.success() {
            if let Ok(version) = String::from_utf8(output.stdout) {
                return Some(version.trim().to_string());
            }
        }
    }
    
    None
}

fn detect_home_manager() -> bool {
    // Check if home-manager command exists
    if Command::new("which")
        .arg("home-manager")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return true;
    }
    
    // Check for home-manager in common locations
    let home_manager_paths = [
        "/nix/var/nix/profiles/per-user/*/home-manager/bin/home-manager",
        "~/.nix-profile/bin/home-manager",
    ];
    
    for path_pattern in &home_manager_paths {
        if let Ok(expanded) = shellexpand::full(path_pattern) {
            if Path::new(&*expanded).exists() {
                return true;
            }
        }
    }
    
    false
}

fn discover_config_locations() -> Vec<NixConfigLocation> {
    let mut locations = Vec::new();
    
    // System configuration
    let system_config = PathBuf::from("/etc/nixos/configuration.nix");
    locations.push(NixConfigLocation {
        config_type: NixConfigType::SystemConfig,
        path: system_config.clone(),
        exists: system_config.exists(),
        has_hyprland_config: system_config.exists() && check_hyprland_in_file(&system_config),
    });
    
    // Home Manager configuration (traditional)
    if let Some(home_dir) = dirs::home_dir() {
        let home_manager_config = home_dir.join(".config/nixpkgs/home.nix");
        locations.push(NixConfigLocation {
            config_type: NixConfigType::HomeManager,
            path: home_manager_config.clone(),
            exists: home_manager_config.exists(),
            has_hyprland_config: home_manager_config.exists() && check_hyprland_in_file(&home_manager_config),
        });
    }
    
    // Look for flake-based configurations
    locations.extend(discover_flake_configs());
    
    locations
}

fn discover_flake_configs() -> Vec<NixConfigLocation> {
    let mut flake_locations = Vec::new();
    
    // Common flake locations to check
    let flake_paths = [
        "/etc/nixos/flake.nix",
        "~/.config/nixos/flake.nix",
        "~/.config/home-manager/flake.nix",
        "./flake.nix", // Current directory
    ];
    
    for path_pattern in &flake_paths {
        if let Ok(expanded) = shellexpand::full(path_pattern) {
            let flake_path = PathBuf::from(&*expanded);
            if flake_path.exists() {
                // Determine if this is a system or home-manager flake
                let config_type = if flake_path.to_string_lossy().contains("home-manager") {
                    NixConfigType::FlakeHomeManager
                } else {
                    NixConfigType::FlakeSystem
                };
                
                flake_locations.push(NixConfigLocation {
                    config_type,
                    path: flake_path.clone(),
                    exists: true,
                    has_hyprland_config: check_hyprland_in_flake(&flake_path),
                });
            }
        }
    }
    
    flake_locations
}

fn check_hyprland_in_file(path: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(path) {
        // Look for Hyprland-related configuration
        let hyprland_indicators = [
            "programs.hyprland",
            "wayland.windowManager.hyprland",
            "hyprland.enable",
            "services.hyprland",
        ];
        
        for indicator in &hyprland_indicators {
            if content.contains(indicator) {
                return true;
            }
        }
    }
    false
}

fn check_hyprland_in_flake(path: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(path) {
        // Look for Hyprland in flake inputs or configuration
        let hyprland_indicators = [
            "hyprland.url",
            "inputs.hyprland",
            "programs.hyprland",
            "wayland.windowManager.hyprland",
        ];
        
        for indicator in &hyprland_indicators {
            if content.contains(indicator) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nixos_detection() {
        // This test will work differently depending on the environment
        let env = detect_nixos_environment();
        
        // On NixOS systems, should detect correctly
        if Path::new("/etc/NIXOS").exists() {
            assert!(env.is_nixos);
        }
    }
    
    #[test]
    fn test_config_location_discovery() {
        let locations = discover_config_locations();
        
        // Should always return at least the system config location
        assert!(!locations.is_empty());
        
        // First location should be system config
        assert_eq!(locations[0].config_type, NixConfigType::SystemConfig);
        assert_eq!(locations[0].path, PathBuf::from("/etc/nixos/configuration.nix"));
    }
}