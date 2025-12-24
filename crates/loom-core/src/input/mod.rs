//! Input handling for LoomWM
//!
//! This module processes keyboard, mouse, touch, and gesture events from
//! the backend and forwards them to focused surfaces or handles compositor
//! keybindings.
//!
//! # Keybindings
//!
//! Compositor shortcuts are processed before forwarding to clients:
//! - `Logo+Q` or `Ctrl+Alt+Backspace`: Quit compositor
//! - `Logo+Return`: Launch terminal (future)
//! - `Logo+Left/Right`: Move focus (future)
//!
//! # Security
//!
//! - Input events are only forwarded to the focused surface
//! - No raw keycodes are exposed to clients (XKB keysyms only)

mod keybindings;

pub use keybindings::{KeyAction, KeyPattern, Keybindings};

use crate::state::LoomState;
use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Device, Event, InputBackend,
        InputEvent, KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
        PointerMotionEvent,
    },
    desktop::WindowSurfaceType,
    input::{
        keyboard::FilterResult,
        pointer::{AxisFrame, ButtonEvent, MotionEvent, RelativeMotionEvent},
    },
    utils::{Logical, Point, SERIAL_COUNTER},
};
use tracing::{debug, trace, warn};

/// Process all input events from a backend
///
/// This is the main entry point for input handling. It dispatches events
/// to the appropriate handlers based on event type.
pub fn process_input_event<B: InputBackend>(state: &mut LoomState, event: InputEvent<B>) {
    match event {
        InputEvent::Keyboard { event } => {
            process_keyboard_event::<B>(state, event);
        }
        InputEvent::PointerMotion { event } => {
            process_pointer_motion::<B>(state, event);
        }
        InputEvent::PointerMotionAbsolute { event } => {
            process_pointer_motion_absolute::<B>(state, event);
        }
        InputEvent::PointerButton { event } => {
            process_pointer_button::<B>(state, event);
        }
        InputEvent::PointerAxis { event } => {
            process_pointer_axis::<B>(state, event);
        }
        InputEvent::DeviceAdded { device } => {
            debug!("Input device added: {}", device.name());
        }
        InputEvent::DeviceRemoved { device } => {
            debug!("Input device removed: {}", device.name());
        }
        // Touch events - forward to seat
        InputEvent::TouchDown { .. }
        | InputEvent::TouchUp { .. }
        | InputEvent::TouchMotion { .. }
        | InputEvent::TouchCancel { .. }
        | InputEvent::TouchFrame { .. } => {
            trace!("Touch event (not yet handled)");
        }
        // Gesture events
        InputEvent::GestureSwipeBegin { .. }
        | InputEvent::GestureSwipeUpdate { .. }
        | InputEvent::GestureSwipeEnd { .. }
        | InputEvent::GesturePinchBegin { .. }
        | InputEvent::GesturePinchUpdate { .. }
        | InputEvent::GesturePinchEnd { .. }
        | InputEvent::GestureHoldBegin { .. }
        | InputEvent::GestureHoldEnd { .. } => {
            trace!("Gesture event (not yet handled)");
        }
        // Tablet events
        InputEvent::TabletToolAxis { .. }
        | InputEvent::TabletToolProximity { .. }
        | InputEvent::TabletToolTip { .. }
        | InputEvent::TabletToolButton { .. } => {
            trace!("Tablet event (not yet handled)");
        }
        // Switch events (lid, tablet mode)
        InputEvent::SwitchToggle { .. } => {
            trace!("Switch event (not yet handled)");
        }
        // Backend-specific events
        InputEvent::Special(_) => {
            trace!("Special backend event");
        }
    }
}

/// Process a keyboard event
///
/// First checks for compositor keybindings, then forwards to the focused
/// surface if not intercepted.
fn process_keyboard_event<B: InputBackend>(state: &mut LoomState, event: B::KeyboardKeyEvent) {
    let serial = SERIAL_COUNTER.next_serial();
    let time = event.time_msec();
    let keycode = event.key_code();
    let key_state = event.state();

    // Get keyboard from seat
    let keyboard = state.seat.get_keyboard().unwrap();

    // Process through XKB and check for compositor keybindings
    let action = keyboard.input(
        state,
        keycode,
        key_state,
        serial,
        time,
        |state, modifiers, handle| {
            // Only check keybindings on press
            if key_state == KeyState::Pressed {
                let keysym = handle.modified_sym();
                if let Some(action) = state.keybindings.process(keysym.raw(), *modifiers) {
                    debug!("Keybinding matched: {:?}", action);
                    return FilterResult::Intercept(Some(action));
                }
            }
            // Forward to client
            FilterResult::Forward
        },
    );

    // Execute the action if one was intercepted
    if let Some(action) = action.flatten() {
        execute_action(state, action);
    }
}

/// Process relative pointer motion
fn process_pointer_motion<B: InputBackend>(state: &mut LoomState, event: B::PointerMotionEvent) {
    let serial = SERIAL_COUNTER.next_serial();
    let delta = event.delta();

    // Update pointer position
    state.pointer_location += delta;
    clamp_pointer_to_output(state);

    // Find surface under pointer
    let under = surface_under_pointer(state);

    // Send motion event to seat
    let pointer = state.seat.get_pointer().unwrap();
    pointer.motion(
        state,
        under.clone(),
        &MotionEvent {
            location: state.pointer_location,
            serial,
            time: event.time_msec(),
        },
    );

    // Send relative motion for gaming/pointer lock
    pointer.relative_motion(
        state,
        under,
        &RelativeMotionEvent {
            delta,
            delta_unaccel: event.delta_unaccel(),
            utime: event.time(),
        },
    );

    pointer.frame(state);
}

