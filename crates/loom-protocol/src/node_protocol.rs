//! loom_node_v1 protocol
//!
//! Allows Wayland clients to:
//! - Request to be treated as a node (instead of traditional window)
//! - Set node metadata (label, type, connections)
//! - Receive canvas position updates

use tracing::debug;

/// Node hints that clients can set
#[derive(Debug, Clone, Default)]
pub struct NodeHints {
    /// Suggested label for the node
    pub label: Option<String>,
    /// Whether this node prefers to be grouped with similar content
    pub groupable: bool,
    /// Content type hint for AI classification
    pub content_type: Option<String>,
    /// Suggested connections to other nodes
    pub suggested_connections: Vec<String>,
}

impl NodeHints {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn groupable(mut self) -> Self {
        self.groupable = true;
        self
    }
}

/// Server-side handler for the node protocol
pub struct NodeProtocolHandler {
    // Will integrate with wayland-server
}

impl NodeProtocolHandler {
    pub fn new() -> Self {
        debug!("Initializing loom_node_v1 protocol handler");
        Self {}
    }
}

impl Default for NodeProtocolHandler {
    fn default() -> Self {
        Self::new()
    }
}
