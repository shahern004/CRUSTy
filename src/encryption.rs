/// Encryption module for AES-256-GCM file encryption and decryption. 
/// 
/// This module provides functionality for:
/// - Generating and managing encryption keys
/// - Encrypting and decrypting individual files
/// - Batch processing multiple files
/// - Progress tracking during operations
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use hkdf::Hkdf;
use sha2::{Sha256, Digest};
use anyhow::Result;
use rand::RngCore;
use std::fs::File;
use std::io::{Read, Write, BufReader};
use std::path::Path;
use thiserror::Error;
use base64::{Engine as _, engine::general_purpose::STANDARD};

/// Error type for encryption operations
#[derive(Debug, Error)]
pub enum EncryptionError {
    /// Error during encryption
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    /// Error during decryption
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    /// Error with the encryption key
    #[error("Key error: {0}")]
    KeyError(String),
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Represents an AES-256-GCM encryption key
#[derive(Clone)]
pub struct EncryptionKey {
    /// The raw key bytes
    pub key: [u8; 32],
}

impl EncryptionKey {
    /// Generate a new random encryption key
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        EncryptionKey { key }
    }
    
    /// Convert the key to a Base64 string for storage
    pub fn to_base64(&self) -> String {
        STANDARD.encode(&self.key)
    }
    
    /// Create a key from a Base64 string
    pub fn from_base64(base64: &str) -> Result<Self, EncryptionError> {
        let key_bytes = STANDARD.decode(base64.as_bytes())
            .map_err(|e| EncryptionError::KeyError(format!("Invalid Base64 encoding: {}", e)))?;
            
        if key_bytes.len() != 32 {
            return Err(EncryptionError::KeyError(
                format!("Invalid key length: expected 32 bytes, got {}", key_bytes.len())
            ));
        }
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        
        Ok(EncryptionKey { key })
    }
    
    /// Derive a recipient-specific key using HKDF
    pub fn for_recipient(&self, recipient_email: &str) -> EncryptionKey {
        // Normalize the email address
        let normalized_email = recipient_email.trim().to_lowercase();
        
        // Create a salt from the email
        let mut hasher = Sha256::new();
        hasher.update(normalized_email.as_bytes());
        let salt = hasher.finalize();
        
        // Derive a new key using HKDF
        let hkdf = Hkdf::<Sha256>::new(Some(&salt), &self.key);
        let mut derived_key = [0u8; 32];
        hkdf.expand(b"CRUSTy-recipient-key", &mut derived_key)
            .expect("HKDF expansion failed");
            
        EncryptionKey { key: derived_key }
    }
}

/// Encrypt raw data using AES-256-GCM
pub fn encrypt_data(data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError> {
    // Create the cipher
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key.key));
    
    // Generate a random nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt the data
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| EncryptionError::Encryption(format!("Encryption failed: {}", e)))?;
    
    // Format: nonce (12 bytes) + ciphertext length (4 bytes) + ciphertext
    let mut result = Vec::with_capacity(12 + 4 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&(ciphertext.len() as u32).to_be_bytes());
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

/// Decrypt raw data using AES-256-GCM
pub fn decrypt_data(data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError> {
    if data.len() < 16 {
        return Err(EncryptionError::Decryption("Data too short".to_string()));
    }
    
    // Extract the nonce
    let nonce = Nonce::from_slice(&data[0..12]);
    
    // Extract the ciphertext length
    let ciphertext_len = u32::from_be_bytes([data[12], data[13], data[14], data[15]]) as usize;
    
    // Verify the data length
    if data.len() < 16 + ciphertext_len {
        return Err(EncryptionError::Decryption("Invalid data length".to_string()));
    }
    
    // Extract the ciphertext
    let ciphertext = &data[16..16 + ciphertext_len];
    
    // Create the cipher
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key.key));
    
    // Decrypt the data
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| EncryptionError::Decryption(format!("Authentication failed: {}", e)))?;
    
    Ok(plaintext)
}

