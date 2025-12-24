//! XDG Shell handler
//!
//! Handles xdg_wm_base protocol for desktop window management.

use crate::state::LoomState;
use smithay::{
    delegate_xdg_shell,
    desktop::{PopupKind, Window},
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::wl_seat::WlSeat,
    },
    utils::Serial,
    wayland::shell::xdg::{
        PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    },
};
use tracing::{debug, warn};

impl XdgShellHandler for LoomState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        debug!("New toplevel surface created");

        let window = Window::new_wayland_window(surface);

        // Place the window at the origin for now
        // TODO: Implement proper window placement on the canvas
        self.space.map_element(window, (0, 0), false);
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        debug!("New popup surface created");

        // For now, just track the popup but don't position it
        // TODO: Implement proper popup positioning
        let _ = self.space.elements().find(|w| {
            w.toplevel()
                .map(|t| {
                    let popup = PopupKind::Xdg(surface.clone());
                    t.wl_surface() == popup.wl_surface()
                })
                .unwrap_or(false)
        });
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        // TODO: Implement popup grab
        warn!("Popup grab requested but not implemented");
    }

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
        // TODO: Implement popup reposition
        warn!("Popup reposition requested but not implemented");
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        debug!("Toplevel destroyed");

        // Remove the window from the space
        let window = self
            .space
            .elements()
            .find(|w| w.toplevel().map(|t| t == &surface).unwrap_or(false))
            .cloned();

        if let Some(window) = window {
            self.space.unmap_elem(&window);
        }
    }

    fn move_request(&mut self, _surface: ToplevelSurface, _seat: WlSeat, _serial: Serial) {
        // TODO: Implement interactive move
        debug!("Move request (not yet implemented)");
    }

    fn resize_request(
        &mut self,
        _surface: ToplevelSurface,
        _seat: WlSeat,
        _serial: Serial,
        _edges: xdg_toplevel::ResizeEdge,
    ) {
        // TODO: Implement interactive resize
        debug!("Resize request (not yet implemented)");
    }

    fn maximize_request(&mut self, surface: ToplevelSurface) {
        debug!("Maximize request");
        surface.send_configure();
    }

    fn unmaximize_request(&mut self, surface: ToplevelSurface) {
        debug!("Unmaximize request");
        surface.send_configure();
    }

    fn fullscreen_request(
        &mut self,
        surface: ToplevelSurface,
        _output: Option<smithay::reexports::wayland_server::protocol::wl_output::WlOutput>,
    ) {
        debug!("Fullscreen request");
        surface.send_configure();
    }

    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        debug!("Unfullscreen request");
        surface.send_configure();
    }

    fn minimize_request(&mut self, _surface: ToplevelSurface) {
        debug!("Minimize request (not yet implemented)");
    }

    fn show_window_menu(
        &mut self,
        _surface: ToplevelSurface,
        _seat: WlSeat,
        _serial: Serial,
        _location: smithay::utils::Point<i32, smithay::utils::Logical>,
    ) {
        debug!("Show window menu request (not yet implemented)");
    }

    fn ack_configure(
        &mut self,
        _surface: WlSurface,
        _configure: smithay::wayland::shell::xdg::Configure,
    ) {
        // Client acknowledged the configure, nothing to do
    }
}

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

delegate_xdg_shell!(LoomState);
