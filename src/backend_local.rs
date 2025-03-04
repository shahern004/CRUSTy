/// Local (software-based) implementation of the encryption backend.
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write, BufReader};

use crate::backend::{EncryptionBackend, LocalBackend};
use crate::encryption::{
    EncryptionKey, EncryptionError,
    encrypt_data, decrypt_data
};

impl EncryptionBackend for LocalBackend {
    fn encrypt_data(&self, data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError> {
        encrypt_data(data, key)
    }
    
    fn decrypt_data(&self, data: &[u8], key: &EncryptionKey) -> Result<Vec<u8>, EncryptionError> {
        decrypt_data(data, key)
    }
    
    fn encrypt_file(
        &self,
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
        let source_file = File::open(source_path)
            .map_err(|e| EncryptionError::Io(e))?;
        
        // Get file size for progress reporting
        let _file_size = source_file.metadata()
            .map_err(|e| EncryptionError::Io(e))?
            .len();
        
        let mut reader = BufReader::new(source_file);
        
        // Read the entire file into memory
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)
            .map_err(|e| EncryptionError::Io(e))?;
        
        // Update progress to indicate file read is complete
        progress_callback(0.5);
        
        // Encrypt the data
        let encrypted_data = self.encrypt_data(&buffer, key)?;
        
        // Write the encrypted data to the destination file
        let mut dest_file = File::create(dest_path)
            .map_err(|e| EncryptionError::Io(e))?;
        
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
    
    fn decrypt_file(
        &self,
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
        let source_file = File::open(source_path)
            .map_err(|e| EncryptionError::Io(e))?;
        
        let mut reader = BufReader::new(source_file);
        
        // Read the entire file into memory
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)
            .map_err(|e| EncryptionError::Io(e))?;
        
        // Update progress to indicate file read is complete
        progress_callback(0.5);
        
        // Decrypt the data
        let decrypted_data = self.decrypt_data(&buffer, key)?;
        
        // Write the decrypted data to the destination file
        let mut dest_file = File::create(dest_path)
            .map_err(|e| EncryptionError::Io(e))?;
        
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
    
    
    fn encrypt_files(
        &self,
        source_paths: &[&Path],
        dest_dir: &Path,
        key: &EncryptionKey,
        progress_callback: impl Fn(usize, f32) + Clone + Send + 'static,
    ) -> Result<Vec<String>, EncryptionError> {
        let mut results = Vec::new();
        
        for (i, &source_path) in source_paths.iter().enumerate() {
            let file_name = source_path.file_name()
                .ok_or_else(|| EncryptionError::Io(
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid source path")
                ))?;
                
            let mut dest_path = dest_dir.to_path_buf();
            dest_path.push(format!("{}.encrypted", file_name.to_string_lossy()));
            
            let progress_cb = {
                let cb = progress_callback.clone();
                let idx = i;
                move |p: f32| cb(idx, p)
            };
            
            match self.encrypt_file(source_path, &dest_path, key, progress_cb) {
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
    
    fn decrypt_files(
        &self,
        source_paths: &[&Path],
        dest_dir: &Path,
        key: &EncryptionKey,
        progress_callback: impl Fn(usize, f32) + Clone + Send + 'static,
    ) -> Result<Vec<String>, EncryptionError> {
        let mut results = Vec::new();
        
        for (i, &source_path) in source_paths.iter().enumerate() {
            let file_stem = source_path.file_stem()
                .ok_or_else(|| EncryptionError::Io(
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid source path")
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
            
            match self.decrypt_file(source_path, &dest_path, key, progress_cb) {
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
    
}
