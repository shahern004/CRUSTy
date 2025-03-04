use eframe::egui::{Ui, RichText, Button, Rounding, ProgressBar, TextEdit, ScrollArea, ComboBox};
use crate::gui::app_core::CrustyApp;
use crate::gui::app_state::{AppState, EncryptionWorkflowStep};
use crate::start_operation::FileOperation;
use crate::gui::file_list::FileOperationType;
use std::path::PathBuf;

/// Encryption workflow screen trait
pub trait EncryptionWorkflowScreen {
    fn show_encryption_workflow(&mut self, ui: &mut Ui);
    fn show_workflow_files_step(&mut self, ui: &mut Ui);
    fn show_workflow_keys_step(&mut self, ui: &mut Ui);
    fn show_workflow_options_step(&mut self, ui: &mut Ui);
    fn show_workflow_execute_step(&mut self, ui: &mut Ui);
}

impl EncryptionWorkflowScreen for CrustyApp {
    fn show_encryption_workflow(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(RichText::new("Encryption Workflow").size(28.0));
            ui.add_space(10.0);
            
            // Workflow steps indicator
            ui.horizontal(|ui| {
                for step in [
                    EncryptionWorkflowStep::Files,
                    EncryptionWorkflowStep::Keys,
                    EncryptionWorkflowStep::Options,
                    EncryptionWorkflowStep::Execute,
                ] {
                    let is_current = self.encryption_workflow_step == step;
                    let is_completed = match (&self.encryption_workflow_step, &step) {
                        (EncryptionWorkflowStep::Keys, EncryptionWorkflowStep::Files) => true,
                        (EncryptionWorkflowStep::Options, EncryptionWorkflowStep::Files) => true,
                        (EncryptionWorkflowStep::Options, EncryptionWorkflowStep::Keys) => true,
                        (EncryptionWorkflowStep::Execute, EncryptionWorkflowStep::Files) => true,
                        (EncryptionWorkflowStep::Execute, EncryptionWorkflowStep::Keys) => true,
                        (EncryptionWorkflowStep::Execute, EncryptionWorkflowStep::Options) => true,
                        _ => false,
                    };
                    
                    let text_color = if is_current {
                        self.theme.accent
                    } else if is_completed {
                        self.theme.success
                    } else {
                        self.theme.text_secondary
                    };
                    
                    let text = RichText::new(step.to_string())
                        .color(text_color)
                        .strong();
                    
                    if ui.add(Button::new(text)
                        .fill(if is_current { self.theme.background } else { self.theme.background })
                        .rounding(Rounding::same(5.0))
                    ).clicked() && is_completed {
                        self.encryption_workflow_step = step;
                    }
                    
                    if step != EncryptionWorkflowStep::Execute {
                        ui.label(RichText::new(" → ").color(self.theme.text_secondary));
                    }
                }
            });
            
            ui.add_space(20.0);
            
            // Display current step content
            match self.encryption_workflow_step {
                EncryptionWorkflowStep::Files => self.show_workflow_files_step(ui),
                EncryptionWorkflowStep::Keys => self.show_workflow_keys_step(ui),
                EncryptionWorkflowStep::Options => self.show_workflow_options_step(ui),
                EncryptionWorkflowStep::Execute => self.show_workflow_execute_step(ui),
            }
            
            ui.add_space(20.0);
            
            // Navigation buttons
            ui.horizontal(|ui| {
                // Back button
                if self.encryption_workflow_step != EncryptionWorkflowStep::Files {
                    if ui.add_sized(
                        [120.0, 40.0],
                        Button::new(RichText::new("← Previous").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.encryption_workflow_step = self.encryption_workflow_step.previous();
                    }
                }
                
                // Cancel button
                if ui.add_sized(
                    [120.0, 40.0],
                    Button::new(RichText::new("Cancel").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    self.state = AppState::Dashboard;
                    self.operation = FileOperation::None;
                }
                
                // Next/Finish button
                let (next_text, next_enabled) = match self.encryption_workflow_step {
                    EncryptionWorkflowStep::Files => (
                        "Next →",
                        !self.selected_files.is_empty() && self.output_dir.is_some()
                    ),
                    EncryptionWorkflowStep::Keys => (
                        "Next →",
                        self.current_key.is_some()
                    ),
                    EncryptionWorkflowStep::Options => (
                        "Next →",
                        true
                    ),
                    EncryptionWorkflowStep::Execute => (
                        "Finish",
                        self.encryption_workflow_complete
                    ),
                };
                
                if ui.add_sized(
                    [120.0, 40.0],
                    Button::new(RichText::new(next_text).color(self.theme.button_text))
                        .fill(if next_enabled { self.theme.accent } else { self.theme.button_normal })
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    if next_enabled {
                        if self.encryption_workflow_step == EncryptionWorkflowStep::Execute {
                            // Finish the workflow
                            self.state = AppState::Dashboard;
                            self.operation = FileOperation::None;
                        } else {
                            // Go to next step
                            self.encryption_workflow_step = self.encryption_workflow_step.next();
                        }
                    } else {
                        // Show error message based on current step
                        match self.encryption_workflow_step {
                            EncryptionWorkflowStep::Files => {
                                self.show_error("Please select files and output directory");
                            },
                            EncryptionWorkflowStep::Keys => {
                                self.show_error("Please select or create an encryption key");
                            },
                            _ => {}
                        }
                    }
                }
            });
        });
    }
    
