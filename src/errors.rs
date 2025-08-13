//! Custom error types for r-hyprconfig application.
//!
//! This module provides structured error handling using the `thiserror` crate
//! to create user-friendly error messages and proper error propagation.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Main application error type
#[derive(Error, Debug)]
pub enum HyprConfigError {
    /// Hyprctl command execution errors
    #[error("Hyprctl command failed: {message}")]
    HyprctlError { message: String },

    /// Configuration file errors
    #[error("Configuration file error at {path}: {source}")]
    ConfigFileError { path: PathBuf, source: io::Error },

    /// Configuration parsing errors
    #[error("Configuration parsing failed: {message}")]
    ConfigParsingError { message: String },

    /// Configuration validation errors
    #[error("Configuration validation failed for '{option}': {message}")]
    ConfigValidationError { option: String, message: String },

    /// File I/O errors
    #[error("File operation failed: {operation} on {path}")]
    FileOperationError {
        operation: String,
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// Permission errors
    #[error("Permission denied: {message}")]
    PermissionError { message: String },

    /// Cache-related errors
    #[error("Cache operation failed: {message}")]
    CacheError { message: String },

    /// NixOS-specific errors
    #[error("NixOS operation failed: {message}")]
    NixOSError { message: String },

    /// Import/Export errors
    #[error("Import/Export operation failed: {operation} - {message}")]
    ImportExportError { operation: String, message: String },

    /// UI-related errors
    #[error("UI operation failed: {message}")]
    UIError { message: String },

    /// Terminal/TUI errors
    #[error("Terminal operation failed: {message}")]
    TerminalError {
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Invalid input errors
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// Network-related errors
    #[error("Network operation failed: {operation} - {message}")]
    NetworkError { operation: String, message: String },

    /// Generic fallback error
    #[error("Operation failed: {message}")]
    GenericError { message: String },
}

/// Configuration-specific error types
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Invalid configuration value
    #[error("Invalid value for '{key}': {value} - {reason}")]
    InvalidValue {
        key: String,
        value: String,
        reason: String,
    },

    /// Missing required configuration
    #[error("Missing required configuration: {key}")]
    MissingRequired { key: String },

    /// Conflicting configuration options
    #[error("Conflicting configuration: {conflict}")]
    ConflictingOptions { conflict: String },

    /// Unsupported configuration option
    #[error("Unsupported configuration option: {key}")]
    UnsupportedOption { key: String },
}

/// Enhanced file operation error types with specific error conditions and recovery strategies
#[derive(Error, Debug)]
pub enum FileError {
    /// File not found
    #[error("File not found: {path}")]
    NotFound { path: PathBuf },

    /// Permission denied
    #[error("Permission denied accessing: {path} (operation: {operation})")]
    PermissionDenied { path: PathBuf, operation: String },

    /// File already exists
    #[error("File already exists: {path}")]
    AlreadyExists { path: PathBuf },

    /// Invalid file format
    #[error("Invalid file format for {path}: expected {expected}, got {actual}")]
    InvalidFormat {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    /// Corruption or invalid content
    #[error("File content is invalid or corrupted: {path} - {reason}")]
    InvalidContent { path: PathBuf, reason: String },

    /// Failed to read file
    #[error("Failed to read file: {path} - {reason}")]
    ReadError { path: PathBuf, reason: String },
    
    /// Failed to write file
    #[error("Failed to write file: {path} - {reason}")]
    WriteError { path: PathBuf, reason: String },
    
    /// Directory creation failed
    #[error("Directory creation failed: {path} - {reason}")]
    DirectoryCreationFailed { path: PathBuf, reason: String },
    
    /// File operation interrupted
    #[error("File operation interrupted: {path} (operation: {operation})")]
    OperationInterrupted { path: PathBuf, operation: String },
    
    /// Disk space insufficient
    #[error("Disk space insufficient for operation on: {path}")]
    InsufficientSpace { path: PathBuf },
    
    /// File locked by another process
    #[error("File locked by another process: {path}")]
    FileLocked { path: PathBuf },
    
    /// Backup operation failed
    #[error("Backup operation failed: {original_path} -> {backup_path} - {reason}")]
    BackupFailed { 
        original_path: PathBuf, 
        backup_path: PathBuf, 
        reason: String 
    },
    
    /// Atomic operation failed
    #[error("Atomic operation failed: {path} - {stage}")]
    AtomicOperationFailed { path: PathBuf, stage: String },
    
    /// File corruption detected
    #[error("File corruption detected: {path} - {details}")]
    CorruptionDetected { path: PathBuf, details: String },
    
    /// Temporary file operation failed
    #[error("Temporary file operation failed: {temp_path} - {reason}")]
    TempFileError { temp_path: PathBuf, reason: String },
}

/// Hyprctl-specific error types
#[derive(Error, Debug)]
pub enum HyprctlError {
    /// Hyprctl command not found
    #[error("Hyprctl command not found - is Hyprland running?")]
    CommandNotFound,

