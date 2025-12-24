//! Smithay protocol handlers
//!
//! This module implements the Wayland protocol handlers required by Smithay.
//! Each handler implements the corresponding delegate trait.

mod compositor;
mod output;
mod seat;
mod shm;
mod xdg_shell;
