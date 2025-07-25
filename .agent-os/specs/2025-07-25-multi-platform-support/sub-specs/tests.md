# Tests Specification

This is the tests coverage details for the spec detailed in @.agent-os/specs/2025-07-25-multi-platform-support/spec.md

> Created: 2025-07-25
> Version: 1.0.0

## Test Coverage

### Unit Tests

**DistributionDetector**
- Test detection of major Linux distributions (Ubuntu, Arch, Fedora, openSUSE, Debian)
- Test NixOS detection with proper identification
- Test fallback behavior for unknown distributions
- Test parsing of various `/etc/os-release` formats
- Test error handling for missing system files
- Test caching mechanism for repeated detections

**ConfigPathManager**
- Test path resolution for each supported distribution
- Test fallback path discovery when standard paths don't exist
- Test permission validation for configuration directories
- Test symlink handling in NixOS environments
- Test path caching and invalidation
- Test error handling for inaccessible paths

**ProfileManager**
- Test profile creation with valid configurations
- Test profile loading and saving operations
- Test profile deletion with proper cleanup
- Test profile listing and filtering
- Test duplicate name handling
- Test profile validation and error recovery
- Test profile export to different formats
- Test profile metadata management

**NixModuleGenerator**
- Test system module generation with various configurations
- Test home-manager module generation
- Test Nix flake output generation
- Test Nix syntax validation
- Test edge cases with complex configurations
- Test error handling for invalid configurations

**Import/Export Framework**
- Test importing from standard Hyprland configurations
- Test importing from NixOS modules
- Test importing from JSON/TOML formats
- Test export format validation
- Test bidirectional conversion accuracy
- Test error handling for malformed input

### Integration Tests

**Cross-Distribution Compatibility**
- Test application behavior on Ubuntu 22.04/24.04
- Test application behavior on Arch Linux
- Test application behavior on Fedora latest
- Test application behavior on NixOS stable/unstable
- Test configuration path discovery across distributions
- Test profile portability between distributions

**NixOS Integration Workflow**
- Test end-to-end NixOS module generation and validation
- Test home-manager integration with proper module structure
- Test Nix flake integration with correct outputs
- Test bidirectional conversion (Nix → internal → Nix)
- Test integration with existing NixOS configurations
- Test handling of NixOS-specific configuration patterns

**Profile Management Workflow**
- Test complete profile lifecycle (create, save, load, delete)
- Test profile switching with configuration application
- Test profile export and import between systems
- Test profile backup and recovery scenarios
- Test concurrent profile operations
- Test profile corruption recovery

**File System Operations**
- Test atomic file operations with concurrent access
- Test backup creation and restoration
- Test permission handling across different user setups
- Test symlink preservation and handling
- Test large configuration file handling
- Test disk space exhaustion scenarios

### Feature Tests

**Distribution Detection Scenarios**
- User runs application on fresh Ubuntu installation
- User runs application on Arch Linux with custom config paths
- User runs application on NixOS with home-manager
- User runs application on unsupported distribution
- User has custom Hyprland installation in non-standard location
- User has multiple Hyprland configurations (system vs user)

**Profile Management Scenarios**
- User creates work profile with specific monitor setup
- User creates gaming profile with different key bindings
- User switches between profiles multiple times
- User exports profile to share with colleague
- User imports profile from different system
- User deletes profile and recovers from backup

**NixOS Integration Scenarios**
- NixOS user exports current config as home-manager module
- NixOS user exports config as system-level module
- NixOS user exports config as flake output
- NixOS user imports existing Nix configuration
- NixOS user validates generated Nix syntax
- NixOS user integrates with existing configuration.nix

**Import/Export Scenarios**
- User imports existing hyprland.conf from different system
- User exports configuration to JSON for version control
- User exports configuration to multiple formats simultaneously
- User imports configuration with validation errors
- User exports large configuration with many bindings
- User handles import/export with special characters and Unicode

## Mocking Requirements

**File System Operations**
- Mock file system access for testing different distributions
- Mock `/etc/os-release` content for distribution testing
- Mock configuration directory structures
- Mock file permissions and access errors
- Mock disk space and I/O errors

**External Process Execution**
- Mock `hyprctl` command execution and responses
- Mock system command execution for distribution detection
- Mock Nix command validation (nix-instantiate)
- Mock file watching and change notifications

**Time-Based Operations**
- Mock system time for testing timestamp generation
- Mock file modification times for cache validation
- Mock timeout scenarios for external operations

**Network and System State**
- Mock system information queries
- Mock environment variables for path resolution
- Mock user home directory and permissions
- Mock system package manager detection

## Test Data and Fixtures

**Distribution Test Data**
```
tests/fixtures/os-release/
├── ubuntu-22.04
├── ubuntu-24.04
├── arch-rolling
├── fedora-39
├── nixos-23.11
├── nixos-unstable
├── opensuse-tumbleweed
└── unknown-distro
```

**Configuration Test Data**
```
tests/fixtures/configs/
├── minimal-hyprland.conf
├── complex-hyprland.conf
├── nix-home-manager.nix
├── nix-system-module.nix
├── invalid-config.conf
└── unicode-config.conf
```

**Profile Test Data**
```
tests/fixtures/profiles/
├── work-profile.json
├── gaming-profile.json
├── minimal-profile.json
├── corrupted-profile.json
└── legacy-format-profile.json
```

## Performance Tests

**Startup Performance**
- Measure application startup time on different distributions
- Measure distribution detection performance
- Measure profile loading time with various profile counts
- Measure cache effectiveness for repeated operations

**Memory Usage**
- Monitor memory usage during large configuration operations
- Test memory efficiency with multiple profiles loaded
- Test memory cleanup after profile operations
- Monitor memory usage during import/export operations

**File I/O Performance**
- Measure profile save/load performance
- Test concurrent file access performance
- Measure backup creation performance
- Test performance with large configuration files

## Error Handling Tests

**File System Errors**
- Test behavior when configuration directory is read-only
- Test behavior when disk space is exhausted
- Test behavior when files are locked by other processes
- Test behavior when symlinks are broken

**Configuration Errors**
- Test handling of malformed Hyprland configurations
- Test handling of invalid Nix syntax in imports
- Test handling of corrupted profile data
- Test handling of schema version mismatches

**System Environment Errors**
- Test behavior when `/etc/os-release` is missing
- Test behavior when home directory is inaccessible
- Test behavior when required system commands are missing
- Test behavior when Hyprland is not installed

**User Input Errors**
- Test handling of invalid profile names
- Test handling of invalid export paths
- Test handling of invalid import sources
- Test handling of duplicate profile operations

## Regression Tests

**Previous Bug Scenarios**
- Test scenarios that caused issues in previous versions
- Test edge cases discovered during development
- Test platform-specific issues that were resolved
- Test performance regressions from optimization changes

**Compatibility Tests**
- Test backward compatibility with older profile formats
- Test forward compatibility with newer configuration options
- Test compatibility between different application versions
- Test compatibility with different Hyprland versions

## Continuous Integration Tests

**Automated Distribution Testing**
- Docker-based testing on multiple Linux distributions
- Automated testing of NixOS integration using Nix containers
- Cross-platform testing for different architectures
- Automated performance regression detection

**Integration Test Pipeline**
- Automated testing of complete user workflows
- End-to-end testing of import/export functionality
- Automated validation of generated Nix modules
- Automated testing of profile operations across platforms