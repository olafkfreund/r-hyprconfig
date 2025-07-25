# Screenshot Generation Guide for r-hyprconfig

> 📸 Instructions for creating documentation screenshots

## Overview

Since r-hyprconfig is a TUI application, we need terminal screenshots showing the interface in action. Here's how to generate professional-looking screenshots for the release.

## Required Screenshots

### 1. **Main Interface** (`main-interface.png`)
- Show the General configuration panel with settings visible
- Highlight the tab navigation at the top
- Display some example configuration values

**Command to capture:**
```bash
r-hyprconfig
# Navigate to General tab, take screenshot
```

### 2. **Configuration Editing** (`editing-config.png`)
- Show an option being edited (like border_size)
- Display the edit mode interface
- Highlight the current value and validation

**Steps:**
1. Start r-hyprconfig
2. Navigate to an integer option (like border_size)
3. Press Enter to edit
4. Take screenshot during edit mode

### 3. **Search Functionality** (`search-feature.png`)
- Show the search interface active
- Display filtered results
- Highlight matching terms

**Steps:**
1. Press `/` to activate search
2. Type "border" or "animation"
3. Take screenshot with search results

### 4. **Help System** (`help-screen.png`)
- Show the comprehensive help screen
- Display keyboard shortcuts
- Show navigation instructions

**Steps:**
1. Press `?` or F1 to open help
2. Take screenshot of help screen

### 5. **Validation Error** (`validation-demo.png`)
- Show validation in action
- Display an error message for invalid input
- Demonstrate user-friendly error handling

**Steps:**
1. Edit a numeric field
2. Enter invalid value (like "abc" for border_size)
3. Press Enter to trigger validation
4. Take screenshot of error popup

### 6. **NixOS Export** (`nixos-export.png`)
- Show NixOS export dialog
- Display configuration preview
- Highlight different export options

**Steps:**
1. Press `n` to open NixOS export
2. Select an export type
3. Take screenshot with preview visible

## Screenshot Requirements

### Technical Specifications
- **Format**: PNG with transparency support
- **Resolution**: At least 1920x1080 for clarity
- **Terminal**: Use a modern terminal with good font rendering
- **Color Scheme**: Default or a professional dark theme
- **Font**: Use a clear monospace font (FiraCode, JetBrains Mono, etc.)

### Composition Guidelines
- **Terminal Size**: Use consistent 120x30 or 100x25 terminal size
- **Window Decorations**: Keep minimal, focus on content
- **Background**: Clean, professional background
- **Contrast**: Ensure text is clearly readable

## Recommended Tools

### For Linux (with GUI)
```bash
# Using gnome-screenshot
gnome-screenshot -w  # Window screenshot

# Using scrot
scrot -s  # Select area

# Using flameshot
flameshot gui  # Interactive screenshot tool
```

### For Terminal-specific Screenshots
```bash
# Using kitty (built-in screenshot)
kitty +kitten icat screenshot.png

# Using wezterm
wezterm cli spawn --class screenshot-demo

# Using termshot (if available)
termshot r-hyprconfig
```

## Alternative: ASCII Art Mockups

If screenshots are difficult to obtain, we can create ASCII art representations:

### Example: Main Interface Mockup
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

## Implementation Plan

### Option A: Real Screenshots (Preferred)
1. Set up a virtual display or use existing GUI environment
2. Configure terminal with professional appearance
3. Launch r-hyprconfig and navigate through each scenario
4. Capture high-quality screenshots
5. Optimize images for web display

### Option B: Mockup Creation
1. Create detailed ASCII art representations
2. Use terminal rendering tools to generate images
3. Create professional-looking terminal mockups
4. Generate SVG versions for scalability

### Option C: Video Demo
1. Record a short screencast of r-hyprconfig in action
2. Extract key frames as screenshots
3. Create an animated GIF showing basic navigation
4. Upload to GitHub releases

## File Naming Convention

Screenshots should be named descriptively:
- `r-hyprconfig-main-interface.png`
- `r-hyprconfig-editing-config.png`
- `r-hyprconfig-search-feature.png`
- `r-hyprconfig-help-screen.png`
- `r-hyprconfig-validation-demo.png`
- `r-hyprconfig-nixos-export.png`

## Immediate Action Items

Since we're in a headless environment, I recommend:

1. **Document the screenshot requirements** (✓ Done above)
2. **Create placeholder images** with ASCII art or descriptions
3. **Update README with screenshot placeholders**
4. **Complete the release with notes about screenshots**
5. **Add screenshots post-release** when GUI environment available

This approach ensures we can complete the v1.0.0 release without being blocked by screenshot generation.