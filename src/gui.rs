use eframe::egui;
use egui::{Ui, Color32, Button, RichText, Stroke, Rounding};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::encryption::EncryptionKey;
use crate::logger::get_logger;
use crate::backend::{EmbeddedConfig, ConnectionType};
use crate::start_operation::FileOperation;
use crate::transfer_gui::{TransferState, TransferReceiveState};
use crate::split_key::TransferPackage;

// Define color theme for the application
pub struct AppTheme {
    pub background: Color32,
    pub accent: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub button_text: Color32,  // New color for button text
    pub button_normal: Color32,
    pub button_hovered: Color32,
    pub button_active: Color32,
    pub error: Color32,
    pub success: Color32,
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
pub enum AppState {
    Main,
    Encrypting,
    Decrypting,
    KeyManagement,
    SplitKeyManagement,
    TransferPreparation,
    TransferReceive,
    ViewLogs,
    About,
}


// Application structure
pub struct CrustyApp {
    pub state: AppState,
    pub theme: AppTheme,
    
    // File paths
    pub selected_files: Vec<PathBuf>,
    pub output_dir: Option<PathBuf>,
    
    // Key management
    pub current_key: Option<EncryptionKey>,
    pub key_path: Option<PathBuf>,
    pub saved_keys: Vec<(String, EncryptionKey)>, // (key_name, key)
    pub new_key_name: String,
    
    // Recipient information
    pub recipient_email: String,
    pub use_recipient: bool,
    
    // Progress tracking
    pub operation: FileOperation,
    pub progress: Arc<Mutex<Vec<f32>>>,
    pub operation_results: Vec<String>,
    pub shared_results: Arc<Mutex<Vec<String>>>, // Shared results for thread communication
    
    // Status and errors
    pub status_message: String,
    pub error_message: String,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    
    // Flag for batch operation
    pub batch_mode: bool,
    
    // Embedded backend configuration
    pub use_embedded_backend: bool,
    pub embedded_config: Option<EmbeddedConfig>,
    pub embedded_connection_type: ConnectionType,
    pub embedded_device_id: String,
    pub embedded_parameters: HashMap<String, String>,
    
    // Transfer functionality
    pub transfer_package: Option<TransferPackage>,
    pub transfer_state: TransferState,
    pub transfer_receive_state: TransferReceiveState,
    pub transfer_share1: String,
    pub transfer_share2: String,
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
            
            recipient_email: String::new(),
            use_recipient: false,
            
            operation: FileOperation::None,
            progress: Arc::new(Mutex::new(Vec::new())),
            operation_results: Vec::new(),
            shared_results: Arc::new(Mutex::new(Vec::new())),
            
            status_message: "Welcome to CRUSTy".to_string(),
            error_message: String::new(),
            last_status: None,
            last_error: None,
            
            batch_mode: false,
            
            use_embedded_backend: false,
            embedded_config: None,
            embedded_connection_type: ConnectionType::Usb,
            embedded_device_id: String::new(),
            embedded_parameters: HashMap::new(),
            
            transfer_package: None,
            transfer_state: TransferState::Initial,
            transfer_receive_state: TransferReceiveState::Initial,
            transfer_share1: String::new(),
            transfer_share2: String::new(),
        }
    }
}