/// Encrypt raw data for a specific recipient
pub fn encrypt_data_for_recipient(data: &[u8], master_key: &EncryptionKey, recipient_email: &str) -> Result<Vec<u8>, EncryptionError> {
    // Derive a recipient-specific key
    let recipient_key = master_key.for_recipient(recipient_email);
    
    // Encrypt the data with the recipient key
    let encrypted_data = encrypt_data(data, &recipient_key)?;
    
    // Format: nonce (12 bytes) + email length (2 bytes) + email + encrypted data
    let email_bytes = recipient_email.as_bytes();
    let email_len = email_bytes.len();
    
    if email_len > 65535 {
        return Err(EncryptionError::Encryption("Email address too long".to_string()));
    }
    
    let mut result = Vec::with_capacity(12 + 2 + email_len + encrypted_data.len());
    result.extend_from_slice(&encrypted_data[0..12]); // Copy the nonce
    result.extend_from_slice(&(email_len as u16).to_be_bytes());
    result.extend_from_slice(email_bytes);
    result.extend_from_slice(&encrypted_data[12..]); // Copy the rest of the encrypted data
    
    Ok(result)
}

/// Decrypt raw data that was encrypted for a specific recipient
pub fn decrypt_data_with_recipient(data: &[u8], master_key: &EncryptionKey) -> Result<(String, Vec<u8>), EncryptionError> {
    if data.len() < 14 {
        return Err(EncryptionError::Decryption("Data too short".to_string()));
    }
    
    // Extract the email length
    let email_len = u16::from_be_bytes([data[12], data[13]]) as usize;
    
    // Verify the data length
    if data.len() < 14 + email_len {
        return Err(EncryptionError::Decryption("Invalid data length".to_string()));
    }
    
    // Extract the email
    let email_bytes = &data[14..14 + email_len];
    let recipient_email = String::from_utf8(email_bytes.to_vec())
        .map_err(|e| EncryptionError::Decryption(format!("Invalid email encoding: {}", e)))?;
    
    // Derive the recipient-specific key
    let recipient_key = master_key.for_recipient(&recipient_email);
    
    // Reconstruct the encrypted data format
    let mut encrypted_data = Vec::with_capacity(data.len() - 2 - email_len);
    encrypted_data.extend_from_slice(&data[0..12]); // Copy the nonce
    encrypted_data.extend_from_slice(&data[14 + email_len..]); // Skip the email length and email
    
    // Decrypt the data
    let plaintext = decrypt_data(&encrypted_data, &recipient_key)?;
    
    Ok((recipient_email, plaintext))
}

/// Encrypt a file using AES-256-GCM
pub fn encrypt_file(
    source_path: &Path,
    dest_path: &Path,
    key: &EncryptionKey,
    progress_callback: impl Fn(f32) + Send + 'static,
) -> Result<(), EncryptionError> {
    // Check if the destination file already exists
    if dest_path.exists() {
        return Err(EncryptionError::Io(
            std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Destination file already exists")
        ));
    }

    // Open the source file
    let source_file = File::open(source_path)?;
    
    // Get file metadata for progress reporting
    let _file_size = source_file.metadata()?.len();
    
    let mut reader = BufReader::new(source_file);
    
    // Read the entire file into memory
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    
    // Update progress to indicate file read is complete
    progress_callback(0.5);
    
    // Encrypt the data
    let encrypted_data = encrypt_data(&buffer, key)?;
    
    // Write the encrypted data to the destination file
    let mut dest_file = File::create(dest_path)?;
    
    dest_file.write_all(&encrypted_data)
        .map_err(|e| {
            // Delete the destination file if there's an error
            let _ = std::fs::remove_file(dest_path);
            EncryptionError::Io(e)
        })?;
    
    // Final progress update
    progress_callback(1.0);
    
    Ok(())
}

