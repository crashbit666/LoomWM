//! The infinite canvas that holds all nodes

use crate::{limits, CanvasError, Connection, Node, NodeId, Result, Viewport};
use std::collections::HashMap;

pub struct Canvas {
    nodes: HashMap<NodeId, Node>,
    connections: Vec<Connection>,
    viewport: Viewport,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            viewport: Viewport::default(),
        }
    }

    /// Add a node to the canvas (with resource limits)
    pub fn add_node(&mut self, node: Node) -> Result<NodeId> {
        if self.nodes.len() >= limits::MAX_NODES {
            return Err(CanvasError::ResourceLimitExceeded(format!(
                "Maximum nodes ({}) exceeded",
                limits::MAX_NODES
            )));
        }

        // Validate coordinates are within bounds
        if !Self::is_valid_coordinate(node.x) || !Self::is_valid_coordinate(node.y) {
            return Err(CanvasError::ResourceLimitExceeded(
                "Node coordinates out of bounds".to_string(),
            ));
        }

        let id = node.id;
        self.nodes.insert(id, node);
        Ok(id)
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    pub fn remove_node(&mut self, id: NodeId) -> Option<Node> {
        // Also remove connections involving this node
        self.connections.retain(|c| c.from != id && c.to != id);
        self.nodes.remove(&id)
    }

    /// Connect two nodes (with resource limits)
    pub fn connect(&mut self, from: NodeId, to: NodeId) -> Result<()> {
        if self.connections.len() >= limits::MAX_CONNECTIONS {
            return Err(CanvasError::ResourceLimitExceeded(format!(
                "Maximum connections ({}) exceeded",
                limits::MAX_CONNECTIONS
            )));
        }

        // Validate nodes exist
        if !self.nodes.contains_key(&from) {
            return Err(CanvasError::NodeNotFound(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(CanvasError::NodeNotFound(to));
        }

        self.connections.push(Connection::new(from, to));
        Ok(())
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    pub fn visible_nodes(&self) -> impl Iterator<Item = &Node> {
        let vp = &self.viewport;
        self.nodes.values().filter(|n| vp.contains(n.x, n.y))
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Check if a coordinate is within valid bounds
    fn is_valid_coordinate(coord: f64) -> bool {
        coord.is_finite() && coord >= limits::MIN_COORDINATE && coord <= limits::MAX_COORDINATE
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}
