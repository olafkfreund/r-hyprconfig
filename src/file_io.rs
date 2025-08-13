//! Enhanced file I/O operations with atomic writes, backup handling, and error recovery
//!
//! This module provides robust file I/O operations that include:
//! - Atomic file operations to prevent corruption
//! - Automatic backup creation and restoration
//! - Comprehensive error handling with specific recovery strategies
//! - File integrity verification
//! - Cross-platform compatibility

use crate::errors::{FileError, FileResult, RecoveryContext, RecoveryStrategy};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::fs as async_fs;
use tokio::time::sleep;

/// Configuration for file operations
#[derive(Debug, Clone)]
pub struct FileOperationConfig {
    /// Enable automatic backup creation before write operations
    pub create_backup: bool,
    /// Backup file suffix (e.g., ".backup")
    pub backup_suffix: String,
    /// Enable atomic write operations using temporary files
    pub atomic_writes: bool,
    /// Temporary file suffix for atomic operations
    pub temp_suffix: String,
    /// Maximum number of retry attempts for failed operations
    pub max_retries: u32,
    /// Base delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,
    /// Enable file integrity verification after write operations
    pub verify_writes: bool,
}

impl Default for FileOperationConfig {
    fn default() -> Self {
        Self {
            create_backup: true,
            backup_suffix: ".backup".to_string(),
            atomic_writes: true,
            temp_suffix: ".tmp".to_string(),
            max_retries: 3,
            retry_delay_ms: 100,
            verify_writes: true,
        }
    }
}

/// Enhanced file I/O operations with error recovery
pub struct FileOperations {
    config: FileOperationConfig,
}

impl FileOperations {
    /// Create a new FileOperations instance with default configuration
    pub fn new() -> Self {
        Self {
            config: FileOperationConfig::default(),
        }
    }

    /// Create a new FileOperations instance with custom configuration
    pub fn with_config(config: FileOperationConfig) -> Self {
        Self { config }
    }

    /// Read file contents with enhanced error handling
    pub async fn read_to_string<P: AsRef<Path>>(&self, path: P) -> FileResult<String> {
        let path = path.as_ref();
        let mut recovery_context = RecoveryContext::new("read file")
            .with_retry(self.config.max_retries, self.config.retry_delay_ms)
            .with_fallback("Try reading as empty file".to_string());

        loop {
            match self.attempt_read_to_string(path).await {
                Ok(content) => return Ok(content),
                Err(error) => {
                    if let Some(strategy) = recovery_context.next_strategy() {
                        match strategy {
                            RecoveryStrategy::Retry { max_attempts, base_delay_ms } => {
                                if recovery_context.attempt_count <= max_attempts {
                                    let delay = base_delay_ms * recovery_context.attempt_count as u64;
                                    eprintln!(
                                        "Retrying file read (attempt {}/{}) after {}ms delay: {}",
                                        recovery_context.attempt_count, max_attempts, delay, error
                                    );
                                    sleep(Duration::from_millis(delay)).await;
                                    continue;
                                }
                            }
                            RecoveryStrategy::Fallback { description } => {
                                eprintln!("Using fallback strategy: {}", description);
                                // For file not found, return empty string as fallback
                                if matches!(error, FileError::NotFound { .. }) {
                                    return Ok(String::new());
                                }
                            }
                            RecoveryStrategy::Abort => return Err(error),
                            _ => return Err(error),
                        }
                    } else {
                        return Err(error);
                    }
                }
            }
        }
    }

