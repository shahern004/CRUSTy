use eframe::egui::{self, Ui, RichText, Button, Rounding, Layout, Align};
use crate::gui::app_core::CrustyApp;
use crate::gui::file_list::FileOperationType;
use crate::start_operation::FileOperation;
use crate::gui::app_state::AppState;

/// Action bar trait for displaying the top action buttons
pub trait ActionBar {
    fn show_action_bar(&mut self, ui: &mut Ui);
}

impl ActionBar for CrustyApp {
    fn show_action_bar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            // Set spacing between buttons
            ui.spacing_mut().item_spacing.x = 10.0;
            
            // Create a button style for the action buttons
            let button_size = egui::vec2(80.0, 80.0);
            let text_size = 14.0;
            let icon_size = 32.0;
            
            // Encrypt button
            let encrypt_button = ui.add_sized(
                button_size,
                Button::new(
                    RichText::new("🔒").size(icon_size)
                )
                .fill(self.theme.button_normal)
                .rounding(Rounding::same(8.0))
            );
            
            // Add label under the button
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(-25.0); // Adjust spacing to position the label under the button
                ui.label(RichText::new("Encrypt").size(text_size));
            });
            
            if encrypt_button.clicked() {
                if !self.selected_files.is_empty() && self.current_key.is_some() {
                    self.operation = FileOperation::Encrypt;
                    
                    // Add files to the file list
                    let files_to_add = self.selected_files.clone();
                    for file in files_to_add {
                        self.add_file_entry(file, FileOperationType::Encrypt);
                    }
                    
                    self.show_status("Starting encryption...");
                } else {
                    self.show_error("Please select files and encryption key");
                }
            }
            
            // Decrypt button
            let decrypt_button = ui.add_sized(
                button_size,
                Button::new(
                    RichText::new("🔓").size(icon_size)
                )
                .fill(self.theme.button_normal)
                .rounding(Rounding::same(8.0))
            );
            
            // Add label under the button
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(-25.0);
                ui.label(RichText::new("Decrypt").size(text_size));
            });
            
            if decrypt_button.clicked() {
                if !self.selected_files.is_empty() && self.current_key.is_some() {
                    self.operation = FileOperation::Decrypt;
                    
                    // Add files to the file list
                    let files_to_add = self.selected_files.clone();
                    for file in files_to_add {
                        self.add_file_entry(file, FileOperationType::Decrypt);
                    }
                    
                    self.show_status("Starting decryption...");
                } else {
                    self.show_error("Please select files and encryption key");
                }
            }
            
            // Stop Operation button
            let stop_button = ui.add_sized(
                button_size,
                Button::new(
                    RichText::new("⛔").size(icon_size)
                )
                .fill(self.theme.button_normal)
                .rounding(Rounding::same(8.0))
            );
            
            // Add label under the button
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(-25.0);
                ui.label(RichText::new("Stop").size(text_size));
            });
            
            if stop_button.clicked() {
                self.operation = FileOperation::None;
                self.show_status("Operation stopped");
            }
            
            // Key Management button
            let key_button = ui.add_sized(
                button_size,
                Button::new(
                    RichText::new("🔑").size(icon_size)
                )
                .fill(self.theme.button_normal)
                .rounding(Rounding::same(8.0))
            );
            
            // Add label under the button
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(-25.0);
                ui.label(RichText::new("Keys").size(text_size));
            });
            
            if key_button.clicked() {
                self.state = AppState::KeyManagement;
                self.show_status("Key management");
            }
            
            // Advanced Options button
            let advanced_button = ui.add_sized(
                button_size,
                Button::new(
                    RichText::new("⚙️").size(icon_size)
                )
                .fill(self.theme.button_normal)
                .rounding(Rounding::same(8.0))
            );
            
            // Add label under the button
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(-25.0);
                ui.label(RichText::new("Advanced").size(text_size));
            });
            
            if advanced_button.clicked() {
                // Toggle between main screen and advanced options
                if self.state == AppState::MainScreen {
                    self.state = AppState::Dashboard;
                    self.show_status("Advanced options");
                } else {
                    self.state = AppState::MainScreen;
                    self.show_status("Main screen");
                }
            }
            
            // Open Files button
            let open_button = ui.add_sized(
                button_size,
                Button::new(
                    RichText::new("📂").size(icon_size)
                )
                .fill(self.theme.button_normal)
                .rounding(Rounding::same(8.0))
            );
            
            // Add label under the button
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(-25.0);
                ui.label(RichText::new("Open").size(text_size));
            });
            
            if open_button.clicked() {
                self.select_files();
            }
        });
    }
}
