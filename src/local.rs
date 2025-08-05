use std::path::PathBuf;
use crate::{FileInfo, FileSystem, FileSystemError, FileContent};

/// A local file system implementation that reads and writes files to the local disk.
/// It can be configured to be writable or read-only.
pub struct LocalFileSystem {
    base_path: PathBuf,
    writable: bool,
}

impl LocalFileSystem {

    /// # LocalFileSystem::new(base_path: &str, writable: bool)
    /// Creates a new instance of `LocalFileSystem`.
    ///
    /// # Arguments
    /// - _base_path:_ The base path where files will be stored.
    /// - _writable:_ If true, the file system allows writing files; otherwise, it is read-only.
    ///
    /// # Returns
    /// Result containing the `LocalFileSystem` instance or an error if the base path is invalid.
    /// # Errors
    /// `FileSystemError` if the base path is not valid for the specified mode.
    pub fn new(base_path: &str, writable: bool) -> Result<Self, FileSystemError> {
        let base_path = PathBuf::from(base_path);
        if writable {
            if !base_path.exists() {
                std::fs::create_dir_all(&base_path)
                    .expect("Failed to create base path for writable local file system");
            }
        } else {
            if !base_path.is_dir() {
                return Err(FileSystemError::from("Base path must be a directory for non-writable local file system"));
            }
            if !base_path.exists() {
                return Err(FileSystemError::from("Base path does not exist for non-writable local file system"));
            }
        }
        Ok(LocalFileSystem {
            base_path,
            writable,
        })
    }

    fn full_path(&self, path: &str) -> PathBuf {
        self.base_path.join(path)
    }

    fn ensure_writable(&self) -> Result<(), FileSystemError> {
        if !self.writable {
            return Err(FileSystemError::from("File system is not writable"));
        }
        Ok(())
    }
}

impl FileSystem for LocalFileSystem {
    fn read_file(&self, path: &str) -> Result<FileContent, FileSystemError> {
        let full_path = self.full_path(path);
        if !full_path.exists() {
            return Err(FileSystemError::from("File does not exist"));
        }
        if !full_path.is_file() {
            return Err(FileSystemError::from("Path is not a file"));
        }
        std::fs::read(full_path).map_err(|e| FileSystemError::from(e.to_string()))
    }

    fn write_file(&self, path: &str, content: FileContent) -> Result<(), FileSystemError> {
        self.ensure_writable()?;
        let full_path = self.full_path(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| FileSystemError::from(e.to_string()))?;
        }
        std::fs::write(full_path, content).map_err(|e| FileSystemError::from(e.to_string()))
    }

    fn delete_file(&self, path: &str) -> Result<(), FileSystemError> {
        self.ensure_writable()?;
        let full_path = self.full_path(path);
        if !full_path.exists() {
            return Err(FileSystemError::from("File does not exist"));
        }
        if !full_path.is_file() {
            return Err(FileSystemError::from("Path is not a file"));
        }
        std::fs::remove_file(full_path).map_err(|e| FileSystemError::from(e.to_string()))
    }

    fn list_files(&self, directory: &str) -> Result<Vec<FileInfo>, FileSystemError> {
        let full_path = self.full_path(directory);
        if !full_path.exists() {
            return Err(FileSystemError::from("Directory does not exist"));
        }
        if !full_path.is_dir() {
            return Err(FileSystemError::from("Path is not a directory"));
        }
        let entries = std::fs::read_dir(full_path)
            .map_err(|e| FileSystemError::from(e.to_string()))?;

        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| FileSystemError::from(e.to_string()))?;
            files.push(FileInfo::from(entry));
        }
        Ok(files)
    }
}


// Tests for the LocalFileSystem
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_local_filesystem_creation() {
        let fs = LocalFileSystem::new("test_dir", true);
        assert!(fs.is_ok());
        let fs = LocalFileSystem::new("test_dir", false);
        assert!(fs.is_ok());
        // Open a writable file system
        let fs = LocalFileSystem::new("test_dir", true);
        assert!(fs.is_ok());
        // Open a non-writable file system
        let fs = LocalFileSystem::new("test_dir_non_existent", false);
        assert!(fs.is_err());
        // Clean up
        std::fs::remove_dir_all("test_dir").ok();
    }

    #[test]
    fn test_local_filesystem_read_write() {
        let fs = LocalFileSystem::new("test_dir_rw", true).unwrap();
        let content = b"Hello, World!";
        let path = "test_file.txt";

        // Write file
        fs.write_file(path, content.to_vec()).unwrap();
        // Read file
        let read_content = fs.read_file(path).unwrap();
        assert_eq!(read_content, content);
        // List files
        let files = fs.list_files(".").unwrap();
        assert!(files.iter().any(|f| f.name == "test_file.txt"));
        // Delete file
        fs.delete_file(path).unwrap();
        // Verify deletion
        let read_result = fs.read_file(path);
        assert!(read_result.is_err());
    }
}
