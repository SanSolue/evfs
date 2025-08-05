
use crate::core::*;
use crate::local::*;
use crate::enc_utils::*;

pub struct LocalEncryptedFileSystem {
    internal: LocalFileSystem,
    enc_util: EncUtils,
}
impl LocalEncryptedFileSystem {
    pub fn new(base_path: &str, writable: bool, key: Vec<u8>) -> Result<Self, FileSystemError> {
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