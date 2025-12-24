//! Keybinding configuration
//!
//! Security note: Command execution is restricted to prevent arbitrary code execution.
//! Only predefined actions and allowlisted applications are permitted.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    /// Key combination (e.g., "Super+Return", "Super+Shift+Q")
    pub key: String,
    /// Action to perform
    pub action: KeybindingAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KeybindingAction {
    /// Open terminal (uses config.general.terminal)
    Terminal,
    /// Open launcher/command palette (uses config.general.launcher)
    Launcher,
    /// Close focused node
    Close,
    /// Toggle fullscreen for focused node
    Fullscreen,
    /// Pan the canvas
    Pan { direction: Direction },
    /// Zoom in/out
    Zoom { direction: ZoomDirection },
    /// Reset view to origin
    ResetView,
    /// Open AI command input
    AiPrompt,
    /// Launch a desktop application by its .desktop file name (safe)
    /// Example: "firefox", "org.gnome.Calculator"
    LaunchApp { app_id: String },
    /// Run a script from ~/.config/loom-wm/scripts/ (restricted)
    RunScript { script_name: String },
    /// Quit the compositor
    Quit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoomDirection {
    In,
    Out,
}

impl Keybinding {
    pub fn new(key: impl Into<String>, action: KeybindingAction) -> Self {
        Self {
            key: key.into(),
            action,
        }
    }

    /// Default keybindings
    pub fn defaults() -> Vec<Self> {
        vec![
            Self::new("Super+Return", KeybindingAction::Terminal),
            Self::new("Super+D", KeybindingAction::Launcher),
            Self::new("Super+Q", KeybindingAction::Close),
            Self::new("Super+F", KeybindingAction::Fullscreen),
            Self::new("Super+Space", KeybindingAction::AiPrompt),
            Self::new("Super+0", KeybindingAction::ResetView),
            Self::new(
                "Super+Plus",
                KeybindingAction::Zoom {
                    direction: ZoomDirection::In,
                },
            ),
            Self::new(
                "Super+Minus",
                KeybindingAction::Zoom {
                    direction: ZoomDirection::Out,
                },
            ),
            Self::new("Super+Shift+Q", KeybindingAction::Quit),
        ]
    }
}

/// Security module for validating and executing actions safely
pub mod security {
    use crate::{ConfigError, Result};
    use std::path::PathBuf;

    /// Validate a script name (must be alphanumeric with underscores/hyphens, no path separators)
    pub fn validate_script_name(name: &str) -> Result<()> {
        // Must not be empty
        if name.is_empty() {
            return Err(ConfigError::SecurityViolation(
                "Script name cannot be empty".to_string(),
            ));
        }

        // Must not contain path separators or special characters
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err(ConfigError::SecurityViolation(
                "Script name contains invalid characters".to_string(),
            ));
        }

        // Only allow alphanumeric, underscore, hyphen, and dot (for extension)
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Err(ConfigError::SecurityViolation(
                "Script name contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the full path to a user script (validated)
    pub fn get_script_path(script_name: &str) -> Result<PathBuf> {
        validate_script_name(script_name)?;

        let scripts_dir = crate::config_dir().join("scripts");
        let script_path = scripts_dir.join(script_name);

        // Ensure the path doesn't escape the scripts directory
        let canonical_scripts = scripts_dir.canonicalize().map_err(|_| {
            ConfigError::SecurityViolation("Scripts directory not found".to_string())
        })?;

        let canonical_script = script_path.canonicalize().map_err(|_| {
            ConfigError::SecurityViolation(format!("Script not found: {}", script_name))
        })?;

        if !canonical_script.starts_with(&canonical_scripts) {
            return Err(ConfigError::SecurityViolation(
                "Script path escapes scripts directory".to_string(),
            ));
        }

        // Check script is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&canonical_script)
                .map_err(|e| ConfigError::ReadError(e.to_string()))?;
            if metadata.permissions().mode() & 0o111 == 0 {
                return Err(ConfigError::SecurityViolation(
                    "Script is not executable".to_string(),
                ));
            }
        }

        Ok(canonical_script)
    }

    /// Validate an app_id (must be a valid desktop file identifier)
    pub fn validate_app_id(app_id: &str) -> Result<()> {
        if app_id.is_empty() {
            return Err(ConfigError::SecurityViolation(
                "App ID cannot be empty".to_string(),
            ));
        }

        // Must not contain path separators
        if app_id.contains('/') || app_id.contains('\\') {
            return Err(ConfigError::SecurityViolation(
                "App ID contains invalid characters".to_string(),
            ));
        }

        // Only allow alphanumeric, dots, and hyphens (standard for desktop file names)
        if !app_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            return Err(ConfigError::SecurityViolation(
                "App ID contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::security::*;

    #[test]
    fn test_valid_script_names() {
        assert!(validate_script_name("my-script.sh").is_ok());
        assert!(validate_script_name("script_name").is_ok());
        assert!(validate_script_name("Script123").is_ok());
    }

    #[test]
    fn test_invalid_script_names() {
        assert!(validate_script_name("").is_err());
        assert!(validate_script_name("../evil").is_err());
        assert!(validate_script_name("/etc/passwd").is_err());
        assert!(validate_script_name("script;rm -rf /").is_err());
        assert!(validate_script_name("script`whoami`").is_err());
    }

    #[test]
    fn test_valid_app_ids() {
        assert!(validate_app_id("firefox").is_ok());
        assert!(validate_app_id("org.gnome.Calculator").is_ok());
        assert!(validate_app_id("my-app").is_ok());
    }

    #[test]
    fn test_invalid_app_ids() {
        assert!(validate_app_id("").is_err());
        assert!(validate_app_id("/usr/bin/evil").is_err());
        assert!(validate_app_id("app;rm -rf /").is_err());
    }
}
