use std::fmt::{Debug, Display};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit, OsRng, rand_core::RngCore};
use crate::{FileContent, FileSystemError};

/// Constants for encryption key size
/// AES-256-GCM requires a 32-byte key
pub const MAX_ENC_KEY_SIZE: usize = 32; // Maximum size for encryption key

/// Type alias for encryption key
pub type EncKey = Vec<u8>;

/// Utility struct for encryption and decryption operations
/// using AES-256-GCM. It provides methods to encrypt and decrypt file content,
/// manage the encryption key, and validate key sizes.
#[derive(Clone, PartialEq, Eq)]
pub struct EncUtils {
    pub key: EncKey,
}

impl Debug for EncUtils {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncUtils {{ key: [REDACTED] }}") // Avoid displaying the key directly
    }
}

impl Default for EncUtils {
    fn default() -> Self {
        // Generate a random key by default
        let key = EncUtils::generate_random_key();
        EncUtils { key }
    }
}
impl Display for EncUtils {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncUtils {{ key: [REDACTED] }}") // Avoid displaying the key directly
    }
}


impl EncUtils {

    /// Creates a new instance of `EncUtils` with the provided encryption key.
    ///
    /// # Arguments
    ///  - _key:_ The encryption key to use for encryption and decryption.
    ///
    /// # Errors
    /// Returns an error if the key is invalid (empty or exceeds maximum size).
    ///
    /// # Returns
    /// Result containing the `EncUtils` instance or an error if the key is invalid.
    pub fn new(key: EncKey) -> Result<Self, FileSystemError> {
        Self::is_valid_key(&key)?;
        Ok(EncUtils { key })
    }

    /// Returns the current encryption key.
    ///
    /// # Returns
    /// A reference to the encryption key.
    pub fn get_key(&self) -> &EncKey {
        &self.key
    }

    /// Sets a new encryption key.
    ///
    /// # Arguments
    /// - _key:_ The new encryption key to set.
    ///
    /// # Returns
    /// Result indicating success or an error if the key is invalid.
    pub fn set_key(&mut self, key: EncKey) -> Result<(), FileSystemError> {
        Self::is_valid_key(&key)?;
        self.key = key;
        Ok(())
    }

    /// Encrypts the provided file content using AES-256-GCM.
    ///
    /// # Arguments
    /// - _content:_ The file content to encrypt.
    ///
    /// # Returns
    /// Result containing the encrypted content or an error if encryption fails.
    pub fn encrypt(&self, content: FileContent) -> Result<FileContent, FileSystemError> {
        // AES-256-GCM expects a 32-byte key and 12-byte nonce
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, content.as_ref()).map_err(|_| FileSystemError::from("Encryption failed"))?;
        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypts the provided file content using AES-256-GCM.
    ///
    /// # Arguments
    /// - _content:_ The encrypted file content to decrypt.
    ///
    /// # Returns
    /// Result containing the decrypted content or an error if decryption fails.
    pub fn decrypt(&self, content: FileContent) -> Result<FileContent, FileSystemError> {
        // The first 12 bytes are the nonce
        if content.len() < 12 {
            return Err(FileSystemError::from("Content too short for decryption"));
        }
        let (nonce_bytes, ciphertext) = content.split_at(12);
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);
        Ok(cipher.decrypt(nonce, ciphertext).unwrap_or_else(|_| vec![]))
    }

    /// Static method to validate the key size.
    ///
    /// # Arguments
    /// - _key:_ The encryption key to validate.
    ///
    /// # Returns
    /// Result indicating success or an error if the key is invalid.
    pub fn is_valid_key(key: &EncKey) -> Result<(), FileSystemError> {
        if key.is_empty() {
            return Err(FileSystemError::from("Encryption key cannot be empty"));
        }
        if key.len() > MAX_ENC_KEY_SIZE {
            return Err(FileSystemError::from(format!(
                "Encryption key exceeds maximum size of {} bytes",
                MAX_ENC_KEY_SIZE
            )));
        }
        Ok(())
    }

    /// Static method to generate a random key.
    ///
    /// # Returns
    /// A random encryption key of size `MAX_ENC_KEY_SIZE`.
    pub fn generate_random_key() -> EncKey {
        let mut key = vec![0u8; MAX_ENC_KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enc_utils() {
        let key = EncUtils::generate_random_key();
        let enc_utils = EncUtils::new(key.clone()).expect("Failed to create EncUtils");
        let content = b"Hello, World!".to_vec();
        let encrypted = enc_utils.encrypt(content.clone()).expect("Encryption failed");
        let decrypted = enc_utils.decrypt(encrypted).expect("Decryption failed");
        assert_eq!(content, decrypted);
    }

    #[test]
    fn test_invalid_key() {
        let invalid_key = vec![0u8; MAX_ENC_KEY_SIZE + 1];
        let result = EncUtils::new(invalid_key);
        assert!(result.is_err(), "Expected error for invalid key size");

        let empty_key = vec![];
        let result = EncUtils::new(empty_key);
        assert!(result.is_err(), "Expected error for empty key");

        let valid_key = vec![0u8; MAX_ENC_KEY_SIZE];
        let result = EncUtils::new(valid_key);
        assert!(result.is_ok(), "Expected success for valid key size");
    }

    #[test]
    fn test_key_get_set() {
        let key = EncUtils::generate_random_key();
        let mut enc_utils = EncUtils::new(key.clone()).expect("Failed to create EncUtils");

        // Test get_key
        assert_eq!(enc_utils.get_key(), &key);

        // Test set_key with valid key
        let new_key = EncUtils::generate_random_key();
        enc_utils.set_key(new_key.clone()).expect("Failed to set new key");
        assert_eq!(enc_utils.get_key(), &new_key);

        // Test set_key with invalid key
        let invalid_key = vec![0u8; MAX_ENC_KEY_SIZE + 1];
        let result = enc_utils.set_key(invalid_key);
        assert!(result.is_err(), "Expected error for invalid key size");
    }
}
