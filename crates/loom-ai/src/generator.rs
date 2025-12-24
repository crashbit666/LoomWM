//! UI Generation
//!
//! Creates visual elements based on AI analysis:
//! - Generate nodes from intents
//! - Create visualizations for data
//! - Suggest layouts

use loom_canvas::{Node, NodeId, NodeType};
use tracing::debug;

pub struct UiGenerator {
    next_node_id: NodeId,
}

impl UiGenerator {
    pub fn new() -> Self {
        Self { next_node_id: 1000 }
    }

    fn allocate_node_id(&mut self) -> Option<NodeId> {
        let id = self.next_node_id;
        self.next_node_id = self.next_node_id.checked_add(1)?;
        Some(id)
    }

    /// Generate a node from content
    pub fn generate_node(&mut self, content: &str, x: f64, y: f64) -> Option<Node> {
        let id = self.allocate_node_id()?;

        debug!(
            "Generating node {} for content length: {}",
            id,
            content.len()
        );

        Some(
            Node::new(
                id,
                NodeType::Generated {
                    content: content.to_string(),
                },
                x,
                y,
            )
            .with_size(400.0, 300.0)
            .with_label(truncate(content, 30)),
        )
    }

    /// Generate a note node
    pub fn generate_note(&mut self, text: &str, x: f64, y: f64) -> Option<Node> {
        let id = self.allocate_node_id()?;

        Some(
            Node::new(
                id,
                NodeType::Note {
                    text: text.to_string(),
                },
                x,
                y,
            )
            .with_size(300.0, 200.0),
        )
    }
}

impl Default for UiGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Safely truncate a string respecting UTF-8 character boundaries
fn truncate(s: &str, max_len: usize) -> String {
    if max_len < 4 {
        return s.chars().take(max_len).collect();
    }

    if s.len() <= max_len {
        return s.to_string();
    }

    // Find a valid UTF-8 boundary for truncation
    let target = max_len.saturating_sub(3);
    let mut end = target;

    // Walk back to find a valid char boundary
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }

    if end == 0 {
        // Edge case: couldn't find a valid boundary
        return "...".to_string();
    }

    format!("{}...", &s[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_ascii() {
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("exact len", 9), "exact len");
    }

    #[test]
    fn test_truncate_utf8() {
        // Japanese characters (3 bytes each)
        assert_eq!(truncate("こんにちは", 10), "こん...");
        // Emoji (4 bytes each)
        assert_eq!(truncate("🎉🎊🎁🎈", 10), "🎉...");
    }

    #[test]
    fn test_truncate_edge_cases() {
        assert_eq!(truncate("ab", 2), "ab");
        assert_eq!(truncate("abc", 3), "abc");
        // With max_len < 4, we just take first max_len chars (no room for "...")
        assert_eq!(truncate("abcd", 3), "abc");
        // With max_len >= 4, we can add "..."
        assert_eq!(truncate("abcdef", 5), "ab...");
    }
}
