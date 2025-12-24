//! Connections between nodes
//!
//! Connections represent relationships between content:
//! - Data flow
//! - Semantic relationships
//! - User-defined links

use crate::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Simple visual link
    Link,
    /// Data flows from one node to another
    DataFlow,
    /// Semantic relationship (AI-inferred)
    Semantic { relationship: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: NodeId,
    pub to: NodeId,
    pub connection_type: ConnectionType,
}

impl Connection {
    pub fn new(from: NodeId, to: NodeId) -> Self {
        Self {
            from,
            to,
            connection_type: ConnectionType::Link,
        }
    }

    pub fn with_type(mut self, connection_type: ConnectionType) -> Self {
        self.connection_type = connection_type;
        self
    }
}
