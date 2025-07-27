use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::platform::ConfigPathManager;
use super::formats::{ConfigFormat, StructuredConfig};

// Allow dead code for import functionality that will be used by TUI in future
#[allow(dead_code)]
/// Different sources for importing configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportSource {
    /// GitHub repository with optional branch/tag
    GitHub {
        url: String,
        branch: Option<String>,
        tag: Option<String>,
        commit: Option<String>,
    },
    /// Local folder containing configuration files
    LocalFolder { path: PathBuf },
    /// Single local configuration file
    LocalFile { path: PathBuf },
    /// HTTP URL to a configuration file
    HttpUrl { url: String },
}

/// Result of scanning for configuration files
#[derive(Debug, Clone)]
pub struct DiscoveredConfig {
    pub path: PathBuf,
    pub format: ConfigFormat,
    pub relative_path: String,
    pub size: u64,
    pub is_main_config: bool,
    pub associated_files: Vec<PathBuf>, // wallpapers, scripts, etc.
}

/// Preview of what will be imported
#[derive(Debug, Clone)]
pub struct ImportPreview {
    pub source: ImportSource,
    pub discovered_configs: Vec<DiscoveredConfig>,
    pub conflicts: Vec<ConflictInfo>,
    pub assets: Vec<AssetInfo>,
    pub total_size: u64,
}

/// Information about configuration conflicts
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub existing_config: String,
    pub new_config: String,
    pub conflict_type: ConflictType,
    pub resolution: ConflictResolution,
}

/// Types of conflicts that can occur
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    KeybindOverride,
    SettingOverride,
    FileReplacement,
    AssetConflict,
}

/// How to resolve conflicts
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    KeepExisting,
    ReplaceWithNew,
    Merge,
    Skip,
    Backup,
}

/// Information about assets (wallpapers, scripts, etc.)
#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub size: u64,
    pub target_path: PathBuf,
}

/// Types of assets that can be imported
#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    Wallpaper,
    Script,
    Font,
    Theme,
    Other,
}

/// Options for import operations
#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub create_backup: bool,
    pub merge_configs: bool,
    pub import_assets: bool,
    pub conflict_resolution: ConflictResolution,
    pub target_profile: Option<String>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            create_backup: true,
            merge_configs: false,
            import_assets: true,
            conflict_resolution: ConflictResolution::Backup,
            target_profile: None,
        }
    }
}

/// Main configuration importer
pub struct ConfigImporter {
    temp_dir: PathBuf,
}

impl ConfigImporter {
    /// Create a new importer
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("r-hyprconfig-import");
        std::fs::create_dir_all(&temp_dir)
            .with_context(|| format!("Failed to create temp directory: {:?}", temp_dir))?;
        
