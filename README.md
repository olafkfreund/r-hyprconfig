# R-Hyprconfig

A modern terminal user interface (TUI) for managing Hyprland window manager configuration, built with Rust and ratatui.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

## Features

- 🎨 **Modern TUI Interface** - Clean, intuitive terminal interface built with ratatui
- ⚡ **Real-time Configuration** - Live configuration changes via hyprctl commands
- 📦 **Organized Panels** - Configuration options organized in easy-to-navigate boxes
- 💾 **Auto-save Support** - Automatic config file management for standard Linux
- 🏗️ **NixOS Compatible** - Special handling for NixOS declarative configurations
- ⌨️ **Keyboard Navigation** - Full keyboard control with intuitive shortcuts

## Installation

### Prerequisites

- Hyprland window manager installed and running
- `hyprctl` command available in PATH

### With Nix (Recommended)

#### Direct run (no installation)
```bash
# Run the latest version directly
nix run github:olafkfreund/r-hyprconfig

# Run from local clone
git clone https://github.com/olafkfreund/r-hyprconfig.git
cd r-hyprconfig
nix run .
```

#### Install to profile
```bash
# Install globally
nix profile install github:olafkfreund/r-hyprconfig

# Install from local clone
nix profile install .
```

#### NixOS Configuration
Add to your NixOS configuration:

```nix
{
  imports = [
    (builtins.getFlake "github:olafkfreund/r-hyprconfig").nixosModules.default
  ];
  
  programs.r-hyprconfig.enable = true;
}
```

#### Home Manager Configuration
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

### From Source (Traditional)

#### Prerequisites
- Rust 1.70 or higher
- pkg-config and OpenSSL development libraries

```bash
git clone https://github.com/olafkfreund/r-hyprconfig.git
cd r-hyprconfig
cargo build --release
```

The binary will be available at `target/release/r-hyprconfig`.

## Usage

### Basic Usage

```bash
# Run the TUI
./r-hyprconfig

# Enable debug mode
./r-hyprconfig --debug
```

### Navigation

- **Tab** / **→** : Navigate to next panel
- **Shift+Tab** / **←** : Navigate to previous panel
- **↑** / **↓** : Navigate within panel
- **Enter** : Select/edit configuration option
- **R** : Reload configuration from Hyprland
- **S** : Save configuration to file
- **Q** / **Esc** : Quit application

## Configuration Panels

The TUI organizes Hyprland configuration into the following panels:

### Core Configuration
- **General** - Basic window and border settings
- **Input** - Keyboard and mouse configuration
- **Decoration** - Visual effects, blur, shadows, rounding

### Advanced Features  
- **Animations** - Animation settings and bezier curves
- **Gestures** - Touchpad and gesture configuration
- **Binds** - Keybindings and shortcuts

### Rules & Misc
- **Window Rules** - Application-specific window behavior
- **Layer Rules** - Layer-specific rendering rules  
- **Misc** - Additional Hyprland options

## Architecture

### Core Components

- **App** (`src/app.rs`) - Main application state and event handling
- **UI** (`src/ui.rs`) - TUI rendering and layout management
- **HyprCtl** (`src/hyprctl.rs`) - Integration with Hyprland via hyprctl commands
- **Config** (`src/config.rs`) - Configuration file management and persistence

### Key Features

#### Real-time Configuration
The application uses `hyprctl` commands to:
- Read current Hyprland configuration values
- Apply changes immediately to the running compositor
- Reload configuration when needed

#### File Management
- **Standard Linux**: Direct modification of `hyprland.conf`
- **NixOS**: Generates NixOS-compatible configuration files
- **Backup**: Automatic backup creation before modifications

#### Modern TUI
- Built with ratatui for cross-platform terminal rendering
- Responsive layout that adapts to terminal size
- Color-coded panels with focus indicators
- Scrollable lists with visual scrollbars

## Configuration

### Application Config

