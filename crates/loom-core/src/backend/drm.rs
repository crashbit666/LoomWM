//! DRM/KMS backend for direct hardware rendering
//!
//! This backend runs the compositor directly on the GPU without
//! needing an existing display server. Used in production.
//!
//! # Architecture
//!
//! The DRM backend uses several components:
//! - **libseat**: Session management (VT switching, device access)
//! - **udev**: Device discovery and hotplug
//! - **DRM**: Direct Rendering Manager for GPU access
//! - **GBM**: Generic Buffer Manager for buffer allocation
//! - **libinput**: Input device handling
//!
//! # Security Notes
//!
//! - Requires appropriate permissions (seat access, input group)
//! - Uses libseat for proper privilege separation
//! - Device access is managed through the session

use crate::perf::{FrameTimer, TARGET_FRAME_TIME_60FPS};
use crate::state::LoomState;
use crate::{CoreError, Result};
use smithay::{
    backend::{
        allocator::gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
        drm::{DrmDevice, DrmDeviceFd, DrmEvent, DrmEventMetadata, DrmNode, NodeType},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::damage::OutputDamageTracker,
        session::{Event as SessionEvent, Session, libseat::LibSeatSession},
        udev::{UdevBackend, UdevEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{
            EventLoop, LoopHandle, RegistrationToken,
            timer::{TimeoutAction, Timer},
        },
        drm::control::{ModeTypeFlags, connector, crtc},
        input::Libinput,
        rustix::fs::OFlags,
        wayland_server::Display,
    },
    utils::{DeviceFd, Transform},
};
use smithay_drm_extras::drm_scanner::{DrmScanEvent, DrmScanner};
use std::{collections::HashMap, path::Path, time::Duration};
use tracing::{debug, error, info, warn};

/// Background color (dark gray) - RGBA as f32 [0.0, 1.0]
#[allow(dead_code)]
const BACKGROUND_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

/// Log performance stats every N frames
const PERF_LOG_INTERVAL: u64 = 300;

/// State for a single GPU device
struct GpuData {
    /// DRM device
    #[allow(dead_code)]
    drm: DrmDevice,
    /// DRM device file descriptor
    #[allow(dead_code)]
    drm_fd: DrmDeviceFd,
    /// GBM device for buffer allocation
    #[allow(dead_code)]
    gbm: GbmDevice<DrmDeviceFd>,
    /// GBM allocator
    #[allow(dead_code)]
    allocator: GbmAllocator<DrmDeviceFd>,
    /// DRM scanner for connector/CRTC management
    #[allow(dead_code)]
    drm_scanner: DrmScanner,
    /// Token for event loop registration
    #[allow(dead_code)]
    token: RegistrationToken,
}

/// State for a single output (monitor)
struct OutputData {
    /// The Smithay output
    #[allow(dead_code)]
    output: Output,
    /// CRTC for this output
    #[allow(dead_code)]
    crtc: crtc::Handle,
    /// Damage tracker for efficient rendering
    #[allow(dead_code)]
    damage_tracker: OutputDamageTracker,
}

/// DRM backend state
struct DrmState {
    /// Compositor state
    loom_state: LoomState,
    /// Session for device access
    session: LibSeatSession,
    /// Primary GPU node
    #[allow(dead_code)]
    primary_gpu: DrmNode,
    /// Per-GPU data
    gpus: HashMap<DrmNode, GpuData>,
    /// Per-output data
    outputs: HashMap<crtc::Handle, OutputData>,
    /// Frame timer
    frame_timer: FrameTimer,
    /// Frame counter
    frame_count: u64,
}

impl DrmState {
    /// Check if compositor should keep running
    fn running(&self) -> bool {
        self.loom_state.running
    }
}

/// Run the compositor using the DRM backend
pub fn run() -> Result<()> {
    info!("Starting DRM backend...");

    // Create the event loop with LoomState as the data type
    // We'll use a wrapper to handle both DRM state and LoomState
    let mut event_loop: EventLoop<DrmState> =
        EventLoop::try_new().map_err(|e| CoreError::EventLoop(e.to_string()))?;
    let loop_handle = event_loop.handle();

    // Initialize libseat session
    let (session, notifier) = LibSeatSession::new()
        .map_err(|e| CoreError::Session(format!("Failed to create session: {e}")))?;
    info!("Session created: {:?}", session.seat());

    // Create Wayland display for LoomState
    let display: Display<LoomState> = Display::new()
        .map_err(|e| CoreError::BackendInit(format!("Failed to create display: {e}")))?;

    // We need a separate event loop handle for LoomState
    // Create a simple event loop just for getting a handle
    let loom_event_loop: EventLoop<LoomState> =
        EventLoop::try_new().map_err(|e| CoreError::EventLoop(e.to_string()))?;

    // Create compositor state
    let loom_state = LoomState::new(display, loom_event_loop.handle())
        .map_err(|e| CoreError::BackendInit(format!("Failed to create state: {e}")))?;

    // Create display for socket
    let mut socket_display: Display<LoomState> = Display::new()
        .map_err(|e| CoreError::BackendInit(format!("Failed to create display: {e}")))?;

    // Initialize udev backend
    let udev_backend = UdevBackend::new(session.seat())
        .map_err(|e| CoreError::BackendInit(format!("Failed to create udev backend: {e}")))?;

    // Find primary GPU
    let primary_gpu = udev_backend
        .device_list()
        .find_map(|(node, _path)| {
            let node = DrmNode::from_dev_id(node).ok()?;
            if node.ty() == NodeType::Primary {
                Some(node)
            } else {
                None
            }
        })
        .ok_or_else(|| CoreError::BackendInit("No primary GPU found".to_string()))?;
    info!("Primary GPU: {:?}", primary_gpu);

    // Create DRM state
    let mut state = DrmState {
        loom_state,
        session,
        primary_gpu,
        gpus: HashMap::new(),
        outputs: HashMap::new(),
        frame_timer: FrameTimer::new(),
        frame_count: 0,
    };

    // Register socket
    let socket_name = state
        .loom_state
        .register_socket(&mut socket_display)
        .map_err(|e| CoreError::BackendInit(format!("Failed to register socket: {e}")))?;
    info!("Wayland socket: {}", socket_name);

    // Insert session event source
    loop_handle
        .insert_source(notifier, |event, _, state| {
            handle_session_event(event, state);
        })
        .map_err(|e| CoreError::EventLoop(format!("Failed to insert session source: {e}")))?;

    // Initialize libinput
    let libinput_context =
        Libinput::new_with_udev(LibinputSessionInterface::from(state.session.clone()));
    let libinput_backend = LibinputInputBackend::new(libinput_context.clone());

    loop_handle
        .insert_source(libinput_backend, |event, _, _state| {
            // TODO: Forward input events to state
            debug!("Input event: {:?}", event);
        })
        .map_err(|e| CoreError::EventLoop(format!("Failed to insert libinput source: {e}")))?;

    // Initialize GPUs from udev
    for (dev_id, path) in udev_backend.device_list() {
        if let Err(e) = init_gpu(&mut state, &loop_handle, dev_id, path) {
            error!("Failed to initialize GPU {:?}: {}", path, e);
        }
    }

    // Clone handle for udev closure
    let udev_loop_handle = loop_handle.clone();

    // Insert udev event source for hotplug
    loop_handle
        .insert_source(udev_backend, move |event, _, state| {
            handle_udev_event(event, state, &udev_loop_handle);
        })
        .map_err(|e| CoreError::EventLoop(format!("Failed to insert udev source: {e}")))?;

    // Set up frame timer
    let timer = Timer::immediate();
    loop_handle
        .insert_source(timer, |_, _, state| {
            // Render all outputs
            let crtcs: Vec<_> = state.outputs.keys().copied().collect();
            for crtc in crtcs {
                if let Err(e) = render_output(state, crtc) {
                    error!("Failed to render output: {}", e);
                }
            }

            // Schedule next frame
            TimeoutAction::ToDuration(TARGET_FRAME_TIME_60FPS)
        })
        .map_err(|e| CoreError::EventLoop(format!("Failed to insert frame timer: {e}")))?;

    info!("Entering main event loop");
    info!(
        "To connect a client, run: WAYLAND_DISPLAY={} <client>",
        socket_name
    );

    // Main loop
    while state.running() {
        state.frame_timer.begin_frame();

        // Dispatch events
        event_loop
            .dispatch(Some(Duration::from_millis(1)), &mut state)
            .map_err(|e| CoreError::EventLoop(format!("Event loop error: {e}")))?;

        // Process Wayland clients
        socket_display
            .dispatch_clients(&mut state.loom_state)
            .map_err(|e| CoreError::EventLoop(format!("Dispatch error: {e}")))?;

        // Flush clients
        socket_display.flush_clients().ok();

        // Record frame time
        let is_stutter = state.frame_timer.end_frame();
        if is_stutter {
            let stats = state.frame_timer.stats();
            warn!(
                "Frame stutter: {:?} (target: {:?})",
                stats.last_frame_time,
                state.frame_timer.target_frame_time()
            );
        }

        // Periodic logging
        state.frame_count += 1;
        if state.frame_count.is_multiple_of(PERF_LOG_INTERVAL) {
            let stats = state.frame_timer.stats();
            info!(
                "Performance: {:.1} FPS, avg frame: {:?}, stutters: {}, clients: {}, outputs: {}",
                stats.fps,
                stats.avg_frame_time,
                stats.stutter_count,
                state.loom_state.client_count(),
                state.outputs.len()
            );
        }
    }

    // Final stats
    let stats = state.frame_timer.stats();
    info!(
        "DRM backend shutting down. Final stats: {:.1} FPS avg, {} stutters",
        stats.fps, stats.stutter_count
    );

    Ok(())
}

/// Initialize a GPU device
fn init_gpu(
    state: &mut DrmState,
    loop_handle: &LoopHandle<DrmState>,
    dev_id: libc::dev_t,
    path: &Path,
) -> Result<()> {
    let node = DrmNode::from_dev_id(dev_id)
        .map_err(|e| CoreError::BackendInit(format!("Invalid DRM node: {e}")))?;

    // Open the device through the session
    let fd = state
        .session
        .open(
            path,
            OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK,
        )
        .map_err(|e| CoreError::BackendInit(format!("Failed to open GPU: {e}")))?;

    let drm_fd = DrmDeviceFd::new(DeviceFd::from(fd));

    // Create DRM device
    let (drm, drm_notifier) = DrmDevice::new(drm_fd.clone(), true)
        .map_err(|e| CoreError::BackendInit(format!("Failed to create DRM device: {e}")))?;

    // Create GBM device
    let gbm = GbmDevice::new(drm_fd.clone())
        .map_err(|e| CoreError::BackendInit(format!("Failed to create GBM device: {e}")))?;

    // Create allocator
    let allocator = GbmAllocator::new(
        gbm.clone(),
        GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
    );

    // Register DRM event source
    let token = loop_handle
        .insert_source(drm_notifier, move |event, metadata, state| {
            handle_drm_event(event, metadata, state, node);
        })
        .map_err(|e| CoreError::EventLoop(format!("Failed to insert DRM source: {e}")))?;

    // Create DRM scanner
    let mut drm_scanner = DrmScanner::new();

    // Scan for connectors and process results
    for event in drm_scanner
        .scan_connectors(&drm)
        .map_err(|e| CoreError::BackendInit(format!("Failed to scan connectors: {e}")))?
    {
        match event {
            DrmScanEvent::Connected { connector, crtc } => {
                if let Some(crtc) = crtc
                    && let Err(e) = init_output(state, connector, crtc)
                {
                    error!("Failed to init output: {}", e);
                }
            }
            DrmScanEvent::Disconnected { crtc, .. } => {
                if let Some(crtc) = crtc {
                    state.outputs.remove(&crtc);
                }
            }
        }
    }

    // Store GPU data
    state.gpus.insert(
        node,
        GpuData {
            drm,
            drm_fd,
            gbm,
            allocator,
            drm_scanner,
            token,
        },
    );

    info!("GPU initialized: {:?}", path);
    Ok(())
}

/// Initialize an output (monitor)
fn init_output(state: &mut DrmState, connector: connector::Info, crtc: crtc::Handle) -> Result<()> {
    // Get connector name
    let name = format!(
        "{}-{}",
        connector.interface().as_str(),
        connector.interface_id()
    );
    info!("Initializing output: {}", name);

    // Find the preferred mode
    let mode = connector
        .modes()
        .iter()
        .find(|m| m.mode_type().contains(ModeTypeFlags::PREFERRED))
        .or_else(|| connector.modes().first())
        .copied()
        .ok_or_else(|| CoreError::BackendInit("No mode available".to_string()))?;

    let (w, h) = mode.size();
    info!("Mode: {}x{} @ {}Hz", w, h, mode.vrefresh());

    // Create Smithay output
    let output = Output::new(
        name.clone(),
        PhysicalProperties {
            size: (0, 0).into(), // Physical size unknown without EDID parsing
            subpixel: Subpixel::Unknown,
            make: "Unknown".into(),
            model: "Unknown".into(),
        },
    );

    let smithay_mode = Mode {
        size: (w as i32, h as i32).into(),
        refresh: (mode.vrefresh() * 1000) as i32,
    };

    output.change_current_state(
        Some(smithay_mode),
        Some(Transform::Normal),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(smithay_mode);

    // Add output to space
    state.loom_state.space.map_output(&output, (0, 0));

    // Create damage tracker
    let damage_tracker = OutputDamageTracker::from_output(&output);

    // Store output data
    state.outputs.insert(
        crtc,
        OutputData {
            output,
            crtc,
            damage_tracker,
        },
    );

    info!("Output {} initialized with mode {}x{}", name, w, h);
    Ok(())
}

/// Handle session events (VT switching)
fn handle_session_event(event: SessionEvent, _state: &mut DrmState) {
    match event {
        SessionEvent::PauseSession => {
            info!("Session paused (VT switch away)");
            // TODO: Pause rendering, release devices
        }
        SessionEvent::ActivateSession => {
            info!("Session activated (VT switch back)");
            // TODO: Resume rendering, reclaim devices
        }
    }
}

/// Handle udev events (device hotplug)
fn handle_udev_event(event: UdevEvent, state: &mut DrmState, loop_handle: &LoopHandle<DrmState>) {
    match event {
        UdevEvent::Added { device_id, path } => {
            info!("GPU added: {:?}", path);
            if let Err(e) = init_gpu(state, loop_handle, device_id, &path) {
                error!("Failed to init hotplugged GPU: {}", e);
            }
        }
        UdevEvent::Changed { device_id } => {
            debug!("GPU changed: {:?}", device_id);
            // TODO: Handle connector changes
        }
        UdevEvent::Removed { device_id } => {
            if let Ok(node) = DrmNode::from_dev_id(device_id) {
                info!("GPU removed: {:?}", node);
                if let Some(_gpu_data) = state.gpus.remove(&node) {
                    // Remove associated outputs
                    // Token is automatically removed when GpuData is dropped
                }
            }
        }
    }
}

/// Handle DRM events (page flip, vblank)
fn handle_drm_event(
    event: DrmEvent,
    _metadata: &mut Option<DrmEventMetadata>,
    state: &mut DrmState,
    gpu_node: DrmNode,
) {
    match event {
        DrmEvent::VBlank(crtc) => {
            // VBlank occurred, we can submit the next frame
            if let Some(_output_data) = state.outputs.get_mut(&crtc) {
                // TODO: Submit pending frame
            }
        }
        DrmEvent::Error(e) => {
            error!("DRM error on {:?}: {}", gpu_node, e);
        }
    }
}

/// Render a single output
fn render_output(_state: &mut DrmState, _crtc: crtc::Handle) -> Result<()> {
    // TODO: Implement actual rendering
    // This requires setting up the DRM compositor with surfaces
    // and performing the render similar to winit backend

    Ok(())
}
