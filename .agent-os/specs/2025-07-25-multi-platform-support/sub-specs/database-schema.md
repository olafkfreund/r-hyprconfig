# Database Schema

This is the database schema implementation for the spec detailed in @.agent-os/specs/2025-07-25-multi-platform-support/spec.md

> Created: 2025-07-25
> Version: 1.0.0

## Storage Strategy

The multi-platform support features will use file-based storage rather than a traditional database, aligning with the application's lightweight TUI nature and cross-platform requirements. All data will be stored in JSON format for human readability and easy debugging.

## Profile Storage Schema

### Primary Profile Storage File
**Location:** `~/.config/r-hyprconfig/profiles.json`

```json
{
  "version": "1.0.0",
  "metadata": {
    "created": "2025-07-25T10:00:00Z",
    "last_modified": "2025-07-25T15:30:00Z",
    "active_profile": "uuid-of-active-profile",
    "total_profiles": 3
  },
  "profiles": {
    "550e8400-e29b-41d4-a716-446655440001": {
      "name": "Work Setup",
      "description": "Productivity-focused configuration with multiple monitors",
      "created": "2025-07-25T10:00:00Z",
      "modified": "2025-07-25T14:00:00Z",
      "tags": ["work", "productivity", "multi-monitor"],
      "distribution_info": {
        "name": "arch",
        "version": "rolling",
        "created_on": "arch"
      },
      "config": {
        "general": {
          "gaps_in": 5,
          "gaps_out": 10,
          "border_size": 2,
          "col.active_border": "rgba(33ccffee) rgba(00ff99ee) 45deg",
          "col.inactive_border": "rgba(595959aa)"
        },
        "decoration": {
          "rounding": 10,
          "drop_shadow": true,
          "shadow_range": 4,
          "shadow_render_power": 3,
          "col.shadow": "rgba(1a1a1aee)"
        },
        "animations": {
          "enabled": true,
          "bezier": [
            "myBezier, 0.05, 0.9, 0.1, 1.05"
          ],
          "animation": [
            "windows, 1, 7, myBezier",
            "windowsOut, 1, 7, default, popin 80%",
            "border, 1, 10, default",
            "borderangle, 1, 8, default",
            "fade, 1, 7, default",
            "workspaces, 1, 6, default"
          ]
        },
        "input": {
          "kb_layout": "us",
          "kb_variant": "",
          "kb_model": "",
          "kb_options": "",
          "kb_rules": "",
          "follow_mouse": 1,
          "touchpad": {
            "natural_scroll": false
          },
          "sensitivity": 0
        },
        "binds": [
          {
            "key": "SUPER, Q, exec, kitty",
            "description": "Launch terminal"
          },
          {
            "key": "SUPER, C, killactive",
            "description": "Close active window"
          }
        ],
        "monitors": [
          {
            "name": "DP-1",
            "resolution": "2560x1440@144",
            "position": "0x0",
            "scale": 1.0
          },
          {
            "name": "HDMI-A-1", 
            "resolution": "1920x1080@60",
            "position": "2560x0",
            "scale": 1.0
          }
        ]
      },
      "export_history": [
        {
          "timestamp": "2025-07-25T14:00:00Z",
          "format": "nix_home_manager",
          "target_path": "/home/user/.config/nixpkgs/hyprland.nix"
        }
      ]
    }
  }
}
```

## Distribution Configuration Cache

### Distribution Detection Cache File
**Location:** `~/.config/r-hyprconfig/distribution.json`

```json
{
  "version": "1.0.0",
  "last_detected": "2025-07-25T10:00:00Z",
  "distribution": {
    "id": "arch",
    "name": "Arch Linux",
    "version": "rolling",
    "version_id": "rolling",
    "pretty_name": "Arch Linux",
    "variant": null,
    "variant_id": null
  },
  "paths": {
    "hyprland_config": "/home/user/.config/hypr/hyprland.conf",
    "hyprland_config_dir": "/home/user/.config/hypr/",
    "application_config": "/home/user/.config/r-hyprconfig/",
    "backup_directory": "/home/user/.config/r-hyprconfig/backups/",
    "profile_storage": "/home/user/.config/r-hyprconfig/profiles.json"
  },
  "capabilities": {
    "supports_nix": false,
    "package_manager": "pacman",
    "init_system": "systemd",
    "config_format": "standard"
  },
  "validation": {
    "paths_verified": true,
    "permissions_ok": true,
    "last_verified": "2025-07-25T10:00:00Z"
  }
}
```

## Export Templates Storage

