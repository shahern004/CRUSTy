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
use anyhow::Result;
use rand::RngCore;
use std::fs::File;
use std::io::{Read, Write, self, BufReader, BufWriter};
use std::path::Path;
use thiserror::Error;
use base64::{Engine as _, engine::general_purpose::STANDARD};

/// Custom error types for encryption operations
#[derive(Error, Debug)]
pub enum EncryptionError {
    /// Errors related to file I/O operations
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    /// Errors that occur during encryption
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    /// Errors that occur during decryption
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    /// Errors related to key management
    #[error("Key error: {0}")]
    KeyError(String),
}

/// Struct to hold and manage AES-256-GCM encryption keys
#[derive(Clone)]
pub struct EncryptionKey {
    key: Key<Aes256Gcm>,
}

impl EncryptionKey {
    /// Generate a new random encryption key
    pub fn generate() -> Self {
        let key = Aes256Gcm::generate_key(OsRng);
        EncryptionKey { key }
    }
    
    /// Convert the key to a base64 string for storage
    pub fn to_base64(&self) -> String {
        STANDARD.encode(self.key)
    }
    
    /// Create a key from a base64 string
    pub fn from_base64(encoded: &str) -> Result<Self, EncryptionError> {
        let key_bytes = STANDARD.decode(encoded)
            .map_err(|e| EncryptionError::KeyError(format!("Invalid base64 key: {}", e)))?;
            
        if key_bytes.len() != 32 {
            return Err(EncryptionError::KeyError(
                format!("Invalid key length: {}, expected 32", key_bytes.len())
            ));
        }
        
        let key = *Key::<Aes256Gcm>::from_slice(&key_bytes);
        Ok(EncryptionKey { key })
    }
    
    /// Save the key to a file
    pub fn save_to_file(&self, path: &Path) -> Result<(), EncryptionError> {
        File::create(path)
            .map_err(EncryptionError::Io)?
            .write_all(self.to_base64().as_bytes())
            .map_err(EncryptionError::Io)
    }
    
    /// Load a key from a file
    pub fn load_from_file(path: &Path) -> Result<Self, EncryptionError> {
        let mut contents = String::new();
        File::open(path)
            .map_err(EncryptionError::Io)?
            .read_to_string(&mut contents)
            .map_err(EncryptionError::Io)?;
            
        Self::from_base64(&contents)
    }
}

/// Encrypts a file using AES-256-GCM encryption
///
/// # Arguments
/// * `source_path` - Path to the file to encrypt
/// * `dest_path` - Path where the encrypted file will be saved
/// * `key` - The encryption key to use
/// * `progress_callback` - Callback function that will be called with progress updates (0.0 to 1.0)
///
/// # Returns
/// * `Ok(())` if encryption was successful
/// * `Err(EncryptionError)` if an error occurred
pub fn encrypt_file(
    source_path: &Path,
    dest_path: &Path,
    key: &EncryptionKey,
    progress_callback: impl Fn(f32) + Send + 'static,
) -> Result<(), EncryptionError> {
    // Check if the destination file already exists
    if dest_path.exists() {
        return Err(EncryptionError::Io(
            io::Error::new(io::ErrorKind::AlreadyExists, "Destination file already exists")
        ));
    }

    // Open the source file
    let source_file = File::open(source_path)
        .map_err(|e| EncryptionError::Io(e))?;
    
    // Get file size for progress reporting
    let file_size = source_file.metadata()
        .map_err(|e| EncryptionError::Io(e))?
        .len();
    
    let reader = BufReader::new(source_file);
    
    // Create the destination file
    let dest_file = File::create(dest_path)
        .map_err(|e| EncryptionError::Io(e))?;
    
    let mut writer = BufWriter::new(dest_file);
    
    // Create the cipher instance with our key
    let cipher = Aes256Gcm::new(&key.key);
    
    // Generate a random nonce (Number used ONCE)
    let mut nonce_bytes = [0u8; 12]; // AES-GCM uses 12-byte nonces
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Write the nonce to the beginning of the output file
    // This is needed for decryption later
    if let Err(e) = writer.write_all(&nonce_bytes) {
        // Delete the destination file if there's an error
        let _ = std::fs::remove_file(dest_path);
        return Err(EncryptionError::Io(e));
    }
    
    // Process the file in chunks to handle large files
    const CHUNK_SIZE: usize = 4096;
    let mut buffer = [0u8; CHUNK_SIZE];
    let mut bytes_read = 0u64;
    
    loop {
        // Read a chunk from the source file
        let n = match reader.get_ref().read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                // Delete the destination file if there's an error
                let _ = std::fs::remove_file(dest_path);
                return Err(EncryptionError::Io(e));
            }
        };
            
        if n == 0 {
            // End of file
            break;
        }
        
        // Encrypt the chunk
        let encrypted_data = match cipher.encrypt(nonce, &buffer[..n]) {
            Ok(data) => data,
            Err(e) => {
                // Delete the destination file if there's an error
                let _ = std::fs::remove_file(dest_path);
                return Err(EncryptionError::Encryption(e.to_string()));
            }
        };
            
        // Write the encrypted data
        if let Err(e) = writer.write_all(&(encrypted_data.len() as u32).to_le_bytes()) {
            // Delete the destination file if there's an error
            let _ = std::fs::remove_file(dest_path);
            return Err(EncryptionError::Io(e));
        }
            
        if let Err(e) = writer.write_all(&encrypted_data) {
            // Delete the destination file if there's an error
            let _ = std::fs::remove_file(dest_path);
            return Err(EncryptionError::Io(e));
        }
            
        // Update progress
        bytes_read += n as u64;
        progress_callback(bytes_read as f32 / file_size as f32);
    }
    
    // Ensure all data is written
    if let Err(e) = writer.flush() {
        // Delete the destination file if there's an error
        let _ = std::fs::remove_file(dest_path);
        return Err(EncryptionError::Io(e));
    }
    
    // Final progress update
    progress_callback(1.0);
    
    Ok(())
}

