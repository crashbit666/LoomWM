//! LoomWM Protocol Extensions
//!
//! Custom Wayland protocols that allow applications to integrate
//! deeply with LoomWM's node-based canvas system.
//!
//! Protocols:
//! - loom_node_v1: Allows apps to declare themselves as nodes
//! - loom_canvas_v1: Allows apps to query canvas state
//! - loom_intent_v1: Allows apps to send intents to the AI system

pub mod node_protocol;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Protocol not supported: {0}")]
    NotSupported(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;

// Protocol version
pub const LOOM_PROTOCOL_VERSION: u32 = 1;