/// Decrypt a file using AES-256-GCM
pub fn decrypt_file(
    source_path: &Path,
    dest_path: &Path,
    key: &EncryptionKey,
    progress_callback: impl Fn(f32) + Send + 'static,
) -> Result<(), EncryptionError> {
    // Check if the destination file already exists
    if dest_path.exists() {
        return Err(EncryptionError::Io(
            std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Destination file already exists")
        ));
    }

    // Open the source file
    let source_file = File::open(source_path)?;
    
    let mut reader = BufReader::new(source_file);
    
    // Read the entire file into memory
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    
    // Update progress to indicate file read is complete
    progress_callback(0.5);
    
    // Decrypt the data
    let decrypted_data = decrypt_data(&buffer, key)?;
    
    // Write the decrypted data to the destination file
    let mut dest_file = File::create(dest_path)?;
    
    dest_file.write_all(&decrypted_data)
        .map_err(|e| {
            // Delete the destination file if there's an error
            let _ = std::fs::remove_file(dest_path);
            EncryptionError::Io(e)
        })?;
    
    // Final progress update
    progress_callback(1.0);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    // Test helper functions
    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    // Key generation tests
    #[test]
    fn test_key_generation() {
        let key = EncryptionKey::generate();
        assert_eq!(key.key.len(), 32);
    }

    #[test]
    fn test_key_serialization() {
        let key = EncryptionKey::generate();
        let base64 = key.to_base64();
        let restored = EncryptionKey::from_base64(&base64).unwrap();
        assert_eq!(key.key, restored.key);
    }

    // Basic encryption/decryption tests
    #[test]
    fn test_encrypt_decrypt_data() {
        let key = EncryptionKey::generate();
        let plaintext = b"CRUSTy secret message";
        
        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_decrypt_invalid_key() {
        let key1 = EncryptionKey::generate();
        let key2 = EncryptionKey::generate();
        let plaintext = b"CRUSTy secret message";
        
        let encrypted = encrypt_data(plaintext, &key1).unwrap();
        let result = decrypt_data(&encrypted, &key2);
        
        assert!(matches!(result, Err(EncryptionError::Decryption(_)))); 
    }

    // File encryption tests
    #[test]
    fn test_file_encryption() {
        let key = EncryptionKey::generate();
        let plain_file = create_test_file("Test file contents");
        let encrypted_file = NamedTempFile::new().unwrap();
        let decrypted_file = NamedTempFile::new().unwrap();

        encrypt_file(plain_file.path(), encrypted_file.path(), &key, |_| {}).unwrap();
        decrypt_file(encrypted_file.path(), decrypted_file.path(), &key, |_| {}).unwrap();

        let mut decrypted = String::new();
        File::open(decrypted_file.path()).unwrap()
            .read_to_string(&mut decrypted).unwrap();
            
        assert_eq!(decrypted, "Test file contents");
    }

    // Recipient-specific encryption tests
    #[test]
    fn test_recipient_encryption() {
        let master_key = EncryptionKey::generate();
        let recipient = "test@crusty.com";
        let plaintext = b"Secret for recipient";
        
        let encrypted = encrypt_data_for_recipient(plaintext, &master_key, recipient).unwrap();
        let (decrypted_recipient, decrypted) = decrypt_data_with_recipient(&encrypted, &master_key).unwrap();
        
        assert_eq!(recipient, decrypted_recipient);
        assert_eq!(plaintext, decrypted.as_slice());
    }

    // Error condition tests
    #[test]
    fn test_invalid_base64_key() {
        let result = EncryptionKey::from_base64("invalid base64");
        assert!(matches!(result, Err(EncryptionError::KeyError(_)))); 
    }

    #[test]
    fn test_corrupted_ciphertext() {
        let key = EncryptionKey::generate();
        let mut corrupted = encrypt_data(b"test", &key).unwrap();
        corrupted[10] ^= 0xFF; // Flip a bit
        
        let result = decrypt_data(&corrupted, &key);
        assert!(matches!(result, Err(EncryptionError::Decryption(_)))); 
    }
}
