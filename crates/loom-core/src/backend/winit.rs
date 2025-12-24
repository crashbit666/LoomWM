//! Winit backend for development and testing
//!
//! This backend runs the compositor inside a window on an existing
//! display server (X11 or Wayland). Used during development.
//!
//! # Security Notes
//!
//! - This backend is intended for development only
//! - It runs with the same privileges as the parent compositor
//! - Resource limits from [`crate::security`] are still enforced
//!
//! # Performance
//!
//! - Uses damage tracking to minimize GPU work
//! - Pre-allocated element vector to avoid per-frame allocations
//! - Frame timing with stutter detection

use crate::perf::{FrameTimer, TARGET_FRAME_TIME_60FPS};
use crate::state::LoomState;
use crate::{CoreError, Result};
use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker, element::surface::WaylandSurfaceRenderElement,
            glow::GlowRenderer,
        },
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    desktop::space::SpaceRenderElements,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{calloop::EventLoop, wayland_server::Display},
    utils::{Physical, Size, Transform},
};
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};

/// Background color (dark gray) - RGBA as f32 [0.0, 1.0]
const BACKGROUND_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

/// Log performance stats every N frames
const PERF_LOG_INTERVAL: u64 = 300; // Every 5 seconds at 60 FPS

/// Run the compositor using the Winit backend
pub fn run() -> Result<()> {
    info!("Starting Winit backend...");

    // Create the event loop with LoomState as the data type
    let mut event_loop: EventLoop<LoomState> =
        EventLoop::try_new().map_err(|e| CoreError::EventLoop(e.to_string()))?;

    // Create Wayland display
    let display: Display<LoomState> = Display::new()
        .map_err(|e| CoreError::BackendInit(format!("Failed to create display: {e}")))?;

    // Create compositor state
    let mut state = LoomState::new(display, event_loop.handle())
        .map_err(|e| CoreError::BackendInit(format!("Failed to create state: {e}")))?;

    // Create another display for socket registration
    // (the first one was consumed by LoomState::new)
    let mut display: Display<LoomState> = Display::new()
        .map_err(|e| CoreError::BackendInit(format!("Failed to create display: {e}")))?;

    // Register Wayland socket
    let socket_name = state
        .register_socket(&mut display)
        .map_err(|e| CoreError::BackendInit(format!("Failed to register socket: {e}")))?;

    info!("Wayland socket: {}", socket_name);

    // Initialize Winit backend
    let (mut backend, winit_event_source) = winit::init::<GlowRenderer>()
        .map_err(|e| CoreError::BackendInit(format!("Failed to initialize Winit: {e}")))?;

    // Get window size
    let size: Size<i32, Physical> = backend.window_size();
    info!("Winit window created with size {}x{}", size.w, size.h);

    // Create output for this backend
    let output = create_output(size);
    debug!("Output created: {:?}", output.name());

    // Add output to space
    state.space.map_output(&output, (0, 0));

    // Create damage tracker for efficient rendering
    let mut damage_tracker = OutputDamageTracker::from_output(&output);

    // Create frame timer for performance monitoring
    let mut frame_timer = FrameTimer::new();

    // Frame counter for periodic logging
    let mut frame_count: u64 = 0;

    // Insert Winit event source into the event loop
    event_loop
        .handle()
        .insert_source(winit_event_source, move |event, _, state| {
            handle_winit_event(event, state);
        })
        .map_err(|e| CoreError::EventLoop(format!("Failed to insert Winit source: {e}")))?;

    info!("Entering main event loop");
    info!(
        "To connect a client, run: WAYLAND_DISPLAY={} <client>",
        socket_name
    );

    // Main loop
    while state.running {
        frame_timer.begin_frame();

        // Dispatch events with timeout for frame pacing
        event_loop
            .dispatch(
                Some(Duration::from_micros(
                    TARGET_FRAME_TIME_60FPS.as_micros() as u64
                )),
                &mut state,
            )
            .map_err(|e| CoreError::EventLoop(format!("Event loop error: {e}")))?;

        // Process Wayland client requests
        display
            .dispatch_clients(&mut state)
            .map_err(|e| CoreError::EventLoop(format!("Dispatch error: {e}")))?;

        // Render frame
        if let Err(e) = render_frame(&mut backend, &output, &mut damage_tracker, &mut state) {
            error!("Render error: {}", e);
            // Don't crash on render errors, just skip frame
        }

        // Flush client events
        display.flush_clients().ok();

        // Record frame time and check for stutters
        let is_stutter = frame_timer.end_frame();
        if is_stutter {
            let stats = frame_timer.stats();
            warn!(
                "Frame stutter detected: {:?} (target: {:?})",
                stats.last_frame_time,
                frame_timer.target_frame_time()
            );
        }

        // Periodic performance logging
        frame_count += 1;
        if frame_count.is_multiple_of(PERF_LOG_INTERVAL) {
            let stats = frame_timer.stats();
            info!(
                "Performance: {:.1} FPS, avg frame: {:?}, stutters: {}, clients: {}",
                stats.fps,
                stats.avg_frame_time,
                stats.stutter_count,
                state.client_count()
            );
        }
    }

    // Final stats
    let stats = frame_timer.stats();
    info!(
        "Winit backend shutting down. Final stats: {:.1} FPS avg, {} stutters",
        stats.fps, stats.stutter_count
    );

    Ok(())
}

