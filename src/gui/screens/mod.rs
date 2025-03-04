// Export all screen modules
pub mod dashboard;
pub mod main_screen;
pub mod about;
pub mod logs;
pub mod key_mgmt;
pub mod encrypt;
pub mod decrypt;
pub mod workflow;

// Re-export screen traits
pub use dashboard::DashboardScreen;
pub use main_screen::MainScreen;
pub use about::AboutScreen;
pub use logs::LogsScreen;
pub use key_mgmt::KeyManagementScreen;
pub use encrypt::EncryptScreen;
pub use decrypt::DecryptScreen;
pub use workflow::EncryptionWorkflowScreen;
