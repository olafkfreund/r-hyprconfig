# Feature Documentation

Comprehensive overview of all r-hyprconfig features and capabilities.

## 🎨 User Interface Features

### Modern Terminal User Interface (TUI)

**Built with ratatui framework for cross-platform compatibility**

- **Responsive Layout**: Automatically adapts to terminal size and resolution
- **Color-coded Panels**: Each configuration category has distinct visual styling
- **Focus Indicators**: Clear visual feedback for current selection and navigation state
- **Scrollable Lists**: Efficient rendering of large configuration sets with visual scrollbars
- **Professional Borders**: Rounded borders and clean typography throughout

**Keyboard Navigation**
- Full keyboard control with no mouse dependency
- Vim-inspired navigation patterns for power users
- Context-sensitive key bindings
- Consistent navigation across all panels and dialogs

**Visual Feedback**
- Real-time status indicators for configuration state
- Visual confirmation of changes and operations
- Progress indicators for long-running operations
- Error highlighting with clear recovery instructions

## ⚡ Configuration Management

### Real-time Configuration Editing

**Direct Hyprland Integration**
- Uses `hyprctl` commands for immediate configuration changes
- No restart required for most configuration options
- Live preview of changes as you type
- Instant feedback on configuration validity

**Configuration Loading**
- Automatic detection of current Hyprland configuration
- Intelligent parsing of `hyprland.conf` files
- Fallback to default values when configuration is missing
- Smart handling of configuration comments and formatting

**Configuration Persistence**
- Automatic backup creation before any modifications
- Timestamp-based backup naming for easy tracking
- Atomic write operations to prevent corruption
- Validation before writing to ensure configuration integrity

### Multi-Panel Organization

**General Configuration Panel**
- Window gaps (inner/outer gap settings)
- Border configuration (size, active/inactive colors)
- Layout options (dwindle, master, etc.)
- Cursor behavior and timeout settings
- Window focus and warping behavior

**Input Configuration Panel**
- Keyboard layout and variant settings
- Repeat rate and delay configuration
- Mouse sensitivity and acceleration
- Touchpad settings and gestures
- Special key behavior (numlock, caps lock)

**Decoration Panel**
- Window rounding radius
- Blur effects configuration (size, passes, ignore opacity)
- Drop shadow settings (range, render power, color)
- Opacity settings for various window states
- Screen shader effects

**Animations Panel**
- Animation curve definitions (bezier curves)
- Animation speed and timing settings
- Window transition effects
- Workspace switch animations
- Fade in/out configurations

**Gestures Panel**
- Touchpad gesture configuration
- Workspace switching gestures
- Window management gestures
- Custom gesture command bindings

**Keybinds Panel**
- Visual keybinding editor with modifier+key selection
- Application launcher shortcuts
- Window management keybinds
- Workspace navigation shortcuts
- Custom command bindings

**Window Rules Panel**
- Application-specific window behavior rules
- Floating window configurations
- Size and position rules
- Opacity and effects rules per application
- Workspace assignment rules

**Layer Rules Panel**
- Layer-specific rendering configurations
- Overlay effects for status bars and panels
- Blur rules for specific UI layers
- Z-order management for complex setups

**Miscellaneous Panel**
- Hyprland logo display settings
- Debug and logging configuration
- Experimental feature toggles
- Performance tuning options

## 🏗️ NixOS Integration

### Automatic Environment Detection

**Detection Methods**
- Checks for `/etc/NIXOS` system file
- Detects `NIX_STORE` environment variable
- Verifies availability of `nixos-rebuild` command
- Scans for Nix store directory at `/nix/store`
- Detects Home Manager installation

**Configuration Discovery**
- Automatic discovery of NixOS configuration files
- Detection of flake-based vs traditional configurations
- Home Manager configuration location discovery
- System vs user configuration differentiation

### Multi-Format NixOS Export

**Supported Configuration Types**

1. **Home Manager Standalone Configuration**
   ```nix
   # ~/.config/nixpkgs/home.nix
   { config, pkgs, ... }: {
     wayland.windowManager.hyprland = {
       enable = true;
       settings = {
         # Generated configuration
       };
     };
   }
   ```

