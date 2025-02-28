/// Logger module for tracking encryption and decryption operations.
///
/// This module provides functionality for:
/// - Logging successful and failed operations
/// - Storing logs in a JSON format
/// - Retrieving log entries for display in the UI
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::Local;
use serde::{Serialize, Deserialize};

/// Structure representing a single log entry
#[derive(Serialize, Deserialize, Clone)]
pub struct LogEntry {
    /// Timestamp when the log entry was created
    pub timestamp: String,
    /// Type of operation (e.g., "Encrypt", "Decrypt", "Load Key")
    pub operation: String,
    /// Path of the file that was processed
    pub file_path: String,
    /// Whether the operation was successful
    pub success: bool,
    /// Detailed message about the operation
    pub message: String,
}

impl LogEntry {
    /// Create a new log entry
    ///
    /// # Arguments
    /// * `operation` - Type of operation
    /// * `file_path` - Path of the file that was processed
    /// * `success` - Whether the operation was successful
    /// * `message` - Detailed message about the operation
    pub fn new(operation: &str, file_path: &str, success: bool, message: &str) -> Self {
        LogEntry {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            operation: operation.to_string(),
            file_path: file_path.to_string(),
            success,
            message: message.to_string(),
        }
    }
}

/// Logger implementation for tracking operations
#[derive(Clone)]
pub struct Logger {
    /// File handle for writing logs
    log_file: Arc<Mutex<File>>,
    /// In-memory cache of log entries
    entries: Arc<Mutex<Vec<LogEntry>>>,
}

impl Logger {
    /// Create a new logger that writes to the specified file
    ///
    /// # Arguments
    /// * `log_path` - Path to the log file
    ///
    /// # Returns
    /// * `io::Result<Logger>` - A new logger instance or an error
    pub fn new(log_path: &Path) -> io::Result<Self> {
        // Create log directory if it doesn't exist
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Open log file (create if it doesn't exist, append if it does)
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
            
        Ok(Logger {
            log_file: Arc::new(Mutex::new(file)),
            entries: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    /// Log an operation
    ///
    /// # Arguments
    /// * `entry` - The log entry to record
    ///
    /// # Returns
    /// * `io::Result<()>` - Success or an error
    pub fn log(&self, entry: LogEntry) -> io::Result<()> {
        // Add log entry to memory cache
        {
            let mut entries = self.entries.lock().unwrap();
            entries.push(entry.clone());
        }
        
        // Write log entry to file
        let json = serde_json::to_string(&entry)?;
        let mut file = self.log_file.lock().unwrap();
        writeln!(file, "{}", json)?;
        file.flush()?;
        
        Ok(())
    }
    
    /// Get all log entries
    ///
    /// # Returns
    /// * `Vec<LogEntry>` - All log entries in memory
    pub fn get_entries(&self) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.clone()
    }
    
    /// Log a successful operation
    ///
    /// # Arguments
    /// * `operation` - Type of operation
    /// * `file_path` - Path of the file that was processed
    /// * `message` - Detailed message about the operation
    ///
    /// # Returns
    /// * `io::Result<()>` - Success or an error
    pub fn log_success(&self, operation: &str, file_path: &str, message: &str) -> io::Result<()> {
        self.log(LogEntry::new(operation, file_path, true, message))
    }
    
    /// Log a failed operation
    ///
    /// # Arguments
    /// * `operation` - Type of operation
    /// * `file_path` - Path of the file that was processed
    /// * `error` - Error message
    ///
    /// # Returns
    /// * `io::Result<()>` - Success or an error
    pub fn log_error(&self, operation: &str, file_path: &str, error: &str) -> io::Result<()> {
        self.log(LogEntry::new(operation, file_path, false, error))
    }
}

// Create a singleton logger for the application
lazy_static::lazy_static! {
    static ref APP_LOGGER: Mutex<Option<Logger>> = Mutex::new(None);
}

/// Initialize the global logger
///
/// # Arguments
/// * `log_path` - Path to the log file
///
/// # Returns
/// * `io::Result<()>` - Success or an error
pub fn init_logger(log_path: &Path) -> io::Result<()> {
    let logger = Logger::new(log_path)?;
    let mut app_logger = APP_LOGGER.lock().unwrap();
    *app_logger = Some(logger);
    Ok(())
}

/// Get the global logger
///
/// # Returns
/// * `Option<Arc<Logger>>` - The global logger or None if not initialized
pub fn get_logger() -> Option<Arc<Logger>> {
    let app_logger = APP_LOGGER.lock().unwrap();
    app_logger.as_ref().map(|logger| Arc::new(logger.clone()))
}