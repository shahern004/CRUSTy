use eframe::egui;
use egui::{Ui, Color32, Button, RichText, Stroke, Rounding};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::encryption::{EncryptionKey, encrypt_file, decrypt_file, encrypt_files, decrypt_files};
use crate::logger::get_logger;

// Define color theme for the application
struct AppTheme {
    background: Color32,
    accent: Color32,
    text_primary: Color32,
    text_secondary: Color32,
    button_text: Color32,  // New color for button text
    button_normal: Color32,
    button_hovered: Color32,
    button_active: Color32,
    error: Color32,
    success: Color32,
}

impl Default for AppTheme {
    fn default() -> Self {
        AppTheme {
            background: Color32::from_rgb(248, 248, 248), // Off-white background
            accent: Color32::from_rgb(220, 50, 50),       // Red accent
            text_primary: Color32::from_rgb(20, 20, 20),  // Near black text
            text_secondary: Color32::from_rgb(100, 100, 100), // Gray text
            button_text: Color32::from_rgb(240, 240, 255), // Light text for buttons that's easier to read
            button_normal: Color32::from_rgb(65, 105, 225), // Royal blue buttons
            button_hovered: Color32::from_rgb(100, 149, 237), // Cornflower blue when hovered
            button_active: Color32::from_rgb(25, 25, 112), // Midnight blue when clicked
            error: Color32::from_rgb(220, 50, 50),        // Red for errors
            success: Color32::from_rgb(50, 180, 50),      // Green for success
        }
    }
}

// Application state enum for different screens
enum AppState {
    Main,
    Encrypting,
    Decrypting,
    KeyManagement,
    ViewLogs,
    About,
}

// Enum for file operations
#[derive(Clone)]
enum FileOperation {
    None,
    Encrypt,
    Decrypt,
    BatchEncrypt,
    BatchDecrypt,
}

// Application structure
pub struct CrustyApp {
    state: AppState,
    theme: AppTheme,
    
    // File paths
    selected_files: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
    
    // Key management
    current_key: Option<EncryptionKey>,
    key_path: Option<PathBuf>,
    saved_keys: Vec<(String, EncryptionKey)>, // (key_name, key)
    new_key_name: String,
    
    // Progress tracking
    operation: FileOperation,
    progress: Arc<Mutex<Vec<f32>>>,
    operation_results: Vec<String>,
    shared_results: Arc<Mutex<Vec<String>>>, // Shared results for thread communication
    
    // Status and errors
    status_message: String,
    error_message: String,
    
    // Flag for batch operation
    batch_mode: bool,
}

impl Default for CrustyApp {
    fn default() -> Self {
        CrustyApp {
            state: AppState::Main,
            theme: AppTheme::default(),
            
            selected_files: Vec::new(),
            output_dir: None,
            
            current_key: None,
            key_path: None,
            saved_keys: Vec::new(),
            new_key_name: String::new(),
            
            operation: FileOperation::None,
            progress: Arc::new(Mutex::new(Vec::new())),
            operation_results: Vec::new(),
            shared_results: Arc::new(Mutex::new(Vec::new())),
            
            status_message: "Welcome to CRUSTy".to_string(),
            error_message: String::new(),
            
            batch_mode: false,
        }
    }
}

