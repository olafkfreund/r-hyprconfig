# Technical Specification

This is the technical specification for the spec detailed in @.agent-os/specs/2025-07-25-multi-platform-support/spec.md

> Created: 2025-07-25
> Version: 1.0.0

## Technical Requirements

- **Distribution Detection System**: Implement robust Linux distribution detection using `/etc/os-release`, `/etc/lsb-release`, and distribution-specific markers
- **Configuration Path Resolution**: Dynamic configuration path discovery supporting standard locations across major distributions (Ubuntu, Arch, Fedora, openSUSE, etc.)
- **NixOS Module Generator**: Generate syntactically correct Nix expressions with proper attribute sets and option handling
- **Profile Storage System**: JSON-based profile storage with metadata, versioning, and validation
- **Import/Export Pipeline**: Modular system supporting multiple input/output formats with validation and error handling
- **File System Abstraction**: Cross-platform file operations with proper permission handling and atomic writes
- **Configuration Validation**: Schema validation for imported configurations and generated exports

## Approach Options

**Option A: Runtime Distribution Detection with Static Path Mapping**
- Pros: Simple implementation, fast execution, predictable behavior
- Cons: Requires maintenance for new distributions, limited flexibility for custom setups

**Option B: Dynamic Path Discovery with Fallback Chain** (Selected)
- Pros: Adaptable to custom configurations, future-proof, handles edge cases gracefully
- Cons: More complex implementation, potential for slower startup on first run

**Option C: User-Configured Path Override System**
- Pros: Maximum flexibility, works with any setup
- Cons: Requires user configuration, poor out-of-box experience

**Rationale:** Option B provides the best balance of automatic functionality and flexibility. The dynamic discovery system can adapt to various environments while maintaining good performance through caching, and the fallback chain ensures reliability across different setups.

## External Dependencies

- **os_info** - Operating system detection and information gathering
- **Justification:** Provides reliable cross-platform OS detection with extensive distribution support

- **walkdir** - Recursive directory traversal for configuration discovery
- **Justification:** Efficient filesystem traversal with filtering capabilities for configuration file discovery

- **regex** - Pattern matching for configuration parsing and validation
- **Justification:** Essential for parsing various configuration file formats and validating NixOS expressions

- **uuid** - Unique identifier generation for profiles and tracking
- **Justification:** Required for generating unique profile identifiers and tracking configuration versions

## Architecture Components

### Distribution Detection Module
```rust
struct DistributionInfo {
    name: String,
    version: String,
    id: String,
    config_paths: Vec<PathBuf>,
    package_manager: PackageManager,
}
```

### Configuration Path Resolver
```rust
trait ConfigPathResolver {
    fn resolve_hyprland_config(&self) -> Result<PathBuf>;
    fn resolve_application_config(&self) -> Result<PathBuf>;
    fn resolve_backup_location(&self) -> Result<PathBuf>;
}
```

### NixOS Module Generator
```rust
struct NixModuleGenerator {
    config: HyprlandConfig,
    target_type: NixTargetType, // System | HomeManager | Flake
}

enum NixTargetType {
    System,
    HomeManager,
    Flake,
}
```

### Profile Management System
```rust
struct ProfileManager {
    profiles: HashMap<String, ConfigProfile>,
    active_profile: Option<String>,
    storage_path: PathBuf,
}

struct ConfigProfile {
    id: String,
    name: String,
    description: Option<String>,
    created: DateTime<Utf8>,
    modified: DateTime<Utf8>,
    config: HyprlandConfig,
    metadata: ProfileMetadata,
}
```

### Import/Export Framework
```rust
trait ConfigExporter {
    fn export(&self, config: &HyprlandConfig, format: ExportFormat) -> Result<String>;
}

trait ConfigImporter {
    fn import(&self, source: &str, format: ImportFormat) -> Result<HyprlandConfig>;
}

enum ExportFormat {
    HyprlandConf,
    NixSystemModule,
    NixHomeManager,
    NixFlake,
    JSON,
    TOML,
}
```

## Implementation Strategy

### Phase 1: Distribution Detection and Path Resolution
1. Implement `DistributionDetector` with support for major distributions
2. Create `ConfigPathResolver` with dynamic path discovery
3. Add fallback mechanisms for unknown distributions
4. Implement configuration path caching for performance

### Phase 2: Profile Management Infrastructure
1. Design and implement profile storage schema
2. Create profile CRUD operations with validation
3. Add profile switching functionality with safety checks
4. Implement profile import/export for sharing

### Phase 3: NixOS Integration
1. Implement NixOS module generation for system-level configuration
2. Add home-manager module generation support
3. Create Nix flake output generation
4. Add bidirectional conversion (Nix → internal format)

### Phase 4: Enhanced Import/Export
1. Implement traditional Hyprland config import
2. Add JSON/TOML export formats
3. Create validation pipeline for imported configurations
4. Add format auto-detection for imports

## Integration Points

### File System Operations
- Atomic file writes with backup creation
- Permission preservation during configuration updates
- Symlink handling for NixOS environments
- Configuration file watching for external changes

### User Interface Integration
- Profile selection dialog in main TUI
- Export format selection with preview
- Import validation feedback with error details
- Distribution detection status display

### Error Handling Strategy
- Graceful degradation for unsupported distributions
- Clear error messages for configuration validation failures
- Recovery mechanisms for corrupted profiles
- Rollback capabilities for failed configuration updates

## Performance Considerations

- **Lazy Loading**: Distribution detection and path resolution only when needed
- **Caching**: Cache discovered paths and distribution information
- **Async Operations**: Non-blocking file I/O for large configuration operations
- **Memory Management**: Efficient handling of large configuration files and multiple profiles