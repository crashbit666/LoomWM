//! Global compositor state
//!
//! Holds all the state needed by Smithay's delegate implementations.
//! This is the central state struct that gets passed through the event loop.
//!
//! # Security
//!
//! Resource limits from [`crate::security`] are enforced here to prevent
//! denial of service attacks from malicious clients.

use crate::input::Keybindings;
use crate::security;
use loom_canvas::Canvas;
use smithay::{
    desktop::{Space, Window},
    input::{Seat, SeatState, pointer::CursorImageStatus},
    reexports::{
        calloop::{Interest, LoopHandle, Mode, PostAction, generic::Generic},
        wayland_server::{
            Display, DisplayHandle,
            backend::{ClientData, ClientId, DisconnectReason},
        },
    },
    utils::{Logical, Point},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        output::OutputManagerState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Client-specific data stored by Smithay
#[derive(Default)]
pub struct ClientState {
    /// Compositor state for this client
    pub compositor_state: CompositorClientState,
    /// Number of surfaces created by this client (for DoS protection)
    pub surface_count: usize,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {
        debug!("Client initialized");
    }

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        debug!("Client disconnected");
    }
}

/// Global compositor state
///
/// This struct holds all the state needed by the compositor,
/// including Smithay protocol handlers and our custom canvas.
pub struct LoomState {
    /// The infinite canvas where nodes live
    pub canvas: Canvas,

    /// Wayland display handle
    pub display_handle: DisplayHandle,

    /// Event loop handle for registering sources
    pub loop_handle: LoopHandle<'static, Self>,

    /// Compositor protocol state (wl_compositor)
    pub compositor_state: CompositorState,

    /// XDG shell state (xdg_wm_base)
    pub xdg_shell_state: XdgShellState,

    /// Shared memory state (wl_shm)
    pub shm_state: ShmState,

    /// Output manager state
    pub output_manager_state: OutputManagerState,

    /// Seat state (input devices)
    pub seat_state: SeatState<Self>,

    /// The primary seat
    pub seat: Seat<Self>,

    /// 2D space for window management
    pub space: Space<Window>,

    /// Current cursor image status
    pub cursor_status: CursorImageStatus,

    /// Current pointer location
    pub pointer_location: Point<f64, Logical>,

    /// Keybindings manager
    pub keybindings: Keybindings,

    /// Whether the compositor should keep running
    pub running: bool,

    /// Socket name for clients to connect
    pub socket_name: Option<String>,

    /// Number of connected clients (for DoS protection)
    client_count: usize,
}

impl LoomState {
    /// Create a new compositor state
    ///
    /// # Arguments
    ///
    /// * `display` - The Wayland display
    /// * `loop_handle` - The event loop handle
    ///
    /// # Returns
    ///
    /// A new `LoomState` instance, or an error if initialization fails
    pub fn new(
        display: Display<Self>,
        loop_handle: LoopHandle<'static, Self>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let display_handle = display.handle();

        // Initialize Smithay protocol handlers
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);

        // Initialize seat (input devices)
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&display_handle, "seat0");

        // Add keyboard capability with default XKB config
        seat.add_keyboard(Default::default(), 200, 25)
            .map_err(|e| format!("Failed to add keyboard: {e}"))?;

        // Add pointer capability
        seat.add_pointer();

        info!("Compositor state initialized with keyboard and pointer");

        Ok(Self {
            canvas: Canvas::new(),
            display_handle,
            loop_handle,
            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            seat,
            space: Space::default(),
            cursor_status: CursorImageStatus::default_named(),
            pointer_location: Point::from((0.0, 0.0)),
            keybindings: Keybindings::new(),
            running: true,
            socket_name: None,
            client_count: 0,
        })
    }

    /// Register a Wayland socket for client connections
    ///
    /// Creates a socket in the XDG_RUNTIME_DIR and registers it
    /// with the event loop.
    pub fn register_socket(
        &mut self,
        display: &mut Display<Self>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Create listening socket
        let socket = ListeningSocketSource::new_auto()?;
        let socket_name = socket.socket_name().to_string_lossy().to_string();

        // Register socket with event loop
        self.loop_handle
            .insert_source(socket, move |client_stream, _, state| {
                // Check client limit before accepting
                if state.client_count >= security::MAX_CLIENTS {
                    warn!(
                        "Rejecting client: max clients ({}) reached",
                        security::MAX_CLIENTS
                    );
                    return;
                }

                // Accept the client
                if let Err(e) = state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                {
                    warn!("Failed to insert client: {}", e);
                } else {
                    state.client_count += 1;
                    debug!("Client connected (total: {})", state.client_count);
                }
            })?;

        // Register display source for processing client requests
        // Clone the poll_fd to get an owned file descriptor with 'static lifetime
        let poll_fd = display
            .backend()
            .poll_fd()
            .try_clone_to_owned()
            .map_err(|e| format!("Failed to clone poll fd: {e}"))?;

        self.loop_handle.insert_source(
            Generic::new(poll_fd, Interest::READ, Mode::Level),
            |_, _, state| {
                // This is safe because we're in the event loop callback
                // and we have mutable access to state
                state.display_handle.flush_clients().ok();
                Ok(PostAction::Continue)
            },
        )?;

        self.socket_name = Some(socket_name.clone());
        info!("Listening on Wayland socket: {}", socket_name);

        Ok(socket_name)
    }

    /// Called when a client disconnects
    pub fn client_disconnected(&mut self) {
        self.client_count = self.client_count.saturating_sub(1);
        debug!("Client disconnected (total: {})", self.client_count);
    }

    /// Get the current number of connected clients
    pub fn client_count(&self) -> usize {
        self.client_count
    }

    /// Check if we can accept more surfaces from a client
    pub fn can_create_surface(&self, client_surfaces: usize) -> bool {
        client_surfaces < security::MAX_SURFACES_PER_CLIENT
            && self.space.elements().count() < security::MAX_TOTAL_SURFACES
    }
}