impl eframe::App for CrustyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update operation results from shared results
        self.update_operation_results();
        
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = self.theme.background;
        style.visuals.widgets.noninteractive.bg_fill = self.theme.background;
        style.visuals.widgets.inactive.bg_fill = self.theme.button_normal;
        style.visuals.widgets.hovered.bg_fill = self.theme.button_hovered;
        style.visuals.widgets.active.bg_fill = self.theme.button_active;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.theme.button_text);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.theme.button_text);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.theme.button_text);
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(self.theme.text_primary);
            
            // App header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("CRUSTy").color(self.theme.accent).size(24.0));
                ui.add_space(20.0);
                
                if ui.add(Button::new(RichText::new("🔑 Keys").color(self.theme.button_text))
                    .fill(self.theme.button_normal)
                    .rounding(Rounding::same(5.0))
                ).clicked() {
                    self.state = AppState::KeyManagement;
                }
                
                if ui.add(Button::new(RichText::new("📋 Logs").color(self.theme.button_text))
                    .fill(self.theme.button_normal)
                    .rounding(Rounding::same(5.0))
                ).clicked() {
                    self.state = AppState::ViewLogs;
                }
                
                if ui.add(Button::new(RichText::new("ℹ️ About").color(self.theme.button_text))
                    .fill(self.theme.button_normal)
                    .rounding(Rounding::same(5.0))
                ).clicked() {
                    self.state = AppState::About;
                }
                
                if !matches!(self.state, AppState::Main) {
                    if ui.add(Button::new(RichText::new("🏠 Home").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(5.0))
                    ).clicked() {
                        self.state = AppState::Main;
                        self.operation = FileOperation::None;
                    }
                }
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Display appropriate screen based on state
            match self.state {
                AppState::Main => self.show_main_screen(ui),
                AppState::Encrypting => self.show_encrypt_screen(ui),
                AppState::Decrypting => self.show_decrypt_screen(ui),
                AppState::KeyManagement => self.show_key_management(ui),
                AppState::ViewLogs => self.show_logs(ui),
                AppState::About => self.show_about(ui),
            }
            
            // Status bar at the bottom
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                if !self.error_message.is_empty() {
                    ui.label(RichText::new(&self.error_message).color(self.theme.error));
                } else {
                    ui.label(RichText::new(&self.status_message).color(self.theme.text_secondary));
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.current_key.is_some() {
                        ui.label(RichText::new("🔑 Key loaded").color(self.theme.success));
                    } else {
                        ui.label(RichText::new("⚠️ No key loaded").color(self.theme.error));
                    }
                });
            });
        });
        
        // Request repaint if operation is in progress
        if !self.progress.lock().unwrap().is_empty() {
            ctx.request_repaint();
        }
    }
}

impl CrustyApp {
    // Helper method to display error messages
    fn show_error(&mut self, message: &str) {
        self.error_message = message.to_string();
        // Log the error if possible
        if let Some(logger) = get_logger() {
            logger.log_error(
                "GUI",
                "application",
                &message
            ).ok();
        }
    }
    
    // Helper method to display status messages
    fn show_status(&mut self, message: &str) {
        self.status_message = message.to_string();
        self.error_message.clear();
    }
    
    // Main screen UI
    fn show_main_screen(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            
            // Operation mode selection
            ui.group(|ui| {
                ui.heading("Operation Mode");
                
                ui.horizontal(|ui| {
                    ui.label("Processing Mode:");
                    ui.radio_value(&mut self.batch_mode, false, "Single File");
                    ui.radio_value(&mut self.batch_mode, true, "Multiple Files");
                });
                
                ui.add_space(10.0);
                
                // File selection section
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
                    
                    if ui.add_sized(
                        [150.0, 30.0], 
                        Button::new(RichText::new("Select Output Directory").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.select_output_dir();
                    }
                });
            });
            
            ui.add_space(10.0);
            
