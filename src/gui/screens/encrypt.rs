use eframe::egui::{Ui, RichText, Button, Rounding, ProgressBar, ScrollArea};
use crate::gui::app_core::CrustyApp;
use crate::gui::app_state::AppState;
use crate::start_operation::FileOperation;
use crate::gui::file_list::FileOperationType;
use std::path::PathBuf;

/// Encrypt screen trait
pub trait EncryptScreen {
    fn show_encrypt_screen(&mut self, ui: &mut Ui);
}

impl EncryptScreen for CrustyApp {
    fn show_encrypt_screen(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(RichText::new("Encrypt Files").size(28.0));
            ui.add_space(10.0);
            
            // File selection section
            ui.group(|ui| {
                ui.heading("File Selection");
                
                ui.horizontal(|ui| {
                    let select_text = if self.batch_mode {
                        "Select Files"
                    } else {
                        "Select File"
                    };
                    
                    if ui.add_sized(
                        [150.0, 30.0],
                        Button::new(RichText::new(select_text).color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.select_files();
                    }
                    
                    ui.checkbox(&mut self.batch_mode, "Batch Mode");
                });
                
                ui.add_space(5.0);
                
                // Display selected files
                if self.selected_files.is_empty() {
                    ui.label("No files selected");
                } else {
                    ui.label(format!("Selected {} file(s)", self.selected_files.len()));
                    
                    ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                        for file in &self.selected_files {
                            ui.label(format!("• {}", file.file_name().unwrap_or_default().to_string_lossy()));
                        }
                    });
                }
            });
            
            ui.add_space(10.0);
            
            // Output directory selection
            ui.group(|ui| {
                ui.heading("Output Directory");
                
                if ui.add_sized(
                    [200.0, 30.0],
                    Button::new(RichText::new("Select Output Directory").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    self.select_output_dir();
                }
                
                if let Some(dir) = &self.output_dir {
                    ui.label(format!("Output directory: {}", dir.display()));
                } else {
                    ui.label("No output directory selected");
                }
            });
            
            ui.add_space(10.0);
            
            // Encryption options
            ui.group(|ui| {
                ui.heading("Encryption Options");
                
                // Key selection
                ui.horizontal(|ui| {
                    ui.label("Encryption Key:");
                    
                    if self.current_key.is_none() {
                        ui.label(RichText::new("No key selected").color(self.theme.error));
                        
                        if ui.add_sized(
                            [120.0, 24.0],
                            Button::new(RichText::new("Select Key").color(self.theme.button_text))
                                .fill(self.theme.button_normal)
                                .rounding(Rounding::same(5.0))
                        ).clicked() {
                            self.state = AppState::KeyManagement;
                        }
                    } else {
                        // Find the name of the current key
                        let key_name = self.current_key.as_ref().map_or_else(
                            || "Unknown key".to_string(),
                            |current_key| {
                                self.saved_keys.iter()
                                    .find_map(|(name, key)| {
                                        if key.to_base64() == current_key.to_base64() {
                                            Some(name.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or_else(|| "Unknown key".to_string())
                            }
                        );
                        
                        ui.label(RichText::new(&key_name).color(self.theme.success));
                        
                        if ui.add_sized(
                            [120.0, 24.0],
                            Button::new(RichText::new("Change Key").color(self.theme.button_text))
                                .fill(self.theme.button_normal)
                                .rounding(Rounding::same(5.0))
                        ).clicked() {
                            self.state = AppState::KeyManagement;
                        }
                    }
                });
                
                // Recipient options
                ui.add_space(5.0);
                ui.checkbox(&mut self.use_recipient, "Encrypt for specific recipient");
                
                if self.use_recipient {
                    ui.horizontal(|ui| {
                        ui.label("Recipient Email:");
                        ui.text_edit_singleline(&mut self.recipient_email);
                    });
                }
                
                // Backend options
                ui.add_space(5.0);
                ui.checkbox(&mut self.use_embedded_backend, "Use hardware encryption");
                
                if self.use_embedded_backend {
                    ui.horizontal(|ui| {
                        ui.label("Connection Type:");
                        ui.radio_value(&mut self.embedded_connection_type, crate::backend::ConnectionType::Usb, "USB");
                        ui.radio_value(&mut self.embedded_connection_type, crate::backend::ConnectionType::Serial, "Serial");
                    });
                }
            });
            
            ui.add_space(20.0);
            
            // Progress section (only shown during encryption)
            if matches!(self.operation, FileOperation::Encrypt) && !self.progress.lock().unwrap().is_empty() {
                ui.group(|ui| {
                    ui.heading("Encryption Progress");
                    
                    let progress = self.progress.lock().unwrap();
                    
                    // Overall progress
                    let overall_progress = if progress.is_empty() {
                        0.0
                    } else {
                        progress.iter().sum::<f32>() / progress.len() as f32
                    };
                    
                    ui.label(format!("Overall Progress: {:.1}%", overall_progress * 100.0));
                    ui.add(ProgressBar::new(overall_progress)
                        .show_percentage()
                        .animate(true));
                    
                    ui.add_space(10.0);
                    
                    // Individual file progress
                    if !self.selected_files.is_empty() && progress.len() == self.selected_files.len() {
                        ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                            for (i, (file, &prog)) in self.selected_files.iter().zip(progress.iter()).enumerate() {
                                ui.label(format!("File {}: {}", i + 1, file.file_name().unwrap_or_default().to_string_lossy()));
                                ui.add(ProgressBar::new(prog)
                                    .show_percentage()
                                    .animate(true));
                                ui.add_space(5.0);
                            }
                        });
                    }
                });
                
                ui.add_space(10.0);
                
                // Results section
                if !self.operation_results.is_empty() {
                    ui.group(|ui| {
                        ui.heading("Results");
                        
                        ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                            for result in &self.operation_results {
                                if result.contains("Error") || result.contains("Failed") {
                                    ui.label(RichText::new(result).color(self.theme.error));
                                } else {
                                    ui.label(RichText::new(result).color(self.theme.success));
                                }
                            }
                        });
                    });
                }
            }
            
            ui.add_space(20.0);
            
            // Action buttons
            ui.horizontal(|ui| {
                let can_encrypt = !self.selected_files.is_empty() && 
                                 self.output_dir.is_some() && 
                                 self.current_key.is_some();
                
                if ui.add_sized(
                    [150.0, 40.0],
                    Button::new(RichText::new("🔒 Encrypt").color(self.theme.button_text))
                        .fill(if can_encrypt { self.theme.accent } else { self.theme.button_normal })
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    if can_encrypt {
                        self.operation = FileOperation::Encrypt;
                        
                        // Add files to the file list
                        let files_to_add: Vec<PathBuf> = self.selected_files.clone();
                        for file in files_to_add {
                            self.add_file_entry(file, FileOperationType::Encrypt);
                        }
                        
                        // Start encryption
                        self.show_status("Starting encryption...");
                    } else {
                        self.show_error("Please select files, output directory, and encryption key");
                    }
                }
                
                // Back button
                if ui.add_sized(
                    [120.0, 40.0],
                    Button::new(RichText::new("Back").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    self.state = AppState::Dashboard;
                    self.operation = FileOperation::None;
                }
            });
        });
    }
}
