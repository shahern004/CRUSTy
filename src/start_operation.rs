use std::path::{Path, PathBuf};
use std::thread;

use crate::backend::BackendFactory;
use crate::gui::CrustyApp;
use crate::logger::get_logger;

/// Enum for file operations
#[derive(Clone)]
pub enum FileOperation {
    None,
    Encrypt,
    Decrypt,
    BatchEncrypt,
    BatchDecrypt,
}

/// Start the selected operation using the appropriate backend
pub fn start_operation(app: &mut CrustyApp) {
        // Reset the progress and results
        {
            let mut progress = app.progress.lock().unwrap();
            progress.clear();
            progress.resize(app.selected_files.len(), 0.0);
        }
        
        // Clear results
        app.operation_results.clear();
        
        let key = app.current_key.clone().unwrap();
        let files: Vec<PathBuf> = app.selected_files.clone();
        let output_dir = app.output_dir.clone().unwrap();
        let progress = app.progress.clone();
        let operation = app.operation.clone();
        let use_recipient = app.use_recipient;
        let recipient_email = app.recipient_email.clone();
        
        // Create the appropriate backend
        let backend = if app.use_embedded_backend {
            // Use embedded backend with connection type and device ID
            let config = crate::backend::EmbeddedConfig {
                connection_type: app.embedded_connection_type.clone(),
                device_id: app.embedded_device_id.clone(),
                parameters: std::collections::HashMap::new(),
            };
            BackendFactory::create_embedded(config)
        } else {
            // Use local backend by default
            BackendFactory::create_local()
        };
        
        // Start an async operation based on selected operation type
        thread::spawn(move || {
            match operation {
                FileOperation::Encrypt => {
                    if let Some(file_path) = files.first() {
                        let file_path = file_path.clone(); // Clone the PathBuf
                        
                        let file_name = file_path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                            
                        let mut output_path = output_dir.clone();
                        output_path.push(format!("{}.encrypted", file_name));
                        
                        let result = if use_recipient && !recipient_email.trim().is_empty() {
                            // Use recipient-based encryption
                            let progress_clone = progress.clone();
                            backend.encrypt_file_for_recipient(
                                &file_path,
                                &output_path,
                                &key,
                                &recipient_email,
                                move |p| {
                                    let mut guard = progress_clone.lock().unwrap();
                                    if !guard.is_empty() {
                                        guard[0] = p;
                                    }
                                }
                            )
                        } else {
                            // Use standard encryption
                            let progress_clone = progress.clone();
                            backend.encrypt_file(
                                &file_path,
                                &output_path,
                                &key,
                                move |p| {
                                    let mut guard = progress_clone.lock().unwrap();
                                    if !guard.is_empty() {
                                        guard[0] = p;
                                    }
                                }
                            )
                        };
                            
                        // Log the result
                        if let Some(logger) = get_logger() {
                            match &result {
                                Ok(_) => {
                                    let operation_name = if use_recipient {
                                        format!("Encrypt for {}", recipient_email)
                                    } else {
                                        "Encrypt".to_string()
                                    };
                                    
                                    logger.log_success(
                                        &operation_name,
                                        &file_path.to_string_lossy(),
                                        "Encryption successful"
                                    ).ok();
                                    
                                    // Store result
                                    let _result_msg = if use_recipient {
                                        format!("Successfully encrypted for {}: {}", recipient_email, file_path.display())
                                    } else {
                                        format!("Successfully encrypted: {}", file_path.display())
                                    };
                                    
                                    // Add to operation_results in the next UI update
                                    let mut guard = progress.lock().unwrap();
                                    if !guard.is_empty() {
                                        guard[0] = 1.0; // Mark as complete
                                    }
                                },
                                Err(e) => {
                                    let error_str = e.to_string();
                                    logger.log_error(
                                        "Encrypt",
                                        &file_path.to_string_lossy(),
                                        &error_str
                                    ).ok();
                                    
                                    // Store error
                                    let _error_msg = format!("Failed to encrypt {}: {}", file_path.display(), error_str);
                                    
                                    // Add to operation_results in the next UI update
                                    let mut guard = progress.lock().unwrap();
                                    if !guard.is_empty() {
                                        guard[0] = 1.0; // Mark as complete
                                    }
                                }
                            }
                        }
                    }
                },
                FileOperation::Decrypt => {
                    if let Some(file_path) = files.first() {
                        let file_name = file_path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                            
                        let file_stem = file_name.to_string();
                        let output_name = if file_stem.ends_with(".encrypted") {
                            file_stem.trim_end_matches(".encrypted").to_string()
                        } else {
                            format!("{}.decrypted", file_stem)
                        };
                        
                        let mut output_path = output_dir.clone();
                        output_path.push(output_name);
                        
                        // Try recipient-based decryption first, fall back to standard decryption if it fails
                        let result = if use_recipient {
                            let progress_clone = progress.clone();
                            match backend.decrypt_file_with_recipient(
                                file_path,
                                &output_path,
                                &key,
                                move |p| {
                                    let mut guard = progress_clone.lock().unwrap();
                                    if !guard.is_empty() {
                                        guard[0] = p;
                                    }
                                }
                            ) {
                                Ok((_email, _)) => {
                                    // Store the detected recipient email
                                    // Add to operation_results in the next UI update
                                    let mut guard = progress.lock().unwrap();
                                    if !guard.is_empty() {
                                        guard[0] = 1.0; // Mark as complete
                                    }
                                    Ok(())
                                },
                                Err(_e) => {
                                    // Fall back to standard decryption
                                    let progress_clone = progress.clone();
                                    backend.decrypt_file(
                                        file_path,
                                        &output_path,
                                        &key,
                                        move |p| {
                                            let mut guard = progress_clone.lock().unwrap();
                                            if !guard.is_empty() {
                                                guard[0] = p;
                                            }
                                        }
                                    )
                                }
                            }
                        } else {
                            // Use standard decryption
                            let progress_clone = progress.clone();
                            backend.decrypt_file(
                                file_path,
                                &output_path,
                                &key,
                                move |p| {
                                    let mut guard = progress_clone.lock().unwrap();
                                    if !guard.is_empty() {
                                        guard[0] = p;
                                    }
                                }
                            )
                        };
                        
                        // Log the result
                        if let Some(logger) = get_logger() {
                            match &result {
                                Ok(_) => {
                                    logger.log_success(
                                        "Decrypt",
                                        &file_path.to_string_lossy(),
                                        "Decryption successful"
                                    ).ok();
                                    
                                    // Store result
                                    let _result_msg = format!("Successfully decrypted: {}", file_path.display());
                                    
                                    // Add to operation_results in the next UI update
                                    let mut guard = progress.lock().unwrap();
                                    if !guard.is_empty() {
                                        guard[0] = 1.0; // Mark as complete
                                    }
                                },
                                Err(e) => {
                                    let error_str = e.to_string();
                                    logger.log_error(
                                        "Decrypt",
                                        &file_path.to_string_lossy(),
                                        &error_str
                                    ).ok();
                                    
                                    // Store error with specific message for wrong key
                                    let _error_msg = if error_str.contains("authentication") || error_str.contains("tag mismatch") {
                                        format!("Failed to decrypt {}: Wrong encryption key used. Please try a different key.", file_path.display())
                                    } else {
                                        format!("Failed to decrypt {}: {}", file_path.display(), error_str)
                                    };
                                    
                                    // Add to operation_results in the next UI update
                                    let mut guard = progress.lock().unwrap();
                                    if !guard.is_empty() {
                                        guard[0] = 1.0; // Mark as complete
                                    }
                                }
                            }
                        }
                    }
                },
                FileOperation::BatchEncrypt => {
                    let progress_clone = progress.clone();
                    
                    // Convert Vec<PathBuf> to Vec<&Path>
                    let path_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
                    
                    let results = if use_recipient && !recipient_email.trim().is_empty() {
                        // Use recipient-based batch encryption
                        backend.encrypt_files_for_recipient(
                            &path_refs,
                            &output_dir,
                            &key,
                            &recipient_email,
                            move |idx, p| {
                                let mut guard = progress_clone.lock().unwrap();
                                if idx < guard.len() {
                                    guard[idx] = p;
                                }
                            }
                        )
                    } else {
                        // Use standard batch encryption
                        backend.encrypt_files(
                            &path_refs,
                            &output_dir,
                            &key,
                            move |idx, p| {
                                let mut guard = progress_clone.lock().unwrap();
                                if idx < guard.len() {
                                    guard[idx] = p;
                                }
                            }
                        )
                    };
                
                    // Log the results
                    if let Some(logger) = get_logger() {
                        if let Ok(results) = &results {
                            for (i, result) in results.iter().enumerate() {
                                let file_path = if i < files.len() {
                                    files[i].to_string_lossy().to_string()
                                } else {
                                    "Unknown file".to_string()
                                };
                                
                                if result.contains("Successfully") {
                                    let operation_name = if use_recipient {
                                        format!("Batch Encrypt for {}", recipient_email)
                                    } else {
                                        "Batch Encrypt".to_string()
                                    };
                                    
                                    logger.log_success(&operation_name, &file_path, result).ok();
                                } else {
                                    logger.log_error("Batch Encrypt", &file_path, result).ok();
                                }
                            }
                        } else if let Err(e) = &results {
                            let error_str = e.to_string();
                            logger.log_error(
                                "Batch Encrypt",
                                "multiple files",
                                &error_str
                            ).ok();
                        }
                    }
                },
                FileOperation::BatchDecrypt => {
                    let progress_clone = progress.clone();
                    
                    // Convert Vec<PathBuf> to Vec<&Path>
                    let path_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
                    
                    // For batch decryption, we always use standard decryption
                    // as we can't know which files might be recipient-encrypted
                    let results = backend.decrypt_files(
                        &path_refs,
                        &output_dir,
                        &key,
                        move |idx, p| {
                            let mut guard = progress_clone.lock().unwrap();
                            if idx < guard.len() {
                                guard[idx] = p;
                            }
                        }
                    );
                    
                    // Log the results
                    if let Some(logger) = get_logger() {
                        if let Ok(results) = &results {
                            for (i, result) in results.iter().enumerate() {
                                let file_path = if i < files.len() {
                                    files[i].to_string_lossy().to_string()
                                } else {
                                    "Unknown file".to_string()
                                };
                                
                                if result.contains("Successfully") {
                                    logger.log_success("Batch Decrypt", &file_path, result).ok();
                                } else {
                                    logger.log_error("Batch Decrypt", &file_path, result).ok();
                                }
                            }
                        } else if let Err(e) = &results {
                            let error_str = e.to_string();
                            logger.log_error(
                                "Batch Decrypt",
                                "multiple files",
                                &error_str
                            ).ok();
                        }
                    }
                },
                _ => {}
            }
            
            // Set all progress values to 1.0 to indicate completion
            {
                let mut guard = progress.lock().unwrap();
                for p in guard.iter_mut() {
                    *p = 1.0;
                }
            }
            
            // Wait a moment before clearing progress
            thread::sleep(std::time::Duration::from_millis(1500));
            
            // Clear the progress to signal completion
            let mut guard = progress.lock().unwrap();
            guard.clear();
        });
}
