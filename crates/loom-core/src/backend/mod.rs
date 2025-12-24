//! Backend initialization for different environments
//!
//! Supports:
//! - DRM/KMS for real hardware (TTY) - enabled with `drm` feature
//! - Winit for development (nested in X11/Wayland) - enabled with `winit` feature

use crate::{CoreError, Result};

#[cfg(any(feature = "backend-drm", feature = "backend-winit"))]
use tracing::info;

#[cfg(feature = "backend-drm")]
pub mod drm;

#[cfg(feature = "backend-winit")]
pub mod winit;

/// Available backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Direct Rendering Manager - for real hardware
    #[cfg(feature = "backend-drm")]
    Drm,
    /// Winit - for development/testing in a window
    #[cfg(feature = "backend-winit")]
    Winit,
}

impl BackendType {
    /// Auto-detect the best backend for the current environment
    pub fn autodetect() -> Result<Self> {
        // If we're running inside an existing display server, prefer Winit
        #[cfg(feature = "backend-winit")]
        if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
            info!("Detected existing display server, using Winit backend");
            return Ok(BackendType::Winit);
        }

        // Otherwise, use DRM for direct hardware access
        #[cfg(feature = "backend-drm")]
        {
            info!("No display server detected, using DRM backend");
            return Ok(BackendType::Drm);
        }

        // No backend available
        #[allow(unreachable_code)]
        Err(CoreError::NoBackendAvailable)
    }
}

/// Run the compositor with the specified backend
pub fn run(backend: BackendType) -> Result<()> {
    match backend {
        #[cfg(feature = "backend-drm")]
        BackendType::Drm => drm::run(),

        #[cfg(feature = "backend-winit")]
        BackendType::Winit => winit::run(),
    }
}

/// Run the compositor with auto-detected backend
pub fn run_auto() -> Result<()> {
    let backend = BackendType::autodetect()?;
    run(backend)
}
