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

use crate::{CoreError, Result};
use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker, element::surface::WaylandSurfaceRenderElement,
            glow::GlowRenderer,
        },
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::{EventLoop, LoopSignal},
    utils::{Physical, Size, Transform},
};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Frame time target for 60 FPS (in milliseconds)
const FRAME_TIME_MS: u64 = 16;

/// Background color (dark gray) - RGBA as f32 [0.0, 1.0]
const BACKGROUND_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

/// Run the compositor using the Winit backend
pub fn run() -> Result<()> {
    info!("Starting Winit backend...");

    // Create the event loop
    let mut event_loop: EventLoop<LoopData> =
        EventLoop::try_new().map_err(|e| CoreError::EventLoop(e.to_string()))?;

    // Initialize Winit backend
    let (mut backend, winit_event_source) = winit::init::<GlowRenderer>()
        .map_err(|e| CoreError::BackendInit(format!("Failed to initialize Winit: {e}")))?;

    // Get window size
    let size: Size<i32, Physical> = backend.window_size();
    info!("Winit window created with size {}x{}", size.w, size.h);

    // Create output for this backend
    let output = create_output(size);
    debug!("Output created: {:?}", output.name());

    // Create damage tracker for efficient rendering
    let mut damage_tracker = OutputDamageTracker::from_output(&output);

    // Loop state
    let loop_signal = event_loop.get_signal();
    let mut data = LoopData {
        running: true,
        loop_signal: loop_signal.clone(),
    };

    // Insert Winit event source into the event loop
    event_loop
        .handle()
        .insert_source(winit_event_source, move |event, _, data| {
            handle_winit_event(event, data);
        })
        .map_err(|e| CoreError::EventLoop(format!("Failed to insert Winit source: {e}")))?;

    info!("Entering main event loop");

    // Main loop
    while data.running {
        // Dispatch events with timeout for frame pacing
        event_loop
            .dispatch(Some(Duration::from_millis(FRAME_TIME_MS)), &mut data)
            .map_err(|e| CoreError::EventLoop(format!("Event loop error: {e}")))?;

        // Render frame
        if let Err(e) = render_frame(&mut backend, &output, &mut damage_tracker) {
            error!("Render error: {}", e);
            // Don't crash on render errors, just skip frame
        }
    }

    info!("Winit backend shutting down");
    Ok(())
}

/// Data passed through the event loop
struct LoopData {
    running: bool,
    loop_signal: LoopSignal,
}

/// Handle Winit window events
fn handle_winit_event(event: WinitEvent, data: &mut LoopData) {
    match event {
        WinitEvent::Resized { size, scale_factor } => {
            debug!(
                "Window resized to {}x{} (scale: {})",
                size.w, size.h, scale_factor
            );
        }
        WinitEvent::Focus(focused) => {
            debug!("Window focus: {}", focused);
        }
        WinitEvent::Input(input_event) => {
            debug!("Input event: {:?}", input_event);
            // TODO: Forward to input handler
        }
        WinitEvent::Redraw => {
            // Handled in main loop
        }
        WinitEvent::CloseRequested => {
            info!("Window close requested");
            data.running = false;
            data.loop_signal.stop();
        }
    }
}

/// Create an output representing the Winit window
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
fn render_frame(
    backend: &mut WinitGraphicsBackend<GlowRenderer>,
    _output: &Output,
    damage_tracker: &mut OutputDamageTracker,
) -> Result<()> {
    // No elements to render yet - just clear to background color
    let elements: Vec<WaylandSurfaceRenderElement<GlowRenderer>> = vec![];

    // Bind the renderer and get framebuffer
    let (renderer, mut framebuffer) = backend
        .bind()
        .map_err(|e| CoreError::Renderer(format!("Failed to bind renderer: {e}")))?;

    // Render with damage tracking
    let render_result = damage_tracker.render_output(
        renderer,
        &mut framebuffer,
        0, // age - 0 means full redraw
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
        }
        Err(e) => {
            warn!("Render output failed: {:?}", e);
        }
    }

    Ok(())
}
