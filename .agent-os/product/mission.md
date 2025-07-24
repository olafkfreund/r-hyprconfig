# Product Mission

> Last Updated: 2025-07-24
> Version: 1.0.0

## Pitch

r-hyprconfig is a modern terminal user interface (TUI) application that helps Hyprland users visually configure their window manager settings by providing an intuitive interface for hyprctl commands with automatic config file management for both standard Linux distributions and NixOS workflows.

## Users

### Primary Customers

- **Hyprland Enthusiasts**: Power users who want a visual interface for configuration management
- **Linux System Administrators**: IT professionals managing multiple Hyprland setups across different systems
- **NixOS Community**: Declarative configuration users needing seamless integration with their workflow

### User Personas

**Alex** (25-35 years old)
- **Role:** Software Developer / Linux Enthusiast
- **Context:** Uses Hyprland as daily driver, frequently tweaks configuration for productivity
- **Pain Points:** Command-line configuration is tedious, hard to remember all hyprctl options, difficult to visualize changes
- **Goals:** Quick configuration changes, visual feedback, maintain clean config files

**Morgan** (30-45 years old)
- **Role:** System Administrator
- **Context:** Manages multiple Linux workstations with Hyprland for development teams
- **Pain Points:** Inconsistent configurations across machines, time-consuming manual setup, lack of standardization
- **Goals:** Standardized configurations, quick deployment, easy maintenance

**Jordan** (20-30 years old)
- **Role:** NixOS User / DevOps Engineer
- **Context:** Uses NixOS for declarative system management, wants Hyprland config to fit the paradigm
- **Pain Points:** Manual config doesn't integrate with NixOS workflow, hard to version control, reproducibility issues
- **Goals:** Declarative configuration, version control integration, reproducible setups

## The Problem

### Configuration Complexity Barrier

Configuring Hyprland through hyprctl commands and config files is complex and intimidating for new users. The command-line interface requires memorizing numerous options and syntax, creating a steep learning curve that prevents adoption.

**Our Solution:** Provide an intuitive TUI that presents all configuration options in organized, visual panels with real-time preview.

### Lack of Visual Feedback

Users cannot easily visualize how configuration changes will affect their desktop environment without applying them first. This trial-and-error approach is inefficient and frustrating.

**Our Solution:** Real-time configuration preview and immediate visual feedback for all setting changes.

### Configuration Management Inconsistency

Managing Hyprland configurations across different Linux distributions and deployment scenarios (especially NixOS) requires different approaches, leading to fragmented workflows and maintenance overhead.

**Our Solution:** Unified configuration interface that automatically handles different deployment targets and file management strategies.

### Poor Discoverability of Features

Hyprland has extensive configuration options that are poorly documented or hard to discover through command-line tools alone. Users often don't know what's possible.

**Our Solution:** Comprehensive interface that exposes all available options with descriptions and examples.

## Differentiators

### Native Rust Performance with Modern TUI

Unlike web-based configuration tools or Python scripts, we provide native Rust performance with a modern terminal interface built on ratatui. This results in instant startup times, minimal resource usage, and seamless integration with terminal workflows.

### Direct hyprctl Integration

Unlike configuration file editors that require restarts, we integrate directly with hyprctl for real-time configuration changes. This provides immediate feedback and allows users to test settings without disrupting their workflow.

### Universal Linux Distribution Support

Unlike distribution-specific tools, we support both traditional Linux distributions and NixOS with intelligent configuration file management. This ensures consistent experience across different deployment scenarios.

## Key Features

### Core Features

- **Interactive TUI Interface:** Clean, organized panels showing all Hyprland configuration options with intuitive navigation
- **Real-time Configuration:** Direct hyprctl integration for immediate setting changes and visual feedback
- **Automatic Config Management:** Intelligent saving and loading of configuration files with backup support
- **Settings Organization:** Logical grouping of related settings in expandable sections for easy discovery
- **Search and Filter:** Quick finding of specific configuration options across all categories

### Platform Features

- **NixOS Integration:** Generate NixOS-compatible configuration modules for declarative system management
- **Cross-distribution Support:** Handle different config file locations and formats across Linux distributions
- **Configuration Profiles:** Save and switch between different configuration presets for various use cases
- **Import/Export:** Easy sharing of configurations between systems and users

### User Experience Features

- **Visual Previews:** Show the effect of settings changes before applying them permanently
- **Help System:** Built-in documentation and examples for each configuration option
- **Undo/Redo:** Safe experimentation with easy rollback of configuration changes