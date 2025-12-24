//! Global compositor state
//!
//! Holds all the state needed by Smithay's delegate implementations

use loom_canvas::Canvas;

#[derive(Default)]
pub struct LoomState {
    pub canvas: Canvas,
    // Will hold:
    // - Smithay compositor state
    // - XDG shell state
    // - Seat (input devices)
    // - Output configuration
}

impl LoomState {
    pub fn new() -> Self {
        Self {
            canvas: Canvas::new(),
        }
    }
}
