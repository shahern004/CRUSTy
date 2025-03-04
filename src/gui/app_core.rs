use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use eframe::egui::{self, Context};

use crate::encryption::EncryptionKey;
use crate::gui::theme::AppTheme;
use crate::gui::app_state::{AppState, EncryptionWorkflowStep};
use crate::gui::file_list::{FileEntry, EnhancedFileList};
use crate::start_operation::FileOperation;
use crate::logger::{Logger, get_logger};


use crate::gui::screens::*;

/// Main application struct
pub struct CrustyApp {
    // UI state
    pub theme: AppTheme,
    pub state: AppState,
    pub status_message: Option<String>,
    pub status_time: Instant,
    pub error_message: Option<String>,
    pub error_time: Instant,
    
    // File operations
    pub selected_files: Vec<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub batch_mode: bool,
    pub operation: FileOperation,
    pub progress: Arc<Mutex<Vec<f32>>>,
    pub operation_results: Vec<String>,
    
    // File list
    pub file_entries: Vec<FileEntry>,
    
    // Encryption
    pub current_key: Option<EncryptionKey>,
    pub saved_keys: Vec<(String, EncryptionKey)>,
    pub new_key_name: String,
    
    // Embedded backend options
    pub use_embedded_backend: bool,
    pub embedded_connection_type: crate::backend::ConnectionType,
    pub embedded_device_id: String,
    
    // Workflow
    pub encryption_workflow_step: EncryptionWorkflowStep,
    pub encryption_workflow_complete: bool,
    
    // Status tracking
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    
    // Logger
    pub logger: Arc<Logger>,
}

// Implement AsRef<AppTheme> for CrustyApp to support EnhancedFileList trait
impl AsRef<AppTheme> for CrustyApp {
    fn as_ref(&self) -> &AppTheme {
        &self.theme
    }
}

// Implement AsMut<Vec<FileEntry>> for CrustyApp to support EnhancedFileList trait
impl AsMut<Vec<FileEntry>> for CrustyApp {
    fn as_mut(&mut self) -> &mut Vec<FileEntry> {
        &mut self.file_entries
    }
}

impl Default for CrustyApp {
    fn default() -> Self {
        Self {
            theme: AppTheme::default(),
            state: AppState::Dashboard,
            status_message: None,
            status_time: Instant::now(),
            error_message: None,
            error_time: Instant::now(),
            
            selected_files: Vec::new(),
            output_dir: None,
            batch_mode: false,
            operation: FileOperation::None,
            progress: Arc::new(Mutex::new(Vec::new())),
            operation_results: Vec::new(),
            
            file_entries: Vec::new(),
            
            current_key: None,
            saved_keys: Vec::new(),
            new_key_name: String::new(),
            
            use_embedded_backend: false,
            embedded_connection_type: crate::backend::ConnectionType::Usb,
            embedded_device_id: String::new(),
            
            encryption_workflow_step: EncryptionWorkflowStep::Files,
            encryption_workflow_complete: false,
            
            last_status: None,
            last_error: None,
            
            logger: get_logger().unwrap_or_else(|| {
                let mut log_path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
                log_path.push("crusty");
                log_path.push("logs");
                std::fs::create_dir_all(&log_path).expect("Failed to create log directory");
                log_path.push("operations.log");
                
                Arc::new(Logger::new(&log_path).expect("Failed to initialize logger"))
            }),
        }
    }
}

impl eframe::App for CrustyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Apply theme to context
        self.theme.apply_to_context(ctx);
        
        // Handle status and error message timeouts
        let now = Instant::now();
        if let Some(_) = &self.status_message {
            if now.duration_since(self.status_time) > Duration::from_secs(5) {
                self.status_message = None;
            }
        }
        if let Some(_) = &self.error_message {
            if now.duration_since(self.error_time) > Duration::from_secs(5) {
                self.error_message = None;
            }
        }
        
        // Handle last status and error messages from closures
        if let Some(status) = self.last_status.take() {
            self.show_status(&status);
        }
        if let Some(error) = self.last_error.take() {
            self.show_error(&error);
        }
        
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.select_files();
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        _frame.close();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.state = AppState::About;
                        ui.close_menu();
                    }
                    if ui.button("View Logs").clicked() {
                        self.state = AppState::Logs;
                        ui.close_menu();
                    }
                });
            });
        });
        
        // Status panel with status and error messages
        egui::TopBottomPanel::top("status_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(status) = &self.status_message {
                    ui.label(egui::RichText::new(status).color(self.theme.success));
                }
                
                if let Some(error) = &self.error_message {
                    ui.label(egui::RichText::new(error).color(self.theme.error));
                }
            });
        });
        
        // Main central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            // Display the current screen based on the application state
            match self.state {
                AppState::Dashboard => self.show_dashboard(ui),
                AppState::MainScreen => self.show_main_screen(ui),
                AppState::EncryptionWorkflow => self.show_encryption_workflow(ui),
                AppState::Encrypting => self.show_encrypt_screen(ui),
                AppState::Decrypting => self.show_decrypt_screen(ui),
                AppState::KeyManagement => self.show_key_management(ui),
                AppState::Logs => self.show_logs(ui),
                AppState::About => self.show_about(ui),
            }
        });
    }
}