2. **System-Level Configuration**
   ```nix
   # /etc/nixos/configuration.nix
   { config, pkgs, ... }: {
     programs.hyprland = {
       enable = true;
       # User configuration typically in Home Manager
     };
   }
   ```

3. **Flake-based Home Manager**
   ```nix
   {
     description = "Home Manager flake with Hyprland";
     inputs = {
       nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
       home-manager.url = "github:nix-community/home-manager";
       hyprland.url = "github:hyprwm/Hyprland";
     };
     outputs = { ... }: {
       homeConfigurations."user" = # ... configuration
     };
   }
   ```

4. **Flake-based NixOS System**
   ```nix
   {
     description = "NixOS flake with Hyprland";
     inputs = {
       nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
       hyprland.url = "github:hyprwm/Hyprland";
     };
     outputs = { ... }: {
       nixosConfigurations.hostname = # ... configuration
     };
   }
   ```

**Export Features**
- Real-time preview of generated Nix configuration
- Proper Nix syntax formatting and indentation
- Automatic input flake management
- Intelligent value type conversion (strings, numbers, booleans)
- Array formatting for keybinds and rules
- Attribute set organization by logical grouping

**Configuration Conversion**
- Traditional hyprland.conf → Nix attribute sets
- Proper escaping of special characters in Nix strings
- Boolean and numeric value type preservation
- Array conversion for multi-value settings
- Comment preservation where applicable

## 🔧 Batch Configuration Management

### Profile Management System

**Profile Creation**
- Capture current configuration state as reusable profiles
- Automatic timestamping with ISO 8601 format
- Optional profile descriptions for documentation
- Metadata tracking (creation time, last modified)
- Profile validation before storage

**Profile Storage**
- Organized storage in `~/.config/r-hyprconfig/profiles/`
- TOML-based profile format for human readability
- Atomic profile operations to prevent corruption
- Profile versioning and backup capabilities

**Profile Operations**
- Profile listing with creation dates and descriptions
- Profile deletion with confirmation prompts
- Profile duplication for creating variants
- Profile import/export for sharing between systems

### Batch Operation Types

**Apply Operation**
- Apply profile settings to current configuration
- Merge profile values with existing settings
- Preserve non-conflicting current settings
- Create backup before applying changes

**Merge Operation**
- Intelligent merging of profile with current settings
- Conflict resolution with user preferences
- Additive approach for arrays (keybinds, rules)
- Smart handling of duplicate configurations

**Replace Operation**
- Complete replacement of current configuration with profile
- Full backup creation before replacement
- Clean slate approach for standardization
- Atomic replacement to prevent partial states

**Backup Operation**
- Create timestamped backup of current configuration
- Include all configuration files and settings
- Metadata tracking for backup management
- Restoration capabilities from backup files

### Enterprise Features

**Multi-Profile Management**
- Centralized profile storage and organization
- Team-based profile categorization
- Profile deployment across multiple machines
- Standardization enforcement capabilities

**Audit and Tracking**
- Operation logging with timestamps
- Change tracking and history
- Rollback capabilities to previous states
- Configuration drift detection

## 💾 Data Management

### Configuration Formats

**Import/Export Support**
- TOML format for human-readable exports
- JSON format for programmatic integration
- Traditional hyprland.conf format compatibility
- NixOS Nix expression generation

**Backup System**
- Automatic backup creation before modifications
- Timestamped backup organization
- Configurable backup retention policies
- One-click restoration from backups

**Data Validation**
- Configuration syntax validation before saving
- Value range checking for numeric settings
- Enum validation for categorical settings
- Cross-reference validation for dependent settings

### File Management

**Intelligent File Handling**
- Atomic write operations for data integrity
- Lock file management to prevent conflicts
- Temporary file cleanup on errors
- Permission checking before operations

**Path Management**
- Automatic directory creation as needed
- Cross-platform path handling
- Environment variable expansion
- Symlink resolution and handling

## 🎨 Themes and Customization

### Built-in Theme System

**Available Themes**
- **Nord**: Cool blue/gray Nordic palette (default)
- **Dracula**: Dark purple vampire-inspired theme
- **Gruvbox**: Warm retro orange/brown color scheme
- **Tokyo Night**: Modern dark theme with blue accents
- **Catppuccin**: Soft pastel dark theme
- **Solarized**: Scientific light/dark balanced palette

