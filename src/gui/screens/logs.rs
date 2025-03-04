use eframe::egui::{Ui, RichText, Button, Rounding, ScrollArea, TextEdit, TextStyle};
use crate::gui::app_core::CrustyApp;
use crate::gui::app_state::AppState;
use crate::logger::get_logger;
use std::path::PathBuf;

/// Logs screen trait
pub trait LogsScreen {
    fn show_logs(&mut self, ui: &mut Ui);
}

impl LogsScreen for CrustyApp {
    fn show_logs(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(RichText::new("Operation Logs").size(28.0));
            ui.add_space(10.0);
            
            // Get log path
            let mut log_path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            log_path.push("crusty");
            log_path.push("logs");
            log_path.push("operations.log");
            
            // Display log path
            ui.horizontal(|ui| {
                ui.label("Log file location:");
                ui.label(RichText::new(format!("{}", log_path.display())).monospace());
                
                if ui.add(Button::new(RichText::new("Open Log Directory").color(self.theme.button_text))
                    .fill(self.theme.button_normal)
                    .rounding(Rounding::same(5.0))
                ).clicked() {
                    if let Some(parent) = log_path.parent() {
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("explorer")
                            .arg(parent)
                            .spawn();
                        
                        #[cfg(target_os = "macos")]
                        let _ = std::process::Command::new("open")
                            .arg(parent)
                            .spawn();
                        
                        #[cfg(target_os = "linux")]
                        let _ = std::process::Command::new("xdg-open")
                            .arg(parent)
                            .spawn();
                    }
                }
            });
            
            ui.add_space(10.0);
            
            // Display log content
            ui.group(|ui| {
                ui.heading("Recent Logs");
                
                let log_content = if log_path.exists() {
                    match std::fs::read_to_string(&log_path) {
                        Ok(content) => content,
                        Err(e) => format!("Error reading log file: {}", e),
                    }
                } else {
                    "No log file found.".to_string()
                };
                
                // Display log content in a scrollable area with monospace font
                ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.add(TextEdit::multiline(&mut log_content.as_str())
                            .font(TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .interactive(false));
                    });
            });
            
            ui.add_space(20.0);
            
            // Log management buttons
            ui.horizontal(|ui| {
                if ui.add_sized(
                    [120.0, 30.0],
                    Button::new(RichText::new("Refresh Logs").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(5.0))
                ).clicked() {
                    // Just refresh the UI to show updated logs
                    self.show_status("Logs refreshed");
                }
                
                if ui.add_sized(
                    [120.0, 30.0],
                    Button::new(RichText::new("Clear Logs").color(self.theme.button_text))
                        .fill(self.theme.error)
                        .rounding(Rounding::same(5.0))
                ).clicked() {
                    // Clear the log file
                    if let Some(_logger) = get_logger() {
                        // We'll just truncate the file instead of calling clear_logs
                        if let Err(e) = std::fs::write(&log_path, "") {
                            self.show_error(&format!("Failed to clear logs: {}", e));
                        } else {
                            self.show_status("Logs cleared successfully");
                        }
                    } else {
                        self.show_error("Logger not initialized");
                    }
                }
                
                // Back button
                if ui.add_sized(
                    [120.0, 30.0],
                    Button::new(RichText::new("Back").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(5.0))
                ).clicked() {
                    self.state = AppState::Dashboard;
                }
            });
        });
    }
}
