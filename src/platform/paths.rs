use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use super::distribution::{DistributionDetector, DistributionInfo, DistributionType};

// Allow dead code for path management functionality that will be used by TUI in future
#[allow(dead_code)]
/// Configuration for different types of paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfiguration {
    pub hyprland_config_dir: PathBuf,
    pub hyprland_config_file: PathBuf,
    pub app_config_dir: PathBuf,
    pub app_cache_dir: PathBuf,
    pub app_data_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub profiles_dir: PathBuf,
    pub exports_dir: PathBuf,
}

impl PathConfiguration {
    /// Create all directories in the configuration
    pub fn create_directories(&self) -> Result<()> {
        let dirs = [
            &self.hyprland_config_dir,
            &self.app_config_dir,
            &self.app_cache_dir,
            &self.app_data_dir,
            &self.backup_dir,
            &self.profiles_dir,
            &self.exports_dir,
        ];

        for dir in &dirs {
            if !dir.exists() {
                std::fs::create_dir_all(dir)
                    .with_context(|| format!("Failed to create directory: {:?}", dir))?;
            }
        }

        Ok(())
    }

    /// Validate that all paths are accessible
    pub fn validate(&self) -> Result<()> {
        // Check if hyprland config directory exists or can be created
        if !self.hyprland_config_dir.exists() {
            if let Some(parent) = self.hyprland_config_dir.parent() {
                if !parent.exists() {
                    anyhow::bail!(
                        "Hyprland config parent directory does not exist: {:?}",
                        parent
                    );
                }
            }
        }

        // Check write permissions for app directories
        for dir in [
            &self.app_config_dir,
            &self.app_cache_dir,
            &self.app_data_dir,
        ] {
            if dir.exists() {
                let test_file = dir.join(".write_test");
                match std::fs::write(&test_file, "test") {
                    Ok(_) => {
                        let _ = std::fs::remove_file(test_file);
                    }
                    Err(e) => {
                        anyhow::bail!("No write permission for directory {:?}: {}", dir, e);
                    }
                }
            }
        }

        Ok(())
    }
}

/// Path cache for performance optimization
static PATH_CACHE: OnceLock<Mutex<Option<PathConfiguration>>> = OnceLock::new();

/// Manages configuration paths across different Linux distributions
pub struct ConfigPathManager;

impl ConfigPathManager {
    /// Get path configuration for the current distribution
    pub fn get_paths() -> Result<PathConfiguration> {
        let cache = PATH_CACHE.get_or_init(|| Mutex::new(None));

        {
            let cached = cache.lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire path cache lock"))?;
            if let Some(ref paths) = *cached {
                return Ok(paths.clone());
            }
        }

        let distribution = DistributionDetector::detect()?;
        let paths = Self::resolve_paths(&distribution)?;

        {
            let mut cached = cache.lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire path cache lock for write"))?;
            *cached = Some(paths.clone());
        }

        Ok(paths)
    }

    /// Clear the path cache (useful for testing)
    pub fn clear_cache() -> Result<()> {
        if let Some(cache) = PATH_CACHE.get() {
            let mut cached = cache.lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire path cache lock for clearing"))?;
            *cached = None;
        }
        Ok(())
    }

    /// Resolve paths based on distribution information
    fn resolve_paths(distribution: &DistributionInfo) -> Result<PathConfiguration> {
        match &distribution.distribution_type {
            DistributionType::NixOS => Self::resolve_nixos_paths(distribution),
            _ => Self::resolve_standard_paths(distribution),
        }
    }

