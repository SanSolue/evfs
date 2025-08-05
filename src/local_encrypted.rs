
use crate::core::*;
use crate::local::*;
use crate::enc_utils::*;

/// A local file system implementation that reads and writes encrypted files to the local disk.
/// It uses the `EncUtils` for encryption and decryption of file contents.
/// It can be configured to be writable or read-only.
pub struct LocalEncryptedFileSystem {
    internal: LocalFileSystem,
    enc_util: EncUtils,
}

impl LocalEncryptedFileSystem {

    /// Creates a new instance of `LocalEncryptedFileSystem`.
    /// # Arguments
    /// - _base_path:_ The base path where files will be stored.
    /// - _writable:_ If true, the file system allows writing files; otherwise,
    ///   it is read-only.
    pub fn new(base_path: &str, writable: bool, key: EncKey) -> Result<Self, FileSystemError> {
        let internal = LocalFileSystem::new(base_path, writable)?;
        let enc_util = EncUtils::new(key)?;
        Ok(LocalEncryptedFileSystem { internal, enc_util })
    }
}

impl FileSystem for LocalEncryptedFileSystem {
    fn read_file(&self, path: &str) -> Result<FileContent, FileSystemError> {
        let content = self.internal.read_file(path)?;
        self.enc_util.decrypt(content)
    }

    fn write_file(&self, path: &str, content: FileContent) -> Result<(), FileSystemError> {
        let encrypted_content = self.enc_util.encrypt(content)?;
        self.internal.write_file(path, encrypted_content)
    }

    fn delete_file(&self, path: &str) -> Result<(), FileSystemError> {
        self.internal.delete_file(path)
    }

    fn list_files(&self, directory: &str) -> Result<Vec<FileInfo>, FileSystemError> {
        self.internal.list_files(directory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_encrypted_file_system() {
        // use rand to generate a secure key
        let key = EncUtils::generate_random_key();
        let fs = LocalEncryptedFileSystem::new("test_dir", true, key).unwrap();

        let content = b"Hello, World!".to_vec();
        fs.write_file("test.txt", content.clone()).unwrap();

        let read_content = fs.read_file("test.txt").unwrap();
        assert_eq!(read_content, content);

        fs.delete_file("test.txt").unwrap();

        // remove test directory
        std::fs::remove_dir_all("test_dir").unwrap_or(());
    }
}