/// Decrypts a file that was encrypted with AES-256-GCM
///
/// # Arguments
/// * `source_path` - Path to the encrypted file
/// * `dest_path` - Path where the decrypted file will be saved
/// * `key` - The encryption key to use (must be the same key used for encryption)
/// * `progress_callback` - Callback function that will be called with progress updates (0.0 to 1.0)
///
/// # Returns
/// * `Ok(())` if decryption was successful
/// * `Err(EncryptionError)` if an error occurred
pub fn decrypt_file(
    source_path: &Path,
    dest_path: &Path,
    key: &EncryptionKey,
    progress_callback: impl Fn(f32) + Send + 'static,
) -> Result<(), EncryptionError> {
    // Check if the destination file already exists
    if dest_path.exists() {
        return Err(EncryptionError::Io(
            io::Error::new(io::ErrorKind::AlreadyExists, "Destination file already exists")
        ));
    }

    // Open the source (encrypted) file
    let mut source_file = File::open(source_path)
        .map_err(|e| EncryptionError::Io(e))?;
    
    // Get file size for progress reporting
    let file_size = source_file.metadata()
        .map_err(|e| EncryptionError::Io(e))?
        .len();
    
    // Create the destination file
    let dest_file = File::create(dest_path)
        .map_err(|e| EncryptionError::Io(e))?;
    
    let mut writer = BufWriter::new(dest_file);
    
    // Read the nonce from the beginning of the file
    let mut nonce_bytes = [0u8; 12];
    let nonce_result = source_file.read_exact(&mut nonce_bytes);
    
    if let Err(e) = nonce_result {
        // Delete the destination file if there's an error
        let _ = std::fs::remove_file(dest_path);
        return Err(EncryptionError::Io(e));
    }
    
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Create the cipher instance with our key
    let cipher = Aes256Gcm::new(&key.key);
    
    // Track progress
    let mut bytes_read = 12u64; // We've read the nonce already
    
    // Process the file in chunks
    loop {
        // Check if we've reached the end of the file
        if bytes_read >= file_size {
            break;
        }
        
        // Read the size of the next encrypted chunk
        let mut size_bytes = [0u8; 4];
        match source_file.read_exact(&mut size_bytes) {
            Ok(_) => {},
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                // Delete the destination file if there's an error
                let _ = std::fs::remove_file(dest_path);
                return Err(EncryptionError::Io(e));
            },
        }
        
        bytes_read += 4;
        let chunk_size = u32::from_le_bytes(size_bytes) as usize;
        
        // Read the encrypted chunk
        let mut encrypted_chunk = vec![0u8; chunk_size];
        match source_file.read_exact(&mut encrypted_chunk) {
            Ok(_) => {},
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // Delete the destination file if there's an error
                let _ = std::fs::remove_file(dest_path);
                return Err(EncryptionError::Decryption(
                    "Unexpected end of file".to_string()
                ));
            },
            Err(e) => {
                // Delete the destination file if there's an error
                let _ = std::fs::remove_file(dest_path);
                return Err(EncryptionError::Io(e));
            },
        }
        
        bytes_read += chunk_size as u64;
        
        // Decrypt the chunk
        let decrypted_data = match cipher.decrypt(nonce, encrypted_chunk.as_ref()) {
            Ok(data) => data,
            Err(e) => {
                // Delete the destination file if there's an error
                let _ = std::fs::remove_file(dest_path);
                
                // Provide a more specific error message for authentication failures
                if e.to_string().contains("authentication") || e.to_string().contains("tag mismatch") {
                    return Err(EncryptionError::Decryption(
                        "Authentication failed: The encryption key is incorrect or the file is corrupted".to_string()
                    ));
                } else {
                    return Err(EncryptionError::Decryption(e.to_string()));
                }
            }
        };
            
        // Write the decrypted data
        if let Err(e) = writer.write_all(&decrypted_data) {
            // Delete the destination file if there's an error
            let _ = std::fs::remove_file(dest_path);
            return Err(EncryptionError::Io(e));
        }
            
        // Update progress
        progress_callback(bytes_read as f32 / file_size as f32);
    }
    
    // Ensure all data is written
    if let Err(e) = writer.flush() {
        // Delete the destination file if there's an error
        let _ = std::fs::remove_file(dest_path);
        return Err(EncryptionError::Io(e));
    }
    
    // Final progress update
    progress_callback(1.0);
    
    Ok(())
}

