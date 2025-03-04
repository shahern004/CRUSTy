use std::path::PathBuf;
use rfd::FileDialog;

use crate::encryption::EncryptionKey;
use crate::gui::file_list::{FileEntry, FileOperationType, FileStatus};
use crate::gui::app_core::CrustyApp;

/// Implementation of action methods for CrustyApp
impl CrustyApp {
    /// Show a status message
    pub fn show_status(&mut self, message: &str) {
        self.status_message = Some(message.to_string());
        self.status_time = std::time::Instant::now();
    }
    
    /// Show an error message
    pub fn show_error(&mut self, message: &str) {
        self.error_message = Some(message.to_string());
        self.error_time = std::time::Instant::now();
    }
    
    /// Select files using a file dialog
    pub fn select_files(&mut self) {
        let mut dialog = FileDialog::new();
        
        if self.batch_mode {
            dialog = dialog.set_title("Select Files to Process");
        } else {
            dialog = dialog.set_title("Select File to Process");
        }
        
        if self.batch_mode {
            if let Some(files) = dialog.pick_files() {
                self.selected_files = files;
                self.show_status(&format!("Selected {} file(s)", self.selected_files.len()));
            }
        } else {
            if let Some(file) = dialog.pick_file() {
                self.selected_files = vec![file];
                self.show_status("Selected 1 file");
            }
        }
    }
    
    /// Select output directory using a file dialog
    pub fn select_output_dir(&mut self) {
        if let Some(dir) = FileDialog::new()
            .set_title("Select Output Directory")
            .pick_folder() {
            self.output_dir = Some(dir.clone());
            self.show_status(&format!("Selected output directory: {}", dir.display()));
        }
    }
    
    /// Generate a new encryption key
    pub fn generate_key(&mut self, name: &str) {
        let key = EncryptionKey::generate();
        self.current_key = Some(key.clone());
        self.saved_keys.push((name.to_string(), key));
        self.show_status(&format!("Generated new key: {}", name));
    }
    
    /// Save the current key to a file
    pub fn save_key_to_file(&mut self) {
        if let Some(key) = &self.current_key {
            if let Some(path) = FileDialog::new()
                .set_title("Save Encryption Key")
                .set_file_name("encryption_key.key")
                .save_file() {
                // Save the key to a file
                let key_base64 = key.to_base64();
                match std::fs::write(&path, key_base64) {
                    Ok(_) => self.show_status(&format!("Key saved to: {}", path.display())),
                    Err(e) => self.show_error(&format!("Failed to save key: {}", e)),
                }
            }
        } else {
            self.show_error("No key selected");
        }
    }
    
    /// Load a key from a file
    pub fn load_key_from_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .set_title("Load Encryption Key")
            .add_filter("Key Files", &["key"])
            .pick_file() {
            // Read the key from a file
            match std::fs::read_to_string(&path) {
                Ok(key_base64) => {
                    match crate::encryption::EncryptionKey::from_base64(&key_base64) {
                        Ok(key) => {
                            // Extract filename without extension as the key name
                            let name = path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("Loaded Key")
                                .to_string();
                            
                            self.current_key = Some(key.clone());
                            self.saved_keys.push((name.clone(), key));
                            self.show_status(&format!("Loaded key: {}", name));
                        },
                        Err(e) => self.show_error(&format!("Failed to load key: {}", e)),
                    }
                },
                Err(e) => self.show_error(&format!("Failed to read key file: {}", e)),
            }
        }
    }
    
    /// Add a file entry to the file list
    pub fn add_file_entry(&mut self, path: PathBuf, operation_type: FileOperationType) {
        let entry = FileEntry::new(path, operation_type);
        self.file_entries.push(entry);
    }
    
    /// Update file status
    pub fn update_file_status(&mut self, index: usize, status: FileStatus) {
        if index < self.file_entries.len() {
            self.file_entries[index].status = status;
        }
    }
    
    /// Set file progress
    pub fn set_file_progress(&mut self, index: usize, progress: f32) {
        if index < self.file_entries.len() {
            self.file_entries[index].set_progress(progress);
        }
    }
    
    /// Set file completed
    pub fn set_file_completed(&mut self, index: usize, result: String) {
        if index < self.file_entries.len() {
            self.file_entries[index].set_completed(result);
        }
    }
    
    /// Set file failed
    pub fn set_file_failed(&mut self, index: usize, error: String) {
        if index < self.file_entries.len() {
            self.file_entries[index].set_failed(error);
        }
    }
    
    /// Remove a file entry from the file list
    pub fn remove_file_entry(&mut self, index: usize) {
        if index < self.file_entries.len() {
            self.file_entries.remove(index);
        }
    }
    
    /// Clear all file entries
    pub fn clear_file_entries(&mut self) {
        self.file_entries.clear();
    }
    
    /// Show the file list in the UI
    pub fn show_file_list(&mut self, ui: &mut eframe::egui::Ui) {
        if self.file_entries.is_empty() {
            ui.label("No files in the list");
            return;
        }
        
        ui.group(|ui| {
            ui.heading("File List");
            
            let mut entry_to_remove = None;
            
            for (i, entry) in self.file_entries.iter().enumerate() {
                ui.horizontal(|ui| {
                    // File name
                    ui.label(&entry.file_name());
                    
                    // Status with color
                    ui.label(eframe::egui::RichText::new(entry.status_text())
                        .color(entry.status_color(&self.theme)));
                    
                    // Operation type
                    let op_text = match entry.operation_type {
                        FileOperationType::Encrypt => "Encrypt",
                        FileOperationType::Decrypt => "Decrypt",
                        FileOperationType::None => "",
                    };
                    if !op_text.is_empty() {
                        ui.label(op_text);
                    }
                    
                    // Result or error message
                    if let Some(result) = &entry.result {
                        ui.label(eframe::egui::RichText::new(result).color(self.theme.success));
                    } else if let Some(error) = &entry.error {
                        ui.label(eframe::egui::RichText::new(error).color(self.theme.error));
                    }
                    
                    // Remove button
                    if ui.button("❌").clicked() {
                        entry_to_remove = Some(i);
                    }
                });
            }
            
            // Handle removal outside the loop
            if let Some(index) = entry_to_remove {
                self.remove_file_entry(index);
            }
            
            // Clear all button
            if ui.button("Clear All").clicked() {
                self.clear_file_entries();
            }
        });
    }
}