    /// Write content to file with atomic operations and backup
    pub async fn write_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        content: &str,
    ) -> FileResult<()> {
        let path = path.as_ref();
        let mut recovery_context = RecoveryContext::new("write file")
            .with_retry(self.config.max_retries, self.config.retry_delay_ms);

        // Create backup if enabled and file exists
        let backup_path = if self.config.create_backup && path.exists() {
            match self.create_backup(path).await {
                Ok(backup) => {
                    eprintln!("Created backup: {}", backup.display());
                    Some(backup)
                }
                Err(e) => {
                    eprintln!("Warning: Failed to create backup: {}", e);
                    None
                }
            }
        } else {
            None
        };

        loop {
            match self.attempt_write_to_file(path, content).await {
                Ok(()) => {
                    // Verify write if enabled
                    if self.config.verify_writes {
                        if let Err(e) = self.verify_file_content(path, content).await {
                            eprintln!("Warning: File verification failed: {}", e);
                            // Don't fail the operation, just warn
                        }
                    }

                    // Clean up backup on success
                    if let Some(backup) = backup_path {
                        let _ = async_fs::remove_file(backup).await; // Ignore errors
                    }
                    return Ok(());
                }
                Err(error) => {
                    if let Some(strategy) = recovery_context.next_strategy() {
                        match strategy {
                            RecoveryStrategy::Retry { max_attempts, base_delay_ms } => {
                                if recovery_context.attempt_count <= max_attempts {
                                    let delay = base_delay_ms * recovery_context.attempt_count as u64;
                                    eprintln!(
                                        "Retrying file write (attempt {}/{}) after {}ms delay: {}",
                                        recovery_context.attempt_count, max_attempts, delay, error
                                    );
                                    sleep(Duration::from_millis(delay)).await;
                                    continue;
                                }
                            }
                            RecoveryStrategy::Abort => {
                                // Restore backup if we have one
                                if let Some(backup) = backup_path {
                                    if let Err(e) = async_fs::copy(&backup, path).await {
                                        eprintln!("Failed to restore backup: {}", e);
                                    } else {
                                        eprintln!("Backup restored after write failure");
                                    }
                                }
                                return Err(error);
                            }
                            _ => return Err(error),
                        }
                    } else {
                        return Err(error);
                    }
                }
            }
        }
    }

    /// Create a timestamped backup of a file
    pub async fn create_backup<P: AsRef<Path>>(&self, path: P) -> FileResult<PathBuf> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(FileError::NotFound {
                path: path.to_path_buf(),
            });
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let backup_path = if let Some(extension) = path.extension() {
            path.with_extension(format!(
                "{}.{}.{}",
                extension.to_string_lossy(),
                self.config.backup_suffix.trim_start_matches('.'),
                timestamp
            ))
        } else {
            PathBuf::from(format!(
                "{}.{}.{}",
                path.to_string_lossy(),
                self.config.backup_suffix.trim_start_matches('.'),
                timestamp
            ))
        };

        async_fs::copy(path, &backup_path)
            .await
            .map_err(|e| FileError::BackupFailed {
                original_path: path.to_path_buf(),
                backup_path: backup_path.clone(),
                reason: e.to_string(),
            })?;

        Ok(backup_path)
    }

    /// Ensure directory exists, creating it if necessary
    pub async fn ensure_directory<P: AsRef<Path>>(&self, path: P) -> FileResult<()> {
        let path = path.as_ref();
        
        if path.exists() {
            if path.is_dir() {
                return Ok(());
            } else {
                return Err(FileError::DirectoryCreationFailed {
                    path: path.to_path_buf(),
                    reason: "Path exists but is not a directory".to_string(),
                });
            }
        }

        async_fs::create_dir_all(path)
            .await
            .map_err(|e| FileError::DirectoryCreationFailed {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })
    }

    /// Copy file with enhanced error handling
    pub async fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        from: P,
        to: Q,
    ) -> FileResult<()> {
        let from = from.as_ref();
        let to = to.as_ref();

        // Ensure destination directory exists
        if let Some(parent) = to.parent() {
            self.ensure_directory(parent).await?;
        }

        async_fs::copy(from, to)
            .await
            .map_err(|e| match e.kind() {
                io::ErrorKind::NotFound => FileError::NotFound {
                    path: from.to_path_buf(),
                },
                io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                    path: from.to_path_buf(),
                    operation: "copy".to_string(),
                },
                _ => FileError::WriteError {
                    path: to.to_path_buf(),
                    reason: e.to_string(),
                },
            })?;

        Ok(())
    }

    /// Remove file with error handling
    pub async fn remove_file<P: AsRef<Path>>(&self, path: P) -> FileResult<()> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(FileError::NotFound {
                path: path.to_path_buf(),
            });
        }

        async_fs::remove_file(path)
            .await
            .map_err(|e| match e.kind() {
                io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                    path: path.to_path_buf(),
                    operation: "delete".to_string(),
                },
                _ => FileError::WriteError {
                    path: path.to_path_buf(),
                    reason: e.to_string(),
                },
            })
    }

    /// Attempt to read file contents (internal method)
    async fn attempt_read_to_string(&self, path: &Path) -> FileResult<String> {
        async_fs::read_to_string(path)
            .await
            .map_err(|e| match e.kind() {
                io::ErrorKind::NotFound => FileError::NotFound {
                    path: path.to_path_buf(),
                },
                io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                    path: path.to_path_buf(),
                    operation: "read".to_string(),
                },
                _ => FileError::ReadError {
                    path: path.to_path_buf(),
                    reason: e.to_string(),
                },
            })
    }

    /// Attempt to write file contents with atomic operations (internal method)
    async fn attempt_write_to_file(&self, path: &Path, content: &str) -> FileResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            self.ensure_directory(parent).await?;
        }

        if self.config.atomic_writes {
            self.atomic_write(path, content).await
        } else {
            self.direct_write(path, content).await
        }
    }

    /// Perform atomic write using temporary file
    async fn atomic_write(&self, path: &Path, content: &str) -> FileResult<()> {
        let temp_path = path.with_extension(
            format!(
                "{}{}",
                path.extension()
                    .map(|e| format!("{}.{}", e.to_string_lossy(), self.config.temp_suffix.trim_start_matches('.')))
                    .unwrap_or_else(|| self.config.temp_suffix.clone()),
                "atomic"
            )
        );

        // Write to temporary file first
        async_fs::write(&temp_path, content)
            .await
            .map_err(|e| FileError::TempFileError {
                temp_path: temp_path.clone(),
                reason: e.to_string(),
            })?;

        // Atomically move temp file to final location
        async_fs::rename(&temp_path, path)
            .await
            .map_err(|e| {
                // Clean up temp file on failure
                let _ = std::fs::remove_file(&temp_path);
                FileError::AtomicOperationFailed {
                    path: path.to_path_buf(),
                    stage: format!("rename temporary file: {}", e),
                }
            })?;

        Ok(())
    }

    /// Perform direct write (non-atomic)
    async fn direct_write(&self, path: &Path, content: &str) -> FileResult<()> {
        async_fs::write(path, content)
            .await
            .map_err(|e| match e.kind() {
                io::ErrorKind::PermissionDenied => FileError::PermissionDenied {
                    path: path.to_path_buf(),
                    operation: "write".to_string(),
                },
                _ => FileError::WriteError {
                    path: path.to_path_buf(),
                    reason: e.to_string(),
                },
            })
    }

    /// Verify file content matches what was written
    async fn verify_file_content(&self, path: &Path, expected_content: &str) -> FileResult<()> {
        let actual_content = async_fs::read_to_string(path)
            .await
            .map_err(|e| FileError::ReadError {
                path: path.to_path_buf(),
                reason: format!("verification read failed: {}", e),
            })?;

        if actual_content != expected_content {
            return Err(FileError::CorruptionDetected {
                path: path.to_path_buf(),
                details: format!(
                    "Content mismatch: expected {} bytes, got {} bytes",
                    expected_content.len(),
                    actual_content.len()
                ),
            });
        }

        Ok(())
    }
}