/// Encrypts multiple files using AES-256-GCM encryption
///
/// # Arguments
/// * `source_paths` - Paths to the files to encrypt
/// * `dest_dir` - Directory where the encrypted files will be saved
/// * `key` - The encryption key to use
/// * `progress_callback` - Callback function that will be called with progress updates
///   The first parameter is the index of the file being processed
///   The second parameter is the progress for that file (0.0 to 1.0)
///
/// # Returns
/// * `Ok(Vec<String>)` with status messages for each file if successful
/// * `Err(EncryptionError)` if an error occurred
pub fn encrypt_files(
    source_paths: &[&Path],
    dest_dir: &Path,
    key: &EncryptionKey,
    progress_callback: impl Fn(usize, f32) + Clone + Send + 'static,
) -> Result<Vec<String>, EncryptionError> {
    let mut results = Vec::new();
    
    for (i, &source_path) in source_paths.iter().enumerate() {
        let file_name = source_path.file_name()
            .ok_or_else(|| EncryptionError::Io(
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid source path")
            ))?;
            
        let mut dest_path = dest_dir.to_path_buf();
        dest_path.push(format!("{}.encrypted", file_name.to_string_lossy()));
        
        let progress_cb = {
            let cb = progress_callback.clone();
            let idx = i;
            move |p: f32| cb(idx, p)
        };
        
        match encrypt_file(source_path, &dest_path, key, progress_cb) {
            Ok(_) => results.push(format!("Successfully encrypted: {}", source_path.display())),
            Err(e) => {
                // Ensure the destination file is removed if it exists
                let _ = std::fs::remove_file(&dest_path);
                results.push(format!("Failed to encrypt {}: {}", source_path.display(), e));
            },
        }
    }
    
    Ok(results)
}

/// Decrypts multiple files that were encrypted with AES-256-GCM
///
/// # Arguments
/// * `source_paths` - Paths to the encrypted files
/// * `dest_dir` - Directory where the decrypted files will be saved
/// * `key` - The encryption key to use (must be the same key used for encryption)
/// * `progress_callback` - Callback function that will be called with progress updates
///   The first parameter is the index of the file being processed
///   The second parameter is the progress for that file (0.0 to 1.0)
///
/// # Returns
/// * `Ok(Vec<String>)` with status messages for each file if successful
/// * `Err(EncryptionError)` if an error occurred
pub fn decrypt_files(
    source_paths: &[&Path],
    dest_dir: &Path,
    key: &EncryptionKey,
    progress_callback: impl Fn(usize, f32) + Clone + Send + 'static,
) -> Result<Vec<String>, EncryptionError> {
    let mut results = Vec::new();
    
    for (i, &source_path) in source_paths.iter().enumerate() {
        let file_stem = source_path.file_stem()
            .ok_or_else(|| EncryptionError::Io(
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid source path")
            ))?;
            
        let mut dest_path = dest_dir.to_path_buf();
        
        // If the file ends with .encrypted, strip it from the output filename
        let file_name = file_stem.to_string_lossy();
        let output_name = if file_name.ends_with(".encrypted") {
            file_name.trim_end_matches(".encrypted").to_string()
        } else {
            format!("{}.decrypted", file_name)
        };
        
        dest_path.push(output_name);
        
        let progress_cb = {
            let cb = progress_callback.clone();
            let idx = i;
            move |p: f32| cb(idx, p)
        };
        
        match decrypt_file(source_path, &dest_path, key, progress_cb) {
            Ok(_) => results.push(format!("Successfully decrypted: {}", source_path.display())),
            Err(e) => {
                // Ensure the destination file is removed if it exists
                let _ = std::fs::remove_file(&dest_path);
                
                // Provide a more specific error message for authentication failures
                let error_msg = if e.to_string().contains("Authentication failed") || 
                                  e.to_string().contains("authentication") || 
                                  e.to_string().contains("tag mismatch") {
                    format!("Failed to decrypt {}: Wrong encryption key used. Please try a different key.", source_path.display())
                } else {
                    format!("Failed to decrypt {}: {}", source_path.display(), e)
                };
                
                results.push(error_msg);
            },
        }
    }
    
    Ok(results)
}