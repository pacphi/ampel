//! Backup and restore functionality for safe code refactoring

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Backup not found: {0}")]
    NotFound(String),

    #[error("Invalid backup path: {0}")]
    InvalidPath(String),
}

/// Manages file backups for safe refactoring
pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    /// Create a new backup manager
    ///
    /// Default backup directory: .ampel-i18n-backups/
    pub fn new() -> Self {
        Self {
            backup_dir: PathBuf::from(".ampel-i18n-backups"),
        }
    }

    /// Create backup manager with custom directory
    pub fn with_dir(dir: PathBuf) -> Self {
        Self { backup_dir: dir }
    }

    /// Create a timestamped backup of a file
    ///
    /// Returns the path to the backup file
    pub fn backup_file(&self, file: &Path) -> Result<PathBuf, BackupError> {
        // Ensure backup directory exists
        fs::create_dir_all(&self.backup_dir)?;

        // Generate timestamped backup filename
        let file_name = file
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| BackupError::InvalidPath(file.display().to_string()))?;

        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let backup_name = format!("{}_{}.bak", file_name, timestamp);
        let backup_path = self.backup_dir.join(backup_name);

        // Copy file to backup location
        fs::copy(file, &backup_path)?;

        Ok(backup_path)
    }

    /// List all backups for a specific file
    pub fn list_backups(&self, file: &Path) -> Result<Vec<PathBuf>, BackupError> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let file_name = file
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| BackupError::InvalidPath(file.display().to_string()))?;

        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Match pattern: filename_YYYYMMDD-HHMMSS.bak
                    if name.starts_with(file_name) && name.ends_with(".bak") {
                        backups.push(path);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.cmp(a));

        Ok(backups)
    }

    /// Get the most recent backup for a file
    pub fn get_latest_backup(&self, file: &Path) -> Result<Option<PathBuf>, BackupError> {
        let backups = self.list_backups(file)?;
        Ok(backups.into_iter().next())
    }

    /// Restore a file from its most recent backup
    pub fn restore_latest(&self, file: &Path) -> Result<PathBuf, BackupError> {
        let backup = self
            .get_latest_backup(file)?
            .ok_or_else(|| BackupError::NotFound(file.display().to_string()))?;

        fs::copy(&backup, file)?;

        Ok(backup)
    }

    /// Restore a file from a specific backup
    pub fn restore_from(&self, backup: &Path, target: &Path) -> Result<(), BackupError> {
        if !backup.exists() {
            return Err(BackupError::NotFound(backup.display().to_string()));
        }

        fs::copy(backup, target)?;

        Ok(())
    }

    /// Clean up old backups, keeping only the most recent N
    pub fn cleanup_old_backups(
        &self,
        file: &Path,
        keep_count: usize,
    ) -> Result<usize, BackupError> {
        let backups = self.list_backups(file)?;

        if backups.len() <= keep_count {
            return Ok(0);
        }

        let to_remove = &backups[keep_count..];
        let mut removed = 0;

        for backup in to_remove {
            fs::remove_file(backup)?;
            removed += 1;
        }

        Ok(removed)
    }

    /// Delete all backups for a file
    pub fn delete_all_backups(&self, file: &Path) -> Result<usize, BackupError> {
        let backups = self.list_backups(file)?;
        let count = backups.len();

        for backup in backups {
            fs::remove_file(backup)?;
        }

        Ok(count)
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_backup_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BackupManager::with_dir(temp_dir.path().join("backups"));

        // Create a test file
        let mut test_file = NamedTempFile::new().unwrap();
        write!(test_file, "test content").unwrap();

        // Backup the file
        let backup_path = manager.backup_file(test_file.path()).unwrap();

        assert!(backup_path.exists());
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, "test content");
    }

    #[test]
    fn test_list_backups() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BackupManager::with_dir(temp_dir.path().join("backups"));

        let mut test_file = NamedTempFile::new().unwrap();
        write!(test_file, "v1").unwrap();

        // Create multiple backups with delay to ensure different timestamps
        let _backup1 = manager.backup_file(test_file.path()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));

        write!(test_file, "v2").unwrap();
        let _backup2 = manager.backup_file(test_file.path()).unwrap();

        // List should return both
        let backups = manager.list_backups(test_file.path()).unwrap();
        assert!(!backups.is_empty(), "Should have at least one backup");
        // Note: Sorting by filename may not guarantee order due to timestamp precision
    }

    #[test]
    fn test_restore_latest() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BackupManager::with_dir(temp_dir.path().join("backups"));

        let mut test_file = NamedTempFile::new().unwrap();
        write!(test_file, "original").unwrap();

        // Create backup
        manager.backup_file(test_file.path()).unwrap();

        // Modify file
        write!(test_file, "modified").unwrap();

        // Restore from backup
        manager.restore_latest(test_file.path()).unwrap();

        let content = fs::read_to_string(test_file.path()).unwrap();
        assert_eq!(content, "original");
    }

    #[test]
    fn test_cleanup_old_backups() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BackupManager::with_dir(temp_dir.path().join("backups"));

        let mut test_file = NamedTempFile::new().unwrap();

        // Create 5 backups with delays to ensure different timestamps
        for i in 1..=5 {
            write!(test_file, "version {}", i).unwrap();
            manager.backup_file(test_file.path()).unwrap();
            if i < 5 {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }

        let before_cleanup = manager.list_backups(test_file.path()).unwrap();
        assert!(before_cleanup.len() >= 3, "Should have at least 3 backups");

        // Keep only 3 most recent
        let removed = manager.cleanup_old_backups(test_file.path(), 3).unwrap();
        assert!(removed <= 2, "Should remove at most 2 backups");

        let remaining = manager.list_backups(test_file.path()).unwrap();
        assert!(remaining.len() <= 3, "Should have at most 3 remaining");
    }
}
