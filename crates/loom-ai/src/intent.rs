//! Intent parsing and representation
//!
//! Converts user input (text, voice, gestures) into structured intents
//! that the compositor can act upon.

use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    /// Open an application
    Launch { app: String },
    /// Search for content
    Search { query: String },
    /// Arrange nodes in a specific way
    Arrange { pattern: ArrangePattern },
    /// Focus on a specific node or content
    Focus { target: String },
    /// Create a new node
    Create {
        node_type: String,
        content: Option<String>,
    },
    /// Connect two nodes
    Connect { from: String, to: String },
    /// General query for the AI
    Query { question: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArrangePattern {
    /// Grid layout
    Grid,
    /// Stack vertically
    Stack,
    /// Radial around a center node
    Radial,
    /// Timeline (horizontal sequence)
    Timeline,
    /// Let AI decide
    Auto,
}

pub struct IntentParser {
    // Will hold LLM client or local model
}

impl IntentParser {
    pub fn new() -> Self {
        debug!("Initializing intent parser");
        Self {}
    }

    /// Parse natural language into an intent
    pub async fn parse(&self, input: &str) -> crate::Result<Intent> {
        debug!("Parsing intent: {}", input);

        // Simple keyword matching for now
        // TODO: Replace with actual LLM integration
        let input_lower = input.to_lowercase();

        if input_lower.starts_with("open ") || input_lower.starts_with("launch ") {
            let app = input
                .split_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ");
            return Ok(Intent::Launch { app });
        }

        if input_lower.starts_with("search ") || input_lower.starts_with("find ") {
            let query = input
                .split_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ");
            return Ok(Intent::Search { query });
        }

        if input_lower.contains("arrange") || input_lower.contains("organize") {
            return Ok(Intent::Arrange {
                pattern: ArrangePattern::Auto,
            });
        }

        // Default to query
        Ok(Intent::Query {
            question: input.to_string(),
        })
    }
}

impl Default for IntentParser {
    fn default() -> Self {
        Self::new()
    }
}