### Export Templates Configuration
**Location:** `~/.config/r-hyprconfig/export_templates.json`

```json
{
  "version": "1.0.0",
  "templates": {
    "nix_system": {
      "name": "NixOS System Module",
      "description": "System-level Hyprland configuration for NixOS",
      "template": "{ config, pkgs, ... }:\n\n{\n  wayland.windowManager.hyprland = {\n    enable = true;\n    settings = {{CONFIG_JSON}};\n  };\n}",
      "file_extension": ".nix",
      "validation_required": true
    },
    "nix_home_manager": {
      "name": "Home Manager Module",
      "description": "User-level Hyprland configuration for home-manager",
      "template": "{ config, pkgs, ... }:\n\n{\n  wayland.windowManager.hyprland = {\n    enable = true;\n    settings = {{CONFIG_JSON}};\n  };\n}",
      "file_extension": ".nix",
      "validation_required": true
    },
    "hyprland_conf": {
      "name": "Traditional Hyprland Config",
      "description": "Standard hyprland.conf format",
      "template": "{{HYPRLAND_CONFIG}}",
      "file_extension": ".conf",
      "validation_required": false
    }
  }
}
```

## Application Settings Schema

### Main Application Configuration
**Location:** `~/.config/r-hyprconfig/config.json`

```json
{
  "version": "1.0.0",
  "settings": {
    "default_export_format": "hyprland_conf",
    "auto_backup": true,
    "backup_retention_days": 30,
    "profile_auto_save": true,
    "distribution_detection": {
      "auto_detect": true,
      "cache_duration_hours": 24,
      "fallback_paths": [
        "~/.config/hypr/",
        "/etc/hypr/",
        "~/.hyprland/"
      ]
    },
    "nix_integration": {
      "enabled": true,
      "default_target": "home_manager",
      "validate_syntax": true,
      "format_output": true
    },
    "profiles": {
      "show_created_date": true,
      "show_distribution_info": true,
      "default_tags": ["custom"],
      "auto_tag_by_distribution": true
    }
  },
  "ui": {
    "show_distribution_in_status": true,
    "profile_preview_enabled": true,
    "export_preview_enabled": true
  },
  "performance": {
    "cache_enabled": true,
    "lazy_load_profiles": true,
    "async_exports": true
  }
}
```

## File Organization Structure

```
~/.config/r-hyprconfig/
├── config.json                    # Main application settings
├── profiles.json                  # Profile storage
├── distribution.json              # Distribution detection cache
├── export_templates.json          # Export format templates
├── backups/                       # Configuration backups
│   ├── 2025-07-25_hyprland.conf.bak
│   ├── 2025-07-24_hyprland.conf.bak
│   └── ...
├── exports/                       # Recent exports
│   ├── work-setup.nix
│   ├── gaming-profile.conf
│   └── ...
└── logs/                          # Application logs
    ├── r-hyprconfig.log
    └── error.log
```

## Data Migration Strategy

### Version Migration Schema
```json
{
  "migrations": [
    {
      "from_version": "0.9.0",
      "to_version": "1.0.0",
      "migration_script": "migrate_v0_9_to_v1_0",
      "backup_required": true,
      "description": "Add distribution info to profiles"
    }
  ]
}
```

## Backup and Recovery

### Automatic Backup Schema
```json
{
  "backup_metadata": {
    "version": "1.0.0",
    "created": "2025-07-25T10:00:00Z",
    "source": "profiles.json",
    "trigger": "profile_deletion",
    "retention_policy": "30_days"
  },
  "backup_data": {
    // Original data before modification
  }
}
```

## Schema Validation Rules

### Profile Validation
- Profile ID must be valid UUID v4
- Profile name must be non-empty string (max 100 chars)
- Creation/modification timestamps must be valid ISO 8601
- Configuration data must conform to Hyprland config schema
- Tags must be array of strings (max 20 tags, 50 chars each)

### Distribution Info Validation
- Distribution ID must match known distribution list or "unknown"
- Paths must be absolute and accessible
- Timestamps must be valid ISO 8601
- Capabilities must be boolean values

### Export Template Validation
- Template must contain valid placeholder syntax
- File extension must start with dot
- Validation rules must be boolean
- Template content must be valid for target format

## Error Handling

### Data Corruption Recovery
- Automatic backup restoration on corrupted data detection
- Schema validation before data loading
- Graceful degradation with default values
- User notification for data recovery actions

### Migration Failures
- Rollback mechanism for failed migrations
- Backup creation before migration attempts
- Error logging with detailed failure information
- Manual recovery options for edge cases