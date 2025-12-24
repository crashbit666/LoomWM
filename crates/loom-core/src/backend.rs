//! Backend initialization for different environments
//!
//! Supports:
//! - DRM/KMS for real hardware (TTY)
//! - Winit for development (nested in X11/Wayland)

use crate::Result;
use tracing::info;

pub enum Backend {
    /// Direct Rendering Manager - for real hardware
    Drm,
    /// Winit - for development/testing
    Winit,
}

impl Backend {
    pub fn autodetect() -> Result<Self> {
        // If we have a display, use winit for development
        if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
            info!("Detected existing display server, using Winit backend");
            Ok(Backend::Winit)
        } else {
            info!("No display server detected, using DRM backend");
            Ok(Backend::Drm)
        }
    }
}
