use std::path::PathBuf;
use std::time::{SystemTime, Duration};
use eframe::egui::{Color32, Ui, RichText, Button, Rounding, ScrollArea};

use crate::gui::theme::AppTheme;

// File status enum for the list-based design
#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Pending,
    InProgress(f32), // Progress percentage (0.0 - 1.0)
    Completed,
    Failed,
}

impl FileStatus {
    // Get a visual representation of the progress
    pub fn progress_bar(&self, width: f32) -> String {
        match self {
            FileStatus::InProgress(progress) => {
                let filled_chars = (progress * width) as usize;
                let empty_chars = width as usize - filled_chars;
                let filled = "█".repeat(filled_chars);
                let empty = "░".repeat(empty_chars);
                format!("{}{}", filled, empty)
            },
            _ => "".to_string(),
        }
    }
}

// File operation type enum
#[derive(Debug, Clone, PartialEq)]
pub enum FileOperationType {
    Encrypt,
    Decrypt,
    None,
}

// File entry struct for the list-based design
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub status: FileStatus,
    pub result: Option<String>,
    pub error: Option<String>,
    pub timestamp: SystemTime,
    pub operation_type: FileOperationType,
    pub file_size: Option<u64>,
}

impl FileEntry {
    pub fn new(path: PathBuf, operation_type: FileOperationType) -> Self {
        // Try to get file size
        let file_size = std::fs::metadata(&path).ok().map(|m| m.len());
        
        FileEntry {
            path,
            status: FileStatus::Pending,
            result: None,
            error: None,
            timestamp: SystemTime::now(),
            operation_type,
            file_size,
        }
    }
    
    pub fn file_name(&self) -> String {
        self.path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown file".to_string())
    }
    
    pub fn file_size_text(&self) -> String {
        match self.file_size {
            Some(size) => {
                if size < 1024 {
                    format!("{} B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else if size < 1024 * 1024 * 1024 {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                } else {
                    format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
                }
            },
            None => "--".to_string(),
        }
    }
    
    pub fn set_progress(&mut self, progress: f32) {
        self.status = FileStatus::InProgress(progress);
    }
    
    pub fn set_completed(&mut self, result: String) {
        self.status = FileStatus::Completed;
        self.result = Some(result);
        self.timestamp = SystemTime::now();
    }
    
    pub fn set_failed(&mut self, error: String) {
        self.status = FileStatus::Failed;
        self.error = Some(error);
        self.timestamp = SystemTime::now();
    }
    
    pub fn status_text(&self) -> String {
        match &self.status {
            FileStatus::Pending => "Pending".to_string(),
            FileStatus::InProgress(progress) => format!("In Progress: {:.1}%", progress * 100.0),
            FileStatus::Completed => "Completed".to_string(),
            FileStatus::Failed => "Failed".to_string(),
        }
    }
    
    pub fn status_color(&self, theme: &AppTheme) -> Color32 {
        match &self.status {
            FileStatus::Pending => theme.text_secondary,
            FileStatus::InProgress(_) => theme.button_normal,
            FileStatus::Completed => theme.success,
            FileStatus::Failed => theme.error,
        }
    }
    
    pub fn elapsed_time(&self) -> Option<Duration> {
        SystemTime::now().duration_since(self.timestamp).ok()
    }
    
    pub fn elapsed_text(&self) -> String {
        if let Some(duration) = self.elapsed_time() {
            let seconds = duration.as_secs();
            if seconds < 60 {
                format!("{} seconds ago", seconds)
            } else if seconds < 3600 {
                format!("{} minutes ago", seconds / 60)
            } else {
                format!("{} hours ago", seconds / 3600)
            }
        } else {
            "Just now".to_string()
        }
    }
    
    pub fn operation_text(&self) -> String {
        match self.operation_type {
            FileOperationType::Encrypt => "Encrypt".to_string(),
            FileOperationType::Decrypt => "Decrypt".to_string(),
            FileOperationType::None => "".to_string(),
        }
    }
    
    pub fn algorithm_text(&self) -> String {
        // For now, we only support AES-256-GCM
        "AES-256".to_string()
    }
}

// Enhanced file list trait
pub trait EnhancedFileList {
    fn show_enhanced_file_list(&mut self, ui: &mut Ui);
}

impl<T> EnhancedFileList for T 
where 
    T: AsMut<Vec<FileEntry>> + AsRef<AppTheme>
{
    fn show_enhanced_file_list(&mut self, ui: &mut Ui) {
        let file_entries = self.as_mut();
        let theme = self.as_ref();
        
        ui.group(|ui| {
            ui.heading("File List");
            
            // Column headers
            ui.horizontal(|ui| {
                ui.label(RichText::new("File").strong()).min_width(200.0);
                ui.label(RichText::new("Size").strong()).min_width(80.0);
                ui.label(RichText::new("Status").strong()).min_width(100.0);
                ui.label(RichText::new("Algorithm").strong()).min_width(80.0);
                ui.label(RichText::new("Date").strong()).min_width(100.0);
                ui.label(RichText::new("Actions").strong()).min_width(100.0);
            });
            
            ui.separator();
            
            // File entries
            if file_entries.is_empty() {
                ui.label("No files in the list. Use the Open button to select files.");
            } else {
                ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    let mut entry_to_remove = None;
                    
                    for (i, entry) in file_entries.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // File name
                            ui.label(&entry.file_name()).min_width(200.0);
                            
                            // File size
                            ui.label(&entry.file_size_text()).min_width(80.0);
                            
                            // Status with color
                            ui.label(
                                RichText::new(entry.status_text())
                                .color(entry.status_color(theme))
                            ).min_width(100.0);
                            
                            // Algorithm
                            ui.label(&entry.algorithm_text()).min_width(80.0);
                            
                            // Date
                            ui.label(entry.elapsed_text()).min_width(100.0);
                            
                            // Actions
                            if ui.add(Button::new(RichText::new("❌").color(theme.button_text))
                                .fill(theme.error)
                                .rounding(Rounding::same(5.0))
                            ).clicked() {
                                entry_to_remove = Some(i);
                            }
                        });
                        
                        // Show progress bar for in-progress files
                        if let FileStatus::InProgress(progress) = entry.status {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label(format!("[{}] {:.1}%", 
                                    FileStatus::InProgress(progress).progress_bar(20.0), 
                                    progress * 100.0
                                ));
                            });
                        }
                    }
                    
                    // Handle removal outside the loop
                    if let Some(index) = entry_to_remove {
                        file_entries.remove(index);
                    }
                });
            }
            
            // Bottom controls for file list
            ui.horizontal(|ui| {
                ui.label(format!("Total: {} file(s)", file_entries.len()));
                
                if !file_entries.is_empty() {
                    if ui.add(Button::new(RichText::new("Clear All").color(theme.button_text))
                        .fill(theme.button_normal)
                        .rounding(Rounding::same(5.0))
                    ).clicked() {
                        file_entries.clear();
                    }
                }
            });
        });
    }
}
