//! Main compositor struct and event loop

use crate::Result;
use tracing::info;

pub struct Compositor {
    // Will hold Smithay's event loop and display
}

impl Compositor {
    pub fn new() -> Result<Self> {
        info!("Initializing LoomWM compositor");
        Ok(Self {})
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting compositor event loop");
        // TODO: Implement Smithay event loop
        Ok(())
    }
}
