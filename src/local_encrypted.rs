use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit, OsRng, rand_core::RngCore};
use crate::core::*;
use crate::local::*;

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
        // AES-256-GCM expects a 32-byte key and 12-byte nonce
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, content.as_ref()).expect("encryption failure!");
        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        result
    }
    fn decrypt(&self, content: FileContent) -> FileContent {
        // The first 12 bytes are the nonce
        if content.len() < 12 {
            return vec![]; // or handle error as needed
        }
        let (nonce_bytes, ciphertext) = content.split_at(12);
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);
        cipher.decrypt(nonce, ciphertext).unwrap_or_else(|_| vec![])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_encrypted_file_system() {
        // use rand to generate a secure key
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
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