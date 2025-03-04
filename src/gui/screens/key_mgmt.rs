use eframe::egui::{Ui, RichText, Button, Rounding, TextEdit, Grid};
use crate::gui::app_core::CrustyApp;
use crate::gui::app_state::AppState;

/// Key management screen trait
pub trait KeyManagementScreen {
    fn show_key_management(&mut self, ui: &mut Ui);
}

impl KeyManagementScreen for CrustyApp {
    fn show_key_management(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(RichText::new("Key Management").size(28.0));
            ui.add_space(10.0);
            
            // Create new key section
            ui.group(|ui| {
                ui.heading("Create New Key");
                
                ui.horizontal(|ui| {
                    ui.label("Key Name:");
                    ui.add(TextEdit::singleline(&mut self.new_key_name)
                        .hint_text("Enter a name for the new key")
                        .desired_width(250.0));
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
            
            // Saved keys section
            ui.group(|ui| {
                ui.heading("Saved Keys");
                
                if self.saved_keys.is_empty() {
                    ui.label("No saved keys. Create a new key or load one from a file.");
                } else {
                    // Display current key
                    let current_key_base64 = self.current_key.as_ref().map(|k| k.to_base64());
                    
                    // Create a table for the keys
                    Grid::new("keys_grid")
                        .num_columns(4)
                        .spacing([20.0, 10.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Header row
                            ui.label(RichText::new("Key Name").strong());
                            ui.label(RichText::new("Status").strong());
                            ui.label(RichText::new("Actions").strong());
                            ui.label(RichText::new("").strong());
                            ui.end_row();
                            
                            // Key rows
                            let mut key_to_remove = None;
                            
                    // Create a temporary vector of key data for the grid
                    let key_data: Vec<(usize, String, String, bool)> = self.saved_keys.iter().enumerate()
                        .map(|(i, (name, key))| {
                            let is_current = current_key_base64.as_ref().map_or(false, |current| current == &key.to_base64());
                            (i, name.clone(), key.to_base64(), is_current)
                        })
                        .collect();
                    
                    for (i, name, _key_base64, is_current) in key_data {
                        // Key name
                        ui.label(if is_current {
                            RichText::new(&name).strong().color(self.theme.success)
                        } else {
                            RichText::new(&name)
                        });
                        
                        // Status
                        ui.label(if is_current {
                            RichText::new("Current").color(self.theme.success)
                        } else {
                            RichText::new("Saved")
                        });
                        
                        // Select button
                        ui.horizontal(|ui| {
                            if ui.add_sized(
                                [80.0, 24.0],
                                Button::new(RichText::new("Select").color(self.theme.button_text))
                                    .fill(self.theme.button_normal)
                                    .rounding(Rounding::same(5.0))
                            ).clicked() {
                                if i < self.saved_keys.len() {
                                    let (_, key) = &self.saved_keys[i];
                                    self.current_key = Some(key.clone());
                                    self.show_status(&format!("Selected key: {}", name));
                                }
                            }
                            
                            if ui.add_sized(
                                [80.0, 24.0],
                                Button::new(RichText::new("Save").color(self.theme.button_text))
                                    .fill(self.theme.button_normal)
                                    .rounding(Rounding::same(5.0))
                            ).clicked() {
                                if i < self.saved_keys.len() {
                                    let (_, key) = &self.saved_keys[i];
                                    self.current_key = Some(key.clone());
                                    self.save_key_to_file();
                                }
                            }
                        });
                        
                        // Delete button
                        if ui.add_sized(
                            [80.0, 24.0],
                            Button::new(RichText::new("Delete").color(self.theme.button_text))
                                .fill(self.theme.error)
                                .rounding(Rounding::same(5.0))
                        ).clicked() {
                            key_to_remove = Some(i);
                        }
                        
                        ui.end_row();
                    }
                            
                            // Handle key removal outside the closure
                            if let Some(idx) = key_to_remove {
                                if idx < self.saved_keys.len() {
                                    // Store the name and key_base64 before removing
                                    let name = self.saved_keys[idx].0.clone();
                                    let key_base64 = self.saved_keys[idx].1.to_base64();
                                    
                                    // Remove the key
                                    self.saved_keys.remove(idx);
                                    
                                    // If we removed the current key, clear it
                                    if let Some(current) = &self.current_key {
                                        if current.to_base64() == key_base64 {
                                            self.current_key = None;
                                        }
                                    }
                                    
                                    self.show_status(&format!("Removed key: {}", name));
                                }
                            }
                        });
                }
                
                ui.add_space(10.0);
                
                // Load key from file button
                if ui.add_sized(
                    [150.0, 30.0],
                    Button::new(RichText::new("Load Key from File").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    self.load_key_from_file();
                }
            });
            
            ui.add_space(20.0);
            
            // Advanced key operations
            ui.group(|ui| {
                ui.heading("Advanced Key Operations");
                
                ui.horizontal(|ui| {
                    if ui.add_sized(
                        [180.0, 35.0],
                        Button::new(RichText::new("Split Key Management").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.state = AppState::SplitKeyManagement;
                    }
                    
                    if ui.add_sized(
                        [180.0, 35.0],
                        Button::new(RichText::new("Transfer Preparation").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.state = AppState::TransferPreparation;
                    }
                    
                    if ui.add_sized(
                        [180.0, 35.0],
                        Button::new(RichText::new("Receive Transfer").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        self.state = AppState::TransferReceive;
                    }
                });
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
