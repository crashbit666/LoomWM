//! LoomWM Core - Wayland compositor built on Smithay
//!
//! This crate handles the low-level compositor functionality:
//! - DRM/KMS backend for direct rendering
//! - libinput for input devices
//! - Wayland protocol handling
//! - Surface management
//!
//! # Security
//!
//! This crate follows security-by-default principles:
//! - Resource limits prevent DoS attacks (see [`security`] module)
//! - No unsafe code without explicit safety documentation
//! - Input validation at all system boundaries

pub mod backend;
pub mod compositor;
mod handlers;
pub mod input;
pub mod security;
pub mod state;

pub use compositor::Compositor;
pub use state::LoomState;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Failed to initialize backend: {0}")]
    BackendInit(String),

    #[error("Failed to create renderer: {0}")]
    Renderer(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Input error: {0}")]
    Input(String),

    #[error("No backend available - compile with 'drm' or 'winit' feature")]
    NoBackendAvailable,

    #[error("Event loop error: {0}")]
    EventLoop(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