    /// Resolve paths for standard Linux distributions
    fn resolve_standard_paths(distribution: &DistributionInfo) -> Result<PathConfiguration> {
        let home_dir = dirs::home_dir().context("Could not determine home directory")?;

        // Standard XDG paths
        let config_home = dirs::config_dir().unwrap_or_else(|| home_dir.join(".config"));
        let cache_home = dirs::cache_dir().unwrap_or_else(|| home_dir.join(".cache"));
        let data_home = dirs::data_dir().unwrap_or_else(|| home_dir.join(".local/share"));

        // Hyprland configuration paths
        let hyprland_config_dir = Self::resolve_hyprland_config_dir(&config_home, distribution)?;
        let hyprland_config_file = hyprland_config_dir.join("hyprland.conf");

        // Application paths
        let app_config_dir = config_home.join("r-hyprconfig");
        let app_cache_dir = cache_home.join("r-hyprconfig");
        let app_data_dir = data_home.join("r-hyprconfig");

        // Specialized directories
        let backup_dir = app_data_dir.join("backups");
        let profiles_dir = app_data_dir.join("profiles");
        let exports_dir = app_data_dir.join("exports");

        Ok(PathConfiguration {
            hyprland_config_dir,
            hyprland_config_file,
            app_config_dir,
            app_cache_dir,
            app_data_dir,
            backup_dir,
            profiles_dir,
            exports_dir,
        })
    }

    /// Resolve paths specifically for NixOS
    fn resolve_nixos_paths(_distribution: &DistributionInfo) -> Result<PathConfiguration> {
        let home_dir = dirs::home_dir().context("Could not determine home directory")?;

        // Check for NixOS-specific environment variables
        let config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir.join(".config"));

