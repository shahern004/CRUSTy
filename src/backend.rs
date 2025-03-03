/// Backend abstraction for encryption operations.
///
/// This module provides a trait-based abstraction for different encryption backends,
/// allowing the application to use either local (software-based) encryption or
/// offload encryption operations to an embedded device.
use std::path::Path;
use crate::encryption::{EncryptionKey, EncryptionError};

/// Trait defining the interface for encryption backends.
pub trait EncryptionBackend {
    /// Encrypts raw data using the provided key.
    fn encrypt_data(&self, data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError>;
    
    /// Decrypts raw data using the provided key.
    fn decrypt_data(&self, data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError>;
    
    /// Encrypts raw data for a specific recipient using their email.
    fn encrypt_data_for_recipient(
        &self, 
        data: &[u8], 
        master_key: &EncryptionKey, 
        recipient_email: &str
    ) -> Result<Vec<u8>, EncryptionError>;
    
    /// Decrypts raw data that was encrypted for a specific recipient.
    fn decrypt_data_with_recipient(
        &self, 
        data: &[u8], 
        master_key: &EncryptionKey
    ) -> Result<(String, Vec<u8>), EncryptionError>;
    
    /// Encrypts a file using the provided key.
    fn encrypt_file(
        &self,
        source_path: &Path,
        dest_path: &Path,
        key: &EncryptionKey,
        progress_callback: impl Fn(f32) + Send + 'static,
    ) -> Result<(), EncryptionError>;
    
    /// Decrypts a file using the provided key.
    fn decrypt_file(
        &self,
        source_path: &Path,
        dest_path: &Path,
        key: &EncryptionKey,
        progress_callback: impl Fn(f32) + Send + 'static,
    ) -> Result<(), EncryptionError>;
    
    /// Encrypts a file for a specific recipient using their email.
    fn encrypt_file_for_recipient(
        &self,
        source_path: &Path,
        dest_path: &Path,
        master_key: &EncryptionKey,
        recipient_email: &str,
        progress_callback: impl Fn(f32) + Send + 'static,
    ) -> Result<(), EncryptionError>;
    
    /// Decrypts a file that was encrypted for a specific recipient.
    fn decrypt_file_with_recipient(
        &self,
        source_path: &Path,
        dest_path: &Path,
        master_key: &EncryptionKey,
        progress_callback: impl Fn(f32) + Send + 'static,
    ) -> Result<(String, ()), EncryptionError>;
    
    /// Encrypts multiple files using the provided key.
    fn encrypt_files(
        &self,
        source_paths: &[&Path],
        dest_dir: &Path,
        key: &EncryptionKey,
        progress_callback: impl Fn(usize, f32) + Clone + Send + 'static,
    ) -> Result<Vec<String>, EncryptionError>;
    
    /// Decrypts multiple files using the provided key.
    fn decrypt_files(
        &self,
        source_paths: &[&Path],
        dest_dir: &Path,
        key: &EncryptionKey,
        progress_callback: impl Fn(usize, f32) + Clone + Send + 'static,
    ) -> Result<Vec<String>, EncryptionError>;
    
    /// Encrypts multiple files for a specific recipient.
    fn encrypt_files_for_recipient(
        &self,
        source_paths: &[&Path],
        dest_dir: &Path,
        master_key: &EncryptionKey,
        recipient_email: &str,
        progress_callback: impl Fn(usize, f32) + Clone + Send + 'static,
    ) -> Result<Vec<String>, EncryptionError>;
}

/// Local (software-based) implementation of the encryption backend.
pub struct LocalBackend;

/// Configuration for the embedded device backend.
pub struct EmbeddedConfig {
    /// Connection type (e.g., USB, UART, Ethernet)
    pub connection_type: ConnectionType,
    /// Device identifier or address
    pub device_id: String,
    /// Additional connection parameters
    pub parameters: std::collections::HashMap<String, String>,
}

/// Connection types for the embedded device.
pub enum ConnectionType {
    /// USB connection
    Usb,
    /// Serial/UART connection
    Serial,
    /// Ethernet/TCP connection
    Ethernet,
}

/// Embedded device implementation of the encryption backend.
pub struct EmbeddedBackend {
    /// Configuration for the embedded device connection
    config: EmbeddedConfig,
    /// Whether the backend is currently connected
    connected: bool,
}

/// Factory for creating encryption backends.
pub struct BackendFactory;

impl BackendFactory {
    /// Creates a new local (software-based) encryption backend.
    pub fn create_local() -> Box<dyn EncryptionBackend> {
        Box::new(LocalBackend)
    }
    
    /// Creates a new embedded device encryption backend with the specified configuration.
    pub fn create_embedded(config: EmbeddedConfig) -> Box<dyn EncryptionBackend> {
        Box::new(EmbeddedBackend {
            config,
            connected: false,
        })
    }
}
