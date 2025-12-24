//! Viewport for the infinite canvas
//!
//! Controls what portion of the canvas is visible:
//! - Pan (scroll the canvas)
//! - Zoom (scale in/out)

use crate::limits;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    /// Center X position in canvas coordinates
    pub x: f64,
    /// Center Y position in canvas coordinates
    pub y: f64,
    /// Zoom level (1.0 = 100%)
    pub zoom: f64,
    /// Screen width in pixels
    pub screen_width: f64,
    /// Screen height in pixels
    pub screen_height: f64,
}

/// Zoom limits
const MIN_ZOOM: f64 = 0.1;
const MAX_ZOOM: f64 = 10.0;

impl Viewport {
    pub fn new(screen_width: f64, screen_height: f64) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            screen_width,
            screen_height,
        }
    }

    /// Pan the viewport by delta pixels (with bounds checking)
    pub fn pan(&mut self, dx: f64, dy: f64) {
        // Validate input
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }

        let new_x = self.x + dx / self.zoom;
        let new_y = self.y + dy / self.zoom;

        // Clamp to valid coordinate range
        self.x = new_x.clamp(limits::MIN_COORDINATE, limits::MAX_COORDINATE);
        self.y = new_y.clamp(limits::MIN_COORDINATE, limits::MAX_COORDINATE);
    }

    /// Zoom in/out centered on a point (with bounds checking)
    pub fn zoom_at(&mut self, factor: f64, center_x: f64, center_y: f64) {
        // Validate input
        if !factor.is_finite() || !center_x.is_finite() || !center_y.is_finite() {
            return;
        }

        let old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);

        // Adjust position to zoom towards the center point
        let zoom_ratio = self.zoom / old_zoom;
        let new_x = center_x - (center_x - self.x) * zoom_ratio;
        let new_y = center_y - (center_y - self.y) * zoom_ratio;

        // Clamp to valid coordinate range
        self.x = new_x.clamp(limits::MIN_COORDINATE, limits::MAX_COORDINATE);
        self.y = new_y.clamp(limits::MIN_COORDINATE, limits::MAX_COORDINATE);
    }

    /// Reset viewport to origin
    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.zoom = 1.0;
    }

    /// Check if a point is within the visible area
    pub fn contains(&self, x: f64, y: f64) -> bool {
        let half_width = (self.screen_width / 2.0) / self.zoom;
        let half_height = (self.screen_height / 2.0) / self.zoom;

        x >= self.x - half_width
            && x <= self.x + half_width
            && y >= self.y - half_height
            && y <= self.y + half_height
    }

    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        let canvas_x = (screen_x - self.screen_width / 2.0) / self.zoom + self.x;
        let canvas_y = (screen_y - self.screen_height / 2.0) / self.zoom + self.y;
        (canvas_x, canvas_y)
    }

    /// Convert canvas coordinates to screen coordinates
    pub fn canvas_to_screen(&self, canvas_x: f64, canvas_y: f64) -> (f64, f64) {
        let screen_x = (canvas_x - self.x) * self.zoom + self.screen_width / 2.0;
        let screen_y = (canvas_y - self.y) * self.zoom + self.screen_height / 2.0;
        (screen_x, screen_y)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(1920.0, 1080.0)
    }
}
