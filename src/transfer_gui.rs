use eframe::egui;
use egui::{Ui, Button, RichText, Rounding, TextEdit, Label};
use std::path::{Path, PathBuf};

use crate::encryption::EncryptionKey;
use crate::split_key::{SplitEncryptionKey, KeyShareManager, TransferPackage, ShareFormat, KeyPurpose, SplitKeyError};
use crate::gui::CrustyApp;

/// Extension trait for CrustyApp to add transfer functionality
pub trait TransferGui {
    /// Show the transfer preparation UI
    fn show_transfer_preparation(&mut self, ui: &mut Ui);
    
    /// Show the transfer receive UI
    fn show_transfer_receive(&mut self, ui: &mut Ui);
}

impl TransferGui for CrustyApp {
    fn show_transfer_preparation(&mut self, ui: &mut Ui) {
        // The implementation is in gui_impl.rs
        // This is just a wrapper to satisfy the trait
        self.show_transfer_preparation_impl(ui);
    }
    
    fn show_transfer_receive(&mut self, ui: &mut Ui) {
        // The implementation is in gui_impl.rs
        // This is just a wrapper to satisfy the trait
        self.show_transfer_receive_impl(ui);
    }
}

/// Transfer state for the GUI
#[derive(Debug, Clone, PartialEq)]
pub enum TransferState {
    /// Initial state
    Initial,
    /// Creating transfer package
    Creating,
    /// Transfer package created
    Created,
    /// Saving shares
    SavingShares,
    /// Shares saved
    SharesSaved,
    /// Error state
    Error(String),
}

/// Transfer receive state for the GUI
#[derive(Debug, Clone, PartialEq)]
pub enum TransferReceiveState {
    /// Initial state
    Initial,
    /// Entering shares
    EnteringShares,
    /// Reconstructing key
    Reconstructing,
    /// Key reconstructed
    Reconstructed,
    /// Error state
    Error(String),
}