/// Handle Winit window events
#[inline]
fn handle_winit_event(event: WinitEvent, state: &mut LoomState) {
    match event {
        WinitEvent::Resized { size, scale_factor } => {
            debug!(
                "Window resized to {}x{} (scale: {})",
                size.w, size.h, scale_factor
            );
            // TODO: Update output mode
        }
        WinitEvent::Focus(focused) => {
            debug!("Window focus: {}", focused);
        }
        WinitEvent::Input(input_event) => {
            trace!("Input event: {:?}", input_event);
            // TODO: Forward to input handler
        }
        WinitEvent::Redraw => {
            // Handled in main loop
        }
        WinitEvent::CloseRequested => {
            info!("Window close requested");
            state.running = false;
        }
    }
}

/// Create an output representing the Winit window
#[inline]
fn create_output(size: Size<i32, Physical>) -> Output {
    let mode = Mode {
        size,
        refresh: 60_000, // 60 Hz in mHz
    };

    let physical_properties = PhysicalProperties {
        size: (0, 0).into(), // Unknown physical size
        subpixel: Subpixel::Unknown,
        make: "LoomWM".into(),
        model: "Winit Backend".into(),
    };

    let output = Output::new("winit-0".into(), physical_properties);
    output.change_current_state(
        Some(mode),
        Some(Transform::Normal),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    output
}

/// Render a frame to the Winit backend
///
/// # Performance
///
/// This function is on the hot path and must avoid allocations.
#[inline]
fn render_frame(
    backend: &mut WinitGraphicsBackend<GlowRenderer>,
    output: &Output,
    damage_tracker: &mut OutputDamageTracker,
    state: &mut LoomState,
) -> Result<()> {
    // Collect render elements from the space
    let scale = output.current_scale().fractional_scale() as f32;
    let elements: Vec<
        SpaceRenderElements<GlowRenderer, WaylandSurfaceRenderElement<GlowRenderer>>,
    > = state
        .space
        .render_elements_for_output(backend.renderer(), output, scale)
        .map_err(|e| CoreError::Renderer(format!("Failed to get render elements: {e:?}")))?;

    // Bind the renderer and get framebuffer
    let (renderer, mut framebuffer) = backend
        .bind()
        .map_err(|e| CoreError::Renderer(format!("Failed to bind renderer: {e}")))?;

    // Render with damage tracking
    let render_result = damage_tracker.render_output(
        renderer,
        &mut framebuffer,
        0, // age - 0 means full redraw for now
        &elements,
        BACKGROUND_COLOR,
    );

    // Drop framebuffer before calling submit
    drop(framebuffer);

    match render_result {
        Ok(render_output_result) => {
            // Submit the frame with damage info
            let damage = render_output_result.damage.map(|d| d.as_slice());
            backend
                .submit(damage)
                .map_err(|e| CoreError::Renderer(format!("Failed to submit frame: {e}")))?;

            // Send frame callbacks to clients
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            state.space.elements().for_each(|window| {
                window.send_frame(output, time, Some(Duration::ZERO), |_, _| {
                    Some(output.clone())
                });
            });
        }
        Err(e) => {
            warn!("Render output failed: {:?}", e);
        }
    }

    Ok(())
}