/// Process absolute pointer motion (from touchpad or tablet)
fn process_pointer_motion_absolute<B: InputBackend>(
    state: &mut LoomState,
    event: B::PointerMotionAbsoluteEvent,
) {
    let serial = SERIAL_COUNTER.next_serial();

    // Get output size for coordinate transformation
    let output_size = state
        .space
        .outputs()
        .next()
        .map(|o| {
            o.current_mode()
                .map(|m| m.size)
                .unwrap_or((1920, 1080).into())
        })
        .unwrap_or((1920, 1080).into());

    // Transform to output coordinates
    state.pointer_location = event.position_transformed(output_size.to_logical(1));

    // Find surface under pointer
    let under = surface_under_pointer(state);

    // Send motion event
    let pointer = state.seat.get_pointer().unwrap();
    pointer.motion(
        state,
        under,
        &MotionEvent {
            location: state.pointer_location,
            serial,
            time: event.time_msec(),
        },
    );

    pointer.frame(state);
}

/// Process pointer button press/release
fn process_pointer_button<B: InputBackend>(state: &mut LoomState, event: B::PointerButtonEvent) {
    let serial = SERIAL_COUNTER.next_serial();
    let button = event.button_code();
    let button_state = event.state();

    // On click, update keyboard focus to window under pointer
    if button_state == ButtonState::Pressed {
        if let Some((window, _)) = state
            .space
            .element_under(state.pointer_location)
            .map(|(w, p)| (w.clone(), p))
        {
            // Raise window to top
            state.space.raise_element(&window, true);

            // Set keyboard focus
            let keyboard = state.seat.get_keyboard().unwrap();
            if let Some(toplevel) = window.toplevel() {
                keyboard.set_focus(state, Some(toplevel.wl_surface().clone()), serial);
            }
        } else {
            // Clicked on background - clear focus
            let keyboard = state.seat.get_keyboard().unwrap();
            keyboard.set_focus(state, None, serial);
        }
    }

    // Send button event
    let pointer = state.seat.get_pointer().unwrap();
    pointer.button(
        state,
        &ButtonEvent {
            button,
            state: button_state,
            serial,
            time: event.time_msec(),
        },
    );

    pointer.frame(state);
}

/// Process pointer axis (scroll) event
fn process_pointer_axis<B: InputBackend>(state: &mut LoomState, event: B::PointerAxisEvent) {
    let source = event.source();

    let mut frame = AxisFrame::new(event.time_msec()).source(source);

    // Handle horizontal axis
    if let Some(amount) = event.amount(Axis::Horizontal) {
        frame = frame.value(Axis::Horizontal, amount);
        if let Some(discrete) = event.amount_v120(Axis::Horizontal) {
            frame = frame.v120(Axis::Horizontal, discrete as i32);
        }
    }

    // Handle vertical axis
    if let Some(amount) = event.amount(Axis::Vertical) {
        frame = frame.value(Axis::Vertical, amount);
        if let Some(discrete) = event.amount_v120(Axis::Vertical) {
            frame = frame.v120(Axis::Vertical, discrete as i32);
        }
    }

    // Handle stop events
    if source == AxisSource::Finger {
        if event.amount(Axis::Horizontal) == Some(0.0) {
            frame = frame.stop(Axis::Horizontal);
        }
        if event.amount(Axis::Vertical) == Some(0.0) {
            frame = frame.stop(Axis::Vertical);
        }
    }

    let pointer = state.seat.get_pointer().unwrap();
    pointer.axis(state, frame);
    pointer.frame(state);
}

/// Execute a compositor action
fn execute_action(state: &mut LoomState, action: KeyAction) {
    match action {
        KeyAction::Quit => {
            debug!("Quit action triggered");
            state.running = false;
        }
        KeyAction::CloseFocused => {
            debug!("Close focused window");
            // TODO: Send close request to focused window
            warn!("CloseFocused not yet implemented");
        }
        KeyAction::FocusNext => {
            debug!("Focus next window");
            // TODO: Cycle focus to next window
            warn!("FocusNext not yet implemented");
        }
        KeyAction::FocusPrev => {
            debug!("Focus previous window");
            // TODO: Cycle focus to previous window
            warn!("FocusPrev not yet implemented");
        }
        KeyAction::ToggleFullscreen => {
            debug!("Toggle fullscreen");
            // TODO: Toggle fullscreen for focused window
            warn!("ToggleFullscreen not yet implemented");
        }
        KeyAction::None => {}
    }
}

/// Clamp pointer position to valid output bounds
fn clamp_pointer_to_output(state: &mut LoomState) {
    // Get the bounding box of all outputs
    let (min_x, min_y, max_x, max_y) = state.space.outputs().fold(
        (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
        |(min_x, min_y, max_x, max_y), output| {
            let geometry = state.space.output_geometry(output).unwrap();
            (
                min_x.min(geometry.loc.x as f64),
                min_y.min(geometry.loc.y as f64),
                max_x.max((geometry.loc.x + geometry.size.w) as f64),
                max_y.max((geometry.loc.y + geometry.size.h) as f64),
            )
        },
    );

    state.pointer_location.x = state.pointer_location.x.clamp(min_x, max_x - 1.0);
    state.pointer_location.y = state.pointer_location.y.clamp(min_y, max_y - 1.0);
}

/// Find the surface under the pointer
fn surface_under_pointer(
    state: &LoomState,
) -> Option<(
    smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
    Point<f64, Logical>,
)> {
    state
        .space
        .element_under(state.pointer_location)
        .and_then(|(window, location)| {
            window
                .surface_under(
                    state.pointer_location - location.to_f64(),
                    WindowSurfaceType::ALL,
                )
                .map(|(surface, surface_loc)| (surface, (surface_loc + location).to_f64()))
        })
}
