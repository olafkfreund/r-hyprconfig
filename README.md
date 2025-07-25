# r-hyprconfig

> 🚀 **A modern TUI for visually configuring Hyprland** 

A production-ready terminal user interface for managing Hyprland window manager configuration with real-time updates, comprehensive validation, and seamless NixOS integration.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Version](https://img.shields.io/badge/version-v1.0.0-green.svg)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)

## 🚀 Features

### 🎨 **Modern TUI Interface**
- Clean, intuitive terminal interface built with ratatui
- Responsive layout that adapts to terminal size
- Color-coded panels with focus indicators
- Scrollable lists with visual scrollbars

### ⚡ **Real-time Configuration**
- Live configuration changes via hyprctl commands
- Instant preview of changes
- No restart required for most settings

### 📦 **Organized Configuration Management**
- Configuration options organized in easy-to-navigate panels
- Logical grouping: General, Input, Decoration, Animations, Gestures, Binds, Rules
- Smart search and filtering across all options

### 🏗️ **NixOS Integration** *(NEW)*
- Automatic NixOS environment detection
- Export configurations in NixOS-compatible format
- Support for multiple NixOS configuration types:
  - Home Manager standalone
  - System-level configuration
  - Flake-based Home Manager
  - Flake-based NixOS system

### 🔧 **Batch Configuration Management** *(NEW)*
- Create and manage multiple configuration profiles
- Apply settings across multiple machines
- Batch operations: Apply, Merge, Replace, Backup
- Perfect for system administrators

### 💾 **Advanced Configuration Handling**
- Automatic config file management
- Smart backup creation before modifications
- Import/Export functionality with TOML format
- Configuration validation and error checking

### ⌨️ **Keyboard Navigation**
- Full keyboard control with intuitive shortcuts
- Vim-like navigation patterns
- Context-sensitive help system

## 🎯 Real-Life Use Cases

### 1. **Daily Hyprland User**
*Sarah is a developer who wants to fine-tune her Hyprland setup*

```bash
# Quick configuration tweaks
r-hyprconfig

# Navigate to General panel
# Adjust gaps_in from 5 to 8 pixels
# Press Enter to edit, type new value
# Press S to save configuration
# Changes applied instantly!
```

**Sarah's Workflow:**
- Opens r-hyprconfig when she wants to adjust window gaps
- Uses the search function (/) to quickly find specific settings
- Tests different animation curves in real-time
- Saves configurations with meaningful names for different workflows

### 2. **System Administrator**
*Mike manages 20+ developer workstations with Hyprland*

```bash
# Create standardized profiles
r-hyprconfig

# Press B for Batch Management
# 1. Create new profile "developer-workstation"
# Configure standard settings:
#   - gaps_in = 5
#   - gaps_out = 10
#   - rounding = 8
#   - Standard keybindings

# Deploy to multiple machines:
# 2. Select existing profile
# 3. Choose "Apply" operation
# Execute across all target machines
```

**Mike's Workflow:**
- Creates company-standard configuration profiles
- Uses batch operations to deploy consistent settings
- Maintains backup profiles for quick rollbacks
- Manages different profiles for different teams (frontend, backend, QA)

### 3. **NixOS User**
*Alex uses NixOS with declarative configuration management*

```bash
# Configure Hyprland graphically, export to Nix
r-hyprconfig

# Make visual changes in TUI
# Press N for NixOS Export
# Choose configuration type:
#   1. Home Manager standalone
#   2. System configuration  
#   3. Flake-based Home Manager ← Alex chooses this
#   4. Flake-based NixOS system

# Preview generated Nix configuration
# Press Enter to export
```

**Generated Nix Configuration:**
```nix
{
  description = "Home Manager flake with Hyprland configuration";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    hyprland.url = "github:hyprwm/Hyprland";
  };
  
  outputs = { nixpkgs, home-manager, hyprland, ... }: {
    homeConfigurations."${USER}" = home-manager.lib.homeManagerConfiguration {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      modules = [
        hyprland.homeManagerModules.default
        {
          wayland.windowManager.hyprland = {
            enable = true;
            settings = {
              general = {
                gaps_in = 5;
                gaps_out = 10;
                border_size = 2;
              };
              decoration = {
                rounding = 8;
                blur = {
                  enabled = true;
                  size = 6;
                };
              };
              bind = [
                "SUPER, Q, exec, kitty"
                "SUPER, W, killactive"
                "SUPER, M, exit"
              ];
              windowrule = [
                "float, ^(kitty)$"
                "opacity 0.8 0.8, ^(Alacritty)$"
              ];
            };
          };
        }
      ];
    };
  };
}
```

### 4. **Gaming Enthusiast**
*Jordan wants different configurations for work and gaming*

```bash
# Create work profile
r-hyprconfig
# Press B → 1 (Create profile)
# Configure: minimal gaps, no animations, productivity keybinds

# Create gaming profile  
# Press B → 1 (Create profile)
# Configure: flashy animations, gaming-optimized keybinds

# Quick switching between profiles
# Press B → 2 → 1 (Apply work profile)
# Press B → 2 → 1 (Apply gaming profile)
```

**Jordan's Profiles:**
- **Work Profile**: Minimal distractions, productivity-focused keybinds
- **Gaming Profile**: Eye-candy animations, gaming-specific window rules
- **Streaming Profile**: Optimized for screen sharing and OBS

## 📱 Application Screenshots

### Main Interface
```
┌─ r-hyprconfig v0.1.0 ─ NixOS Detected ─ Theme: Nord ────────────────────────────┐
│ ⚙️  Hyprland Configuration Manager                                               │
│ 🏠 Config: ~/.config/hypr/hyprland.conf                                        │
│ 📦 NixOS: Home Manager + Flakes                                                │
└──────────────────────────────────────────────────────────────────────────────┘

┌─ Configuration Panels ──────────────────────────────────────────────────────────┐
│ [General] [Input] [Decoration] [Animations] [Gestures] [Binds] [Rules] [Misc]   │
└──────────────────────────────────────────────────────────────────────────────┘

┌─ General Configuration ──────────────────────────────────────────────┐
│ ✓ gaps_in                              5           │ Inner window gaps │
│   gaps_out                            10           │ Outer window gaps │
│   border_size                          2           │ Window border size│
│   col.active_border                 0xff5e81ac     │ Active border     │
│   col.inactive_border               0xff3b4252     │ Inactive border   │
│   cursor_inactive_timeout             0           │ Hide cursor delay │
│   layout                             dwindle      │ Default layout    │
│   no_cursor_warps                     false       │ Disable warping   │
│   resize_on_border                    false       │ Resize on border  │
│   extend_border_grab_area              5          │ Border grab area  │
└──────────────────────────────────────────────────────────────────────┘

┌─ Help ─────────────────────────────────────────────────────────────────────────┐
│ Navigation: ↑/↓ Select  Tab/Shift+Tab Switch Panel  Enter Edit  Esc Cancel     │
│ Actions: S Save  R Reload  N NixOS Export  B Batch  E Export  M Import         │
└──────────────────────────────────────────────────────────────────────────────┘
```

### NixOS Export Dialog
```
┌─ Export as NixOS Configuration ──────────────────────────────────────────────────┐
│                                                                                  │
│ Choose NixOS configuration type:                                                 │
│                                                                                  │
│ 1. Home Manager (standalone)       - ~/.config/nixpkgs/home.nix                │
│ 2. System Configuration           - /etc/nixos/configuration.nix                │
│ 3. Flake Home Manager     ← [*]   - flake.nix with Home Manager               │
│ 4. Flake NixOS System             - flake.nix with NixOS                       │
│                                                                                  │
│ ┌─ Preview ────────────────────────────────────────────────────────────────────┐ │
│ │ {                                                                           │ │
│ │   wayland.windowManager.hyprland = {                                       │ │
│ │     enable = true;                                                          │ │
│ │     settings = {                                                            │ │
│ │       general = {                                                           │ │
│ │         gaps_in = 5;                                                        │ │
│ │         gaps_out = 10;                                                      │ │
│ │         border_size = 2;                                                    │ │
│ │       };                                                                    │ │
│ │       bind = [                                                              │ │
│ │         "SUPER, Q, exec, kitty"                                            │ │
│ │         "SUPER, W, killactive"                                             │ │
│ │       ];                                                                    │ │
│ │     };                                                                      │ │
│ │   };                                                                        │ │
│ │ }                                                                           │ │
│ └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                  │
│ Press Enter to export • Esc to cancel                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Batch Management Dialog
```
┌─ Batch Configuration Management ─────────────────────────────────────────────────┐
│                                                                                  │
│ 🔧 Batch Configuration Management                                                │
│                                                                                  │
│ Manage configuration profiles:                                                   │
│                                                                                  │
│ 1. Create new profile from current config                                       │
│ 2. Select existing profile for operations                                       │
│ 3. Delete profile                                                               │
│                                                                                  │
│ Current profiles:                                                                │
│ • developer-workstation-v1 (created: 2025-01-15)                              │
│ • gaming-setup (created: 2025-01-14)                                           │
│ • minimal-productivity (created: 2025-01-10)                                   │
│                                                                                  │
│ Press Esc to cancel                                                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 🛠️ Installation

### Prerequisites

- Hyprland window manager (v0.50.1+ recommended)
- `hyprctl` command available in PATH
- Linux x86_64 (ARM64 via source compilation)

### Quick Install (Recommended)

**GitHub Releases:**
```bash
# Download latest release
wget https://github.com/olafkfreund/r-hyprconfig/releases/download/v1.0.0/r-hyprconfig_v1.0.0_amd64.deb

# Install (Debian/Ubuntu)
sudo dpkg -i r-hyprconfig_v1.0.0_amd64.deb

# Install (Fedora/RHEL/CentOS)  
sudo rpm -i r-hyprconfig-v1.0.0-1.x86_64.rpm
```

**From Source:**
```bash
git clone https://github.com/olafkfreund/r-hyprconfig.git
cd r-hyprconfig
cargo build --release
sudo cp target/release/r-hyprconfig /usr/local/bin/
```

**With Nix:**
```bash
# Run directly
nix run github:olafkfreund/r-hyprconfig/v1.0.0

# Or install to profile
nix profile install github:olafkfreund/r-hyprconfig/v1.0.0
```

### NixOS System Configuration

Add to your NixOS configuration:

```nix
{
  imports = [
    (builtins.getFlake "github:olafkfreund/r-hyprconfig").nixosModules.default
  ];
  
  programs.r-hyprconfig.enable = true;
}
```

### Home Manager Configuration

Add to your Home Manager configuration:

```nix
{
  imports = [
    (builtins.getFlake "github:olafkfreund/r-hyprconfig").homeManagerModules.default
  ];
  
  programs.r-hyprconfig = {
    enable = true;
    settings = {
      backup_enabled = true;
      auto_save = false;
      nixos_mode = true;
    };
  };
}
```

### From Source (Traditional Linux)

```bash
# Prerequisites: Rust 1.70+, pkg-config, OpenSSL dev libraries
git clone https://github.com/olafkfreund/r-hyprconfig.git
cd r-hyprconfig
cargo build --release

# Binary available at target/release/r-hyprconfig
./target/release/r-hyprconfig
```

### Package Managers

```bash
# AUR (Arch Linux)
yay -S r-hyprconfig-git

# Cargo (Rust package manager)
cargo install r-hyprconfig

# Homebrew (macOS/Linux)
brew install olafkfreund/tap/r-hyprconfig
```

## 📋 Usage Guide

### Basic Navigation

| Key | Action |
|-----|--------|
| `Tab` / `→` | Navigate to next panel |
| `Shift+Tab` / `←` | Navigate to previous panel |
| `↑` / `↓` | Navigate within panel |
| `Page Up` / `Page Down` | Scroll by page |
| `Home` / `End` | Go to first/last item |

### Configuration Management

| Key | Action |
|-----|--------|
| `Enter` | Edit selected configuration option |
| `S` | Save configuration to file |
| `R` | Reload configuration from Hyprland |
| `E` | Export configuration (TOML) |
| `M` | Import configuration |

### Advanced Features

| Key | Action |
|-----|--------|
| `N` | Export as NixOS configuration |
| `B` | Batch configuration management |
| `T` | Cycle through themes |
| `/` | Search configuration options |
| `?` / `F1` | Show help overlay |
| `Q` / `Esc` | Quit application |

### Search and Filtering

Press `/` to enter search mode:

```bash
# Search examples:
/gaps          # Find all gap-related settings
/border        # Find border configuration
/animation     # Find animation settings
/bind          # Find keybinding options
```

### Configuration Editing

When you press `Enter` on a setting:

1. **Text Values**: Direct text input with cursor
2. **Boolean Values**: Toggle with Space
3. **Numeric Values**: Type new number
4. **Select Options**: Use ↑/↓ to choose from predefined options
5. **Keybinds**: Special editor for modifier+key combinations

## 🏗️ Configuration Panels

### 1. General Panel
Core window management settings:
- Window gaps (inner/outer)
- Border configuration (size, colors)
- Layout options (dwindle, master)
- Cursor behavior

### 2. Input Panel  
Keyboard and mouse configuration:
- Keyboard layout and options
- Mouse sensitivity and acceleration  
- Touchpad settings
- Special key behaviors

### 3. Decoration Panel
Visual appearance settings:
- Window rounding
- Blur effects and intensity
- Drop shadows
- Opacity settings

### 4. Animations Panel
Animation configuration:
- Animation curves (bezier definitions)
- Animation speeds
- Window transition effects
- Workspace animations

### 5. Gestures Panel
Touchpad gesture configuration:
- Workspace switching gestures
- Window management gestures
- Custom gesture commands

### 6. Binds Panel
Keybinding management:
- Application launchers
- Window management shortcuts
- Workspace navigation
- Custom commands

### 7. Window Rules Panel
Application-specific window behavior:
- Floating rules for specific applications
- Size and position rules
- Opacity and effects rules
- Workspace assignment rules

### 8. Layer Rules Panel
Layer-specific rendering rules:
- Overlay effects for bars/panels
- Blur rules for specific layers
- Z-order management

### 9. Misc Panel
Additional Hyprland options:
- Hyprland logo display
- Debug settings
- Experimental features

## 🔧 Advanced Features

### NixOS Integration

The application automatically detects NixOS environments and provides seamless integration:

#### Detection Methods
- Checks for `/etc/NIXOS` file
- Detects `NIX_STORE` environment variable
- Verifies `nixos-rebuild` command availability
- Scans for Nix store directory (`/nix/store`)

#### Supported Configuration Types

1. **Home Manager Standalone**
   ```nix
   # ~/.config/nixpkgs/home.nix
   { config, pkgs, ... }: {
     wayland.windowManager.hyprland = {
       enable = true;
       settings = {
         # Your settings here
       };
     };
   }
   ```

2. **System Configuration**
   ```nix
   # /etc/nixos/configuration.nix
   { config, pkgs, ... }: {
     programs.hyprland = {
       enable = true;
       # User configuration in Home Manager
     };
   }
   ```

3. **Flake-based Home Manager**
   ```nix
   {
     description = "Home Manager flake with Hyprland";
     inputs = {
       hyprland.url = "github:hyprwm/Hyprland";
       # ... other inputs
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
       hyprland.url = "github:hyprwm/Hyprland";
     };
     outputs = { ... }: {
       nixosConfigurations.hostname = # ... configuration
     };
   }
   ```

### Batch Configuration Management

Perfect for system administrators managing multiple Hyprland installations:

#### Profile Management
- **Create Profiles**: Capture current configuration as reusable profiles
- **Profile Metadata**: Automatic timestamping and descriptions
- **Profile Storage**: Organized storage in `~/.config/r-hyprconfig/profiles/`

#### Batch Operations

1. **Apply**: Apply profile settings to current configuration
2. **Merge**: Intelligently merge profile with existing settings
3. **Replace**: Replace entire configuration with profile
4. **Backup**: Create backup before applying changes

#### Real-World Batch Scenarios

**Scenario 1: New Employee Onboarding**
```bash
# System admin creates standard developer profile
r-hyprconfig
# Configure optimal developer settings
# Press B → 1 to create "new-developer-2025" profile

# Deploy to new employee machines
# Press B → 2 → Select "new-developer-2025"
# Press 1 for Apply operation
# Consistent setup across all machines!
```

**Scenario 2: Seasonal Configuration Updates**
```bash
# Create "summer-theme" profile with bright colors
# Create "winter-theme" profile with dark colors
# Use batch operations to deploy seasonally
```

**Scenario 3: Team-Specific Configurations**
```bash
# Frontend team: Focus on visual tools
# Backend team: Terminal-heavy workflows  
# QA team: Multi-monitor optimized
# Each team gets optimized profile
```

## 📂 Configuration Files

### Application Configuration
Location: `~/.config/r-hyprconfig/config.toml`

```toml
hyprland_config_path = "/home/user/.config/hypr/hyprland.conf"
backup_enabled = true
auto_save = false
nixos_mode = false
theme = "Nord"

[nixos]
config_type = "HomeManager"
export_path = "/home/user/.config/nixos-exports/"

[batch]
profile_directory = "/home/user/.config/r-hyprconfig/profiles/"
auto_backup = true
```

### Profile Storage
Location: `~/.config/r-hyprconfig/profiles/`

```
profiles/
├── developer-workstation-v1.toml
├── gaming-setup.toml
├── minimal-productivity.toml
└── metadata.json
```

### Export Directory
Location: `~/.config/r-hyprconfig/exports/`

```
exports/
├── hyprland_export_20250115_143022.toml
├── nixos-exports/
│   ├── hyprland_nixos_export_20250115_143045.nix
│   └── hyprland_nixos_export_20250114_091234.nix
└── backups/
    └── hyprland_backup_20250115_143022.conf
```

## 🎨 Themes

Built-in themes for different preferences:

### Available Themes
- **Nord**: Cool blue/gray palette (default)
- **Dracula**: Dark purple vampire theme
- **Gruvbox**: Warm retro colors
- **Tokyo Night**: Modern dark theme
- **Catppuccin**: Pastel dark theme
- **Solarized**: Classic light/dark scientific theme

### Theme Switching
Press `T` to cycle through themes, or configure in `config.toml`:

```toml
theme = "Nord"  # Nord, Dracula, Gruvbox, TokyoNight, Catppuccin, Solarized
```

## 🔍 Troubleshooting

### Common Issues

#### "hyprctl not found"
**Problem**: Hyprland is not installed or not in PATH
**Solution**: 
```bash
# Verify Hyprland installation
which hyprctl
# Should return path like /usr/bin/hyprctl

# If not found, install Hyprland first
```

#### "Permission denied writing config"
**Problem**: Cannot write to Hyprland config file
**Solution**:
```bash
# Check file permissions
ls -la ~/.config/hypr/hyprland.conf

# Fix permissions if needed
chmod 644 ~/.config/hypr/hyprland.conf
```

#### "NixOS export not available"
**Problem**: Running on non-NixOS system
**Solution**: NixOS export features are only available on NixOS systems. Use regular export (E key) instead.

#### Configuration not taking effect
**Problem**: Changes saved but not visible in Hyprland
**Solution**:
```bash
# Manual reload
hyprctl reload

# Or restart Hyprland
# Mod + Shift + M (default exit keybind)
```

### Debug Mode

Run with debug output for troubleshooting:

```bash
r-hyprconfig --debug

# Shows detailed logging:
# - Configuration file operations
# - hyprctl command execution
# - Error stack traces
# - Performance metrics
```

### Log Files

Application logs are stored in:
- Linux: `~/.local/share/r-hyprconfig/logs/`
- macOS: `~/Library/Application Support/r-hyprconfig/logs/`

## 🤝 Contributing

We welcome contributions! Here's how to get started:

### Development Setup

```bash
# Clone repository
git clone https://github.com/olafkfreund/r-hyprconfig.git
cd r-hyprconfig

# Development with Nix (recommended)
nix develop

# Or traditional Rust development
cargo build
cargo test
cargo run -- --debug
```

### Development Environment Features

The Nix flake provides:
- **Rust toolchain** (stable with rust-analyzer, clippy, rustfmt)
- **Development tools** (cargo-watch, cargo-audit, etc.)
- **System dependencies** (pkg-config, OpenSSL)
- **Pre-commit hooks** (formatting, linting, security audit)

### Contribution Guidelines

1. **Code Style**: Follow `cargo fmt` and `cargo clippy` recommendations
2. **Testing**: Add tests for new functionality
3. **Documentation**: Update README and inline docs
4. **Commits**: Use conventional commit messages

### Areas for Contribution

- **New Themes**: Add more color schemes
- **Configuration Options**: Support additional Hyprland settings
- **Platform Support**: Testing on different distributions
- **UI Improvements**: Enhanced TUI components
- **Documentation**: Examples, tutorials, use cases

## 📈 Roadmap

### Current Version (v1.0.0) ✅ STABLE RELEASE
- ✅ Complete TUI interface with intuitive navigation
- ✅ Real-time hyprctl integration and configuration editing
- ✅ Comprehensive configuration validation system
- ✅ NixOS integration with export functionality
- ✅ Batch configuration management for system admins
- ✅ Multi-theme support with 6 built-in themes
- ✅ Search and filtering across all options
- ✅ Automatic backup and restore functionality
- ✅ Cross-distribution Linux compatibility
- ✅ Production-ready stability and error handling

### Upcoming Features (v1.1.0)
- [ ] Undo/redo functionality for configuration changes
- [ ] Configuration diff viewer and comparison tools
- [ ] Enhanced batch operations with scheduling
- [ ] Plugin system for custom configuration modules
- [ ] Improved validation with contextual error messages
- [ ] Configuration templates and presets

### Future Plans (v1.2.0+)
- [ ] Remote configuration management via SSH
- [ ] Configuration synchronization between machines
- [ ] Integration with popular Hyprland theme repositories
- [ ] Visual timeline for configuration history
- [ ] Community configuration sharing platform
- [ ] Advanced theming and customization tools

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Hyprland](https://hyprland.org/) - The amazing Wayland compositor
- [ratatui](https://github.com/ratatui-org/ratatui) - Excellent TUI framework
- [NixOS](https://nixos.org/) - Inspiration for declarative configuration
- The Rust community for the incredible ecosystem

## 📞 Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/olafkfreund/r-hyprconfig/issues)
- **Discussions**: [Community discussions](https://github.com/olafkfreund/r-hyprconfig/discussions)
- **Discord**: Join the Hyprland community for real-time help

---

**Made with ❤️ for the Hyprland community**