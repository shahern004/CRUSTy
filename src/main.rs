// Set the Windows subsystem to "windows" to hide the console window
#![cfg_attr(windows, windows_subsystem = "windows")]

/// CRUSTy
///
/// A secure file encryption application using AES-256-GCM encryption.
/// Features:
/// - Encrypt and decrypt individual files
/// - Batch processing of multiple files
/// - Key management (generation, saving, loading)
/// - Operation logging
/// - Progress tracking
mod encryption;
mod logger;
mod gui;
mod backend;
mod backend_local;
mod backend_embedded;
mod start_operation;
mod split_key;
mod split_key_gui;
mod transfer_gui;
mod gui_impl;
mod test_transfer;

use eframe::{run_native, NativeOptions};
use gui::CrustyApp;
use std::path::PathBuf;

/// Application entry point
fn main() -> Result<(), eframe::Error> {
    // Initialize logger
    let mut log_path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    log_path.push("crusty");
    log_path.push("logs");
    std::fs::create_dir_all(&log_path).expect("Failed to create log directory");
    log_path.push("operations.log");
    
    logger::init_logger(&log_path).expect("Failed to initialize logger");
    
    let app = CrustyApp::default();
    
    // Configure window options
    let window_options = NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(800.0, 600.0)),
        resizable: true,
        vsync: true,
        ..Default::default()
    };

    // Start the GUI application
    run_native(
        "CRUSTy",
        window_options,
        Box::new(|_cc| Box::new(app)),
    )
}
