use eframe::egui::{Ui, RichText, Button, Rounding};
use crate::gui::app_core::CrustyApp;
use crate::gui::app_state::AppState;

/// About screen trait
pub trait AboutScreen {
    fn show_about(&mut self, ui: &mut Ui);
}

impl AboutScreen for CrustyApp {
    fn show_about(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(RichText::new("About CRUSTy").size(28.0));
            ui.add_space(10.0);
            
            ui.label("CRUSTy - Cryptographic Rust Utility");
            ui.label("Version 1.0.0");
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.heading("Description");
                ui.label("CRUSTy is a secure file encryption application using AES-256-GCM encryption.");
                ui.label("It provides a user-friendly interface for encrypting and decrypting files.");
                ui.add_space(10.0);
                
                ui.label("Features:");
                ui.label("• Encrypt and decrypt individual files");
                ui.label("• Batch processing of multiple files");
                ui.label("• Key management (generation, saving, loading)");
                ui.label("• Split-key functionality for enhanced security");
                ui.label("• Recipient-specific encryption");
                ui.label("• Operation logging");
                ui.label("• Progress tracking");
                ui.label("• Support for hardware encryption via embedded devices");
            });
            
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.heading("Technical Details");
                ui.label("• Built with Rust and eframe/egui for the GUI");
                ui.label("• Uses AES-256-GCM for authenticated encryption");
                ui.label("• Implements HKDF for recipient-specific key derivation");
                ui.label("• Supports both software-based and hardware-based encryption backends");
                ui.label("• File operations are performed with progress tracking");
            });
            
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.heading("License");
                ui.label("This software is licensed under the MIT License.");
                ui.label("Copyright © 2025 CRUSTy Team");
            });
            
            ui.add_space(20.0);
            
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
    }
}