        let cache_home = std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir.join(".cache"));

        let data_home = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir.join(".local/share"));

        // NixOS-specific Hyprland paths
        let hyprland_config_dir = if std::env::var("HOME_MANAGER_CONFIG").is_ok() {
            // Using home-manager
            config_home.join("hypr")
        } else {
            // System-wide NixOS configuration
            config_home.join("hypr")
        };

        let hyprland_config_file = hyprland_config_dir.join("hyprland.conf");

        // Application paths (same as standard for user data)
        let app_config_dir = config_home.join("r-hyprconfig");
        let app_cache_dir = cache_home.join("r-hyprconfig");
        let app_data_dir = data_home.join("r-hyprconfig");

        // NixOS-specific directories
        let backup_dir = app_data_dir.join("backups");
        let profiles_dir = app_data_dir.join("profiles");
        let exports_dir = app_data_dir.join("exports");

        Ok(PathConfiguration {
            hyprland_config_dir,
            hyprland_config_file,
            app_config_dir,
            app_cache_dir,
            app_data_dir,
            backup_dir,
            profiles_dir,
            exports_dir,
        })
    }

    /// Resolve Hyprland configuration directory for different distributions
    fn resolve_hyprland_config_dir(
        config_home: &Path,
        distribution: &DistributionInfo,
    ) -> Result<PathBuf> {
        let hypr_dir = config_home.join("hypr");

        // Check for distribution-specific overrides
        match &distribution.distribution_type {
            DistributionType::Arch | DistributionType::Manjaro => {
                // Arch-based distributions typically use ~/.config/hypr
                Ok(hypr_dir)
            }
            DistributionType::Ubuntu | DistributionType::Debian => {
                // Debian-based distributions might have different conventions
                // Check for existing directories in order of preference
                let candidates = [hypr_dir.clone(), config_home.join("hyprland")];

                for candidate in &candidates {
                    if candidate.exists() {
                        return Ok(candidate.clone());
                    }
                }

                // Default to standard path
                Ok(hypr_dir)
            }
            DistributionType::Fedora => {
                // Fedora follows XDG standards
                Ok(hypr_dir)
            }
            DistributionType::OpenSUSE => {
                // openSUSE typically follows XDG standards
                Ok(hypr_dir)
            }
            _ => {
                // Default case for other distributions
                Ok(hypr_dir)
            }
        }
    }

    /// Find existing Hyprland configuration files
    pub fn find_existing_configs() -> Result<Vec<PathBuf>> {
        let paths = Self::get_paths()?;
        let mut configs = Vec::new();

        // Check primary config file
        if paths.hyprland_config_file.exists() {
            configs.push(paths.hyprland_config_file.clone());
        }

        // Check for alternative config files
        let config_dir = &paths.hyprland_config_dir;
        if config_dir.exists() {
            let alternative_names = ["hyprland.conf", "config", "hypr.conf", "hyprland.config"];

            for name in &alternative_names {
                let config_file = config_dir.join(name);
                if config_file.exists() && !configs.contains(&config_file) {
                    configs.push(config_file);
                }
            }
        }

        Ok(configs)
    }

    /// Create a backup of the current configuration
    pub fn create_backup(source: &Path) -> Result<PathBuf> {
        let paths = Self::get_paths()?;
        paths.create_directories()?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("hyprland_{}.conf", timestamp);
        let backup_path = paths.backup_dir.join(backup_name);

        std::fs::copy(source, &backup_path).with_context(|| {
            format!(
                "Failed to create backup from {:?} to {:?}",
                source, backup_path
            )
        })?;

        Ok(backup_path)
    }

    /// List available backups
    pub fn list_backups() -> Result<Vec<PathBuf>> {
        let paths = Self::get_paths()?;
        let mut backups = Vec::new();

        if paths.backup_dir.exists() {
            for entry in std::fs::read_dir(&paths.backup_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("hyprland_") && name.ends_with(".conf") {
                            backups.push(path);
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| {
            let a_meta = std::fs::metadata(a).ok();
            let b_meta = std::fs::metadata(b).ok();

            match (a_meta, b_meta) {
                (Some(a_meta), Some(b_meta)) => b_meta
                    .modified()
                    .unwrap_or(std::time::UNIX_EPOCH)
                    .cmp(&a_meta.modified().unwrap_or(std::time::UNIX_EPOCH)),
                _ => std::cmp::Ordering::Equal,
            }
        });

        Ok(backups)
    }

    /// Get configuration export path for a specific format
    pub fn get_export_path(name: &str, format: &str) -> Result<PathBuf> {
        let paths = Self::get_paths()?;
        paths.create_directories()?;

        let filename = match format {
            "nix" | "nixos" => format!("{}.nix", name),
            "toml" => format!("{}.toml", name),
            "json" => format!("{}.json", name),
            "conf" | "hyprland" => format!("{}.conf", name),
            _ => format!("{}.{}", name, format),
        };

        Ok(paths.exports_dir.join(filename))
    }

    /// Initialize directories for the current distribution
    pub fn initialize() -> Result<()> {
        let paths = Self::get_paths()?;
        paths.create_directories()?;
        paths.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_configuration_create_directories() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        let config = PathConfiguration {
            hyprland_config_dir: base_path.join("hypr"),
            hyprland_config_file: base_path.join("hypr/hyprland.conf"),
            app_config_dir: base_path.join("r-hyprconfig"),
            app_cache_dir: base_path.join("cache/r-hyprconfig"),
            app_data_dir: base_path.join("data/r-hyprconfig"),
            backup_dir: base_path.join("data/r-hyprconfig/backups"),
            profiles_dir: base_path.join("data/r-hyprconfig/profiles"),
            exports_dir: base_path.join("data/r-hyprconfig/exports"),
        };

        config.create_directories()?;

        assert!(config.hyprland_config_dir.exists());
        assert!(config.app_config_dir.exists());
        assert!(config.app_cache_dir.exists());
        assert!(config.app_data_dir.exists());
        assert!(config.backup_dir.exists());
        assert!(config.profiles_dir.exists());
        assert!(config.exports_dir.exists());

        Ok(())
    }

    #[test]
    fn test_path_configuration_validate() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        let config = PathConfiguration {
            hyprland_config_dir: base_path.join("hypr"),
            hyprland_config_file: base_path.join("hypr/hyprland.conf"),
            app_config_dir: base_path.join("r-hyprconfig"),
            app_cache_dir: base_path.join("cache/r-hyprconfig"),
            app_data_dir: base_path.join("data/r-hyprconfig"),
            backup_dir: base_path.join("data/r-hyprconfig/backups"),
            profiles_dir: base_path.join("data/r-hyprconfig/profiles"),
            exports_dir: base_path.join("data/r-hyprconfig/exports"),
        };

        config.create_directories()?;
        config.validate()?;

        Ok(())
    }

    #[test]
    fn test_resolve_standard_paths() -> Result<()> {
        // Mock distribution info
        let distribution = DistributionInfo {
            distribution_type: DistributionType::Ubuntu,
            version: Some("20.04".to_string()),
            version_id: Some("20.04".to_string()),
            name: "Ubuntu".to_string(),
            pretty_name: Some("Ubuntu 20.04 LTS".to_string()),
            id: "ubuntu".to_string(),
            id_like: Some(vec!["debian".to_string()]),
            home_url: None,
            support_url: None,
            bug_report_url: None,
        };

        let paths = ConfigPathManager::resolve_standard_paths(&distribution)?;

        assert!(paths.hyprland_config_dir.to_string_lossy().contains("hypr"));
        assert!(paths
            .hyprland_config_file
            .to_string_lossy()
            .contains("hyprland.conf"));
        assert!(paths
            .app_config_dir
            .to_string_lossy()
            .contains("r-hyprconfig"));

        Ok(())
    }

    #[test]
    fn test_resolve_nixos_paths() -> Result<()> {
        // Mock NixOS distribution info
        let distribution = DistributionInfo {
            distribution_type: DistributionType::NixOS,
            version: Some("23.11".to_string()),
            version_id: Some("23.11".to_string()),
            name: "NixOS".to_string(),
            pretty_name: Some("NixOS 23.11".to_string()),
            id: "nixos".to_string(),
            id_like: None,
            home_url: None,
            support_url: None,
            bug_report_url: None,
        };

        let paths = ConfigPathManager::resolve_nixos_paths(&distribution)?;

        assert!(paths.hyprland_config_dir.to_string_lossy().contains("hypr"));
        assert!(paths
            .hyprland_config_file
            .to_string_lossy()
            .contains("hyprland.conf"));
        assert!(paths
            .app_config_dir
            .to_string_lossy()
            .contains("r-hyprconfig"));

        Ok(())
    }

    #[test]
    fn test_find_existing_configs() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let hypr_dir = temp_dir.path().join("hypr");
        std::fs::create_dir_all(&hypr_dir)?;

        // Create test config files
        std::fs::write(hypr_dir.join("hyprland.conf"), "# test config")?;
        std::fs::write(hypr_dir.join("config"), "# another config")?;

        // This test would require mocking the path resolution
        // For now, we'll just test that the function doesn't panic
        let _ = ConfigPathManager::find_existing_configs();

        Ok(())
    }

    #[test]
    fn test_create_backup() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source_file = temp_dir.path().join("test_config.conf");
        std::fs::write(&source_file, "# test configuration")?;

        // This test would require mocking the path resolution
        // For now, we'll just test that the function signature is correct
        let _ = ConfigPathManager::create_backup(&source_file);

        Ok(())
    }

    #[test]
    fn test_get_export_path() -> Result<()> {
        // Test different export formats
        let formats = ["nix", "nixos", "toml", "json", "conf", "hyprland", "yaml"];

        for format in &formats {
            let _ = ConfigPathManager::get_export_path("test_config", format);
        }

        Ok(())
    }

    #[test]
    fn test_resolve_hyprland_config_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_home = temp_dir.path();

        let distributions = [
            DistributionInfo {
                distribution_type: DistributionType::Arch,
                version: None,
                version_id: None,
                name: "Arch Linux".to_string(),
                pretty_name: None,
                id: "arch".to_string(),
                id_like: None,
                home_url: None,
                support_url: None,
                bug_report_url: None,
            },
            DistributionInfo {
                distribution_type: DistributionType::Ubuntu,
                version: None,
                version_id: None,
                name: "Ubuntu".to_string(),
                pretty_name: None,
                id: "ubuntu".to_string(),
                id_like: Some(vec!["debian".to_string()]),
                home_url: None,
                support_url: None,
                bug_report_url: None,
            },
        ];

        for distribution in &distributions {
            let result = ConfigPathManager::resolve_hyprland_config_dir(config_home, distribution)?;
            assert!(result.to_string_lossy().contains("hypr"));
        }

        Ok(())
    }
}