impl Default for FileOperations {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common file operations
pub struct FileUtils;

impl FileUtils {
    /// Safely write content to a file with automatic backup and error recovery
    pub async fn safe_write<P: AsRef<Path>>(
        path: P,
        content: &str,
    ) -> FileResult<()> {
        let ops = FileOperations::new();
        ops.write_to_file(path, content).await
    }

    /// Read file content with retry on failure
    pub async fn resilient_read<P: AsRef<Path>>(path: P) -> FileResult<String> {
        let ops = FileOperations::new();
        ops.read_to_string(path).await
    }

    /// Create a backup of a file with timestamp
    pub async fn backup_file<P: AsRef<Path>>(path: P) -> FileResult<PathBuf> {
        let ops = FileOperations::new();
        ops.create_backup(path).await
    }

    /// Ensure a directory path exists
    pub async fn ensure_directory<P: AsRef<Path>>(path: P) -> FileResult<()> {
        let ops = FileOperations::new();
        ops.ensure_directory(path).await
    }

    /// Perform atomic file replacement
    pub async fn atomic_replace<P: AsRef<Path>>(
        path: P,
        content: &str,
    ) -> FileResult<PathBuf> {
        let path = path.as_ref();
        let ops = FileOperations::with_config(FileOperationConfig {
            atomic_writes: true,
            create_backup: true,
            verify_writes: true,
            ..Default::default()
        });

        // Create backup first
        let backup_path = if path.exists() {
            Some(ops.create_backup(path).await?)
        } else {
            None
        };

        // Perform atomic write
        ops.write_to_file(path, content).await?;

        Ok(backup_path.unwrap_or_else(|| PathBuf::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_file_operations_basic() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let ops = FileOperations::new();

        // Test write
        let content = "Hello, World!";
        ops.write_to_file(&file_path, content).await.unwrap();

        // Test read
        let read_content = ops.read_to_string(&file_path).await.unwrap();
        assert_eq!(content, read_content);
    }

    #[tokio::test]
    async fn test_atomic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("atomic_test.txt");
        
        let config = FileOperationConfig {
            atomic_writes: true,
            create_backup: true,
            verify_writes: true,
            ..Default::default()
        };
        let ops = FileOperations::with_config(config);

        // Write initial content
        ops.write_to_file(&file_path, "initial").await.unwrap();
        assert!(file_path.exists());

        // Update with atomic operation
        ops.write_to_file(&file_path, "updated").await.unwrap();
        
        let content = ops.read_to_string(&file_path).await.unwrap();
        assert_eq!("updated", content);
    }

    #[tokio::test]
    async fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("backup_test.txt");
        let ops = FileOperations::new();

        // Create initial file
        fs::write(&file_path, "original content").unwrap();

        // Create backup
        let backup_path = ops.create_backup(&file_path).await.unwrap();
        assert!(backup_path.exists());

        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!("original content", backup_content);
    }

    #[tokio::test]
    async fn test_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested").join("directory");
        let ops = FileOperations::new();

        ops.ensure_directory(&nested_path).await.unwrap();
        assert!(nested_path.exists());
        assert!(nested_path.is_dir());
    }

    #[tokio::test]
    async fn test_file_utils() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("utils_test.txt");

        // Test safe write
        FileUtils::safe_write(&file_path, "test content").await.unwrap();

        // Test resilient read
        let content = FileUtils::resilient_read(&file_path).await.unwrap();
        assert_eq!("test content", content);

        // Test backup
        let backup_path = FileUtils::backup_file(&file_path).await.unwrap();
        assert!(backup_path.exists());
    }
}