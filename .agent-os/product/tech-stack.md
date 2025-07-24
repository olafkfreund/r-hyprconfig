# Technical Stack

> Last Updated: 2025-07-24
> Version: 1.0.0

## Core Technologies

### Application Framework
- **Language:** Rust
- **Version:** 1.70+
- **Edition:** 2021

### User Interface
- **TUI Framework:** ratatui
- **Version:** Latest stable
- **Terminal Backend:** crossterm

## Dependencies

### Core Libraries
- **ratatui:** Terminal user interface framework
- **crossterm:** Cross-platform terminal manipulation
- **clap:** Command-line argument parsing
- **serde:** Serialization/deserialization for configuration
- **tokio:** Async runtime for non-blocking operations

### Configuration Management
- **toml:** Configuration file format parsing
- **dirs:** Cross-platform directory discovery
- **fs_extra:** Enhanced filesystem operations

### System Integration
- **hyprctl Integration:** Direct process execution and IPC
- **File Watching:** Configuration file change detection
- **Process Management:** Hyprland service communication

## Build System

### Package Manager
- **Build Tool:** Cargo
- **Version:** Latest stable with Rust toolchain
- **Package Registry:** crates.io

### Development Dependencies
- **Testing:** Built-in Rust testing framework
- **Benchmarking:** criterion for performance testing
- **Linting:** clippy for code quality
- **Formatting:** rustfmt for consistent style

## Deployment

### Distribution
- **Target:** Linux x86_64
- **Package Format:** Cargo binary, AUR package, Nix package
- **Installation:** cargo install, package managers

### Configuration Locations
- **Standard Linux:** ~/.config/hypr/
- **NixOS:** Managed through Nix configuration
- **Application Config:** ~/.config/r-hyprconfig/

## Infrastructure

### Version Control
- **Repository:** Git-based development
- **Platform:** GitHub or similar
- **CI/CD:** GitHub Actions for automated testing and releases

### Testing Environment
- **Unit Tests:** Rust built-in testing
- **Integration Tests:** Hyprland environment testing
- **Platform Testing:** Multiple Linux distribution validation