/// Application state enum
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Dashboard,
    MainScreen,
    EncryptionWorkflow,
    Encrypting,
    Decrypting,
    KeyManagement,
    Logs,
    About,
}

/// Encryption workflow step enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionWorkflowStep {
    Files,
    Keys,
    Options,
    Execute,
}

impl EncryptionWorkflowStep {
    /// Get the next step in the workflow
    pub fn next(&self) -> Self {
        match self {
            Self::Files => Self::Keys,
            Self::Keys => Self::Options,
            Self::Options => Self::Execute,
            Self::Execute => Self::Execute, // No next step after Execute
        }
    }
    
    /// Get the previous step in the workflow
    pub fn previous(&self) -> Self {
        match self {
            Self::Files => Self::Files, // No previous step before Files
            Self::Keys => Self::Files,
            Self::Options => Self::Keys,
            Self::Execute => Self::Options,
        }
    }
}

impl ToString for EncryptionWorkflowStep {
    fn to_string(&self) -> String {
        match self {
            Self::Files => "Files".to_string(),
            Self::Keys => "Keys".to_string(),
            Self::Options => "Options".to_string(),
            Self::Execute => "Execute".to_string(),
        }
    }
}
