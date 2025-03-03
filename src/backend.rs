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
#[derive(Clone)]
pub struct EmbeddedConfig {
    /// Connection type (e.g., USB, UART, Ethernet)
    pub connection_type: ConnectionType,
    /// Device identifier or address
    pub device_id: String,
    /// Additional connection parameters
    pub parameters: std::collections::HashMap<String, String>,
}

/// Connection types for the embedded device.
#[derive(Debug, Clone, PartialEq)]
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
    pub config: EmbeddedConfig,
    /// Whether the backend is currently connected
    pub connected: bool,
}

/// Enum-based backend that can be either local or embedded
pub enum Backend {
    /// Local (software-based) backend
    Local(LocalBackend),
    /// Embedded device backend
    Embedded(EmbeddedBackend),
}

impl Backend {
    /// Encrypts raw data using the provided key.
    pub fn encrypt_data(&self, data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError> {
        match self {
            Backend::Local(backend) => backend.encrypt_data(data, key),
            Backend::Embedded(backend) => backend.encrypt_data(data, key),
        }
    }
    
    /// Decrypts raw data using the provided key.
    pub fn decrypt_data(&self, data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError> {
        match self {
            Backend::Local(backend) => backend.decrypt_data(data, key),
            Backend::Embedded(backend) => backend.decrypt_data(data, key),
        }
    }
    
    /// Encrypts raw data for a specific recipient using their email.
    pub fn encrypt_data_for_recipient(
        &self, 
        data: &[u8], 
        master_key: &EncryptionKey, 
        recipient_email: &str
    ) -> Result<Vec<u8>, EncryptionError> {
        match self {
            Backend::Local(backend) => backend.encrypt_data_for_recipient(data, master_key, recipient_email),
            Backend::Embedded(backend) => backend.encrypt_data_for_recipient(data, master_key, recipient_email),
        }
    }
    
    /// Decrypts raw data that was encrypted for a specific recipient.
    pub fn decrypt_data_with_recipient(
        &self, 
        data: &[u8], 
        master_key: &EncryptionKey
    ) -> Result<(String, Vec<u8>), EncryptionError> {
        match self {
            Backend::Local(backend) => backend.decrypt_data_with_recipient(data, master_key),
            Backend::Embedded(backend) => backend.decrypt_data_with_recipient(data, master_key),
        }
    }
    
    /// Encrypts a file using the provided key.
    pub fn encrypt_file<F>(
        &self,
        source_path: &Path,
        dest_path: &Path,
        key: &EncryptionKey,
        progress_callback: F,
    ) -> Result<(), EncryptionError>
    where
        F: Fn(f32) + Send + 'static,
    {
        match self {
            Backend::Local(backend) => backend.encrypt_file(source_path, dest_path, key, progress_callback),
            Backend::Embedded(backend) => backend.encrypt_file(source_path, dest_path, key, progress_callback),
        }
    }
    
    /// Decrypts a file using the provided key.
    pub fn decrypt_file<F>(
        &self,
        source_path: &Path,
        dest_path: &Path,
        key: &EncryptionKey,
        progress_callback: F,
    ) -> Result<(), EncryptionError>
    where
        F: Fn(f32) + Send + 'static,
    {
        match self {
            Backend::Local(backend) => backend.decrypt_file(source_path, dest_path, key, progress_callback),
            Backend::Embedded(backend) => backend.decrypt_file(source_path, dest_path, key, progress_callback),
        }
    }
    
    /// Encrypts a file for a specific recipient using their email.
    pub fn encrypt_file_for_recipient<F>(
        &self,
        source_path: &Path,
        dest_path: &Path,
        master_key: &EncryptionKey,
        recipient_email: &str,
        progress_callback: F,
    ) -> Result<(), EncryptionError>
    where
        F: Fn(f32) + Send + 'static,
    {
        match self {
            Backend::Local(backend) => backend.encrypt_file_for_recipient(
                source_path, dest_path, master_key, recipient_email, progress_callback
            ),
            Backend::Embedded(backend) => backend.encrypt_file_for_recipient(
                source_path, dest_path, master_key, recipient_email, progress_callback
            ),
        }
    }
    
    /// Decrypts a file that was encrypted for a specific recipient.
    pub fn decrypt_file_with_recipient<F>(
        &self,
        source_path: &Path,
        dest_path: &Path,
        master_key: &EncryptionKey,
        progress_callback: F,
    ) -> Result<(String, ()), EncryptionError>
    where
        F: Fn(f32) + Send + 'static,
    {
        match self {
            Backend::Local(backend) => backend.decrypt_file_with_recipient(
                source_path, dest_path, master_key, progress_callback
            ),
            Backend::Embedded(backend) => backend.decrypt_file_with_recipient(
                source_path, dest_path, master_key, progress_callback
            ),
        }
    }
    
    /// Encrypts multiple files using the provided key.
    pub fn encrypt_files<F>(
        &self,
        source_paths: &[&Path],
        dest_dir: &Path,
        key: &EncryptionKey,
        progress_callback: F,
    ) -> Result<Vec<String>, EncryptionError>
    where
        F: Fn(usize, f32) + Clone + Send + 'static,
    {
        match self {
            Backend::Local(backend) => backend.encrypt_files(
                source_paths, dest_dir, key, progress_callback
            ),
            Backend::Embedded(backend) => backend.encrypt_files(
                source_paths, dest_dir, key, progress_callback
            ),
        }
    }
    
    /// Decrypts multiple files using the provided key.
    pub fn decrypt_files<F>(
        &self,
        source_paths: &[&Path],
        dest_dir: &Path,
        key: &EncryptionKey,
        progress_callback: F,
    ) -> Result<Vec<String>, EncryptionError>
    where
        F: Fn(usize, f32) + Clone + Send + 'static,
    {
        match self {
            Backend::Local(backend) => backend.decrypt_files(
                source_paths, dest_dir, key, progress_callback
            ),
            Backend::Embedded(backend) => backend.decrypt_files(
                source_paths, dest_dir, key, progress_callback
            ),
        }
    }
    
    /// Encrypts multiple files for a specific recipient.
    pub fn encrypt_files_for_recipient<F>(
        &self,
        source_paths: &[&Path],
        dest_dir: &Path,
        master_key: &EncryptionKey,
        recipient_email: &str,
        progress_callback: F,
    ) -> Result<Vec<String>, EncryptionError>
    where
        F: Fn(usize, f32) + Clone + Send + 'static,
    {
        match self {
            Backend::Local(backend) => backend.encrypt_files_for_recipient(
                source_paths, dest_dir, master_key, recipient_email, progress_callback
            ),
            Backend::Embedded(backend) => backend.encrypt_files_for_recipient(
                source_paths, dest_dir, master_key, recipient_email, progress_callback
            ),
        }
    }
}

/// Factory for creating encryption backends.
pub struct BackendFactory;

impl BackendFactory {
    /// Creates a new local (software-based) encryption backend.
    pub fn create_local() -> Backend {
        Backend::Local(LocalBackend)
    }
    
    /// Creates a new embedded device encryption backend with the specified configuration.
    pub fn create_embedded(config: EmbeddedConfig) -> Backend {
        Backend::Embedded(EmbeddedBackend {
            config,
            connected: false,
        })
    }
}
