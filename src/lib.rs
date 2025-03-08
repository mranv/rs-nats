//! Library module for RS-NATS

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Default NATS server URL
pub const DEFAULT_NATS_URL: &str = "nats://localhost:4222";

/// Default subject prefix for all messages
pub const DEFAULT_SUBJECT_PREFIX: &str = "rs-support";

/// Error types for RS-NATS
#[derive(Error, Debug)]
pub enum RsNatsError {
    #[error("NATS connection error: {0}")]
    ConnectionError(String),
    
    #[error("Command execution error: {0}")]
    CommandError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Operation not supported on this platform")]
    PlatformNotSupported,
}

/// Commands that can be sent to clients
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Command {
    Ping,
    Execute(String),
    GetSystemInfo,
    Shutdown,
    LogEvent { level: LogLevel, message: String },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Ping => write!(f, "Ping"),
            Command::Execute(cmd) => write!(f, "Execute: {}", cmd),
            Command::GetSystemInfo => write!(f, "GetSystemInfo"),
            Command::Shutdown => write!(f, "Shutdown"),
            Command::LogEvent { level, message } => write!(f, "Log [{}]: {}", level, message),
        }
    }
}

/// Log levels for message logging
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// System information for client machines
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub username: String,
    pub os_type: String,
    pub os_version: Option<String>,
}

/// Result of a command execution
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub command_type: CommandType,
}

/// Type of command that was executed
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CommandType {
    Shell,
    Internal,
}

/// Get a unique client ID based on the machine
pub fn get_client_id() -> String {
    // Use fallible version instead of deprecated hostname()
    let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string());
    let username = whoami::username();
    
    format!("{}-{}", username, hostname)
}

/// Get the system's OS type
pub fn get_os_type() -> String {
    if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else if cfg!(target_os = "macos") {
        "macOS".to_string()
    } else {
        "Unknown".to_string()
    }
}