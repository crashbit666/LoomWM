//! Nodes represent content on the canvas
//!
//! A node can be:
//! - A Wayland surface (application window)
//! - A generated UI element (from AI)
//! - A data visualization
//! - A group of other nodes

use serde::{Deserialize, Serialize};

pub type NodeId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// A Wayland surface (application)
    Surface { surface_id: u64 },
    /// AI-generated content
    Generated { content: String },
    /// A group containing other nodes
    Group { children: Vec<NodeId> },
    /// A text note
    Note { text: String },
    /// An image or media
    Media { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale: f64,
    pub label: Option<String>,
}

impl Node {
    pub fn new(id: NodeId, node_type: NodeType, x: f64, y: f64) -> Self {
        Self {
            id,
            node_type,
            x,
            y,
            width: 800.0,
            height: 600.0,
            scale: 1.0,
            label: None,
        }
    }

    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        (self.x, self.y, self.x + self.width, self.y + self.height)
    }
}
