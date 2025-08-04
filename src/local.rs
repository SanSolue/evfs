use std::path::PathBuf;
use crate::{FileInfo, FileSystem, FileSystemError, FileContent};

pub struct LocalFileSystem {
    base_path: PathBuf,
    writable: bool,
}

impl LocalFileSystem {
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

#[cfg(feature = "enc")]
mod encrypted {
use super::*;
    pub struct LocalEncryptedFileSystem {
        internal: LocalFileSystem,
        key: Vec<u8>,
    }
    impl LocalEncryptedFileSystem {
        pub fn new(base_path: &str, writable: bool, key: Vec<u8>) -> Result<Self, FileSystemError> {
            let internal = LocalFileSystem::new(base_path, writable)?;
            Ok(LocalEncryptedFileSystem { internal, key })
        }
        fn encrypt(&self, content: FileContent) -> FileContent {
            content
        }
        fn decrypt(&self, content: FileContent) -> FileContent {
            content
        }
    }
    impl FileSystem for LocalEncryptedFileSystem {
        fn read_file(&self, path: &str) -> Result<FileContent, FileSystemError> {
            let content = self.internal.read_file(path)?;
            Ok(self.decrypt(content))
        }

        fn write_file(&self, path: &str, content: FileContent) -> Result<(), FileSystemError> {
            let encrypted_content = self.encrypt(content);
            self.internal.write_file(path, encrypted_content)
        }

        fn delete_file(&self, path: &str) -> Result<(), FileSystemError> {
            self.internal.delete_file(path)
        }

        fn list_files(&self, directory: &str) -> Result<Vec<FileInfo>, FileSystemError> {
            self.internal.list_files(directory)
        }
    }
}

#[cfg(feature = "enc")]
pub use encrypted::LocalEncryptedFileSystem;