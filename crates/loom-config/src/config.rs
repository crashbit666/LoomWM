//! Main configuration struct

use crate::{ConfigError, Result, keybindings::Keybinding, theme::Theme};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use tracing::{debug, info};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// Canvas settings
    #[serde(default)]
    pub canvas: CanvasConfig,

    /// AI settings
    #[serde(default)]
    pub ai: AiConfig,

    /// Theme settings
    #[serde(default)]
    pub theme: Theme,

    /// Keybindings
    #[serde(default)]
    pub keybindings: Vec<Keybinding>,
}

// Manual Debug impl to avoid leaking sensitive data in logs
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("general", &self.general)
            .field("canvas", &self.canvas)
            .field("ai", &self.ai)
            .field("theme", &"[...]")
            .field(
                "keybindings",
                &format!("[{} bindings]", self.keybindings.len()),
            )
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Enable debug logging
    #[serde(default)]
    pub debug: bool,

    /// Default terminal application
    #[serde(default = "default_terminal")]
    pub terminal: String,

    /// Default launcher command
    #[serde(default)]
    pub launcher: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasConfig {
    /// Initial zoom level
    #[serde(default = "default_zoom")]
    pub initial_zoom: f64,

    /// Zoom sensitivity (mouse wheel)
    #[serde(default = "default_zoom_sensitivity")]
    pub zoom_sensitivity: f64,

    /// Pan sensitivity
    #[serde(default = "default_pan_sensitivity")]
    pub pan_sensitivity: f64,

    /// Show grid in background
    #[serde(default = "default_true")]
    pub show_grid: bool,

    /// Grid spacing in pixels
    #[serde(default = "default_grid_spacing")]
    pub grid_spacing: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Enable AI features
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// AI service URL (for remote LLM)
    #[serde(default)]
    pub service_url: Option<String>,

    /// API key for AI service (WARNING: stored in plaintext, prefer env var LOOM_AI_API_KEY)
    #[serde(default)]
    pub api_key: Option<String>,

    /// Use local model instead of remote
    #[serde(default)]
    pub use_local: bool,

    /// Local model path
    #[serde(default)]
    pub local_model_path: Option<String>,
}

// Manual Debug impl to redact API key
impl fmt::Debug for AiConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AiConfig")
            .field("enabled", &self.enabled)
            .field("service_url", &self.service_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .field("use_local", &self.use_local)
            .field("local_model_path", &self.local_model_path)
            .finish()
    }
}

impl AiConfig {
    /// Get API key from config or environment variable (env var takes precedence)
    pub fn get_api_key(&self) -> Option<String> {
        std::env::var("LOOM_AI_API_KEY")
            .ok()
            .or_else(|| self.api_key.clone())
    }
}

impl Config {
    /// Load config from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = crate::config_file();

        if config_path.exists() {
            Self::load_from(&config_path)
        } else {
            info!("No config file found, using defaults");
            Ok(Self::default())
        }
    }

    /// Load config from a specific path (must be within config directory)
    pub fn load_from(path: &Path) -> Result<Self> {
        // Security: Validate path is within allowed config directory
        let canonical = path
            .canonicalize()
            .map_err(|e| ConfigError::ReadError(format!("Cannot resolve path: {}", e)))?;

        let config_dir = crate::config_dir();

        // Create config dir if it doesn't exist (for canonicalize to work)
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .map_err(|e| ConfigError::ReadError(e.to_string()))?;
        }

        let canonical_config_dir = config_dir
            .canonicalize()
            .map_err(|e| ConfigError::ReadError(format!("Cannot resolve config dir: {}", e)))?;

        if !canonical.starts_with(&canonical_config_dir) {
            return Err(ConfigError::SecurityViolation(format!(
                "Config file must be within {:?}, got {:?}",
                canonical_config_dir, canonical
            )));
        }

        debug!("Loading config from: {:?}", canonical);

        let content = std::fs::read_to_string(&canonical)
            .map_err(|e| ConfigError::ReadError(e.to_string()))?;

        // Limit config file size to prevent DoS (1MB max)
        const MAX_CONFIG_SIZE: usize = 1024 * 1024;
        if content.len() > MAX_CONFIG_SIZE {
            return Err(ConfigError::SecurityViolation(
                "Config file exceeds maximum size of 1MB".to_string(),
            ));
        }

        toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = crate::config_file();

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigError::ReadError(e.to_string()))?;
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        std::fs::write(&config_path, content).map_err(|e| ConfigError::ReadError(e.to_string()))?;

        info!("Config saved to: {:?}", config_path);
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            canvas: CanvasConfig::default(),
            ai: AiConfig::default(),
            theme: Theme::default(),
            keybindings: Keybinding::defaults(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            debug: false,
            terminal: default_terminal(),
            launcher: None,
        }
    }
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self {
            initial_zoom: default_zoom(),
            zoom_sensitivity: default_zoom_sensitivity(),
            pan_sensitivity: default_pan_sensitivity(),
            show_grid: true,
            grid_spacing: default_grid_spacing(),
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_url: None,
            api_key: None,
            use_local: false,
            local_model_path: None,
        }
    }
}

fn default_terminal() -> String {
    "foot".to_string()
}

fn default_zoom() -> f64 {
    1.0
}

fn default_zoom_sensitivity() -> f64 {
    0.1
}

fn default_pan_sensitivity() -> f64 {
    1.0
}

fn default_grid_spacing() -> f64 {
    50.0
}

fn default_true() -> bool {
    true
}
