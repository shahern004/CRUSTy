use eframe::egui;
use egui::{Ui, Button, RichText, Rounding, TextEdit, ScrollArea};
use std::path::{Path, PathBuf};

use crate::encryption::EncryptionKey;
use crate::split_key::{SplitEncryptionKey, KeyShareManager, SplitKeyError, ShareFormat, KeyPurpose, TransferPackage};
use crate::gui::CrustyApp;
use crate::transfer_gui::{TransferState, TransferReceiveState};

/// Implementation of split-key and transfer functionality for CrustyApp
impl CrustyApp {
    /// Show the split-key management UI implementation
    pub fn show_split_key_management_impl(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading("Split-Key Management");
            ui.add_space(20.0);
            
            // Explanation of split-key functionality
            ui.group(|ui| {
                ui.heading("About Split-Key Encryption");
                ui.label("Split-key encryption divides your encryption key into multiple shares.");
                ui.label("You need a minimum number of shares to reconstruct the key.");
                ui.label("This provides enhanced security through multi-party authorization.");
                
                ui.add_space(10.0);
                ui.label("CRUSTy uses a 2-of-3 scheme:");
                ui.label("• Primary Share: Stored in your OS credential store");
                ui.label("• Secondary Share: Stored as a file in a location you choose");
                ui.label("• Recovery Share: Generated as a QR code for you to print or save");
                
                ui.add_space(10.0);
                ui.label("You need any 2 of these 3 shares to decrypt your files.");
            });
            
            ui.add_space(20.0);
            
            // Create split key section
            ui.group(|ui| {
                ui.heading("Create Split Key");
                
                if self.current_key.is_none() {
                    ui.label(RichText::new("You need to select or create a key first").color(self.theme.error));
                } else {
                    if ui.add_sized(
                        [220.0, 40.0],
                        Button::new(RichText::new("Create Split Key").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        match self.create_split_key() {
                            Ok(split_key) => {
                                match self.store_split_key(&split_key) {
                                    Ok(_) => {
                                        self.show_status("Split key created and stored successfully");
                                    },
                                    Err(e) => {
                                        self.show_error(&format!("Failed to store split key: {}", e));
                                    }
                                }
                            },
                            Err(e) => {
                                self.show_error(&format!("Failed to create split key: {}", e));
                            }
                        }
                    }
                }
            });
            
            ui.add_space(20.0);
            
            // Reconstruct key section
            ui.group(|ui| {
                ui.heading("Reconstruct Key from Shares");
                
                ui.label("To reconstruct your key, you need:");
                ui.label("1. Your primary share (automatically retrieved from OS credential store)");
                ui.label("2. Either your secondary share file OR your recovery share QR code");
                
                if ui.add_sized(
                    [220.0, 40.0],
                    Button::new(RichText::new("Select Secondary Share File").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    // This would normally use a native file dialog
                    // For now, we'll just use a placeholder path
                    let secondary_share_path = PathBuf::from("secondary_share.key");
                    
                    match self.reconstruct_key(&secondary_share_path) {
                        Ok(key) => {
                            self.current_key = Some(key.clone());
                            let name = "Reconstructed Key".to_string();
                            self.saved_keys.push((name.clone(), key));
                            self.show_status(&format!("Key '{}' reconstructed and selected", name));
                        },
                        Err(e) => {
                            self.show_error(&format!("Failed to reconstruct key: {}", e));
                        }
                    }
                }
                
                ui.add_space(10.0);
                
                if ui.add_sized(
                    [220.0, 40.0],
                    Button::new(RichText::new("Scan Recovery Share QR Code").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    self.show_status("QR code scanning not implemented in this version");
                }
            });
            
            ui.add_space(20.0);
            
            // Back button
            if ui.add(Button::new(RichText::new("Back to Key Management").color(self.theme.button_text))
                .fill(self.theme.button_normal)
                .rounding(Rounding::same(5.0))
            ).clicked() {
                self.state = crate::gui::AppState::KeyManagement;
            }
        });
    }
    
    /// Create a split key from the current key
    pub fn create_split_key(&mut self) -> Result<SplitEncryptionKey, SplitKeyError> {
        if let Some(key) = &self.current_key {
            // Create a split key with threshold 2 and 3 shares
            SplitEncryptionKey::new(key, 2, 3, KeyPurpose::Standard)
        } else {
            Err(SplitKeyError::Key("No key selected".to_string()))
        }
    }
    
    /// Store a split key
    pub fn store_split_key(&mut self, split_key: &SplitEncryptionKey) -> Result<(), SplitKeyError> {
        // Create a key share manager
        let app_name = "CRUSTy";
        let share_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let share_dir = share_dir.join("crusty").join("shares");
        
        let key_share_manager = KeyShareManager::new(app_name, &share_dir)?;
        
        // Store the primary share in the OS credential store
        key_share_manager.store_primary_share(split_key)?;
        
        // Save the secondary share to a file
        let secondary_share_path = key_share_manager.save_secondary_share(
            split_key,
            "secondary_share.key",
            ShareFormat::Binary
        )?;
        
        // Generate and save a recovery share in text format
        let recovery_share_path = key_share_manager.save_recovery_share(
            split_key,
            "recovery_share.txt",
            ShareFormat::Text
        )?;
        
        // Show paths to the user
        self.show_status(&format!(
            "Secondary share saved to: {}\nRecovery share saved to: {}",
            secondary_share_path.display(),
            recovery_share_path.display()
        ));
        
        Ok(())
    }
    
    /// Show the transfer preparation UI implementation
    pub fn show_transfer_preparation_impl(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading("Prepare for Transfer");
            ui.add_space(20.0);
            
            // Explanation of transfer functionality
            ui.group(|ui| {
                ui.heading("About Secure Transfer");
                ui.label("This feature helps you securely transfer encrypted files to others.");
                ui.label("It creates a special transfer key that is split into multiple shares.");
                ui.label("You send different shares through different channels for security.");
                
                ui.add_space(10.0);
                ui.label("The process works like this:");
                ui.label("1. Select a file to encrypt for transfer");
                ui.label("2. Create a transfer package with multiple key shares");
                ui.label("3. Send the encrypted file through one channel");
                ui.label("4. Send key shares through different channels");
                ui.label("5. The recipient needs the file and enough shares to decrypt");
                
                ui.add_space(10.0);
                ui.label("This provides enhanced security for out-of-band transfers.");
            });
            
            ui.add_space(20.0);
            
            // Create transfer package section
            ui.group(|ui| {
                ui.heading("Create Transfer Package");
                
                if self.current_key.is_none() {
                    ui.label(RichText::new("You need to select or create a key first").color(self.theme.error));
                } else {
                    if ui.add_sized(
                        [220.0, 40.0],
                        Button::new(RichText::new("Create Transfer Package").color(self.theme.button_text))
                            .fill(self.theme.button_normal)
                            .rounding(Rounding::same(8.0))
                    ).clicked() {
                        match self.create_transfer_package() {
                            Ok(package) => {
                                self.transfer_package = Some(package);
                                self.transfer_state = TransferState::Created;
                                self.show_status("Transfer package created successfully");
                            },
                            Err(e) => {
                                self.transfer_state = TransferState::Error(e.to_string());
                                self.show_error(&format!("Failed to create transfer package: {}", e));
                            }
                        }
                    }
                }
            });
            
            ui.add_space(20.0);
            
            // Display shares section (only shown if package is created)
            if self.transfer_state == TransferState::Created || 
               self.transfer_state == TransferState::SharesSaved {
                if let Some(ref package) = self.transfer_package {
                    ui.group(|ui| {
                        ui.heading("Transfer Shares");
                        
                        ui.label(format!("Threshold: {} of {} shares needed", 
                                        package.get_threshold(), 
                                        package.get_shares_count()));
                        
                        ui.add_space(10.0);
                        
                        // Display each share
                        for i in 0..package.get_shares_count() {
                            ui.group(|ui| {
                                ui.heading(format!("Share {}", i + 1));
                                
                                let share_text_result = package.get_share_text(i);
                                let mnemonic_result = package.get_share_mnemonic(i);
                                
                                if let Ok(share_text) = share_text_result {
                                    // Display the share text in a scrollable area
                                    ScrollArea::vertical().max_height(80.0).show(ui, |ui| {
                                        ui.add(TextEdit::multiline(&mut share_text.to_string())
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(3)
                                            .interactive(false));
                                    });
                                    
                                    // Option to save this share
                                    let share_path = dirs::data_dir()
                                        .unwrap_or_else(|| PathBuf::from("."))
                                        .join("crusty")
                                        .join("shares")
                                        .join(format!("transfer_share_{}.txt", i + 1));
                                    
                                    let share_path_str = format!("{}", share_path.display());
                                    let share_index = i;
                                    
                                    if ui.add_sized(
                                        [150.0, 30.0],
                                        Button::new(RichText::new("Save Share").color(self.theme.button_text))
                                            .fill(self.theme.button_normal)
                                            .rounding(Rounding::same(5.0))
                                    ).clicked() {
                                        // This would normally use a native file dialog
                                        // For now, we'll just use a placeholder path
                                        // Save the share to a file
                                        if let Err(e) = package.save_share_to_file(share_index, &share_path) {
                                            // Store the error message to display after the closure
                                            let error_msg = format!("Failed to save share: {}", e);
                                            ui.ctx().request_repaint(); // Request a repaint to show the error
                                            
                                            // Request a repaint to update the UI
                                            ui.ctx().request_repaint();
                                            
                                            // We'll set an error flag that will be checked outside the closure
                                            self.last_error = Some(error_msg);
                                        } else {
                                            // Store success message to display after the closure
                                            let success_msg = format!("Share {} saved to: {}", 
                                                                    share_index + 1, 
                                                                    share_path_str);
                                            
                                            // Request a repaint to update the UI
                                            ui.ctx().request_repaint();
                                            
                                            // We'll set a success flag that will be checked outside the closure
                                            self.last_status = Some(success_msg);
                                            self.transfer_state = TransferState::SharesSaved;
                                        }
                                    }
                                    
                                    // Option to view as mnemonic
                                    if let Ok(mnemonic) = mnemonic_result {
                                        let mnemonic_str = mnemonic.clone();
                                        let share_index = i;
                                        
                                        let mnemonic_button = ui.add_sized(
                                            [150.0, 30.0],
                                            Button::new(RichText::new("View as Mnemonic").color(self.theme.button_text))
                                                .fill(self.theme.button_normal)
                                                .rounding(Rounding::same(5.0))
                                        );
                                        
                                        if mnemonic_button.clicked() {
                                            // Store the mnemonic message to display after the closure
                                            let mnemonic_msg = format!("Share {} mnemonic: {}", share_index + 1, mnemonic_str);
                                            
                                            // Request a repaint to update the UI
                                            ui.ctx().request_repaint();
                                            
                                            // We'll set a success flag that will be checked outside the closure
                                            self.last_status = Some(mnemonic_msg);
                                        }
                                    }
                                } else {
                                    ui.label(RichText::new("Error retrieving share").color(self.theme.error));
                                }
                            });
                        }
                    });
                }
            }
            
            ui.add_space(20.0);
            
            // Back button
            if ui.add(Button::new(RichText::new("Back to Key Management").color(self.theme.button_text))
                .fill(self.theme.button_normal)
                .rounding(Rounding::same(5.0))
            ).clicked() {
                self.state = crate::gui::AppState::KeyManagement;
            }
        });
    }
    
    /// Show the transfer receive UI implementation
    pub fn show_transfer_receive_impl(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.heading("Receive Transfer");
            ui.add_space(20.0);
            
            // Explanation of receive functionality
            ui.group(|ui| {
                ui.heading("About Receiving Transfers");
                ui.label("This feature helps you decrypt files that were sent to you.");
                ui.label("You'll need to enter the key shares you received.");
                ui.label("Once you have enough shares, you can reconstruct the key and decrypt the file.");
                
                ui.add_space(10.0);
                ui.label("The process works like this:");
                ui.label("1. Enter the key shares you received");
                ui.label("2. Reconstruct the encryption key");
                ui.label("3. Use the key to decrypt the file");
            });
            
            ui.add_space(20.0);
            
            // Enter shares section
            ui.group(|ui| {
                ui.heading("Enter Key Shares");
                
                // Share 1 input
                ui.label("Share 1:");
                ui.add(TextEdit::multiline(&mut self.transfer_share1)
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .hint_text("Enter the first key share here..."));
                
                ui.add_space(10.0);
                
                // Share 2 input
                ui.label("Share 2:");
                ui.add(TextEdit::multiline(&mut self.transfer_share2)
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .hint_text("Enter the second key share here..."));
                
                ui.add_space(10.0);
                
                // Option to load from file
                if ui.add_sized(
                    [150.0, 30.0],
                    Button::new(RichText::new("Load Share from File").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(5.0))
                ).clicked() {
                    // This would normally use a native file dialog
                    // For now, we'll just show a message
                    self.show_status("File dialog would open here to select a share file");
                }
                
                ui.add_space(10.0);
                
                // Reconstruct key button
                if ui.add_sized(
                    [220.0, 40.0],
                    Button::new(RichText::new("Reconstruct Key").color(self.theme.button_text))
                        .fill(self.theme.button_normal)
                        .rounding(Rounding::same(8.0))
                ).clicked() {
                    if !self.transfer_share1.is_empty() && !self.transfer_share2.is_empty() {
                        match self.reconstruct_key_from_transfer_shares() {
                            Ok(key) => {
                                self.current_key = Some(key.clone());
                                let name = "Transfer Key".to_string();
                                self.saved_keys.push((name.clone(), key));
                                self.transfer_receive_state = TransferReceiveState::Reconstructed;
                                self.show_status(&format!("Key '{}' reconstructed and selected", name));
                            },
                            Err(e) => {
                                self.transfer_receive_state = TransferReceiveState::Error(e.to_string());
                                self.show_error(&format!("Failed to reconstruct key: {}", e));
                            }
                        }
                    } else {
                        self.show_error("Please enter both key shares");
                    }
                }
            });
            
            ui.add_space(20.0);
            
            // Back button
            if ui.add(Button::new(RichText::new("Back to Key Management").color(self.theme.button_text))
                .fill(self.theme.button_normal)
                .rounding(Rounding::same(5.0))
            ).clicked() {
                self.state = crate::gui::AppState::KeyManagement;
            }
        });
    }
    
    /// Create a transfer package
    pub fn create_transfer_package(&mut self) -> Result<TransferPackage, SplitKeyError> {
        if let Some(key) = &self.current_key {
            // Create a key share manager
            let app_name = "CRUSTy";
            let share_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            let share_dir = share_dir.join("crusty").join("shares");
            
            let key_share_manager = KeyShareManager::new(app_name, &share_dir)?;
            
            // Create a transfer package with threshold 2 and 3 shares
            key_share_manager.create_transfer_package(key, 2, 3)
        } else {
            Err(SplitKeyError::Key("No key selected".to_string()))
        }
    }
    
    /// Reconstruct a key from transfer shares
    pub fn reconstruct_key_from_transfer_shares(&mut self) -> Result<EncryptionKey, SplitKeyError> {
        // Create a key share manager
        let app_name = "CRUSTy";
        let share_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let share_dir = share_dir.join("crusty").join("shares");
        
        let key_share_manager = KeyShareManager::new(app_name, &share_dir)?;
        
        // Reconstruct the key from the provided shares
        let shares = vec![
            self.transfer_share1.clone(),
            self.transfer_share2.clone(),
        ];
        
        key_share_manager.reconstruct_key_from_text_shares(&shares)
    }
    
    /// Reconstruct a key from shares
    pub fn reconstruct_key(&mut self, secondary_share_path: &Path) -> Result<EncryptionKey, SplitKeyError> {
        // Create a key share manager
        let app_name = "CRUSTy";
        let share_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let share_dir = share_dir.join("crusty").join("shares");
        
        let key_share_manager = KeyShareManager::new(app_name, &share_dir)?;
        
        // Reconstruct the key from the primary share and the secondary share
        key_share_manager.reconstruct_key(secondary_share_path)
    }
}
