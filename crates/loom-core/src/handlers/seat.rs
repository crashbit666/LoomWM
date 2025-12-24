//! Seat (input) handler
//!
//! Handles wl_seat protocol for keyboard, pointer, and touch input.

use crate::state::LoomState;
use smithay::{
    delegate_seat,
    input::{Seat, SeatHandler, SeatState, pointer::CursorImageStatus},
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
};
use tracing::debug;

impl SeatHandler for LoomState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        self.cursor_status = image;
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, focused: Option<&Self::KeyboardFocus>) {
        debug!("Focus changed to: {:?}", focused.map(|s| s.id()));
    }
}

delegate_seat!(LoomState);