    /// Hyprland is not running
    #[error("Hyprland is not running or not accessible")]
    HyprlandNotRunning,

    /// Invalid hyprctl option
    #[error("Invalid hyprctl option: {option}")]
    InvalidOption { option: String },

    /// Hyprctl command timeout
    #[error("Hyprctl command timed out after {timeout_ms}ms: {command}")]
    Timeout { command: String, timeout_ms: u64 },

    /// Failed to parse hyprctl output
    #[error("Failed to parse hyprctl output for command '{command}': {reason}")]
    ParseError { command: String, reason: String },

    /// Hyprctl command execution failed
    #[error("Hyprctl command failed: {command} - {stderr}")]
    ExecutionFailed { command: String, stderr: String },
}

/// Result type alias using our custom error
pub type Result<T> = std::result::Result<T, HyprConfigError>;

/// Configuration result type alias
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

/// File operation result type alias
pub type FileResult<T> = std::result::Result<T, FileError>;

/// Hyprctl result type alias
pub type HyprctlResult<T> = std::result::Result<T, HyprctlError>;

impl From<io::Error> for HyprConfigError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::PermissionDenied => HyprConfigError::PermissionError {
                message: error.to_string(),
            },
            _ => HyprConfigError::GenericError {
                message: error.to_string(),
            },
        }
    }
}

impl From<ConfigError> for HyprConfigError {
    fn from(error: ConfigError) -> Self {
        match error {
            ConfigError::InvalidValue { key, value, reason } => {
                HyprConfigError::ConfigValidationError {
                    option: format!("{}={}", key, value),
                    message: reason,
                }
            }
            _ => HyprConfigError::ConfigParsingError {
                message: error.to_string(),
            },
        }
    }
}

impl From<FileError> for HyprConfigError {
    fn from(error: FileError) -> Self {
        match error {
            FileError::PermissionDenied { path, operation } => HyprConfigError::PermissionError {
                message: format!("Cannot access file {} for operation '{}': Permission denied", path.display(), operation),
            },
            FileError::NotFound { path } => HyprConfigError::FileOperationError {
                operation: "read".to_string(),
                path,
                source: io::Error::new(io::ErrorKind::NotFound, "File not found"),
            },
            _ => HyprConfigError::GenericError {
                message: error.to_string(),
            },
        }
    }
}

impl From<HyprctlError> for HyprConfigError {
    fn from(error: HyprctlError) -> Self {
        HyprConfigError::HyprctlError {
            message: error.to_string(),
        }
    }
}

/// Helper functions for creating specific error types
impl HyprConfigError {
    /// Create a configuration validation error
    pub fn config_validation(option: impl Into<String>, message: impl Into<String>) -> Self {
        HyprConfigError::ConfigValidationError {
            option: option.into(),
            message: message.into(),
        }
    }

    /// Create a file operation error
    pub fn file_operation(
        operation: impl Into<String>,
        path: impl Into<PathBuf>,
        source: io::Error,
    ) -> Self {
        HyprConfigError::FileOperationError {
            operation: operation.into(),
            path: path.into(),
            source,
        }
    }

    /// Create a hyprctl error
    pub fn hyprctl(message: impl Into<String>) -> Self {
        HyprConfigError::HyprctlError {
            message: message.into(),
        }
    }

    /// Create a permission error
    pub fn permission(message: impl Into<String>) -> Self {
        HyprConfigError::PermissionError {
            message: message.into(),
        }
    }

    /// Create a NixOS error
    pub fn nixos(message: impl Into<String>) -> Self {
        HyprConfigError::NixOSError {
            message: message.into(),
        }
    }

    /// Create an import/export error
    pub fn import_export(operation: impl Into<String>, message: impl Into<String>) -> Self {
        HyprConfigError::ImportExportError {
            operation: operation.into(),
            message: message.into(),
        }
    }
}

/// Helper functions for creating config errors
impl ConfigError {
    /// Create an invalid value error
    pub fn invalid_value(
        key: impl Into<String>,
        value: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        ConfigError::InvalidValue {
            key: key.into(),
            value: value.into(),
            reason: reason.into(),
        }
    }

    /// Create a missing required error
    pub fn missing_required(key: impl Into<String>) -> Self {
        ConfigError::MissingRequired { key: key.into() }
    }
}

