//! Input handling via libinput
//!
//! Processes keyboard, mouse, touch, and gesture events

use tracing::debug;

pub struct InputHandler {
    // Will hold libinput context
}

impl InputHandler {
    pub fn new() -> Self {
        debug!("Initializing input handler");
        Self {}
    }
}