        Ok(Self { temp_dir })
    }

    /// Import configuration from any source with preview
    pub async fn import_with_preview(
        &self,
        source: ImportSource,
        options: ImportOptions,
    ) -> Result<(ImportPreview, Option<StructuredConfig>)> {
        let preview = self.preview_import(&source).await?;
        
        if preview.conflicts.is_empty() || options.conflict_resolution != ConflictResolution::Skip {
            let config = self.execute_import(&source, &options).await?;
            Ok((preview, Some(config)))
        } else {
            Ok((preview, None))
        }
    }

    /// Preview what will be imported without actually importing
    pub async fn preview_import(&self, source: &ImportSource) -> Result<ImportPreview> {
        let discovered_configs = self.discover_configs(source).await?;
        let conflicts = self.detect_conflicts(&discovered_configs).await?;
        let assets = self.discover_assets(source).await?;
        
        let total_size = discovered_configs.iter().map(|c| c.size).sum::<u64>()
            + assets.iter().map(|a| a.size).sum::<u64>();

        Ok(ImportPreview {
            source: source.clone(),
            discovered_configs,
            conflicts,
            assets,
            total_size,
        })
    }

    /// Execute the actual import
    pub async fn execute_import(
        &self,
        source: &ImportSource,
        options: &ImportOptions,
    ) -> Result<StructuredConfig> {
        let configs = self.discover_configs(source).await?;
        
        if configs.is_empty() {
            anyhow::bail!("No configuration files found in source");
        }

        // Find the main configuration file
        let main_config = configs.iter()
            .find(|c| c.is_main_config)
            .or_else(|| configs.first())
            .context("No main configuration file found")?;

        // Create backup if requested
        if options.create_backup {
            self.create_backup().await?;
        }

        // Parse the main configuration
        let mut structured_config = self.parse_config_file(&main_config.path, main_config.format.clone()).await?;

        // Merge additional configs if requested
        if options.merge_configs {
            for config in configs.iter().filter(|c| !c.is_main_config) {
                let additional = self.parse_config_file(&config.path, config.format.clone()).await?;
                self.merge_configs(&mut structured_config, additional)?;
            }
        }

        // Import assets if requested
        if options.import_assets {
            let assets = self.discover_assets(source).await?;
            self.import_assets(&assets).await?;
        }

        // Set source information
        if let ImportSource::GitHub { url, .. } = source {
            structured_config.set_source_url(url);
        }
        structured_config.touch();

        Ok(structured_config)
    }

    /// Discover configuration files in the source
    async fn discover_configs(&self, source: &ImportSource) -> Result<Vec<DiscoveredConfig>> {
        match source {
            ImportSource::GitHub { url, branch, tag, commit } => {
                let repo_path = self.clone_repository(url, branch.as_deref(), tag.as_deref(), commit.as_deref()).await?;
                self.scan_directory_for_configs(&repo_path, &repo_path)
            }
            ImportSource::LocalFolder { path } => {
                self.scan_directory_for_configs(path, path)
            }
            ImportSource::LocalFile { path } => {
                self.scan_single_file(path)
            }
            ImportSource::HttpUrl { url } => {
                let file_path = self.download_file(url).await?;
                self.scan_single_file(&file_path)
            }
        }
    }

    /// Clone a git repository
    async fn clone_repository(
        &self,
        url: &str,
        branch: Option<&str>,
        tag: Option<&str>,
        commit: Option<&str>,
    ) -> Result<PathBuf> {
        let repo_name = url.split('/').last()
            .and_then(|name| name.strip_suffix(".git"))
            .unwrap_or("repo");
        
        let target_path = self.temp_dir.join(format!("{}_{}", repo_name, uuid::Uuid::new_v4()));
        
        // Use git2 for cloning
        let mut builder = git2::build::RepoBuilder::new();
        
        if let Some(branch) = branch {
            builder.branch(branch);
        }
        
        let repo = builder.clone(url, &target_path)
            .with_context(|| format!("Failed to clone repository: {}", url))?;

        // Checkout specific tag or commit if specified
        if let Some(tag) = tag {
            let (object, _reference) = repo.revparse_ext(tag)
                .with_context(|| format!("Failed to find tag: {}", tag))?;
            repo.checkout_tree(&object, None)
                .with_context(|| format!("Failed to checkout tag: {}", tag))?;
        } else if let Some(commit) = commit {
            let oid = git2::Oid::from_str(commit)
                .with_context(|| format!("Invalid commit hash: {}", commit))?;
            let commit = repo.find_commit(oid)
                .with_context(|| format!("Failed to find commit: {}", commit))?;
            repo.checkout_tree(commit.as_object(), None)
                .with_context(|| format!("Failed to checkout commit: {}", commit.id()))?;
        }

        Ok(target_path)
    }

    /// Download a file from HTTP URL
    async fn download_file(&self, url: &str) -> Result<PathBuf> {
        let response = reqwest::get(url).await
            .with_context(|| format!("Failed to download file: {}", url))?;
        
        if !response.status().is_success() {
            anyhow::bail!("HTTP error {}: {}", response.status(), url);
        }

        let filename = url.split('/').last().unwrap_or("downloaded_config");
        let file_path = self.temp_dir.join(filename);
        
        let content = response.bytes().await
            .context("Failed to read response body")?;
        
        std::fs::write(&file_path, content)
            .with_context(|| format!("Failed to write downloaded file: {:?}", file_path))?;

        Ok(file_path)
    }

    /// Scan a directory for configuration files
    fn scan_directory_for_configs(&self, dir_path: &Path, base_path: &Path) -> Result<Vec<DiscoveredConfig>> {
        let mut configs = Vec::new();
        
        // Common Hyprland config file patterns
        let config_patterns = [
            "hyprland.conf",
            "hypr.conf", 
            "config.conf",
            "hyprland.config",
            ".hyprland.conf",
        ];

        // Common locations within repos
        let common_paths = [
            ".config/hypr/",
            "hypr/",
            "config/hypr/",
            "dotfiles/.config/hypr/",
            "home/.config/hypr/",
            "",
        ];

        for entry in WalkDir::new(dir_path)
            .max_depth(5) // Prevent infinite recursion
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if !path.is_file() {
                continue;
            }

            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Check if it's a known config file
            let is_config_file = config_patterns.iter().any(|pattern| {
                filename.eq_ignore_ascii_case(pattern) || 
                filename.ends_with(".conf") ||
                filename.ends_with(".config")
            });

            if is_config_file {
                let format = ConfigFormat::from_path(&path.to_path_buf())
                    .unwrap_or(ConfigFormat::HyprlandConf);
                
                let relative_path = path.strip_prefix(base_path)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                let metadata = std::fs::metadata(path)
                    .with_context(|| format!("Failed to read metadata for: {:?}", path))?;

                // Determine if this is likely the main config
                let is_main_config = config_patterns.iter().any(|pattern| {
                    filename.eq_ignore_ascii_case(pattern)
                }) || common_paths.iter().any(|common_path| {
                    relative_path.starts_with(common_path)
                });

                // Find associated files (wallpapers, scripts in same directory)
                let associated_files = self.find_associated_files(path.parent().unwrap_or(path))?;

                configs.push(DiscoveredConfig {
                    path: path.to_path_buf(),
                    format,
                    relative_path,
                    size: metadata.len(),
                    is_main_config,
                    associated_files,
                });
            }
        }

        // Sort by priority (main configs first, then by size)
        configs.sort_by(|a, b| {
            match (a.is_main_config, b.is_main_config) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.size.cmp(&a.size), // Larger files first
            }
        });

        Ok(configs)
    }

    /// Scan a single file
    fn scan_single_file(&self, file_path: &Path) -> Result<Vec<DiscoveredConfig>> {
        if !file_path.exists() {
            anyhow::bail!("File does not exist: {:?}", file_path);
        }

        let format = ConfigFormat::from_path(&file_path.to_path_buf())
            .unwrap_or(ConfigFormat::HyprlandConf);

        let metadata = std::fs::metadata(file_path)
            .with_context(|| format!("Failed to read metadata for: {:?}", file_path))?;

        let filename = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let associated_files = if let Some(parent) = file_path.parent() {
            self.find_associated_files(parent)?
        } else {
            Vec::new()
        };

        Ok(vec![DiscoveredConfig {
            path: file_path.to_path_buf(),
            format,
            relative_path: filename.to_string(),
            size: metadata.len(),
            is_main_config: true,
            associated_files,
        }])
    }

    /// Find associated files (wallpapers, scripts, etc.)
    fn find_associated_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut associated = Vec::new();
        
        let asset_extensions = [
            // Images
            "jpg", "jpeg", "png", "webp", "bmp", "gif", "svg",
            // Scripts
            "sh", "bash", "zsh", "fish", "py", "lua",
            // Themes
            "css", "scss", "theme",
            // Other config files
            "json", "toml", "yaml", "yml",
        ];

        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {:?}", dir))?
        {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if asset_extensions.iter().any(|&ae| ae.eq_ignore_ascii_case(ext)) {
                        associated.push(path);
                    }
                }
            }
        }

        Ok(associated)
    }

    /// Parse a configuration file into structured format
    async fn parse_config_file(&self, path: &Path, format: ConfigFormat) -> Result<StructuredConfig> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        match format {
            ConfigFormat::HyprlandConf => self.parse_hyprland_conf(&content, path),
            ConfigFormat::Json => self.parse_json_config(&content),
            ConfigFormat::Toml => self.parse_toml_config(&content),
            ConfigFormat::Yaml => self.parse_yaml_config(&content),
            ConfigFormat::RHyprConfig => self.parse_rhypr_config(&content),
            _ => anyhow::bail!("Unsupported format for import: {:?}", format),
        }
    }

    /// Parse Hyprland .conf format
    fn parse_hyprland_conf(&self, content: &str, path: &Path) -> Result<StructuredConfig> {
        let name = path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("Imported Configuration")
            .to_string();

        let mut config = StructuredConfig::new(&name);
        
        // TODO: Improve config parsing
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // Parse settings by category
                if key.starts_with("general:") {
                    self.parse_general_setting(&mut config, key, value)?;
                } else if key.starts_with("input:") {
                    self.parse_input_setting(&mut config, key, value)?;
                } else if key.starts_with("decoration:") {
                    self.parse_decoration_setting(&mut config, key, value)?;
                } else if key.starts_with("bind") {
                    // Parse keybinds that have = format
                    if let Some(bind) = self.parse_keybind_line(line) {
                        config.keybinds.push(bind);
                    }
                } else if key.starts_with("windowrule") {
                    // Parse window rules
                    if let Some(rule) = self.parse_window_rule_line(line) {
                        config.window_rules.push(rule);
                    }
                }
            }
        }

        config.touch();
        Ok(config)
    }

    /// Parse other formats (JSON, TOML, YAML)
    fn parse_json_config(&self, content: &str) -> Result<StructuredConfig> {
        serde_json::from_str(content)
            .context("Failed to parse JSON configuration")
    }

    fn parse_toml_config(&self, content: &str) -> Result<StructuredConfig> {
        toml::from_str(content)
            .context("Failed to parse TOML configuration")
    }

    fn parse_yaml_config(&self, content: &str) -> Result<StructuredConfig> {
        serde_yaml::from_str(content)
            .context("Failed to parse YAML configuration")
    }

    fn parse_rhypr_config(&self, content: &str) -> Result<StructuredConfig> {
        // Skip metadata comments and parse JSON content
        let json_start = content.find('{')
            .context("Invalid r-hyprconfig format: no JSON content found")?;
        
        let json_content = &content[json_start..];
        serde_json::from_str(json_content)
            .context("Failed to parse r-hyprconfig JSON content")
    }

    // Helper parsing methods would go here...
    fn parse_general_setting(&self, config: &mut StructuredConfig, key: &str, value: &str) -> Result<()> {
        match key {
            "general:gaps_in" => config.general.gaps_in = value.parse().ok(),
            "general:gaps_out" => config.general.gaps_out = value.parse().ok(),
            "general:border_size" => config.general.border_size = value.parse().ok(),
            "general:col.active_border" => config.general.col_active_border = Some(value.to_string()),
            "general:col.inactive_border" => config.general.col_inactive_border = Some(value.to_string()),
            _ => {
                config.custom_settings.insert(key.to_string(), value.to_string());
            }
        }
        Ok(())
    }

    fn parse_input_setting(&self, config: &mut StructuredConfig, key: &str, value: &str) -> Result<()> {
        match key {
            "input:kb_layout" => config.input.kb_layout = Some(value.to_string()),
            "input:follow_mouse" => config.input.follow_mouse = value.parse().ok(),
            "input:sensitivity" => config.input.sensitivity = value.parse().ok(),
            _ => {
                config.custom_settings.insert(key.to_string(), value.to_string());
            }
        }
        Ok(())
    }

    fn parse_decoration_setting(&self, config: &mut StructuredConfig, key: &str, value: &str) -> Result<()> {
        match key {
            "decoration:rounding" => config.decoration.rounding = value.parse().ok(),
            "decoration:blur:enabled" => config.decoration.blur_enabled = value.parse().ok(),
            "decoration:blur:size" => config.decoration.blur_size = value.parse().ok(),
            _ => {
                config.custom_settings.insert(key.to_string(), value.to_string());
            }
        }
        Ok(())
    }

    fn parse_keybind_line(&self, line: &str) -> Option<super::formats::KeybindEntry> {
        // TODO: Improve keybind parsing
        if let Some(equals_pos) = line.find('=') {
            let bind_type = line[..equals_pos].trim();
            let bind_part = line[equals_pos + 1..].trim();
            let parts: Vec<&str> = bind_part.split(',').map(|s| s.trim()).collect();
            
            if parts.len() >= 3 {
                return Some(super::formats::KeybindEntry {
                    bind_type: bind_type.to_string(),
                    modifiers: vec![parts[0].to_string()],
                    key: parts[1].to_string(),
                    dispatcher: parts[2].to_string(),
                    args: if parts.len() > 3 { Some(parts[3..].join(",")) } else { None },
                    description: None,
                });
            }
        }
        None
    }

    fn parse_window_rule_line(&self, line: &str) -> Option<super::formats::WindowRuleEntry> {
        if let Some(equals_pos) = line.find('=') {
            let rule_part = &line[equals_pos + 1..].trim();
            let parts: Vec<&str> = rule_part.splitn(2, ',').map(|s| s.trim()).collect();
            
            if parts.len() == 2 {
                return Some(super::formats::WindowRuleEntry {
                    rule: parts[0].to_string(),
                    window_identifier: parts[1].to_string(),
                    description: None,
                });
            }
        }
        None
    }

    /// Detect conflicts with existing configuration
    async fn detect_conflicts(&self, _configs: &[DiscoveredConfig]) -> Result<Vec<ConflictInfo>> {
        // This would compare with existing config and detect conflicts
        // For now, return empty (no conflicts)
        Ok(Vec::new())
    }

    /// Discover assets (wallpapers, scripts, etc.)
    async fn discover_assets(&self, source: &ImportSource) -> Result<Vec<AssetInfo>> {
        let mut assets = Vec::new();
        
        let search_path = match source {
            ImportSource::GitHub { .. } => {
                // Would use the cloned repo path
                return Ok(assets);
            }
            ImportSource::LocalFolder { path } => path.clone(),
            ImportSource::LocalFile { path } => {
                if let Some(parent) = path.parent() {
                    parent.to_path_buf()
                } else {
                    return Ok(assets);
                }
            }
            ImportSource::HttpUrl { .. } => return Ok(assets),
        };

        // Scan for assets
        for entry in WalkDir::new(&search_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(asset_type) = self.classify_asset(path) {
                let metadata = std::fs::metadata(path)?;
                let target_path = self.determine_asset_target_path(path, asset_type.clone())?;
                
                assets.push(AssetInfo {
                    path: path.to_path_buf(),
                    asset_type,
                    size: metadata.len(),
                    target_path,
                });
            }
        }

        Ok(assets)
    }

    /// Classify an asset by its file extension and path
    fn classify_asset(&self, path: &Path) -> Option<AssetType> {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" | "png" | "webp" | "bmp" | "gif" | "svg" => Some(AssetType::Wallpaper),
                "sh" | "bash" | "zsh" | "fish" | "py" | "lua" => Some(AssetType::Script),
                "ttf" | "otf" | "woff" | "woff2" => Some(AssetType::Font),
                "css" | "scss" | "theme" => Some(AssetType::Theme),
                _ => Some(AssetType::Other),
            }
        } else {
            None
        }
    }

    /// Determine where an asset should be placed
    fn determine_asset_target_path(&self, _source_path: &Path, asset_type: AssetType) -> Result<PathBuf> {
        let paths = ConfigPathManager::get_paths()?;
        
        match asset_type {
            AssetType::Wallpaper => Ok(paths.app_data_dir.join("wallpapers")),
            AssetType::Script => Ok(paths.app_data_dir.join("scripts")),
            AssetType::Font => Ok(paths.app_data_dir.join("fonts")),
            AssetType::Theme => Ok(paths.app_data_dir.join("themes")),
            AssetType::Other => Ok(paths.app_data_dir.join("assets")),
        }
    }

    /// Create backup of current configuration
    async fn create_backup(&self) -> Result<PathBuf> {
        let paths = ConfigPathManager::get_paths()?;
        let backup_path = ConfigPathManager::create_backup(&paths.hyprland_config_file)?;
        Ok(backup_path)
    }

    /// Merge two configurations
    fn merge_configs(&self, base: &mut StructuredConfig, additional: StructuredConfig) -> Result<()> {
        // Merge keybinds
        base.keybinds.extend(additional.keybinds);
        
        // Merge window rules
        base.window_rules.extend(additional.window_rules);
        
        // Merge layer rules  
        base.layer_rules.extend(additional.layer_rules);
        
        // Merge custom settings
        base.custom_settings.extend(additional.custom_settings);
        
        // Merge other settings (taking non-None values from additional)
        if additional.general.gaps_in.is_some() {
            base.general.gaps_in = additional.general.gaps_in;
        }
        if additional.general.gaps_out.is_some() {
            base.general.gaps_out = additional.general.gaps_out;
        }
        // ... continue for other settings
        
        base.touch();
        Ok(())
    }

    /// Import assets to their target locations
    async fn import_assets(&self, assets: &[AssetInfo]) -> Result<()> {
        for asset in assets {
            // Create target directory
            if let Some(parent) = asset.target_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create asset directory: {:?}", parent))?;
            }
            
            // Copy asset file
            let target_file = asset.target_path.join(
                asset.path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("asset"))
            );
            
            std::fs::copy(&asset.path, &target_file)
                .with_context(|| format!("Failed to copy asset from {:?} to {:?}", asset.path, target_file))?;
        }
        Ok(())
    }
}

