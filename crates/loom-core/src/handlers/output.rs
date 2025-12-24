//! Output handler
//!
//! Handles wl_output and xdg_output protocols for display management.

use crate::state::LoomState;
use smithay::{delegate_output, wayland::output::OutputHandler};

impl OutputHandler for LoomState {}

delegate_output!(LoomState);
