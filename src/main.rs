//! LoomWM - A fluid, AI-driven Wayland compositor
//!
//! Weaving your digital intent.
//!
//! LoomWM breaks away from the traditional desktop paradigm of
//! overlapping windows. Instead, it presents an infinite canvas
//! where applications become nodes that can be arranged, connected,
//! and manipulated spatially.

use tracing::{Level, error, info};
use tracing_subscriber::{EnvFilter, fmt};

fn main() {
    // Initialize logging
    let filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    fmt().with_env_filter(filter).with_target(true).init();

    info!("Starting LoomWM - Weaving your digital intent");

    // Load configuration
    let config = match loom_config::Config::load() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load config: {}, using defaults", e);
            loom_config::Config::default()
        }
    };

    // Initialize the compositor
    match run(config) {
        Ok(()) => info!("LoomWM shut down cleanly"),
        Err(e) => {
            error!("LoomWM exited with error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run(_config: loom_config::Config) -> Result<(), Box<dyn std::error::Error>> {
    // Run compositor with auto-detected backend
    loom_core::backend::run_auto()?;
    Ok(())
}
