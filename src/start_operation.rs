// Start the selected operation
    fn start_operation(&mut self) {
        // Reset the progress and results
        {
            let mut progress = self.progress.lock().unwrap();
            progress.clear();
            progress.resize(self.selected_files.len(), 0.0);
        }
        
        // Clear results
        self.operation_results.clear();
        {
            let mut shared_results = self.shared_results.lock().unwrap();
            shared_results.clear();
        }
        
        let key = self.current_key.clone().unwrap();
        let files: Vec<PathBuf> = self.selected_files.clone();
        let output_dir = self.output_dir.clone().unwrap();
        let progress = self.progress.clone();
        let operation = self.operation.clone();
        let shared_results = self.shared_results.clone();
        let use_recipient = self.use_recipient;
        let recipient_email = self.recipient_email.clone();
        
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
                        
                        let progress_clone = progress.clone();
                        
                        let result = if use_recipient && !recipient_email.trim().is_empty() {
                            // Use recipient-based encryption
                            encrypt_file_for_recipient(
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
                            encrypt_file(
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
                                    let result_msg = if use_recipient {
                                        format!("Successfully encrypted for {}: {}", recipient_email, file_path.display())
                                    } else {
                                        format!("Successfully encrypted: {}", file_path.display())
                                    };
                                    
                                    if let Ok(mut results) = shared_results.lock() {
                                        results.push(result_msg);
                                    }
                                },
                                Err(e) => {
                                    logger.log_error(
                                        "Encrypt",
                                        &file_path.to_string_lossy(),
                                        &e.to_string()
                                    ).ok();
                                    
                                    // Store error
                                    let error_msg = format!("Failed to encrypt {}: {}", file_path.display(), e);
                                    if let Ok(mut results) = shared_results.lock() {
                                        results.push(error_msg);
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
                        
                        let progress_clone = progress.clone();
                        
                        // Try recipient-based decryption first, fall back to standard decryption if it fails
                        let result = if use_recipient {
                            match decrypt_file_with_recipient(
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
                                Ok((email, _)) => {
                                    // Store the detected recipient email
                                    if let Ok(mut results) = shared_results.lock() {
                                        results.push(format!("Detected recipient: {}", email));
                                    }
                                    Ok(())
                                },
                                Err(e) => {
                                    // Fall back to standard decryption
                                    decrypt_file(
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
                            decrypt_file(
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
                                    let result_msg = format!("Successfully decrypted: {}", file_path.display());
                                    if let Ok(mut results) = shared_results.lock() {
                                        results.push(result_msg);
                                    }
                                },
                                Err(e) => {
                                    logger.log_error(
                                        "Decrypt",
                                        &file_path.to_string_lossy(),
                                        &e.to_string()
                                    ).ok();
                                    
                                    // Store error with specific message for wrong key
                                    let error_msg = if e.to_string().contains("authentication") || e.to_string().contains("tag mismatch") {
                                        format!("Failed to decrypt {}: Wrong encryption key used. Please try a different key.", file_path.display())
                                    } else {
                                        format!("Failed to decrypt {}: {}", file_path.display(), e)
                                    };
                                    
                                    if let Ok(mut results) = shared_results.lock() {
                                        results.push(error_msg);
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
                        encrypt_files_for_recipient(
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
                        encrypt_files(
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
                                    
                                    // Store result
                                    if let Ok(mut op_results) = shared_results.lock() {
                                        op_results.push(result.clone());
                                    }
                                } else {
                                    logger.log_error("Batch Encrypt", &file_path, result).ok();
                                    
                                    // Store error
                                    if let Ok(mut op_results) = shared_results.lock() {
                                        op_results.push(result.clone());
                                    }
                                }
                            }
                        } else if let Err(e) = &results {
                            logger.log_error(
                                "Batch Encrypt",
                                "multiple files",
                                &e.to_string()
                            ).ok();
                            
                            // Store error
                            let error_msg = format!("Batch encryption failed: {}", e);
                            if let Ok(mut op_results) = shared_results.lock() {
                                op_results.push(error_msg);
                            }
                        }
                    }
                },
                FileOperation::BatchDecrypt => {
                    let progress_clone = progress.clone();
                    
                    // Convert Vec<PathBuf> to Vec<&Path>
                    let path_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
                    
                    // For batch decryption, we always use standard decryption
                    // as we can't know which files might be recipient-encrypted
                    let results = decrypt_files(
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
                                    
                                    // Store result
                                    if let Ok(mut op_results) = shared_results.lock() {
                                        op_results.push(result.clone());
                                    }
                                } else {
                                    logger.log_error("Batch Decrypt", &file_path, result).ok();
                                    
                                    // Store error with specific message for wrong key
                                    let error_msg = if result.contains("authentication") || result.contains("tag mismatch") {
                                        format!("Failed to decrypt {}: Wrong encryption key used. Please try a different key.", file_path)
                                    } else {
                                        result.clone()
                                    };
                                    
                                    if let Ok(mut op_results) = shared_results.lock() {
                                        op_results.push(error_msg);
                                    }
                                }
                            }
                        } else if let Err(e) = &results {
                            logger.log_error(
                                "Batch Decrypt",
                                "multiple files",
                                &e.to_string()
                            ).ok();
                            
                            // Store error with specific message for wrong key
                            let error_msg = if e.to_string().contains("authentication") || e.to_string().contains("tag mismatch") {
                                format!("Batch decryption failed: Wrong encryption key used. Please try a different key.")
                            } else {
                                format!("Batch decryption failed: {}", e)
                            };
                            
                            if let Ok(mut op_results) = shared_results.lock() {
                                op_results.push(error_msg);
                            }
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
