//! Theme configuration for LoomWM

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Background color (hex)
    #[serde(default = "default_background")]
    pub background: String,

    /// Grid color (hex)
    #[serde(default = "default_grid")]
    pub grid: String,

    /// Node border color
    #[serde(default = "default_node_border")]
    pub node_border: String,

    /// Node border color when focused
    #[serde(default = "default_node_border_focused")]
    pub node_border_focused: String,

    /// Connection line color
    #[serde(default = "default_connection")]
    pub connection: String,

    /// Text color
    #[serde(default = "default_text")]
    pub text: String,

    /// Accent color (for highlights, AI elements)
    #[serde(default = "default_accent")]
    pub accent: String,

    /// Node border width
    #[serde(default = "default_border_width")]
    pub border_width: f32,

    /// Node corner radius
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f32,

    /// Font family
    #[serde(default = "default_font")]
    pub font_family: String,

    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: default_background(),
            grid: default_grid(),
            node_border: default_node_border(),
            node_border_focused: default_node_border_focused(),
            connection: default_connection(),
            text: default_text(),
            accent: default_accent(),
            border_width: default_border_width(),
            corner_radius: default_corner_radius(),
            font_family: default_font(),
            font_size: default_font_size(),
        }
    }
}

// Dark theme defaults (modern, minimal)
fn default_background() -> String {
    "#0a0a0f".to_string()
}

fn default_grid() -> String {
    "#1a1a2e".to_string()
}

fn default_node_border() -> String {
    "#2d2d44".to_string()
}

fn default_node_border_focused() -> String {
    "#6366f1".to_string() // Indigo accent
}

fn default_connection() -> String {
    "#4f46e5".to_string()
}

fn default_text() -> String {
    "#e2e8f0".to_string()
}

fn default_accent() -> String {
    "#8b5cf6".to_string() // Purple for AI elements
}

fn default_border_width() -> f32 {
    2.0
}

fn default_corner_radius() -> f32 {
    8.0
}

fn default_font() -> String {
    "Inter".to_string()
}

fn default_font_size() -> f32 {
    14.0
}
