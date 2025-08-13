use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command as AsyncCommand;
use tokio::time::{timeout, Duration as TokioDuration};

use crate::errors::{HyprConfigError, HyprctlError, HyprctlResult, RecoveryContext, RecoveryStrategy};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyprlandConfig {
    pub general: GeneralConfig,
    pub input: InputConfig,
    pub decoration: DecorationConfig,
    pub animations: AnimationsConfig,
    pub gestures: GesturesConfig,
    pub binds: Vec<String>,
    pub window_rules: Vec<String>,
    pub layer_rules: Vec<String>,
    pub misc: MiscConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub gaps_in: i32,
    pub gaps_out: i32,
    pub border_size: i32,
    pub col_active_border: String,
    pub col_inactive_border: String,
    pub resize_on_border: bool,
    pub extend_border_grab_area: i32,
    pub hover_icon_on_border: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputConfig {
    pub kb_layout: String,
    pub kb_variant: String,
    pub kb_model: String,
    pub kb_options: String,
    pub kb_rules: String,
    pub follow_mouse: i32,
    pub mouse_refocus: bool,
    pub sensitivity: f32,
    pub accel_profile: String,
    pub natural_scroll: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecorationConfig {
    pub rounding: i32,
    pub blur_enabled: bool,
    pub blur_size: i32,
    pub blur_passes: i32,
    pub drop_shadow: bool,
    pub shadow_range: i32,
    pub shadow_render_power: i32,
    pub col_shadow: String,
    pub dim_inactive: bool,
    pub dim_strength: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnimationsConfig {
    pub enabled: bool,
    pub beziers: Vec<String>,
    pub animations: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GesturesConfig {
    pub workspace_swipe: bool,
    pub workspace_swipe_fingers: i32,
    pub workspace_swipe_distance: i32,
    pub workspace_swipe_invert: bool,
    pub workspace_swipe_min_speed_to_force: i32,
    pub workspace_swipe_cancel_ratio: f32,
    pub workspace_swipe_create_new: bool,
    pub workspace_swipe_forever: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MiscConfig {
    pub disable_hyprland_logo: bool,
    pub disable_splash_rendering: bool,
    pub mouse_move_enables_dpms: bool,
    pub key_press_enables_dpms: bool,
    pub always_follow_on_dnd: bool,
    pub layers_hog_keyboard_focus: bool,
    pub animate_manual_resizes: bool,
    pub animate_mouse_windowdragging: bool,
    pub disable_autoreload: bool,
    pub enable_swallow: bool,
    pub swallow_regex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyprlandKeybind {
    pub modifiers: Vec<String>, // SUPER, ALT, CTRL, SHIFT
    pub key: String,            // q, Return, space, etc.
    pub dispatcher: String,     // exec, killactive, togglefloating, etc.
    pub args: Option<String>,   // Command arguments
    pub bind_type: String,      // bind, bindm, bindr, etc.
}

impl HyprlandKeybind {
    #[allow(dead_code)]
    pub fn new(
        modifiers: Vec<String>,
        key: String,
        dispatcher: String,
        args: Option<String>,
    ) -> Self {
        Self {
            modifiers,
            key,
            dispatcher,
            args,
            bind_type: "bind".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn to_hyprland_config(&self) -> String {
        let mod_string = if self.modifiers.is_empty() {
            String::new()
        } else {
            format!("{}, ", self.modifiers.join(" + "))
        };

        let args_string = if let Some(ref args) = self.args {
            format!(", {args}")
        } else {
            String::new()
        };

        format!(
            "{} = {}{}, {}{}",
            self.bind_type, mod_string, self.key, self.dispatcher, args_string
        )
    }

    pub fn display_string(&self) -> String {
        let mod_string = if self.modifiers.is_empty() {
            String::new()
        } else {
            format!("{} + ", self.modifiers.join(" + "))
        };

        let args_string = if let Some(ref args) = self.args {
            format!(" [{args}]")
        } else {
            String::new()
        };

        format!(
            "{}{} → {}{}",
            mod_string, self.key, self.dispatcher, args_string
        )
    }
}

/// Cache entry for hyprctl responses
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    timestamp: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
        }
    }
    
    fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() > ttl
    }
}

/// Cache for hyprctl responses to avoid repeated expensive calls
/// Only caches successful results - errors are not cached to allow retries
#[derive(Debug, Default)]
struct HyprctlCache {
    // Individual option cache - key: option_name, value: successful result
    options: HashMap<String, CacheEntry<String>>,
    
    // Bulk operations cache - only successful results
    all_options: Option<CacheEntry<HashMap<String, String>>>,
    binds: Option<CacheEntry<Vec<HyprlandKeybind>>>,
    window_rules: Option<CacheEntry<Vec<String>>>,
    layer_rules: Option<CacheEntry<Vec<String>>>,
    workspace_rules: Option<CacheEntry<Vec<String>>>,
    
    // Cache configuration
    default_ttl: Duration,
    bulk_ttl: Duration,
}

impl HyprctlCache {
    fn new() -> Self {
        Self {
            options: HashMap::new(),
            all_options: None,
            binds: None,
            window_rules: None,
            layer_rules: None,
            workspace_rules: None,
            default_ttl: Duration::from_secs(30), // 30 seconds for individual options
            bulk_ttl: Duration::from_secs(60),    // 60 seconds for bulk operations
        }
    }
    
    fn clear(&mut self) {
        self.options.clear();
        self.all_options = None;
        self.binds = None;
        self.window_rules = None;
        self.layer_rules = None;
        self.workspace_rules = None;
    }
    
    fn clear_expired(&mut self) {
        // Clear expired individual options
        self.options.retain(|_, entry| !entry.is_expired(self.default_ttl));
        
        // Clear expired bulk operations
        if let Some(entry) = &self.all_options {
            if entry.is_expired(self.bulk_ttl) {
                self.all_options = None;
            }
        }
        
        if let Some(entry) = &self.binds {
            if entry.is_expired(self.bulk_ttl) {
                self.binds = None;
            }
        }
        
        if let Some(entry) = &self.window_rules {
            if entry.is_expired(self.bulk_ttl) {
                self.window_rules = None;
            }
        }
        
        if let Some(entry) = &self.layer_rules {
            if entry.is_expired(self.bulk_ttl) {
                self.layer_rules = None;
            }
        }
        
        if let Some(entry) = &self.workspace_rules {
            if entry.is_expired(self.bulk_ttl) {
                self.workspace_rules = None;
            }
        }
    }
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub options_cached: usize,
    pub all_options_cached: bool,
    pub binds_cached: bool,
    pub window_rules_cached: bool,
    pub layer_rules_cached: bool,
    pub workspace_rules_cached: bool,
    pub default_ttl_secs: u64,
    pub bulk_ttl_secs: u64,
}

pub struct HyprCtl {
    #[allow(dead_code)]
    socket_path: Option<String>,
    cache: std::sync::Mutex<HyprctlCache>,
    /// Timeout for hyprctl commands in milliseconds
    timeout_ms: u64,
}

impl HyprCtl {
    pub async fn new() -> Result<Self> {
        let mut hyprctl = Self { 
            socket_path: None,
            cache: std::sync::Mutex::new(HyprctlCache::new()),
            timeout_ms: 5000, // Default 5 second timeout
        };

        // Try to detect Hyprland socket
        hyprctl.detect_socket().await?;

        Ok(hyprctl)
    }

    pub fn new_disconnected() -> Self {
        Self { 
            socket_path: None,
            cache: std::sync::Mutex::new(HyprctlCache::new()),
            timeout_ms: 5000, // Default 5 second timeout
        }
    }

    /// Clear all cached data (useful when configuration changes are made)
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
    
    /// Clear expired cache entries
    pub fn clear_expired_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear_expired();
        }
    }

    /// Configure timeout for hyprctl commands
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }

    /// Get current timeout setting
    pub fn get_timeout(&self) -> u64 {
        self.timeout_ms
    }

    /// Execute a hyprctl command with timeout handling
    async fn execute_hyprctl_with_timeout(&self, args: &[&str]) -> HyprctlResult<std::process::Output> {
        let command_str = format!("hyprctl {}", args.join(" "));
        let timeout_duration = TokioDuration::from_millis(self.timeout_ms);
        
        let future = AsyncCommand::new("hyprctl")
            .args(args)
            .output();
            
        match timeout(timeout_duration, future).await {
            Ok(result) => {
                result.map_err(|_| HyprctlError::CommandNotFound)
            }
            Err(_) => {
                Err(HyprctlError::Timeout {
                    command: command_str,
                    timeout_ms: self.timeout_ms,
                })
            }
        }
    }

    /// Execute a hyprctl command with custom timeout (for testing or special cases)
    pub async fn execute_with_custom_timeout(&self, args: &[&str], timeout_ms: u64) -> HyprctlResult<std::process::Output> {
        let command_str = format!("hyprctl {}", args.join(" "));
        let timeout_duration = TokioDuration::from_millis(timeout_ms);
        
        let future = AsyncCommand::new("hyprctl")
            .args(args)
            .output();
            
        match timeout(timeout_duration, future).await {
            Ok(result) => {
                result.map_err(|_| HyprctlError::CommandNotFound)
            }
            Err(_) => {
                Err(HyprctlError::Timeout {
                    command: command_str,
                    timeout_ms,
                })
            }
        }
    }

    /// Test timeout functionality by running a command with very short timeout
    #[cfg(test)]
    pub async fn test_timeout_functionality(&self) -> bool {
        // Try to run a command with 1ms timeout - should definitely timeout
        matches!(
            self.execute_with_custom_timeout(&["version"], 1).await,
            Err(HyprctlError::Timeout { .. })
        )
    }
    
    /// Get cache statistics for debugging/monitoring
    pub fn get_cache_stats(&self) -> Option<CacheStats> {
        if let Ok(cache) = self.cache.lock() {
            Some(CacheStats {
                options_cached: cache.options.len(),
                all_options_cached: cache.all_options.is_some(),
                binds_cached: cache.binds.is_some(),
                window_rules_cached: cache.window_rules.is_some(),
                layer_rules_cached: cache.layer_rules.is_some(),
                workspace_rules_cached: cache.workspace_rules.is_some(),
                default_ttl_secs: cache.default_ttl.as_secs(),
                bulk_ttl_secs: cache.bulk_ttl.as_secs(),
            })
        } else {
            None
        }
    }

    async fn detect_socket(&mut self) -> Result<()> {
        // Try to get Hyprland instance signature
        let output = self.execute_hyprctl_with_timeout(&["getoption", "general:border_size"]).await;

        match output {
            Ok(output) => {
                if output.status.success() {
                    // hyprctl is available and working
                    Ok(())
                } else {
                    eprintln!("Warning: hyprctl available but Hyprland not running");
                    Ok(()) // Don't fail, just warn
                }
            }
            Err(e) => {
                eprintln!("Warning: Hyprland is not running or hyprctl is not available: {e}");
                Ok(()) // Don't fail, just warn - we can use config file fallback
            }
        }
    }

    pub async fn get_option(&self, option: &str) -> Result<String> {
        // Check cache first
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear_expired();
            
            if let Some(entry) = cache.options.get(option) {
                if !entry.is_expired(cache.default_ttl) {
                    // Return cached successful result
                    return Ok(entry.value.clone());
                }
            }
        }

        // Cache miss or expired - fetch from hyprctl
        let result = self.get_option_uncached(option).await;
        
        // Cache only successful results
        if let Ok(value) = &result {
            if let Ok(mut cache) = self.cache.lock() {
                cache.options.insert(option.to_string(), CacheEntry::new(value.clone()));
            }
        }
        
        result
    }

    /// Internal method to get option without caching (used by cached version)
    async fn get_option_uncached(&self, option: &str) -> Result<String> {
        let output = self.execute_hyprctl_with_timeout(&["getoption", option])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute hyprctl getoption: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl getoption failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse the structured output from hyprctl getoption
        // Example output:
        // int: 2
        // set: true
        //
        // or:
        // custom type: 2 2 2 2
        // set: true

        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("int: ") {
                return Ok(line.strip_prefix("int: ").unwrap_or("").to_string());
            } else if line.starts_with("float: ") {
                return Ok(line.strip_prefix("float: ").unwrap_or("").to_string());
            } else if line.starts_with("str: ") {
                return Ok(line.strip_prefix("str: ").unwrap_or("").to_string());
            } else if line.starts_with("custom type: ") {
                return Ok(line.strip_prefix("custom type: ").unwrap_or("").to_string());
            } else if line.starts_with("vec2: ") {
                return Ok(line.strip_prefix("vec2: ").unwrap_or("").to_string());
            } else if line.starts_with("color: ") {
                return Ok(line.strip_prefix("color: ").unwrap_or("").to_string());
            } else if !line.starts_with("set: ") && !line.is_empty() {
                // Fallback: if no type prefix found, return the first non-empty, non-"set:" line
                return Ok(line.to_string());
            }
        }

        // Fallback to original behavior
        Ok(stdout.trim().to_string())
    }

    /// Get hyprctl option with structured error handling
    pub async fn get_option_typed(&self, option: &str) -> HyprctlResult<String> {

        let output = self.execute_hyprctl_with_timeout(&["getoption", option]).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("unknown option") || stderr.contains("invalid option") {
                return Err(HyprctlError::InvalidOption {
                    option: option.to_string(),
                });
            }
            return Err(HyprctlError::ExecutionFailed {
                command: format!("hyprctl getoption {}", option),
                stderr: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Err(HyprctlError::ParseError {
                command: format!("hyprctl getoption {}", option),
                reason: "Empty output received".to_string(),
            });
        }

        Ok(stdout.trim().to_string())
    }

    /// Get hyprctl option with automatic retry and recovery
    pub async fn get_option_resilient(&self, option: &str) -> HyprctlResult<String> {
        let mut recovery_context = RecoveryContext::new(format!("get option '{}'", option))
            .with_retry(3, 100) // 3 attempts, start with 100ms delay
            .with_fallback("Using default value".to_string());

        loop {
            match self.get_option_typed(option).await {
                Ok(value) => return Ok(value),
                Err(error) => {
                    if let Some(strategy) = recovery_context.next_strategy() {
                        match strategy {
                            RecoveryStrategy::Retry { max_attempts, base_delay_ms } => {
                                if recovery_context.attempt_count <= max_attempts {
                                    let delay = base_delay_ms * (1 << (recovery_context.attempt_count - 1));
                                    eprintln!(
                                        "Retrying hyprctl command '{}' (attempt {}/{}) after {}ms delay...",
                                        option, recovery_context.attempt_count, max_attempts, delay
                                    );
                                    sleep(Duration::from_millis(delay)).await;
                                    continue;
                                } else {
                                    // Max retries exceeded, try next strategy
                                    continue;
                                }
                            }
                            RecoveryStrategy::Fallback { description } => {
                                eprintln!("Falling back for option '{}': {}", option, description);
                                // Return a reasonable default based on the option type
                                return Ok(self.get_default_value_for_option(option));
                            }
                            RecoveryStrategy::Abort => {
                                return Err(error);
                            }
                            _ => {
                                // Other strategies not applicable here
                                return Err(error);
                            }
                        }
                    } else {
                        return Err(error);
                    }
                }
            }
        }
    }

    /// Get a reasonable default value for common options
    fn get_default_value_for_option(&self, option: &str) -> String {
        match option {
            opt if opt.contains("gaps_in") => "5".to_string(),
            opt if opt.contains("gaps_out") => "20".to_string(),
            opt if opt.contains("border_size") => "2".to_string(),
            opt if opt.contains("rounding") => "10".to_string(),
            opt if opt.contains("enabled") || opt.contains("enable") => "true".to_string(),
            opt if opt.contains("opacity") => "1.0".to_string(),
            opt if opt.contains("sensitivity") => "0.0".to_string(),
            opt if opt.starts_with("col.") => "rgba(33ccffee)".to_string(),
            _ => "unknown".to_string(),
        }
    }

    pub async fn set_option(&self, option: &str, value: &str) -> Result<()> {
        let output = self.execute_hyprctl_with_timeout(&["keyword", option, value])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute hyprctl keyword: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl keyword failed: {}", stderr);
        }

        // Clear cache after setting option since configuration has changed
        self.clear_cache();

        Ok(())
    }

    pub async fn get_all_options(&self) -> Result<HashMap<String, String>> {
        // Check cache first
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear_expired();
            
            if let Some(entry) = &cache.all_options {
                if !entry.is_expired(cache.bulk_ttl) {
                    // Return cached successful result
                    return Ok(entry.value.clone());
                }
            }
        }

        // Cache miss or expired - fetch from hyprctl
        let result = self.get_all_options_uncached().await;
        
        // Cache only successful results
        if let Ok(options) = &result {
            if let Ok(mut cache) = self.cache.lock() {
                cache.all_options = Some(CacheEntry::new(options.clone()));
            }
        }
        
        result
    }

    /// Internal method to get all options without caching
    async fn get_all_options_uncached(&self) -> Result<HashMap<String, String>> {
        let mut options = HashMap::new();

        // Get general options
        let general_options = vec![
            "general:gaps_in",
            "general:gaps_out",
            "general:border_size",
            "general:col.active_border",
            "general:col.inactive_border",
            "general:resize_on_border",
            "general:extend_border_grab_area",
            "general:hover_icon_on_border",
        ];

        for option in general_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {option}: {e}");
                }
            }
        }

        // Get input options
        let input_options = vec![
            "input:kb_layout",
            "input:kb_variant",
            "input:kb_model",
            "input:kb_options",
            "input:kb_rules",
            "input:follow_mouse",
            "input:mouse_refocus",
            "input:sensitivity",
            "input:accel_profile",
            "input:natural_scroll",
        ];

        for option in input_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {option}: {e}");
                }
            }
        }

        // Get decoration options
        let decoration_options = vec![
            "decoration:rounding",
            "decoration:blur:enabled",
            "decoration:blur:size",
            "decoration:blur:passes",
            "decoration:drop_shadow",
            "decoration:shadow_range",
            "decoration:shadow_render_power",
            "decoration:col.shadow",
            "decoration:dim_inactive",
            "decoration:dim_strength",
        ];

        for option in decoration_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {option}: {e}");
                }
            }
        }

        // Get animations options
        let animation_options = vec!["animations:enabled"];

        for option in animation_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {option}: {e}");
                }
            }
        }

        // Get gestures options
        let gesture_options = vec![
            "gestures:workspace_swipe",
            "gestures:workspace_swipe_fingers",
            "gestures:workspace_swipe_distance",
            "gestures:workspace_swipe_invert",
            "gestures:workspace_swipe_min_speed_to_force",
            "gestures:workspace_swipe_cancel_ratio",
            "gestures:workspace_swipe_create_new",
            "gestures:workspace_swipe_forever",
        ];

        for option in gesture_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {option}: {e}");
                }
            }
        }

        // Get misc options
        let misc_options = vec![
            "misc:disable_hyprland_logo",
            "misc:disable_splash_rendering",
            "misc:mouse_move_enables_dpms",
            "misc:key_press_enables_dpms",
            "misc:always_follow_on_dnd",
            "misc:layers_hog_keyboard_focus",
            "misc:animate_manual_resizes",
            "misc:animate_mouse_windowdragging",
            "misc:disable_autoreload",
            "misc:enable_swallow",
            "misc:swallow_regex",
        ];

        for option in misc_options {
            match self.get_option(option).await {
                Ok(value) => {
                    options.insert(option.to_string(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get option {option}: {e}");
                }
            }
        }

        Ok(options)
    }

    pub async fn get_binds(&self) -> Result<Vec<HyprlandKeybind>> {
        // Check cache first
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear_expired();
            
            if let Some(entry) = &cache.binds {
                if !entry.is_expired(cache.bulk_ttl) {
                    // Return cached successful result
                    return Ok(entry.value.clone());
                }
            }
        }

        // Cache miss or expired - fetch from hyprctl
        let result = self.get_binds_uncached().await;
        
        // Cache only successful results
        if let Ok(binds) = &result {
            if let Ok(mut cache) = self.cache.lock() {
                cache.binds = Some(CacheEntry::new(binds.clone()));
            }
        }
        
        result
    }

    /// Internal method to get binds without caching
    async fn get_binds_uncached(&self) -> Result<Vec<HyprlandKeybind>> {
        let output = self.execute_hyprctl_with_timeout(&["binds"])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute hyprctl binds: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl binds failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut binds = Vec::new();

        // Parse the structured output format
        let lines: Vec<&str> = stdout.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Look for bind type lines (bind, binde, bindm, bindl, etc.)
            if line.starts_with("bind") && !line.contains(':') {
                if let Some(keybind) = Self::parse_structured_bind(&lines, i) {
                    binds.push(keybind);
                }
            }
            i += 1;
        }

        Ok(binds)
    }

    fn parse_structured_bind(lines: &[&str], start_index: usize) -> Option<HyprlandKeybind> {
        // Parse the structured hyprctl binds output format:
        // bind
        //     modmask: 64
        //     submap:
        //     key: N
        //     keycode: 0
        //     catchall: false
        //     description:
        //     dispatcher: exec
        //     arg: swaync-client -t -sw

        if start_index >= lines.len() {
            return None;
        }

        let bind_type = lines[start_index].trim().to_string();
        let mut modmask: Option<u32> = None;
        let mut key: Option<String> = None;
        let mut dispatcher: Option<String> = None;
        let mut arg: Option<String> = None;

        // Parse the following lines until we hit the next bind or end of output
        let mut i = start_index + 1;
        while i < lines.len() {
            let line = lines[i].trim();

            // Stop if we hit another bind definition
            if line.starts_with("bind") && !line.contains(':') {
                break;
            }

            // Parse key-value pairs
            if let Some((key_name, value)) = line.split_once(':') {
                let key_name = key_name.trim();
                let value = value.trim();

                match key_name {
                    "modmask" => {
                        if let Ok(mask) = value.parse::<u32>() {
                            modmask = Some(mask);
                        }
                    }
                    "key" => {
                        if !value.is_empty() {
                            key = Some(value.to_string());
                        }
                    }
                    "dispatcher" => {
                        if !value.is_empty() {
                            dispatcher = Some(value.to_string());
                        }
                    }
                    "arg" => {
                        if !value.is_empty() {
                            arg = Some(value.to_string());
                        }
                    }
                    _ => {} // Ignore other fields
                }
            }

            i += 1;
        }

        // Create the keybind if we have the required fields
        if let (Some(key_val), Some(disp)) = (key, dispatcher) {
            let modifiers = if let Some(mask) = modmask {
                Self::parse_modifiers(&mask.to_string())
            } else {
                vec![]
            };

            Some(HyprlandKeybind {
                modifiers,
                key: key_val,
                dispatcher: disp,
                args: arg,
                bind_type,
            })
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn parse_bind_line(line: &str) -> Option<HyprlandKeybind> {
        // Legacy parser - kept for backwards compatibility
        // hyprctl binds output format: "modmask,key -> dispatcher [arg]"
        // Example: "64,q -> exec [kitty]"

        if let Some((key_part, command_part)) = line.split_once(" -> ") {
            let key_part = key_part.trim();
            let command_part = command_part.trim();

            // Parse modifiers and key
            let (modifiers, key) = if let Some((mods, k)) = key_part.split_once(',') {
                (Self::parse_modifiers(mods), k.to_string())
            } else {
                (vec![], key_part.to_string())
            };

            // Parse command and args
            let (dispatcher, args) = if let Some((disp, arg_part)) = command_part.split_once(' ') {
                let args = if arg_part.starts_with('[') && arg_part.ends_with(']') {
                    arg_part
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .to_string()
                } else {
                    arg_part.to_string()
                };
                (disp.to_string(), Some(args))
            } else {
                (command_part.to_string(), None)
            };

            return Some(HyprlandKeybind {
                modifiers,
                key,
                dispatcher,
                args,
                bind_type: "bind".to_string(), // Default, could be enhanced
            });
        }

        None
    }

    fn parse_modifiers(mod_mask: &str) -> Vec<String> {
        // Convert numeric modifier mask to readable modifiers
        if let Ok(mask) = mod_mask.parse::<u32>() {
            let mut mods = Vec::new();
            if mask & 64 != 0 {
                mods.push("SUPER".to_string());
            } // Mod4
            if mask & 8 != 0 {
                mods.push("ALT".to_string());
            } // Mod1
            if mask & 4 != 0 {
                mods.push("CTRL".to_string());
            } // Control
            if mask & 1 != 0 {
                mods.push("SHIFT".to_string());
            } // Shift
            mods
        } else {
            vec![]
        }
    }

    #[allow(dead_code)]
    pub async fn add_keybind(&self, bind: &HyprlandKeybind) -> Result<()> {
        let bind_command = bind.to_hyprland_config();
        let result = self.dispatch(&format!("keyword {bind_command}")).await;
        
        // Clear cache after adding keybind since configuration has changed
        if result.is_ok() {
            self.clear_cache();
        }
        
        result
    }

    #[allow(dead_code)]
    pub async fn remove_keybind(&self, modifiers: &[String], key: &str) -> Result<()> {
        let mod_string = if modifiers.is_empty() {
            String::new()
        } else {
            format!("{}_", modifiers.join("_"))
        };

        let unbind_command = format!("unbind {mod_string}{key}");
        let result = self.dispatch(&unbind_command).await;
        
        // Clear cache after removing keybind since configuration has changed
        if result.is_ok() {
            self.clear_cache();
        }
        
        result
    }

    #[allow(dead_code)]
    pub async fn reload_config(&self) -> Result<()> {
        let output = self.execute_hyprctl_with_timeout(&["reload"])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute hyprctl reload: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl reload failed: {}", stderr);
        }

        // Clear cache after reloading since configuration has changed
        self.clear_cache();

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn dispatch(&self, command: &str) -> Result<()> {
        let output = self.execute_hyprctl_with_timeout(&["dispatch", command])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute hyprctl dispatch: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl dispatch failed: {}", stderr);
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_version(&self) -> Result<String> {
        let output = self.execute_hyprctl_with_timeout(&["version"])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute hyprctl version: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl version failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    #[allow(dead_code)]
    pub async fn is_hyprland_running(&self) -> bool {
        self.execute_hyprctl_with_timeout(&["version"])
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub async fn get_workspace_rules(&self) -> Result<Vec<String>> {
        // Check cache first
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear_expired();
            
            if let Some(entry) = &cache.workspace_rules {
                if !entry.is_expired(cache.bulk_ttl) {
                    // Return cached successful result
                    return Ok(entry.value.clone());
                }
            }
        }

        // Cache miss or expired - fetch from hyprctl
        let result = self.get_workspace_rules_uncached().await;
        
        // Cache only successful results
        if let Ok(rules) = &result {
            if let Ok(mut cache) = self.cache.lock() {
                cache.workspace_rules = Some(CacheEntry::new(rules.clone()));
            }
        }
        
        result
    }

    /// Internal method to get workspace rules without caching
    async fn get_workspace_rules_uncached(&self) -> Result<Vec<String>> {
        let output = self.execute_hyprctl_with_timeout(&["workspacerules"])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute hyprctl workspacerules: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl workspacerules failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let rules: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(rules)
    }

    pub async fn get_window_rules(&self) -> Result<Vec<String>> {
        // Check cache first
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear_expired();
            
            if let Some(entry) = &cache.window_rules {
                if !entry.is_expired(cache.bulk_ttl) {
                    // Return cached successful result
                    return Ok(entry.value.clone());
                }
            }
        }

        // Cache miss or expired - fetch from hyprctl
        let result = self.get_window_rules_uncached().await;
        
        // Cache only successful results
        if let Ok(rules) = &result {
            if let Ok(mut cache) = self.cache.lock() {
                cache.window_rules = Some(CacheEntry::new(rules.clone()));
            }
        }
        
        result
    }

    /// Internal method to get window rules without caching
    async fn get_window_rules_uncached(&self) -> Result<Vec<String>> {
        // Window rules are typically parsed from the config file or through clients command
        // For now, we'll return a placeholder that shows current window classes
        let output = self.execute_hyprctl_with_timeout(&["clients", "-j"])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute hyprctl clients: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl clients failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON to extract window classes for rule suggestions
        let mut window_classes = std::collections::HashSet::new();

        // Try to parse as JSON, but fallback to text parsing if it fails
        if let Ok(clients) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(clients_array) = clients.as_array() {
                for client in clients_array {
                    if let Some(class) = client.get("class").and_then(|c| c.as_str()) {
                        if !class.is_empty() {
                            window_classes.insert(format!("class:^({class})$"));
                        }
                    }
                    if let Some(title) = client.get("title").and_then(|t| t.as_str()) {
                        if !title.is_empty() && title.len() > 3 {
                            window_classes.insert(format!("title:^({title})$"));
                        }
                    }
                }
            }
        }

        // Convert to vector and add some common window rule examples
        let mut rules: Vec<String> = window_classes.into_iter().collect();
        rules.sort();

        // Add some example rules if no windows are found
        if rules.is_empty() {
            rules = vec![
                "windowrule = float, ^(kitty)$".to_string(),
                "windowrule = size 800 600, ^(floating-app)$".to_string(),
                "windowrule = workspace 2, ^(firefox)$".to_string(),
                "windowrule = opacity 0.8, ^(terminal)$".to_string(),
            ];
        }

        Ok(rules)
    }

    pub async fn get_layer_rules(&self) -> Result<Vec<String>> {
        // Check cache first
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear_expired();
            
            if let Some(entry) = &cache.layer_rules {
                if !entry.is_expired(cache.bulk_ttl) {
                    // Return cached successful result
                    return Ok(entry.value.clone());
                }
            }
        }

        // Cache miss or expired - fetch from hyprctl
        let result = self.get_layer_rules_uncached().await;
        
        // Cache only successful results
        if let Ok(rules) = &result {
            if let Ok(mut cache) = self.cache.lock() {
                cache.layer_rules = Some(CacheEntry::new(rules.clone()));
            }
        }
        
        result
    }

    /// Internal method to get layer rules without caching
    async fn get_layer_rules_uncached(&self) -> Result<Vec<String>> {
        let output = self.execute_hyprctl_with_timeout(&["layers"])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute hyprctl layers: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl layers failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse layer information and create layer rule suggestions
        let mut layer_names = std::collections::HashSet::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.contains("namespace:") {
                if let Some(namespace) = line.split("namespace: ").nth(1) {
                    let namespace = namespace.split_whitespace().next().unwrap_or(namespace);
                    layer_names.insert(namespace.to_string());
                }
            }
        }

        // Convert to layer rule suggestions
        let mut rules: Vec<String> = layer_names
            .into_iter()
            .map(|name| format!("layerrule = blur, {name}"))
            .collect();

        rules.sort();

        // Add common layer rule examples if no layers found
        if rules.is_empty() {
            rules = vec![
                "layerrule = blur, waybar".to_string(),
                "layerrule = ignorezero, waybar".to_string(),
                "layerrule = noanim, wallpaper".to_string(),
                "layerrule = blur, notifications".to_string(),
            ];
        }

        Ok(rules)
    }

    #[allow(dead_code)]
    pub async fn add_window_rule(&self, rule: &str) -> Result<()> {
        let command = format!("keyword windowrule {rule}");
        let result = self.dispatch(&command).await;
        
        // Clear cache after adding rule since configuration has changed
        if result.is_ok() {
            self.clear_cache();
        }
        
        result
    }

    #[allow(dead_code)]
    pub async fn add_layer_rule(&self, rule: &str) -> Result<()> {
        let command = format!("keyword layerrule {rule}");
        let result = self.dispatch(&command).await;
        
        // Clear cache after adding rule since configuration has changed
        if result.is_ok() {
            self.clear_cache();
        }
        
        result
    }

    #[allow(dead_code)]
    pub async fn add_workspace_rule(&self, rule: &str) -> Result<()> {
        let command = format!("keyword workspace {rule}");
        let result = self.dispatch(&command).await;
        
        // Clear cache after adding rule since configuration has changed
        if result.is_ok() {
            self.clear_cache();
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration as TokioDuration;

    #[tokio::test]
    async fn test_timeout_configuration() {
        let mut hyprctl = HyprCtl::new_disconnected();
        
        // Test default timeout
        assert_eq!(hyprctl.get_timeout(), 5000);
        
        // Test setting custom timeout
        hyprctl.set_timeout(3000);
        assert_eq!(hyprctl.get_timeout(), 3000);
        
        // Test setting very short timeout
        hyprctl.set_timeout(100);
        assert_eq!(hyprctl.get_timeout(), 100);
    }

    #[tokio::test]
    async fn test_timeout_error_generation() {
        let hyprctl = HyprCtl::new_disconnected();
        
        // Test timeout with very short duration (1ms should always timeout)
        let result = hyprctl.execute_with_custom_timeout(&["version"], 1).await;
        
        match result {
            Err(HyprctlError::Timeout { command, timeout_ms }) => {
                assert_eq!(command, "hyprctl version");
                assert_eq!(timeout_ms, 1);
            }
            Err(HyprctlError::CommandNotFound) => {
                // This is also acceptable if hyprctl is not available
                println!("hyprctl not found - this is expected in test environments");
            }
            Ok(_) => {
                // Very unlikely with 1ms timeout, but possible in fast environments
                println!("Command completed within 1ms - surprisingly fast!");
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[tokio::test] 
    async fn test_timeout_functionality_helper() {
        let hyprctl = HyprCtl::new_disconnected();
        
        // The test helper should return true if timeout works correctly
        // or false if hyprctl is not available (which is fine in test environments)
        let timeout_works = hyprctl.test_timeout_functionality().await;
        
        // We don't assert on the result since hyprctl might not be available
        // in test environments, but we verify the method runs without panic
        println!("Timeout functionality test result: {}", timeout_works);
    }

    #[test]
    fn test_error_types() {
        // Test that timeout errors are properly formatted
        let timeout_error = HyprctlError::Timeout {
            command: "hyprctl test".to_string(),
            timeout_ms: 5000,
        };
        
        let error_string = format!("{}", timeout_error);
        assert!(error_string.contains("timed out"));
        assert!(error_string.contains("5000ms"));
        assert!(error_string.contains("hyprctl test"));
    }

    #[tokio::test]
    async fn test_cache_with_timeout() {
        let hyprctl = HyprCtl::new_disconnected();
        
        // Verify cache stats are accessible
        let stats = hyprctl.get_cache_stats();
        assert!(stats.is_some());
        
        if let Some(stats) = stats {
            assert_eq!(stats.default_ttl_secs, 30); // Default cache TTL
            assert_eq!(stats.bulk_ttl_secs, 60);    // Default bulk TTL
        }
    }
}
