use eframe::egui::{Ui, RichText, Button, Rounding, TopBottomPanel};
use crate::gui::app_core::CrustyApp;
use crate::gui::app_state::AppState;
use crate::gui::action_bar::ActionBar;
use crate::gui::file_list::EnhancedFileList;
use crate::start_operation::FileOperation;

/// Dashboard screen trait
pub trait DashboardScreen {
    fn show_dashboard(&mut self, ui: &mut Ui);
}

impl DashboardScreen for CrustyApp {
    fn show_dashboard(&mut self, ui: &mut Ui) {
        // Add the action bar at the top
        TopBottomPanel::top("action_bar_panel").show_inside(ui, |ui| {
            ui.add_space(5.0);
            self.show_action_bar(ui);
            ui.add_space(5.0);
        });
        
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading(RichText::new("CRUSTy Dashboard").size(24.0));
            ui.label("Secure file encryption with AES-256-GCM");
            ui.add_space(20.0);
            
            // Main actions section
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    ui.heading("Encryption");
                    ui.add_space(5.0);
                    ui.label("Encrypt files with AES-256-GCM");
                    ui.add_space(10.0);
                    
                    if ui.add_sized(
                        [200.0, 40.0],
                        Button::new(RichText::new("🔒 Encrypt Files").color(self.theme.button_text))
                            .fill(self.theme.accent)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.operation = FileOperation::Encrypt;
                        self.state = AppState::EncryptionWorkflow;
                        self.encryption_workflow_step = crate::gui::app_state::EncryptionWorkflowStep::Files;
                        self.encryption_workflow_complete = false;
                        self.show_status("Starting encryption workflow");
                    }
                });
                
                ui.add_space(40.0);
                
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    ui.heading("Decryption");
                    ui.add_space(5.0);
                    ui.label("Decrypt previously encrypted files");
                    ui.add_space(10.0);
                    
                    if ui.add_sized(
                        [200.0, 40.0],
                        Button::new(RichText::new("🔓 Decrypt Files").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.operation = FileOperation::Decrypt;
                        self.state = AppState::Decrypting;
                        self.show_status("Starting decryption");
                    }
                });
            });
            
            ui.add_space(40.0);
            
            // Use the enhanced file list
            self.show_enhanced_file_list(ui);
            
            ui.add_space(10.0);
            
            // Switch to main screen button
            if ui.add_sized(
                [200.0, 35.0],
                Button::new(RichText::new("Go to Main Screen").color(self.theme.button_text))
                    .fill(self.theme.button_normal)
                    .rounding(Rounding::same(8.0))
            ).clicked() {
                self.state = AppState::MainScreen;
                self.show_status("Switched to main screen");
            }
        });
    }
}
