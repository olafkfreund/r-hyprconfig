# API Specification

This is the API specification for the spec detailed in @.agent-os/specs/2025-07-25-multi-platform-support/spec.md

> Created: 2025-07-25
> Version: 1.0.0

## Internal API Structure

The multi-platform support functionality will be exposed through internal Rust APIs that the TUI components can utilize. This specification outlines the key interfaces and their expected behavior.

## Distribution Detection API

### `DistributionDetector`

**Purpose:** Detect the current Linux distribution and provide distribution-specific information
**Integration:** Used by the application at startup and when configuration paths need resolution

```rust
pub struct DistributionDetector;

impl DistributionDetector {
    pub fn detect() -> Result<DistributionInfo, DetectionError>;
    pub fn is_nixos() -> bool;
    pub fn get_supported_distributions() -> Vec<String>;
}
```

**Methods:**
- `detect()` - Returns comprehensive distribution information including name, version, and configuration paths
- `is_nixos()` - Quick check for NixOS to enable special handling
- `get_supported_distributions()` - Lists all distributions with explicit support

## Configuration Path Management API

### `ConfigPathManager`

**Purpose:** Resolve and manage configuration file paths across different distributions
**Integration:** Central service for all configuration file operations

```rust
pub struct ConfigPathManager {
    distribution: DistributionInfo,
    cache: PathCache,
}

impl ConfigPathManager {
    pub fn new(distribution: DistributionInfo) -> Self;
    pub fn hyprland_config_path(&self) -> Result<PathBuf, PathError>;
    pub fn application_config_path(&self) -> Result<PathBuf, PathError>;
    pub fn profile_storage_path(&self) -> Result<PathBuf, PathError>;
    pub fn backup_directory(&self) -> Result<PathBuf, PathError>;
    pub fn validate_path(&self, path: &Path) -> Result<(), ValidationError>;
}
```

## Profile Management API

### `ProfileManager`

**Purpose:** Handle creation, storage, and management of configuration profiles
**Integration:** Accessible from main TUI for profile operations

```rust
pub struct ProfileManager {
    storage_path: PathBuf,
    profiles: HashMap<String, ConfigProfile>,
}

impl ProfileManager {
    pub fn new(storage_path: PathBuf) -> Result<Self, ProfileError>;
    pub fn create_profile(&mut self, name: String, config: HyprlandConfig) -> Result<String, ProfileError>;
    pub fn load_profile(&self, id: &str) -> Result<ConfigProfile, ProfileError>;
    pub fn save_profile(&mut self, profile: ConfigProfile) -> Result<(), ProfileError>;
    pub fn delete_profile(&mut self, id: &str) -> Result<(), ProfileError>;
    pub fn list_profiles(&self) -> Vec<ProfileSummary>;
    pub fn apply_profile(&self, id: &str) -> Result<(), ProfileError>;
    pub fn export_profile(&self, id: &str, format: ExportFormat) -> Result<String, ExportError>;
}
```

## Import/Export API

### `ConfigImporter`

**Purpose:** Import configurations from various sources and formats
**Integration:** Used in import dialogs and profile creation workflows

```rust
pub trait ConfigImporter {
    fn can_import(&self, source: &str) -> bool;
    fn import(&self, source: &str) -> Result<HyprlandConfig, ImportError>;
    fn validate(&self, source: &str) -> Result<ValidationResult, ImportError>;
}

pub struct HyprlandConfigImporter;
pub struct NixConfigImporter;
pub struct JsonConfigImporter;
```

### `ConfigExporter`

**Purpose:** Export configurations to various formats
**Integration:** Used in export dialogs with format selection

```rust
pub trait ConfigExporter {
    fn supported_formats(&self) -> Vec<ExportFormat>;
    fn export(&self, config: &HyprlandConfig, format: ExportFormat) -> Result<String, ExportError>;
    fn preview(&self, config: &HyprlandConfig, format: ExportFormat) -> Result<String, ExportError>;
}

pub struct StandardExporter;
pub struct NixExporter;
```

## NixOS Integration API

### `NixModuleGenerator`

**Purpose:** Generate NixOS-compatible configuration modules
**Integration:** Specialized export functionality for NixOS users

```rust
pub struct NixModuleGenerator;

impl NixModuleGenerator {
    pub fn generate_system_module(&self, config: &HyprlandConfig) -> Result<String, NixError>;
    pub fn generate_home_manager_module(&self, config: &HyprlandConfig) -> Result<String, NixError>;
    pub fn generate_flake_output(&self, config: &HyprlandConfig) -> Result<String, NixError>;
    pub fn validate_nix_syntax(&self, module: &str) -> Result<(), NixError>;
}
```

## Error Types

### `DetectionError`
- `UnsupportedDistribution` - Distribution not recognized or supported
- `MissingFiles` - Required system files not found
- `ParseError` - Cannot parse distribution information

### `PathError`
- `NotFound` - Configuration path doesn't exist
- `PermissionDenied` - Insufficient permissions to access path
- `InvalidPath` - Path is malformed or invalid

### `ProfileError`
- `NotFound` - Profile ID doesn't exist
- `DuplicateName` - Profile name already exists
- `InvalidConfig` - Configuration data is invalid
- `StorageError` - Cannot read/write profile storage

### `ImportError`
- `UnsupportedFormat` - Import format not supported
- `ParseError` - Cannot parse source configuration
- `ValidationError` - Configuration doesn't meet requirements

### `ExportError`
- `UnsupportedFormat` - Export format not supported
- `GenerationError` - Cannot generate output in requested format
- `ValidationError` - Generated output fails validation

## TUI Integration Points

### Main Menu Integration
- Add "Profiles" menu item for profile management
- Add "Import/Export" menu item for configuration transfer
- Display current distribution information in status bar

### Profile Management Dialog
```rust
pub struct ProfileDialog {
    manager: ProfileManager,
    selected_profile: Option<String>,
}

impl ProfileDialog {
    pub fn show_profile_list(&mut self) -> Result<(), TuiError>;
    pub fn create_profile_dialog(&mut self) -> Result<(), TuiError>;
    pub fn apply_profile_dialog(&mut self) -> Result<(), TuiError>;
}
```

### Import/Export Dialog
```rust
pub struct ImportExportDialog {
    importers: Vec<Box<dyn ConfigImporter>>,
    exporters: Vec<Box<dyn ConfigExporter>>,
}

impl ImportExportDialog {
    pub fn show_import_dialog(&mut self) -> Result<HyprlandConfig, TuiError>;
    pub fn show_export_dialog(&mut self, config: &HyprlandConfig) -> Result<(), TuiError>;
}
```

## Configuration Storage Format

### Profile Storage Schema
```json
{
  "version": "1.0",
  "profiles": {
    "profile_id": {
      "name": "Profile Name",
      "description": "Optional description",
      "created": "2025-07-25T10:00:00Z",
      "modified": "2025-07-25T10:00:00Z",
      "config": {
        // Hyprland configuration data
      },
      "metadata": {
        "tags": ["work", "gaming"],
        "distribution": "arch",
        "version": "1.0"
      }
    }
  }
}
```

## Service Integration

### Application Startup
1. `DistributionDetector::detect()` - Identify current distribution
2. `ConfigPathManager::new()` - Initialize path management
3. `ProfileManager::new()` - Load existing profiles
4. Cache results for performance

### Configuration Operations
1. Validate paths using `ConfigPathManager::validate_path()`
2. Create backups before modifications
3. Apply changes through appropriate APIs
4. Update profile storage if needed