R-Hyprconfig stores its settings in `~/.config/r-hyprconfig/config.toml`:

```toml
hyprland_config_path = "/home/user/.config/hypr/hyprland.conf"
backup_enabled = true
auto_save = false
nixos_mode = false
```

### NixOS Integration

When running on NixOS, the application automatically detects the environment and:
- Generates NixOS-compatible configuration syntax
- Saves to a separate file for manual integration
- Provides instructions for incorporating changes

## Development

### Quick Start with Nix + devenv

The easiest way to get started with development:

```bash
# Clone the repository
git clone https://github.com/olafkfreund/r-hyprconfig.git
cd r-hyprconfig

# Enter the development environment
nix develop

# Or with direnv (automatically loads when entering directory)
echo "use flake" > .envrc
direnv allow

# Start developing
cargo run -- --debug
```

### Development Environment Features

The Nix flake provides a complete development environment with:

- **Rust toolchain** (stable, with rust-analyzer, clippy, rustfmt)
- **Development tools** (cargo-watch, cargo-audit, cargo-edit, etc.)
- **System dependencies** (pkg-config, OpenSSL, etc.)
- **Pre-commit hooks** (formatting, linting, security audit)
- **Task automation** (via justfile and devenv scripts)

### Available Commands

With the development environment loaded:

```bash
# Development commands
just build          # Build debug version
just run            # Run the application
just test           # Run tests
just lint           # Format and lint code
just watch          # Auto-rebuild on changes

# Nix commands
nix build           # Build with Nix
nix run             # Run with Nix
nix develop         # Enter dev shell
devenv up           # Start development services

# Package management
just add-dep <crate>    # Add dependency
just update-deps        # Update all dependencies
just audit             # Security audit
```

### Project Structure

```
├── src/
│   ├── main.rs           # Entry point and CLI parsing
│   ├── app.rs            # Application state and event loop
│   ├── ui.rs             # TUI rendering and layout
│   ├── hyprctl.rs        # Hyprland integration
│   └── config.rs         # Configuration management
├── templates/
│   └── default_hyprland.conf  # Default configuration template
├── .devcontainer/        # VS Code dev container config
├── .github/workflows/    # CI/CD workflows
├── flake.nix            # Nix flake with dev environment
├── justfile             # Task automation
└── .envrc               # direnv configuration
```

### Building

#### With Nix (Recommended)
```bash
# Build with Nix (reproducible)
nix build .#r-hyprconfig

# Development build
nix develop --command cargo build

# Release build
nix develop --command cargo build --release
```

#### Traditional Cargo
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
```

### VS Code Development

The project includes a complete VS Code devcontainer setup:

1. Install the "Dev Containers" extension
2. Open the project in VS Code
3. Click "Reopen in Container" when prompted
4. The development environment will be automatically set up

### Docker Development

```bash
# Build development container
docker build --target development -t r-hyprconfig-dev .

# Run development container
docker run -it --rm -v $(pwd):/workspace r-hyprconfig-dev

# Build production container
docker build -t r-hyprconfig .
```

### Dependencies

- **ratatui** - Terminal user interface framework
- **crossterm** - Cross-platform terminal manipulation
- **tokio** - Async runtime for hyprctl commands
- **serde** - Serialization for configuration files
- **clap** - Command line argument parsing

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

### Development Guidelines

1. Follow Rust best practices and idioms
2. Maintain compatibility with the latest stable Rust
3. Add tests for new functionality
4. Update documentation for user-facing changes

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Hyprland](https://hyprland.org/) - The amazing wayland compositor this tool is built for
- [ratatui](https://github.com/ratatui-org/ratatui) - The excellent TUI framework
- The Rust community for the amazing ecosystem

## Roadmap

- [ ] Configuration value editing and validation
- [ ] Real-time preview of changes
- [ ] Configuration import/export
- [ ] Plugin system for custom configurations
- [ ] Integration with popular Hyprland themes
