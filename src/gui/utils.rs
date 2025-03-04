use eframe::egui::{Ui, Button, RichText, Rounding, Response};
use crate::gui::theme::AppTheme;

/// Create a styled button with consistent appearance
pub fn styled_button(ui: &mut Ui, text: &str, theme: &AppTheme, size: Option<[f32; 2]>) -> Response {
    let button = Button::new(RichText::new(text).color(theme.button_text))
        .fill(theme.button_normal)
        .rounding(Rounding::same(5.0));
    
    if let Some(size) = size {
        ui.add_sized(size, button)
    } else {
        ui.add(button)
    }
}

/// Create a styled primary button with consistent appearance
pub fn styled_primary_button(ui: &mut Ui, text: &str, theme: &AppTheme, size: Option<[f32; 2]>) -> Response {
    let button = Button::new(RichText::new(text).color(theme.button_text))
        .fill(theme.accent)
        .rounding(Rounding::same(8.0));
    
    if let Some(size) = size {
        ui.add_sized(size, button)
    } else {
        ui.add(button)
    }
}

/// Create a styled error button with consistent appearance
pub fn styled_error_button(ui: &mut Ui, text: &str, theme: &AppTheme, size: Option<[f32; 2]>) -> Response {
    let button = Button::new(RichText::new(text).color(theme.button_text))
        .fill(theme.error)
        .rounding(Rounding::same(5.0));
    
    if let Some(size) = size {
        ui.add_sized(size, button)
    } else {
        ui.add(button)
    }
}

/// Format a file size in human-readable format
pub fn format_file_size(size_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if size_bytes >= GB {
        format!("{:.2} GB", size_bytes as f64 / GB as f64)
    } else if size_bytes >= MB {
        format!("{:.2} MB", size_bytes as f64 / MB as f64)
    } else if size_bytes >= KB {
        format!("{:.2} KB", size_bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", size_bytes)
    }
}