/// Helper functions for creating file errors with enhanced context
impl FileError {
    /// Create a read error with specific reason
    pub fn read_error(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        FileError::ReadError {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create a write error with specific reason
    pub fn write_error(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        FileError::WriteError {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create a permission denied error with operation context
    pub fn permission_denied(path: impl Into<PathBuf>, operation: impl Into<String>) -> Self {
        FileError::PermissionDenied {
            path: path.into(),
            operation: operation.into(),
        }
    }

    /// Create a backup failed error
    pub fn backup_failed(
        original_path: impl Into<PathBuf>,
        backup_path: impl Into<PathBuf>,
        reason: impl Into<String>,
    ) -> Self {
        FileError::BackupFailed {
            original_path: original_path.into(),
            backup_path: backup_path.into(),
            reason: reason.into(),
        }
    }

    /// Create an atomic operation failed error
    pub fn atomic_operation_failed(path: impl Into<PathBuf>, stage: impl Into<String>) -> Self {
        FileError::AtomicOperationFailed {
            path: path.into(),
            stage: stage.into(),
        }
    }

    /// Create a temporary file error
    pub fn temp_file_error(temp_path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        FileError::TempFileError {
            temp_path: temp_path.into(),
            reason: reason.into(),
        }
    }

    /// Create a directory creation failed error
    pub fn directory_creation_failed(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        FileError::DirectoryCreationFailed {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Determine if this error is recoverable with retry
    pub fn is_retryable(&self) -> bool {
        match self {
            FileError::FileLocked { .. } => true,
            FileError::OperationInterrupted { .. } => true,
            FileError::TempFileError { .. } => true,
            FileError::WriteError { reason, .. } => {
                reason.contains("device busy") || 
                reason.contains("temporary") ||
                reason.contains("try again")
            }
            FileError::ReadError { reason, .. } => {
                reason.contains("device busy") ||
                reason.contains("temporary")
            }
            _ => false,
        }
    }

    /// Determine if this error suggests insufficient space
    pub fn suggests_space_issue(&self) -> bool {
        match self {
            FileError::InsufficientSpace { .. } => true,
            FileError::WriteError { reason, .. } => {
                reason.contains("No space left") ||
                reason.contains("Disk full") ||
                reason.contains("insufficient space")
            }
            FileError::AtomicOperationFailed { stage, .. } => {
                stage.contains("space") || stage.contains("disk")
            }
            _ => false,
        }
    }

    /// Determine if this error suggests permission issues
    pub fn suggests_permission_issue(&self) -> bool {
        match self {
            FileError::PermissionDenied { .. } => true,
            FileError::WriteError { reason, .. } => {
                reason.contains("Permission denied") ||
                reason.contains("Access denied") ||
                reason.contains("Forbidden")
            }
            FileError::ReadError { reason, .. } => {
                reason.contains("Permission denied") ||
                reason.contains("Access denied")
            }
            _ => false,
        }
    }

    /// Get recovery strategy for this file error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            FileError::FileLocked { .. } => RecoveryStrategy::Retry {
                max_attempts: 3,
                base_delay_ms: 100,
            },
            FileError::OperationInterrupted { .. } => RecoveryStrategy::Retry {
                max_attempts: 2,
                base_delay_ms: 50,
            },
            FileError::InsufficientSpace { path } => RecoveryStrategy::UserIntervention {
                message: format!(
                    "Insufficient disk space for operation on {}. Please free up space and try again.",
                    path.display()
                ),
            },
            FileError::PermissionDenied { path, operation } => RecoveryStrategy::UserIntervention {
                message: format!(
                    "Permission denied for {} operation on {}. Please check file permissions or run with appropriate privileges.",
                    operation, path.display()
                ),
            },
            FileError::CorruptionDetected { path, .. } => RecoveryStrategy::UserIntervention {
                message: format!(
                    "File corruption detected in {}. Please restore from backup or recreate the file.",
                    path.display()
                ),
            },
            FileError::NotFound { path } => RecoveryStrategy::Fallback {
                description: format!(
                    "File {} not found, will attempt to create it or use default values",
                    path.display()
                ),
            },
            FileError::TempFileError { .. } => RecoveryStrategy::Retry {
                max_attempts: 2,
                base_delay_ms: 200,
            },
            _ => RecoveryStrategy::Abort,
        }
    }

    /// Get user-friendly error message with recovery suggestions
    pub fn user_message(&self) -> String {
        match self {
            FileError::NotFound { path } => {
                format!("File '{}' was not found. It may have been moved or deleted.", path.display())
            }
            FileError::PermissionDenied { path, operation } => {
                format!(
                    "Permission denied when trying to {} '{}'. Please check the file permissions.",
                    operation, path.display()
                )
            }
            FileError::InsufficientSpace { path } => {
                format!("Not enough disk space to complete the operation on '{}'. Please free up some space.", path.display())
            }
            FileError::FileLocked { path } => {
                format!("File '{}' is currently in use by another application. Please close it and try again.", path.display())
            }
            FileError::CorruptionDetected { path, details } => {
                format!("File '{}' appears to be corrupted ({}). Consider restoring from backup.", path.display(), details)
            }
            FileError::BackupFailed { original_path, reason, .. } => {
                format!("Failed to create backup of '{}': {}. The original file was not modified.", original_path.display(), reason)
            }
            FileError::AtomicOperationFailed { path, stage } => {
                format!("Safe file operation failed for '{}' during {}: The file was not modified to prevent corruption.", path.display(), stage)
            }
            _ => self.to_string(),
        }
    }
}

/// Helper functions for creating hyprctl errors
impl HyprctlError {
    /// Create a timeout error
    pub fn timeout(command: impl Into<String>, timeout_ms: u64) -> Self {
        HyprctlError::Timeout {
            command: command.into(),
            timeout_ms,
        }
    }

    /// Create a parse error
    pub fn parse_error(command: impl Into<String>, reason: impl Into<String>) -> Self {
        HyprctlError::ParseError {
            command: command.into(),
            reason: reason.into(),
        }
    }

    /// Create an execution failed error
    pub fn execution_failed(command: impl Into<String>, stderr: impl Into<String>) -> Self {
        HyprctlError::ExecutionFailed {
            command: command.into(),
            stderr: stderr.into(),
        }
    }

    /// Check if this error suggests Hyprland is not running
    pub fn suggests_hyprland_down(&self) -> bool {
        match self {
            HyprctlError::HyprlandNotRunning => true,
            HyprctlError::CommandNotFound => true,
            HyprctlError::ExecutionFailed { stderr, .. } => {
                stderr.contains("No such file or directory") || 
                stderr.contains("Connection refused") ||
                stderr.contains("could not connect")
            }
            _ => false,
        }
    }

    /// Check if this error is recoverable with retry
    pub fn is_retryable(&self) -> bool {
        match self {
            HyprctlError::Timeout { .. } => true,
            HyprctlError::ExecutionFailed { stderr, .. } => {
                stderr.contains("busy") || 
                stderr.contains("temporary") ||
                stderr.contains("try again")
            }
            _ => false,
        }
    }

    /// Get user-friendly error message with recovery suggestions
    pub fn user_message(&self) -> String {
        match self {
            HyprctlError::CommandNotFound => {
                "Hyprctl command not found. Please ensure Hyprland is installed and running.".to_string()
            }
            HyprctlError::HyprlandNotRunning => {
                "Hyprland is not running. Please start Hyprland and try again.".to_string()
            }
            HyprctlError::InvalidOption { option } => {
                format!("Configuration option '{}' is not supported by your Hyprland version. Please check the documentation or update Hyprland.", option)
            }
            HyprctlError::Timeout { command, timeout_ms } => {
                format!("Command '{}' timed out after {}ms. Hyprland may be busy - try again in a moment.", command, timeout_ms)
            }
            HyprctlError::ParseError { command, reason } => {
                format!("Failed to understand Hyprland's response to '{}': {}. This may indicate a version compatibility issue.", command, reason)
            }
            HyprctlError::ExecutionFailed { command, stderr } => {
                if stderr.contains("could not connect") {
                    format!("Cannot connect to Hyprland. Make sure Hyprland is running and try again.")
                } else {
                    format!("Hyprland command '{}' failed: {}. Please check your configuration.", command, stderr)
                }
            }
        }
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff
    Retry { max_attempts: u32, base_delay_ms: u64 },
    /// Fall back to a default value or alternative approach
    Fallback { description: String },
    /// Ask user for manual intervention
    UserIntervention { message: String },
    /// Skip this operation and continue
    Skip { reason: String },
    /// Abort the entire operation
    Abort,
}

/// Recovery context for error handling
#[derive(Debug)]
pub struct RecoveryContext {
    pub operation: String,
    pub attempt_count: u32,
    pub strategies: Vec<RecoveryStrategy>,
}

impl RecoveryContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            attempt_count: 0,
            strategies: Vec::new(),
        }
    }

    pub fn with_retry(mut self, max_attempts: u32, base_delay_ms: u64) -> Self {
        self.strategies.push(RecoveryStrategy::Retry {
            max_attempts,
            base_delay_ms,
        });
        self
    }

    pub fn with_fallback(mut self, description: impl Into<String>) -> Self {
        self.strategies.push(RecoveryStrategy::Fallback {
            description: description.into(),
        });
        self
    }

    pub fn with_user_intervention(mut self, message: impl Into<String>) -> Self {
        self.strategies.push(RecoveryStrategy::UserIntervention {
            message: message.into(),
        });
        self
    }

    pub fn next_strategy(&mut self) -> Option<RecoveryStrategy> {
        if self.attempt_count < self.strategies.len() as u32 {
            let strategy = self.strategies[self.attempt_count as usize].clone();
            self.attempt_count += 1;
            Some(strategy)
        } else {
            Some(RecoveryStrategy::Abort)
        }
    }
}