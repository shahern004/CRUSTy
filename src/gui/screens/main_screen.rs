use eframe::egui::{Ui, RichText, Button, Rounding, ScrollArea, ComboBox, Label, TopBottomPanel};
use crate::gui::app_core::CrustyApp;
use crate::gui::file_list::{FileOperationType, EnhancedFileList};
use crate::gui::action_bar::ActionBar;
use std::path::PathBuf;

/// Main screen trait
pub trait MainScreen {
    fn show_main_screen(&mut self, ui: &mut Ui);
}

impl MainScreen for CrustyApp {
    fn show_main_screen(&mut self, ui: &mut Ui) {
        // Add the action bar at the top
        TopBottomPanel::top("action_bar_panel").show_inside(ui, |ui| {
            ui.add_space(5.0);
            self.show_action_bar(ui);
            ui.add_space(5.0);
        });
        
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            
            // Tabs for Recent Files and Secured Folders
            ui.horizontal(|ui| {
                if ui.selectable_label(true, "Recent Files").clicked() {
                    // Already on Recent Files tab
                }
                if ui.selectable_label(false, "Secured Folders").clicked() {
                    // Switch to Secured Folders tab (not implemented yet)
                }
            });
            
            ui.separator();
            
            // Operation mode selection (moved to a more compact area)
            ui.horizontal(|ui| {
                ui.label("Processing Mode:");
                ui.radio_value(&mut self.batch_mode, false, "Single File");
                ui.radio_value(&mut self.batch_mode, true, "Multiple Files");
                
                ui.separator();
                
                if ui.add_sized(
                    [150.0, 24.0], 
                    Button::new(RichText::new("Select Output Directory").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(5.0))
                ).clicked() {
                    self.select_output_dir();
                }
            });
            
            ui.add_space(5.0);
            
            // Display selected files
            if !self.selected_files.is_empty() {
                ui.group(|ui| {
                    ui.heading("Selected Files");
                    
                    let mut file_to_remove = None;
                    
                    ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
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
            
            // Use the enhanced file list
            self.show_enhanced_file_list(ui);
            
            // Key selection in a more compact form
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Current Key:");
                        
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
                        
                        ui.add_sized(
                            [150.0, 24.0],
                            Label::new(
                                RichText::new(&current_key_name)
                                    .color(if self.current_key.is_some() { self.theme.success } else { self.theme.error })
                                    .strong()
                            )
                        );
                        
                        // Dropdown for key selection
                        let mut selected_key_index = None;
                        let key_names: Vec<String> = self.saved_keys.iter()
                            .map(|(name, _)| name.clone())
                            .collect();
                        
                        ComboBox::from_label("Select")
                            .selected_text(&current_key_name)
                            .width(150.0)
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
                        
                        // Handle key selection
                        if let Some(idx) = selected_key_index {
                            if idx < self.saved_keys.len() {
                                let (_, key) = &self.saved_keys[idx];
                                self.current_key = Some(key.clone());
                                self.show_status(&format!("Selected key: {}", key_names[idx]));
                            }
                        }
                        
                        if ui.add_sized(
                            [100.0, 24.0],
                            Button::new(RichText::new("New Key").color(self.theme.button_text))
                                .fill(self.theme.button_normal)
                                .rounding(Rounding::same(5.0))
                        ).clicked() {
                            self.new_key_name = format!("Key {}", self.saved_keys.len() + 1);
                            let key_name = self.new_key_name.clone();
                            self.generate_key(&key_name);
                            self.new_key_name.clear();
                        }
                    });
                });
            });
        });
    }
}
