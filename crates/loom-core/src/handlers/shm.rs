//! Shared memory handler
//!
//! Handles wl_shm protocol for CPU-accessible buffers.

use crate::state::LoomState;
use smithay::{delegate_shm, wayland::shm::ShmHandler};

impl ShmHandler for LoomState {
    fn shm_state(&self) -> &smithay::wayland::shm::ShmState {
        &self.shm_state
    }
}

delegate_shm!(LoomState);
