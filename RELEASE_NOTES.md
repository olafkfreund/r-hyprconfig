# r-hyprconfig v1.0.0 Release Notes

> 🎉 **First Stable Release**  
> Released: 2025-07-25  
> Compatibility: Hyprland 0.50.1+

## 🌟 What is r-hyprconfig?

r-hyprconfig is a modern terminal user interface (TUI) application that helps Hyprland users visually configure their window manager settings through an intuitive interface. It provides real-time configuration management with automatic config file handling for both standard Linux distributions and NixOS workflows.

## ✨ Key Features

### 🖥️ **Interactive TUI Interface**
- Clean, organized panels showing all Hyprland configuration options
- Intuitive Tab-based navigation between configuration sections
- Real-time configuration preview and editing
- Built-in help system and keyboard shortcuts

### ⚡ **Real-time Hyprland Integration**
- Direct `hyprctl` integration for immediate setting changes
- Live configuration loading from running Hyprland instance
- Automatic configuration reload and validation
- Seamless switching between runtime and persistent changes

### 🛡️ **Robust Configuration Management**
- Automatic backup creation before any changes
- Comprehensive input validation preventing invalid configurations
- Support for all Hyprland configuration types (integers, floats, colors, etc.)
- Smart parsing of keybinds, window rules, and layer rules

### 🏗️ **Multi-Platform Support**
- **NixOS Integration**: Generate NixOS-compatible configuration modules
- **Cross-distribution**: Works on any Linux distribution with Hyprland
- **Configuration Profiles**: Save and switch between different setups
- **Import/Export**: Easy sharing of configurations between systems

### 🎨 **Advanced User Experience**
- **Search and Filter**: Quick finding of specific configuration options
- **Theme System**: Multiple color schemes for personalized experience
- **Configuration Preview**: See changes before applying them
- **Validation System**: Prevent invalid settings with helpful error messages

## 🚀 Installation

### Quick Install (Recommended)

**From GitHub Releases:**
```bash
# Download the latest release
wget https://github.com/olafkfreund/r-hyprconfig/releases/download/v1.0.0/r-hyprconfig_v1.0.0_amd64.deb

# Install (Debian/Ubuntu)
sudo dpkg -i r-hyprconfig_v1.0.0_amd64.deb

# Install (Fedora/RHEL)
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
nix run github:olafkfreund/r-hyprconfig/v1.0.0
```

### System Requirements
- Linux x86_64 (ARM64 via source compilation)
- Hyprland window manager (0.50.1+ recommended)
- Terminal with Unicode support

## 📖 Usage

### Basic Usage
```bash
# Start the interactive TUI
r-hyprconfig

# Test functionality without opening TUI
r-hyprconfig --test-save

# Enable debug mode for troubleshooting
r-hyprconfig --debug
```

### Navigation
- **Tab/Shift+Tab**: Switch between configuration sections
- **↑/↓**: Navigate options within a section
- **Enter**: Edit selected option
- **s**: Save all changes to config file
- **r**: Reload configuration from Hyprland
- **q**: Quit application
- **?/F1**: Show help

### Configuration Sections
1. **General**: Gaps, borders, layouts, colors
2. **Input**: Keyboard, mouse, touchpad settings
3. **Decoration**: Blur, shadows, rounding, animations
4. **Animations**: Animation curves and timing
5. **Gestures**: Workspace swipe and gesture controls
6. **Binds**: Keybind management and editing
7. **Window Rules**: Application-specific window behavior
8. **Layer Rules**: Layer-specific rendering rules
9. **Misc**: Additional Hyprland options

## 🔧 Advanced Features

### NixOS Integration
Generate NixOS-compatible configurations:
- Press `n` to open NixOS export dialog
- Choose between Home Manager, System Config, or Flakes
- Real-time preview of generated Nix configuration
- Export directly to `.nix` files

### Configuration Management
- **Profiles**: Press `b` to manage configuration profiles
- **Themes**: Press `t` to cycle through color themes
- **Export/Import**: Press `e`/`m` for configuration backup/restore
- **Search**: Press `/` to search for specific options

### Validation and Safety
- All configuration changes are validated before saving
- Automatic backup creation with timestamps
- Graceful error handling with descriptive messages
- Rollback capability for problematic changes

## 🐛 Known Issues & Limitations

### Current Limitations
- TUI-only interface (no GUI integration)
- Requires terminal environment for operation
- Some advanced Hyprland features may need manual configuration

### Workarounds
- Use `--debug` flag for troubleshooting connection issues
- Fallback to config file editing if hyprctl unavailable
- Check `/path/to/config` for backup files if recovery needed

## 🔄 Migration from Other Tools

### From Manual Configuration
1. Run `r-hyprconfig` to import your existing `hyprland.conf`
2. All current settings will be automatically loaded
3. Make changes through the TUI interface
4. Save to update your configuration file

### From Hyprctl Scripts
- r-hyprconfig integrates directly with hyprctl
- All existing hyprctl commands remain functional
- Runtime changes made with hyprctl will be detected
- Use the TUI to make changes persistent

## 🛠️ Technical Details

### Architecture
- **Language**: Rust (memory-safe, high-performance)
- **TUI Framework**: ratatui with crossterm backend
- **Configuration**: TOML-based application settings
- **Integration**: Direct hyprctl process execution

### Performance
- **Startup Time**: < 1 second
- **Memory Usage**: Minimal (~10MB)
- **Configuration Loading**: Sub-second for large configs
- **Real-time Updates**: Immediate via hyprctl

### Security
- Input validation for all configuration options
- Safe file operations with proper permissions
- No elevated privileges required
- Sanitized command execution

## 📊 Development Stats

- **Development Time**: 3 months intensive development
- **Lines of Code**: ~6,000 lines of Rust
- **Test Coverage**: Comprehensive with real Hyprland testing
- **Bug Reports**: 3 critical issues found and resolved
- **Performance**: Optimized for speed and memory efficiency

## 🙏 Acknowledgments

- **Hyprland Community**: For the amazing window manager
- **Rust Community**: For excellent tooling and libraries
- **ratatui**: For the fantastic TUI framework
- **Beta Testers**: For early feedback and bug reports

## 🔮 What's Next?

### Planned for v1.1.0
- Undo/redo functionality for configuration changes
- Configuration diff viewer and comparison tools
- Enhanced plugin system for community extensions
- Improved batch configuration management

### Future Roadmap
- Multi-monitor setup wizard
- Configuration sharing platform integration
- Advanced theming and customization options
- Performance monitoring integration

## 🐛 Bug Reports & Feature Requests

Found a bug or have a feature request?

- **GitHub Issues**: https://github.com/olafkfreund/r-hyprconfig/issues
- **Documentation**: https://github.com/olafkfreund/r-hyprconfig/blob/main/README.md
- **Discussions**: https://github.com/olafkfreund/r-hyprconfig/discussions

## 📄 License

r-hyprconfig is released under the MIT License. See [LICENSE](LICENSE) for details.

---

**🎉 Thank you for using r-hyprconfig!**

We hope this tool makes managing your Hyprland configuration easier and more enjoyable. Happy tiling! 🚀