//! Compositor protocol handler
//!
//! Handles wl_compositor and wl_surface protocols.

use crate::state::{ClientState, LoomState};
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor,
    reexports::wayland_server::{
        Resource,
        protocol::{wl_buffer, wl_surface::WlSurface},
    },
    wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState},
};
use tracing::trace;

impl CompositorHandler for LoomState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(
        &self,
        client: &'a smithay::reexports::wayland_server::Client,
    ) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        trace!("Surface commit: {:?}", surface.id());

        // Handle buffer submission
        on_commit_buffer_handler::<Self>(surface);

        // Handle XDG shell commits
        if let Some(window) = self
            .space
            .elements()
            .find(|w| {
                w.toplevel()
                    .map(|t| t.wl_surface() == surface)
                    .unwrap_or(false)
            })
            .cloned()
        {
            window.on_commit();
        }
    }

    fn destroyed(&mut self, _surface: &WlSurface) {
        trace!("Surface destroyed");
    }
}

impl smithay::wayland::buffer::BufferHandler for LoomState {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {
        // No-op: buffer cleanup is handled automatically
    }
}

delegate_compositor!(LoomState);