impl eframe::App for CrustyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update operation results from shared results
        self.update_operation_results();
        
        // Check for any status or error messages set by UI callbacks
        if let Some(status) = self.last_status.take() {
            self.show_status(&status);
        }
        
        if let Some(error) = self.last_error.take() {
            self.show_error(&error);
        }
        
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
                
                // Reordered buttons with Home first (only shown when not on Main screen)
                if !matches!(self.state, AppState::Main) {
                    if ui.add(Button::new(RichText::new("🏠 Home").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(5.0))
                    ).clicked() {
                        self.state = AppState::Main;
                        self.operation = FileOperation::None;
                    }
                }
                
                // Renamed from "Keys" to "Key Management"
                if ui.add(Button::new(RichText::new("🔑 Key Management").color(self.theme.button_text))
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
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Wrap all content in a ScrollArea to enable scrolling when content exceeds window height
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Display appropriate screen based on state
                match self.state {
                    AppState::Main => self.show_main_screen(ui),
                    AppState::Encrypting => self.show_encrypt_screen(ui),
                    AppState::Decrypting => self.show_decrypt_screen(ui),
                    AppState::KeyManagement => self.show_key_management(ui),
                    AppState::SplitKeyManagement => self.show_split_key_management(ui),
                    AppState::TransferPreparation => self.show_transfer_preparation(ui),
                    AppState::TransferReceive => self.show_transfer_receive(ui),
                    AppState::ViewLogs => self.show_logs(ui),
                    AppState::About => self.show_about(ui),
                }
            });
            
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
    // Select files using native file dialog
    pub fn select_files(&mut self) {
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
    pub fn select_output_dir(&mut self) {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Select Output Directory")
            .pick_folder() {
            self.output_dir = Some(dir);
            self.show_status(&format!("Output directory set to: {}", self.output_dir.as_ref().unwrap().display()));
        }
    }
    
    // Helper method to display error messages
    pub fn show_error(&mut self, message: &str) {
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
    pub fn show_status(&mut self, message: &str) {
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
            
            // Recipient information
            ui.group(|ui| {
                ui.heading("Recipient Information");
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.use_recipient, "Use recipient-specific encryption");
                });
                
                if self.use_recipient {
                    ui.horizontal(|ui| {
                        ui.label("Recipient Email:");
                        ui.text_edit_singleline(&mut self.recipient_email);
                    });
                    
                    ui.label(RichText::new("The recipient's email will be used to derive a unique encryption key.").color(self.theme.text_secondary));
                    ui.label(RichText::new("The recipient must use the same master key to decrypt the file.").color(self.theme.text_secondary));
                }
            });
            
            ui.add_space(20.0);
            
            // Embedded backend configuration
            ui.group(|ui| {
                ui.heading("Embedded System Integration");
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.use_embedded_backend, "Use embedded system for cryptographic operations");
                });
                
                if self.use_embedded_backend {
                    ui.add_space(5.0);
                    
                    // Connection type selection
                    ui.horizontal(|ui| {
                        ui.label("Connection Type:");
                        ui.radio_value(&mut self.embedded_connection_type, ConnectionType::Usb, "USB");
                        ui.radio_value(&mut self.embedded_connection_type, ConnectionType::Serial, "Serial/UART");
                        ui.radio_value(&mut self.embedded_connection_type, ConnectionType::Ethernet, "Ethernet");
                    });
                    
                    // Device ID input
                    ui.horizontal(|ui| {
                        ui.label("Device ID/Address:");
                        ui.text_edit_singleline(&mut self.embedded_device_id);
                    });
                    
                    // Connection parameters
                    ui.collapsing("Advanced Connection Parameters", |ui| {
                        // Add/remove parameter fields
                        let mut param_to_remove = None;
                        let mut new_param = false;
                        let mut new_param_name = String::new();
                        let mut new_param_value = String::new();
                        
                        ui.horizontal(|ui| {
                            ui.label("Parameter Name:");
                            ui.text_edit_singleline(&mut new_param_name);
                            ui.label("Value:");
                            ui.text_edit_singleline(&mut new_param_value);
                            
                            if ui.button("Add").clicked() && !new_param_name.is_empty() {
                                new_param = true;
                            }
                        });
                        
                        if new_param {
                            self.embedded_parameters.insert(new_param_name.clone(), new_param_value.clone());
                            new_param_name.clear();
                            new_param_value.clear();
                        }
                        
                        // Display existing parameters
                        for (name, value) in &self.embedded_parameters {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}: {}", name, value));
                                if ui.button("🗑️").clicked() {
                                    param_to_remove = Some(name.clone());
                                }
                            });
                        }
                        
                        if let Some(param) = param_to_remove {
                            self.embedded_parameters.remove(&param);
                        }
                    });
                    
                    ui.add_space(5.0);
                    
                    // Connection status and buttons
                    ui.horizontal(|ui| {
                        let status_text = if self.embedded_config.is_some() {
                            RichText::new("✓ Configuration ready").color(self.theme.success)
                        } else {
                            RichText::new("⚠️ Not configured").color(self.theme.error)
                        };
                        
                        ui.label(status_text);
                        
                        if ui.button("Apply Configuration").clicked() {
                            // Create the embedded configuration
                            if !self.embedded_device_id.is_empty() {
                                self.embedded_config = Some(EmbeddedConfig {
                                    connection_type: self.embedded_connection_type.clone(),
                                    device_id: self.embedded_device_id.clone(),
                                    parameters: self.embedded_parameters.clone(),
                                });
                                self.show_status("Embedded backend configuration applied");
                            } else {
                                self.show_error("Please enter a device ID/address");
                            }
                        }
                        
                        if ui.button("Reset").clicked() {
                            self.embedded_config = None;
                            self.show_status("Embedded backend configuration reset");
                        }
                    });
                    
                    ui.add_space(5.0);
                    ui.label(RichText::new("Note: The embedded system must be properly set up with the CRUSTy C/C++ API.").color(self.theme.text_secondary));
                    ui.label(RichText::new("Cryptographic operations will be offloaded to the embedded device when enabled.").color(self.theme.text_secondary));
                }
            });
            
            ui.add_space(20.0);
            
            // Operation buttons
            if !self.selected_files.is_empty() && self.output_dir.is_some() && self.current_key.is_some() {
                ui.group(|ui| {
                    ui.heading("Start Operation");
                    
                    // Show warning if recipient mode is enabled but no email is provided
                    if self.use_recipient && self.recipient_email.trim().is_empty() {
                        ui.label(RichText::new("⚠️ Please enter a recipient email address").color(self.theme.error));
                    }
                    
                    ui.horizontal(|ui| {
                        let encrypt_button_enabled = !self.use_recipient || !self.recipient_email.trim().is_empty();
                        
                        if ui.add_enabled(
                            encrypt_button_enabled,
                            Button::new(RichText::new("Encrypt").color(self.theme.button_text).size(18.0))
                                .fill(self.theme.button_normal)
                                .rounding(Rounding::same(8.0))
                                .min_size(egui::vec2(140.0, 50.0))
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
                    
                    if self.use_recipient && self.recipient_email.trim().is_empty() {
                        ui.label(RichText::new("• Enter a recipient email address").color(self.theme.error));
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
                    if ui.button("Back").clicked() {
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
                    if ui.button("Back").clicked() {
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
                        if let Some(key) = &self.current_key {
                            self.save_key_to_file();
                        } else {
                            self.show_error("Please select a key first");
                        }
                    }
                });
            });
            
            ui.add_space(20.0);
            
            // Split key management
            ui.group(|ui| {
                ui.heading("Advanced Key Management");
                
                ui.horizontal(|ui| {
                    if ui.add_sized(
                        [220.0, 40.0],
                        Button::new(RichText::new("Split Key Management").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.state = AppState::SplitKeyManagement;
                    }
                    
                    if ui.add_sized(
                        [220.0, 40.0],
                        Button::new(RichText::new("Transfer Preparation").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.state = AppState::TransferPreparation;
                    }
                });
                
                ui.add_space(10.0);
                
                if ui.add_sized(
                    [220.0, 40.0],
                    Button::new(RichText::new("Receive Transfer").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    self.state = AppState::TransferReceive;
                }
            });
        });
    }
    
    // Split key management UI
    fn show_split_key_management(&mut self, ui: &mut Ui) {
        self.show_split_key_management_impl(ui);
    }
    
    // Transfer preparation UI
    fn show_transfer_preparation(&mut self, ui: &mut Ui) {
        self.show_transfer_preparation_impl(ui);
    }
    
    // Transfer receive UI
    fn show_transfer_receive(&mut self, ui: &mut Ui) {
        self.show_transfer_receive_impl(ui);
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
            ui.heading(RichText::new("About CRUSTy").size(24.0));
            ui.add_space(10.0);
            
            ui.label("CRUSTy - Cryptographic Rust Utility");
            ui.label("Version 0.1.0");
            ui.add_space(10.0);
            
            ui.label("A secure file encryption application using AES-256-GCM encryption.");
            ui.label("Built with Rust and eframe for cross-platform compatibility.");
            
            ui.add_space(20.0);
            ui.heading("Features");
            ui.label("• AES-256-GCM authenticated encryption");
            ui.label("• Single file and batch processing");
            ui.label("• Key management with import/export");
            ui.label("• Split-key encryption for enhanced security");
            ui.label("• Secure transfer functionality");
            ui.label("• Embedded system integration");
            
            ui.add_space(20.0);
            ui.label(RichText::new("© 2025 CRUSTy Team").color(self.theme.text_secondary));
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
                ui.label(RichText::new("Operation Complete").color(self.theme.success));
                ui.add_space(10.0);
                
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for result in &self.operation_results {
                        let success = !result.contains("Failed");
                        let color = if success {
                            self.theme.success
                        } else {
                            self.theme.error
                        };
                        ui.label(RichText::new(result).color(color));
                    }
                });
                
                ui.add_space(10.0);
                if ui.button("Back to Main").clicked() {
                    self.state = AppState::Main;
                    self.operation = FileOperation::None;
                    self.operation_results.clear();
                }
            } else {
                // Operation not started
                ui.label("Operation starting...");
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
        }
    }
    
    // Update operation results from shared results
    fn update_operation_results(&mut self) {
        let mut shared_results = self.shared_results.lock().unwrap();
        if !shared_results.is_empty() {
            self.operation_results.extend(shared_results.drain(..));
        }
    }
    
    // Load key from file using native file dialog
    fn load_key_from_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Load Key File")
            .add_filter("Key Files", &["key"])
            .pick_file() {
            
            // Read the file content
            let result = std::fs::read_to_string(&path).map_err(|e| {
                format!("Failed to read key file: {}", e)
            }).and_then(|content| {
                // Parse the Base64 content to create a key
                EncryptionKey::from_base64(&content.trim()).map_err(|e| {
                    format!("Invalid key format: {}", e)
                })
            });
            
            match result {
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
                            &e
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
                
                // Convert the key to Base64 and write to file
                let base64 = key.to_base64();
                let result = std::fs::write(&path, base64).map_err(|e| {
                    format!("Failed to write key file: {}", e)
                });
                
                match result {
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
                                &e
                            ).ok();
                        }
                    }
                }
            }
        }
    }
    
    // Start the selected operation
    fn start_operation(&mut self) {
        crate::start_operation::start_operation(self);
    }
}