            // Display selected files
            if !self.selected_files.is_empty() {
                ui.group(|ui| {
                    ui.heading("Selected Files");
                    
                    let mut file_to_remove = None;
                    
                    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        for (i, file) in self.selected_files.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}. {}", i + 1, file.file_name().unwrap_or_default().to_string_lossy()));
                                
                                if ui.add(Button::new(RichText::new("❌").color(self.theme.button_text))
                                    .fill(self.theme.error)
                                    .rounding(Rounding::same(5.0))
                                ).clicked() {
                                    file_to_remove = Some(i);
                                }
                            });
                        }
                    });
                    
                    // Handle file removal outside the closure
                    if let Some(idx) = file_to_remove {
                        self.selected_files.remove(idx);
                        if self.selected_files.is_empty() {
                            self.show_status("All files removed");
                        } else {
                            self.show_status(&format!("Removed file, {} remaining", self.selected_files.len()));
                        }
                    }
                    
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label(format!("Total: {} file(s)", self.selected_files.len()));
                        
                        if ui.add(Button::new(RichText::new("Clear All").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(5.0))
                        ).clicked() {
                            self.selected_files.clear();
                            self.show_status("All files cleared");
                        }
                    });
                });
            }
            
            // Display output directory
            if let Some(dir) = &self.output_dir {
                ui.group(|ui| {
                    ui.heading("Output Directory");
                    ui.label(format!("{}", dir.display()));
                });
            }
            
            ui.add_space(20.0);
            
            // Key selection - IMPROVED VERSION
            ui.group(|ui| {
                ui.heading("Encryption Key");
                
                if self.saved_keys.is_empty() {
                    ui.label("No keys available. Please create a key.");
                    
                    if ui.add_sized(
                        [150.0, 30.0],
                        Button::new(RichText::new("Create New Key").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        // Show a popup for key creation instead of navigating away
                        self.new_key_name = "New Key".to_string();
                        let key = EncryptionKey::generate();
                        self.saved_keys.push((self.new_key_name.clone(), key.clone()));
                        self.current_key = Some(key);
                        self.show_status(&format!("New key '{}' generated and selected", self.new_key_name));
                        self.new_key_name.clear();
                    }
                    
                    if ui.add_sized(
                        [150.0, 30.0],
                        Button::new(RichText::new("Load Key from File").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.load_key_from_file();
                    }
                } else {
                    // Display current key with visual emphasis
                    let current_key_name = self.current_key.as_ref().map_or_else(
                        || "No key selected".to_string(),
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
                    
                    // Display current key with visual emphasis
                    ui.horizontal(|ui| {
                        ui.label("Current Key:");
                        ui.add_sized(
                            [200.0, 24.0],
                            egui::Label::new(
                                RichText::new(&current_key_name)
                                    .color(if self.current_key.is_some() { self.theme.success } else { self.theme.error })
                                    .strong()
                            )
                        );
                    });
                    
                    ui.add_space(5.0);
                    
                    // Dropdown for key selection - fixed to avoid borrowing issues
                    let mut selected_key_index = None;
                    
                    // Create a temporary vector of key names for the dropdown
                    let key_names: Vec<String> = self.saved_keys.iter()
                        .map(|(name, _)| name.clone())
                        .collect();
                    
                    egui::ComboBox::from_label("Select Key")
                        .selected_text(&current_key_name)
                        .width(250.0)
                        .show_ui(ui, |ui| {
                            for (i, name) in key_names.iter().enumerate() {
                                let is_selected = name == &current_key_name;
                                
                                if ui.selectable_label(is_selected, name).clicked() {
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
                    
                    ui.add_space(5.0);
                    
                    // Key management buttons
                    ui.horizontal(|ui| {
                        if ui.add_sized(
                            [120.0, 30.0],
                            Button::new(RichText::new("Create New Key").color(self.theme.button_text))
                                .fill(self.theme.button_normal)
                                .rounding(Rounding::same(8.0))
                        ).clicked() {
                            // Create a new key with a default name
                            let mut unique_name = "New Key".to_string();
                            let mut counter = 1;
                            
                            while self.saved_keys.iter().any(|(name, _)| name == &unique_name) {
                                unique_name = format!("New Key {}", counter);
                                counter += 1;
                            }
                            
                            let key = EncryptionKey::generate();
                            self.saved_keys.push((unique_name.clone(), key.clone()));
                            self.current_key = Some(key);
                            self.show_status(&format!("New key '{}' generated and selected", unique_name));
                        }
                        
                        if ui.add_sized(
                            [120.0, 30.0],
                            Button::new(RichText::new("Load from File").color(self.theme.button_text))
                                .fill(self.theme.button_normal)
                                .rounding(Rounding::same(8.0))
                        ).clicked() {
                            self.load_key_from_file();
                        }
                        
                        if ui.add_sized(
                            [120.0, 30.0],
                            Button::new(RichText::new("Advanced...").color(self.theme.button_text))
                                .fill(self.theme.button_normal)
                                .rounding(Rounding::same(8.0))
                        ).clicked() {
                            self.state = AppState::KeyManagement;
                        }
                    });
                }
            });
            
            ui.add_space(20.0);
            
            // Operation buttons
            if !self.selected_files.is_empty() && self.output_dir.is_some() && self.current_key.is_some() {
                ui.group(|ui| {
                    ui.heading("Start Operation");
                    
                    ui.horizontal(|ui| {
                        if ui.add_sized(
                            [140.0, 50.0],
                            Button::new(RichText::new("Encrypt").color(self.theme.button_text).size(18.0))
                                .fill(self.theme.button_normal)
                                .rounding(Rounding::same(8.0))
                        ).clicked() {
                            self.operation = if self.batch_mode {
                                FileOperation::BatchEncrypt
                            } else {
                                FileOperation::Encrypt
                            };
                            self.start_operation();
                            self.state = AppState::Encrypting;
                        }
                        
                        if ui.add_sized(
                            [140.0, 50.0],
                            Button::new(RichText::new("Decrypt").color(self.theme.button_text).size(18.0))
                                .fill(self.theme.button_normal)
                                .rounding(Rounding::same(8.0))
                        ).clicked() {
                            self.operation = if self.batch_mode {
                                FileOperation::BatchDecrypt
                            } else {
                                FileOperation::Decrypt
                            };
                            self.start_operation();
                            self.state = AppState::Decrypting;
                        }
                    });
                });
            } else {
                // Show what's missing
                ui.group(|ui| {
                    ui.heading("Required to Start");
                    
                    if self.selected_files.is_empty() {
                        ui.label(RichText::new("• Select at least one file").color(self.theme.error));
                    } else {
                        ui.label(RichText::new("✓ Files selected").color(self.theme.success));
                    }
                    
                    if self.output_dir.is_none() {
                        ui.label(RichText::new("• Select an output directory").color(self.theme.error));
                    } else {
                        ui.label(RichText::new("✓ Output directory selected").color(self.theme.success));
                    }
                    
                    if self.current_key.is_none() {
                        ui.label(RichText::new("• Select or create an encryption key").color(self.theme.error));
                    } else {
                        ui.label(RichText::new("✓ Encryption key selected").color(self.theme.success));
                    }
                });
            }
        });
    }
    
    // Encryption screen UI
    fn show_encrypt_screen(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading("Encrypting Files");
            ui.add_space(20.0);
            
            match self.operation {
                FileOperation::Encrypt | FileOperation::BatchEncrypt => {
                    // Show progress for encryption operation
                    self.show_operation_progress(ui);
                },
                _ => {
                    ui.label("No encryption operation in progress");
                    if ui.add(Button::new(RichText::new("Back").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(5.0))
                    ).clicked() {
                        self.state = AppState::Main;
                    }
                }
            }
        });
    }
    
    // Decryption screen UI
    fn show_decrypt_screen(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading("Decrypting Files");
            ui.add_space(20.0);
            
            match self.operation {
                FileOperation::Decrypt | FileOperation::BatchDecrypt => {
                    // Show progress for decryption operation
                    self.show_operation_progress(ui);
                },
                _ => {
                    ui.label("No decryption operation in progress");
                    if ui.add(Button::new(RichText::new("Back").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(5.0))
                    ).clicked() {
                        self.state = AppState::Main;
                    }
                }
            }
        });
    }
    
    // Key management screen UI
    fn show_key_management(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading("Key Management");
            ui.add_space(20.0);
            
            // Key generation section
            ui.group(|ui| {
                ui.heading("Generate New Key");
                
                // Key name input field
                ui.horizontal(|ui| {
                    ui.label("Key Name:");
                    ui.text_edit_singleline(&mut self.new_key_name);
                });
                
                if ui.add_sized(
                    [220.0, 40.0],
                    Button::new(RichText::new("Generate Key").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    if self.new_key_name.trim().is_empty() {
                        self.show_error("Please enter a name for the key");
                    } else {
                        let key = EncryptionKey::generate();
                        self.saved_keys.push((self.new_key_name.clone(), key.clone()));
                        self.current_key = Some(key);
                        self.show_status(&format!("New key '{}' generated and selected", self.new_key_name));
                        self.new_key_name.clear();
                    }
                }
            });
            
            ui.add_space(20.0);
            
            // Saved keys section
            ui.group(|ui| {
                ui.heading("Saved Keys");
                
                if self.saved_keys.is_empty() {
                    ui.label("No keys saved yet");
                } else {
                    let mut key_to_remove = None;
                    let mut key_to_select = None;
                    
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for (i, (name, key)) in self.saved_keys.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let is_selected = self.current_key.as_ref().map_or(false, |current| {
                                    // Compare keys by their base64 representation
                                    current.to_base64() == key.to_base64()
                                });
                                
                                if ui.selectable_label(is_selected, name).clicked() {
                                    key_to_select = Some(key.clone());
                                }
                                
                                let key_preview = {
                                    let key_base64 = key.to_base64();
                                    let preview_length = key_base64.len().min(10);
                                    format!("{}...", &key_base64[..preview_length])
                                };
                                
                                ui.label(key_preview);
                                
                                if ui.button("🗑️").clicked() {
                                    key_to_remove = Some((i, name.clone()));
                                }
                            });
                        }
                    });
                    
                    // Handle key selection outside the closure
                    if let Some(key) = key_to_select {
                        self.current_key = Some(key);
                        self.show_status("Key selected");
                    }
                    
                    // Handle key removal outside the closure
                    if let Some((idx, name)) = key_to_remove {
                        // If we're deleting the currently selected key, deselect it
                        if let Some(current_key) = &self.current_key {
                            if current_key.to_base64() == self.saved_keys[idx].1.to_base64() {
                                self.current_key = None;
                            }
                        }
                        
                        // Remove the key from the list
                        self.saved_keys.remove(idx);
                        self.show_status(&format!("Key '{}' removed", name));
                    }
                }
            });
            
            ui.add_space(20.0);
            
            // Import/Export section
            ui.group(|ui| {
                ui.heading("Import/Export Keys");
                
                ui.horizontal(|ui| {
                    if ui.add_sized(
                        [150.0, 40.0],
                        Button::new(RichText::new("Load Key from File").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.load_key_from_file();
                    }
                    
                    if !self.saved_keys.is_empty() && ui.add_sized(
                        [150.0, 40.0],
                        Button::new(RichText::new("Save Key to File").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        if let Some(_key) = &self.current_key {
                            self.save_key_to_file();
                        } else {
                            self.show_error("Please select a key first");
                        }
                    }
                });
            });
        });
    }
    
    // Logs screen UI
    fn show_logs(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading("Operation Logs");
            ui.add_space(10.0);
            
            if let Some(logger) = get_logger() {
                let entries = logger.get_entries();
                
                if entries.is_empty() {
                    ui.label("No logs yet");
                } else {
                    egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                        for entry in entries {
                            ui.group(|ui| {
                                let color = if entry.success {
                                    self.theme.success
                                } else {
                                    self.theme.error
                                };
                                
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&entry.timestamp).color(self.theme.text_secondary));
                                    ui.label(RichText::new(&entry.operation).strong().color(color));
                                });
                                
                                ui.label(format!("File: {}", entry.file_path));
                                ui.label(RichText::new(&entry.message).color(color));
                            });
                            ui.add_space(5.0);
                        }
                    });
                }
            } else {
                ui.label(RichText::new("Logger not initialized").color(self.theme.error));
            }
        });
    }
    
    // About screen UI
    fn show_about(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(RichText::new("About CRUSTy").size(28.0).color(self.theme.accent));
            ui.add_space(10.0);
            
            ui.label(RichText::new("Version 1.0").size(16.0).color(self.theme.text_primary));
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.label(RichText::new("CRUSTy - A Secure File Encryption Application").strong().size(16.0).color(self.theme.text_primary));
                ui.add_space(10.0);
                
                ui.label(RichText::new("CRUSTy is a secure file encryption application that uses AES-256-GCM encryption to protect your files.").color(self.theme.text_primary));
                ui.label(RichText::new("It provides a simple and intuitive interface for encrypting and decrypting files with strong cryptographic protection.").color(self.theme.text_primary));
                
                ui.add_space(10.0);
                ui.label(RichText::new("Features:").strong().color(self.theme.text_primary));
                ui.label(RichText::new("• AES-256-GCM encryption for strong security").color(self.theme.text_primary));
                ui.label(RichText::new("• Single file and batch processing").color(self.theme.text_primary));
                ui.label(RichText::new("• Key management (generation, saving, loading)").color(self.theme.text_primary));
                ui.label(RichText::new("• Operation logging").color(self.theme.text_primary));
                ui.label(RichText::new("• Progress tracking").color(self.theme.text_primary));
                
                ui.add_space(10.0);
                ui.label(RichText::new("Created by:").strong().color(self.theme.text_primary));
                ui.label(RichText::new("Shawn Ahern").color(self.theme.text_primary));
                
                ui.add_space(10.0);
                ui.label(RichText::new("Disclaimer:").strong().color(self.theme.error));
                ui.label(RichText::new("This application is for learning and research purposes only.").color(self.theme.text_primary));
                ui.label(RichText::new("While it uses strong encryption algorithms, it has not been audited for security vulnerabilities.").color(self.theme.text_primary));
                ui.label(RichText::new("Use at your own risk for sensitive data.").color(self.theme.text_primary));
            });
            
            ui.add_space(20.0);
            ui.group(|ui| {
                ui.label(RichText::new("Technical Information").strong().size(16.0).color(self.theme.text_primary));
                ui.add_space(5.0);
                
                ui.label(RichText::new("• Built with Rust and egui framework").color(self.theme.text_primary));
                ui.label(RichText::new("• Uses AES-256-GCM from the aes-gcm crate").color(self.theme.text_primary));
                ui.label(RichText::new("• Secure random key generation via rand crate").color(self.theme.text_primary));
                ui.label(RichText::new("• Native file dialogs provided by rfd").color(self.theme.text_primary));
                
                ui.add_space(5.0);
                ui.label(RichText::new("© 2025 Shawn Ahern. All rights reserved.").size(12.0).color(self.theme.text_primary));
            });
        });
    }
    
    // Show operation progress UI
    fn show_operation_progress(&mut self, ui: &mut Ui) {
        let progress = {
            let guard = self.progress.lock().unwrap();
            guard.clone()
        };
        
        if progress.is_empty() {
            // Operation not started or complete
            if !self.operation_results.is_empty() {
                // Operation complete
                
                // Check if there were any errors
                let has_errors = self.operation_results.iter().any(|r| r.contains("Failed"));
                
                if has_errors {
                    ui.label(RichText::new("⚠️ OPERATION COMPLETED WITH ERRORS").size(24.0).color(self.theme.error).strong());
                } else {
                    ui.label(RichText::new("✅ OPERATION COMPLETE").size(24.0).color(self.theme.success).strong());
                }
                
                ui.add_space(10.0);
                
                // Display results with better formatting
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for result in &self.operation_results {
                        let success = !result.contains("Failed");
                        let color = if success {
                            self.theme.success
                        } else {
                            self.theme.error
                        };
                        
                        // Special formatting for wrong key errors
                        if result.contains("Wrong encryption key") {
                            ui.group(|ui| {
                                ui.label(RichText::new("🔑 WRONG ENCRYPTION KEY").size(18.0).color(self.theme.error).strong());
                                ui.label(RichText::new(result).color(self.theme.error));
                                ui.label(RichText::new("Please try a different key or check that you're using the correct key for this file.").color(self.theme.error));
                            });
                        } else {
                            ui.label(RichText::new(result).color(color));
                        }
                    }
                });
                
                ui.add_space(10.0);
                if ui.add(Button::new(RichText::new("Back to Main").color(self.theme.button_text))
                    .fill(self.theme.button_normal)
                    .rounding(Rounding::same(5.0))
                ).clicked() {
                    self.state = AppState::Main;
                    self.operation = FileOperation::None;
                    self.operation_results.clear();
                    // Clear selected files after operation completes
                    self.selected_files.clear();
                    self.show_status("Operation completed. Files have been cleared for next operation.");
                }
            } else {
                // Operation not started
                ui.label("Operation starting...");
                
                // Check if we have results in shared_results
                self.update_operation_results();
                if !self.operation_results.is_empty() {
                    // Force a repaint to show the results
                    ui.ctx().request_repaint();
                }
            }
        } else {
            // Operation in progress
            ui.label(format!("Processing {} files...", progress.len()));
            
            for (i, p) in progress.iter().enumerate() {
                let file_name = if i < self.selected_files.len() {
                    self.selected_files[i].file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| format!("File {}", i + 1))
                } else {
                    format!("File {}", i + 1)
                };
                
                ui.label(format!("{}: {:.1}%", file_name, p * 100.0));
                
                // Progress bar
                let progress_bar = egui::ProgressBar::new(*p)
                    .show_percentage()
                    .animate(true);
                ui.add(progress_bar);
            }
            
            // Check if all operations are complete
            let all_complete = progress.iter().all(|p| (*p - 1.0).abs() < 0.01);
            if all_complete {
                // Update results before progress is cleared
                self.update_operation_results();
                
                // Request a repaint to show the results
                ui.ctx().request_repaint();
            }
        }
    }
    
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
                        let result = encrypt_file(
                            &file_path,
                            &output_path,
                            &key,
                            move |p| {
                                let mut guard = progress_clone.lock().unwrap();
                                if !guard.is_empty() {
                                    guard[0] = p;
                                }
                            }
                        );
                            
                        // Log the result
                        if let Some(logger) = get_logger() {
                            match &result {
                                Ok(_) => {
                                    logger.log_success(
                                        "Encrypt",
                                        &file_path.to_string_lossy(),
                                        "Encryption successful"
                                    ).ok();
                                    
                                    // Store result
                                    let result_msg = format!("Successfully encrypted: {}", file_path.display());
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
                        let result = decrypt_file(
                            file_path,
                            &output_path,
                            &key,
                            move |p| {
                                let mut guard = progress_clone.lock().unwrap();
                                if !guard.is_empty() {
                                    guard[0] = p;
                                }
                            }
                        );
                        
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
                    
                    let results = encrypt_files(
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
                                    logger.log_success("Batch Encrypt", &file_path, result).ok();
                                    
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
    
    // Update operation results from shared results
    fn update_operation_results(&mut self) {
        if let Ok(shared_results) = self.shared_results.lock() {
            if !shared_results.is_empty() {
                self.operation_results = shared_results.clone();
            }
        }
    }
    
    // Select files using native file dialog
    fn select_files(&mut self) {
        let fd = if matches!(self.operation, FileOperation::Decrypt) {
            rfd::FileDialog::new().add_filter("Encrypted Files", &["encrypted"])
        } else {
            rfd::FileDialog::new().add_filter("All Files", &["*"])
        };

        let files = if self.batch_mode {
            fd.set_title("Select Files").pick_files()
        } else {
            fd.set_title("Select a File").pick_file().map(|file| vec![file])
        };

        if let Some(files) = files {
            if !files.is_empty() {
                self.selected_files = files;
                self.show_status(&format!("Selected {} files", self.selected_files.len()));
            }
        }
    }
    
    // Select output directory using native file dialog
    fn select_output_dir(&mut self) {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Select Output Directory")
            .pick_folder() {
            self.output_dir = Some(dir);
            self.show_status(&format!("Output directory set to: {}", self.output_dir.as_ref().unwrap().display()));
        }
    }
    
    // Load key from file using native file dialog
    fn load_key_from_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Load Key File")
            .add_filter("Key Files", &["key"])
            .pick_file() {
            
            match EncryptionKey::load_from_file(&path) {
                Ok(key) => {
                    // Generate a name for the key based on the file name
                    let key_name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Imported Key")
                        .to_string();
                    
                    // Check if a key with this name already exists
                    let mut unique_name = key_name.clone();
                    let mut counter = 1;
                    
                    while self.saved_keys.iter().any(|(name, _)| name == &unique_name) {
                        unique_name = format!("{} ({})", key_name, counter);
                        counter += 1;
                    }
                    
                    // Add the key to our saved keys
                    self.saved_keys.push((unique_name.clone(), key.clone()));
                    
                    // Set as current key
                    self.current_key = Some(key);
                    self.key_path = Some(path.clone());
                    self.show_status(&format!("Key '{}' loaded from: {}", unique_name, path.display()));
                    
                    // Log successful key load
                    if let Some(logger) = get_logger() {
                        logger.log_success(
                            "Load Key",
                            &path.to_string_lossy(),
                            "Key loaded successfully"
                        ).ok();
                    }
                },
                Err(e) => {
                    self.show_error(&format!("Failed to load key: {}", e));
                    
                    // Log failed key load
                    if let Some(logger) = get_logger() {
                        logger.log_error(
                            "Load Key",
                            &path.to_string_lossy(),
                            &e.to_string()
                        ).ok();
                    }
                }
            }
        }
    }
    
    // Save key to file using native file dialog
    fn save_key_to_file(&mut self) {
        if let Some(key) = &self.current_key {
            // Find the name of the current key
            let key_name = self.saved_keys.iter()
                .find_map(|(name, k)| {
                    if k.to_base64() == key.to_base64() {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "key".to_string());
            
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Save Key File")
                .set_file_name(&format!("{}.key", key_name))
                .add_filter("Key Files", &["key"])
                .save_file() {
                
                match key.save_to_file(&path) {
                    Ok(_) => {
                        self.key_path = Some(path.clone());
                        self.show_status(&format!("Key '{}' saved to: {}", key_name, path.display()));
                        
                        // Log successful key save
                        if let Some(logger) = get_logger() {
                            logger.log_success(
                                "Save Key",
                                &path.to_string_lossy(),
                                "Key saved successfully"
                            ).ok();
                        }
                    },
                    Err(e) => {
                        self.show_error(&format!("Failed to save key: {}", e));
                        
                        // Log failed key save
                        if let Some(logger) = get_logger() {
                            logger.log_error(
                                "Save Key",
                                &path.to_string_lossy(),
                                &e.to_string()
                            ).ok();
                        }
                    }
                }
            }
        }
    }
}