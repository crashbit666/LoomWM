//! LoomWM Canvas - Infinite canvas and node system
//!
//! This is the heart of LoomWM's fluid interface:
//! - Infinite 2D canvas with pan/zoom
//! - Node-based content organization
//! - Connections between nodes
//! - Spatial memory layout

pub mod canvas;
pub mod node;
pub mod connection;
pub mod viewport;

pub use canvas::Canvas;
pub use node::{Node, NodeId, NodeType};
pub use connection::Connection;
pub use viewport::Viewport;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanvasError {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Invalid connection: {0}")]
    InvalidConnection(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}

/// Security limits for canvas resources
pub mod limits {
    /// Maximum number of nodes allowed on the canvas
    pub const MAX_NODES: usize = 10_000;
    /// Maximum number of connections allowed
    pub const MAX_CONNECTIONS: usize = 100_000;
    /// Maximum canvas coordinate (prevents floating point issues)
    pub const MAX_COORDINATE: f64 = 1_000_000.0;
    /// Minimum canvas coordinate
    pub const MIN_COORDINATE: f64 = -1_000_000.0;
}

pub type Result<T> = std::result::Result<T, CanvasError>;
