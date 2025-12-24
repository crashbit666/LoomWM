//! LoomWM Configuration
//!
//! Handles all configuration:
//! - User preferences
//! - Keybindings
//! - Theme settings
//! - AI service configuration

pub mod config;
pub mod keybindings;
pub mod theme;

pub use config::Config;
pub use keybindings::{Keybinding, KeybindingAction};
pub use theme::Theme;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(String),

    #[error("Failed to parse config: {0}")]
    ParseError(String),

    #[error("Invalid keybinding: {0}")]
    InvalidKeybinding(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

/// Get the config directory path
pub fn config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("loom-wm")
}

/// Get the default config file path
pub fn config_file() -> std::path::PathBuf {
    config_dir().join("config.toml")
}
