use eframe::egui::Ui;

use crate::gui::CrustyApp;

/// Extension trait for CrustyApp to add split-key functionality
pub trait SplitKeyGui {
    /// Show the split-key management UI
    fn show_split_key_management(&mut self, ui: &mut Ui);
}

impl SplitKeyGui for CrustyApp {
    fn show_split_key_management(&mut self, ui: &mut Ui) {
        // The implementation is in gui_impl.rs
        // This is just a wrapper to satisfy the trait
        self.show_split_key_management_impl(ui);
    }
}