impl Drop for ConfigImporter {
    fn drop(&mut self) {
        // Clean up temporary directory
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_importer_new() -> Result<()> {
        let importer = ConfigImporter::new()?;
        assert!(importer.temp_dir.exists());
        Ok(())
    }

    #[test]
    fn test_scan_single_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_file = temp_dir.path().join("hyprland.conf");
        std::fs::write(&config_file, "general:gaps_in = 10\n")?;
        
        let importer = ConfigImporter::new()?;
        let configs = importer.scan_single_file(&config_file)?;
        
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].format, ConfigFormat::HyprlandConf);
        assert!(configs[0].is_main_config);
        
        Ok(())
    }

    #[test]
    fn test_parse_hyprland_conf() -> Result<()> {
        let content = r#"
# Test configuration
general:gaps_in = 10
general:gaps_out = 20
input:kb_layout = us

bind = SUPER, Return, exec, kitty
windowrule = float, ^(kitty)$
"#;
        
        let importer = ConfigImporter::new()?;
        let config = importer.parse_hyprland_conf(content, Path::new("test.conf"))?;
        
        assert_eq!(config.general.gaps_in, Some(10));
        assert_eq!(config.general.gaps_out, Some(20));
        assert_eq!(config.input.kb_layout, Some("us".to_string()));
        assert_eq!(config.keybinds.len(), 1);
        assert_eq!(config.window_rules.len(), 1);
        
        Ok(())
    }

    #[test]
    fn test_classify_asset() {
        let importer = ConfigImporter::new().unwrap();
        
        assert_eq!(importer.classify_asset(Path::new("wallpaper.jpg")), Some(AssetType::Wallpaper));
        assert_eq!(importer.classify_asset(Path::new("script.sh")), Some(AssetType::Script));
        assert_eq!(importer.classify_asset(Path::new("font.ttf")), Some(AssetType::Font));
        assert_eq!(importer.classify_asset(Path::new("theme.css")), Some(AssetType::Theme));
        assert_eq!(importer.classify_asset(Path::new("readme.txt")), Some(AssetType::Other));
    }

    #[test]
    fn test_import_options_default() {
        let options = ImportOptions::default();
        assert!(options.create_backup);
        assert!(!options.merge_configs);
        assert!(options.import_assets);
        assert_eq!(options.conflict_resolution, ConflictResolution::Backup);
    }

    #[test]
    fn test_conflict_resolution_types() {
        // Test that all conflict resolution types exist
        let resolutions = [
            ConflictResolution::KeepExisting,
            ConflictResolution::ReplaceWithNew,
            ConflictResolution::Merge,
            ConflictResolution::Skip,
            ConflictResolution::Backup,
        ];
        
        for resolution in &resolutions {
            // Just test that they exist and can be cloned
            let _cloned = resolution.clone();
        }
    }
}