    // Files step
    fn show_workflow_files_step(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Step 1: Select Files");
            
            ui.add_space(10.0);
            
            // File selection
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
                
                ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    for file in &self.selected_files {
                        ui.label(format!("• {}", file.file_name().unwrap_or_default().to_string_lossy()));
                    }
                });
            }
            
            ui.add_space(10.0);
            
            // Output directory selection
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
    }
    
    // Keys step
    fn show_workflow_keys_step(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Step 2: Select Encryption Key");
            
            ui.add_space(10.0);
            
            // Current key display
            if self.current_key.is_none() {
                ui.label(RichText::new("No key selected").color(self.theme.error));
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
                
                ui.horizontal(|ui| {
                    ui.label("Current Key:");
                    ui.label(RichText::new(&key_name).color(self.theme.success).strong());
                });
            }
            
            ui.add_space(10.0);
            
            // Key selection options
            ui.horizontal(|ui| {
                // Create new key
                ui.vertical(|ui| {
                    ui.heading("Create New Key");
                    
                    ui.horizontal(|ui| {
                        ui.label("Key Name:");
                        ui.add(TextEdit::singleline(&mut self.new_key_name)
                            .hint_text("Enter a name for the new key")
                            .desired_width(200.0));
                    });
                    
                    ui.add_space(5.0);
                    
                    if ui.add_sized(
                        [150.0, 30.0],
                        Button::new(RichText::new("Generate Key").color(self.theme.button_text))
                            .fill(self.theme.accent)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        if self.new_key_name.is_empty() {
                            self.show_error("Please enter a name for the key");
                        } else {
                    let key_name = self.new_key_name.clone();
                    self.generate_key(&key_name);
                            self.new_key_name.clear();
                        }
                    }
                });
                
                ui.add_space(20.0);
                
                // Select existing key
                ui.vertical(|ui| {
                    ui.heading("Select Existing Key");
                    
                    if self.saved_keys.is_empty() {
                        ui.label("No saved keys available");
                    } else {
                        // Create a temporary vector of key names for the dropdown
                        let key_names: Vec<String> = self.saved_keys.iter()
                            .map(|(name, _)| name.clone())
                            .collect();
                        
                        let current_key_name = self.current_key.as_ref().map_or_else(
                            || "Select a key".to_string(),
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
                        
                        let mut selected_key_index = None;
                        
                        ComboBox::from_label("Select Key")
                            .selected_text(&current_key_name)
                            .width(250.0)
                            .show_ui(ui, |ui| {
                                for (i, name) in key_names.iter().enumerate() {
                                    if ui.selectable_label(
                                        current_key_name == *name,
                                        name
                                    ).clicked() {
                                        selected_key_index = Some(i);
                                    }
                                }
                            });
                        
                        // Handle key selection outside the closure
                        if let Some(idx) = selected_key_index {
                            if idx < self.saved_keys.len() {
                                let (name, key) = &self.saved_keys[idx];
                                self.current_key = Some(key.clone());
                                self.show_status(&format!("Selected key: {}", name));
                            }
                        }
                    }
                    
                    ui.add_space(5.0);
                    
                    if ui.add_sized(
                        [150.0, 30.0],
                        Button::new(RichText::new("Load Key from File").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.load_key_from_file();
                    }
                });
            });
        });
    }
    
    // Options step
    fn show_workflow_options_step(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Step 3: Encryption Options");
            
            ui.add_space(10.0);
            
            // Recipient options
            ui.heading("Recipient Options");
            ui.checkbox(&mut self.use_recipient, "Encrypt for specific recipient");
            
            if self.use_recipient {
                ui.horizontal(|ui| {
                    ui.label("Recipient Email:");
                    ui.add(TextEdit::singleline(&mut self.recipient_email)
                        .hint_text("Enter recipient's email address")
                        .desired_width(250.0));
                });
                
                ui.label("The recipient will need the same key to decrypt the files.");
            }
            
            ui.add_space(10.0);
            
            // Backend options
            ui.heading("Encryption Backend");
            ui.checkbox(&mut self.use_embedded_backend, "Use hardware encryption");
            
            if self.use_embedded_backend {
                ui.horizontal(|ui| {
                    ui.label("Connection Type:");
                    ui.radio_value(&mut self.embedded_connection_type, crate::backend::ConnectionType::Usb, "USB");
                    ui.radio_value(&mut self.embedded_connection_type, crate::backend::ConnectionType::Serial, "Serial");
                });
                
                ui.horizontal(|ui| {
                    ui.label("Device ID:");
                    ui.text_edit_singleline(&mut self.embedded_device_id);
                });
                
                ui.label("Hardware encryption offloads cryptographic operations to a dedicated device.");
            } else {
                ui.label("Software encryption uses your computer's CPU for cryptographic operations.");
            }
        });
    }
    
    // Execute step
    fn show_workflow_execute_step(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Step 4: Execute Encryption");
            
            ui.add_space(10.0);
            
            // Summary
            ui.heading("Encryption Summary");
            
            ui.label(format!("Files to encrypt: {} file(s)", self.selected_files.len()));
            ui.label(format!("Output directory: {}", self.output_dir.as_ref().unwrap_or(&PathBuf::from("")).display()));
            
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
            
            ui.label(format!("Encryption key: {}", key_name));
            
            if self.use_recipient {
                ui.label(format!("Recipient: {}", self.recipient_email));
            }
            
            ui.label(format!("Backend: {}", if self.use_embedded_backend { "Hardware" } else { "Software" }));
            
            ui.add_space(20.0);
            
            // Execute button
            let can_encrypt = !self.selected_files.is_empty() && 
                             self.output_dir.is_some() && 
                             self.current_key.is_some();
            
            if !self.encryption_workflow_complete {
                if ui.add_sized(
                    [200.0, 40.0],
                    Button::new(RichText::new("🔒 Start Encryption").color(self.theme.button_text))
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
                        self.encryption_workflow_complete = true;
                    } else {
                        self.show_error("Please complete all previous steps");
                    }
                }
            } else {
                // Progress section
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
                
                ui.add_space(10.0);
                
                // Results section
                if !self.operation_results.is_empty() {
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
                }
            }
        });
    }
}