**Theme Features**
- Consistent color application across all UI elements
- High contrast for accessibility
- Color-blind friendly palette options
- Customizable accent colors
- Theme persistence across sessions

**Customization Options**
- Runtime theme switching with 'T' key
- Configuration file theme setting
- Custom theme definition support
- Per-user theme preferences

## 🔍 Search and Navigation

### Advanced Search System

**Search Capabilities**
- Full-text search across all configuration options
- Real-time filtering as you type
- Cross-panel search results
- Regular expression support for power users

**Search Features**
- Debounced search for performance
- Search result caching for speed
- Progressive search for large datasets
- Search history and recall

**Navigation Enhancements**
- Quick jump to search results
- Breadcrumb navigation for deep configurations
- Recently accessed items tracking
- Bookmark system for frequently used settings

### Accessibility Features

**Keyboard Accessibility**
- Full keyboard navigation (no mouse required)
- Consistent key bindings across all interfaces
- Tab order optimization for screen readers
- Keyboard shortcut customization

**Visual Accessibility**
- High contrast theme options
- Configurable font sizes
- Color-blind friendly palettes
- Clear focus indicators

## ⚙️ System Integration

### Hyprland Integration

**hyprctl Interface**
- Direct communication with running Hyprland instance
- Real-time configuration query and modification
- Event listening for external configuration changes
- Graceful handling of Hyprland restarts

**Configuration File Management**
- Intelligent parsing of hyprland.conf files
- Preservation of comments and formatting
- Backup creation before modifications
- Syntax validation and error reporting

### Platform Support

**Linux Distribution Compatibility**
- Arch Linux and AUR package support
- Ubuntu/Debian package compatibility
- Fedora/RHEL support
- NixOS native integration
- Generic Linux binary distribution

**Desktop Environment Integration**
- XDG desktop file for application launchers
- Proper application categorization
- Desktop notifications for important events
- System tray integration where appropriate

## 🛡️ Error Handling and Recovery

### Robust Error Management

**Error Detection**
- Configuration syntax validation
- File system error handling
- Network connectivity issues (for updates)
- Permission and access control errors

**Recovery Mechanisms**
- Automatic backup restoration on corruption
- Graceful degradation when features unavailable
- Clear error messages with suggested solutions
- Safe mode operation for troubleshooting

**User Feedback**
- Clear error message presentation
- Suggested solutions for common problems
- Debug mode for detailed troubleshooting
- Log file generation for support

### Data Integrity

**Validation Systems**
- Pre-save configuration validation
- Post-save verification
- Checksum verification for critical files
- Atomic operation guarantees

**Backup and Recovery**
- Automatic backup creation
- Manual backup triggering
- Point-in-time recovery options
- Configuration history tracking

## 📊 Performance Features

### Optimization

**Memory Efficiency**
- Lazy loading of configuration data
- Efficient data structures for large configs
- Memory cleanup on operations
- Minimal baseline memory footprint

**Rendering Performance**
- Optimized TUI rendering pipeline
- Virtualized lists for large datasets
- Selective redraw for changed areas
- Frame rate limiting for battery efficiency

**Startup Optimization**
- Fast cold start times (<200ms)
- Incremental loading strategies
- Configuration caching where appropriate
- Parallel initialization of components

### Scalability

**Large Configuration Handling**
- Efficient handling of complex configurations
- Pagination for large setting lists
- Progressive loading of configuration sections
- Search indexing for fast lookups

**Multi-User Support**
- Per-user configuration isolation
- Shared system configuration awareness
- Permission-based feature availability
- Profile sharing between users

## 🔧 Developer Features

### Extensibility

**Plugin Architecture** (Planned)
- Custom configuration module support
- Third-party theme integration
- Custom export format plugins
- External tool integration

**API Integration**
- REST API for remote management (planned)
- Configuration synchronization protocols
- Webhook support for external notifications
- Integration with configuration management systems

### Development Tools

**Debug Features**
- Comprehensive logging system
- Debug mode with detailed tracing
- Performance profiling capabilities
- Configuration state inspection

**Testing Support**
- Mock mode for development
- Test configuration sets
- Automated testing framework integration
- Configuration validation testing

This comprehensive feature set makes r-hyprconfig a powerful tool for managing Hyprland configurations across different use cases, from individual users to enterprise deployments.