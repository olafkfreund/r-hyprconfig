# r-hyprconfig Screenshots

> 📸 Visual documentation for the v1.0.0 release

## Screenshot Placeholders

This directory contains placeholder descriptions for screenshots. Real screenshots will be added when a GUI environment is available.

### Main Interface
![Main Interface](placeholder-main-interface.png)
*The main r-hyprconfig interface showing the General configuration panel with tab navigation*

### Configuration Editing
![Configuration Editing](placeholder-editing-config.png)  
*Editing a configuration option with real-time validation*

### Search Functionality
![Search Feature](placeholder-search-feature.png)
*Using the search function to filter configuration options*

### Help System
![Help Screen](placeholder-help-screen.png)
*Comprehensive help screen with keyboard shortcuts*

### Validation Demo
![Validation Demo](placeholder-validation-demo.png)
*User-friendly validation error handling*

### NixOS Export
![NixOS Export](placeholder-nixos-export.png)
*NixOS configuration export dialog with preview*

## ASCII Art Preview

### Main Interface Mockup
```
┌─ r-hyprconfig v1.0.0 ─────────────────────────────────────────────────────┐
│ [General] [Input] [Decoration] [Animations] [Gestures] [Binds] [Rules]    │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│ General Configuration                                                      │
│                                                                            │
│ ► gaps_in                    2 2 2 2              [Inner window gaps]     │
│   gaps_out                   5 5 5 5              [Outer window gaps]     │
│   border_size                2                    [Window border width]   │
│   col.active_border          0xffbebebe          [Active border color]   │
│   col.inactive_border        0xff595959          [Inactive border color] │
│   resize_on_border           true                 [Resize by dragging]    │
│   hover_icon_on_border       true                 [Show resize cursor]    │
│                                                                            │
│ Use ↑↓ to navigate, Enter to edit, Tab for next section                   │
│ Press 's' to save, 'r' to reload, '?' for help, 'q' to quit              │
└────────────────────────────────────────────────────────────────────────────┘
```

### Edit Mode Mockup
```
┌─ r-hyprconfig v1.0.0 - Editing border_size ───────────────────────────────┐
│ [General] [Input] [Decoration] [Animations] [Gestures] [Binds] [Rules]    │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│ General Configuration                                                      │
│                                                                            │
│   gaps_in                    2 2 2 2              [Inner window gaps]     │
│   gaps_out                   5 5 5 5              [Outer window gaps]     │
│ ┌─border_size────────────────────────────────────────────────────────────┐ │
│ │ Current Value: 2                                                       │ │
│ │ New Value: [3_]                                                        │ │
│ │                                                                        │ │
│ │ Border width in pixels (integer, non-negative)                        │ │
│ │                                                                        │ │
│ │ Press Enter to confirm, Esc to cancel                                 │ │
│ └────────────────────────────────────────────────────────────────────────┘ │
│   col.active_border          0xffbebebe          [Active border color]   │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

## TODO: Real Screenshots

When a GUI environment becomes available:
1. Launch r-hyprconfig in a properly configured terminal
2. Capture screenshots for each scenario described above
3. Replace these placeholders with actual images
4. Update README.md with real screenshot links

## Specifications
- **Format**: PNG with transparency
- **Resolution**: 1920x1080 minimum  
- **Terminal Size**: 120x30 characters
- **Font**: Professional monospace (FiraCode/JetBrains Mono)
- **Theme**: Clean dark theme with good contrast