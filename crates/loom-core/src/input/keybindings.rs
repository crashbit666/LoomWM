//! Keybinding system for LoomWM
//!
//! Handles compositor-level keyboard shortcuts that are processed before
//! being forwarded to clients.
//!
//! # Default Keybindings
//!
//! - `Logo+Q`: Quit the compositor
//! - `Ctrl+Alt+Backspace`: Quit the compositor (alternative)
//! - `Logo+W`: Close focused window
//! - `Alt+Tab`: Focus next window
//! - `Alt+Shift+Tab`: Focus previous window
//! - `Logo+F`: Toggle fullscreen

use smallvec::SmallVec;
use smithay::input::keyboard::{ModifiersState, keysyms};

/// Actions that can be triggered by keybindings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Do nothing (used for suppressed key releases)
    None,
    /// Quit the compositor
    Quit,
    /// Close the currently focused window
    CloseFocused,
    /// Focus the next window
    FocusNext,
    /// Focus the previous window
    FocusPrev,
    /// Toggle fullscreen for focused window
    ToggleFullscreen,
}

/// A single keybinding pattern
#[derive(Debug, Clone)]
pub struct KeyPattern {
    /// Required modifiers
    pub modifiers: ModifiersState,
    /// The keysym to match
    pub keysym: u32,
    /// Action to execute
    pub action: KeyAction,
}

impl KeyPattern {
    /// Create a new key pattern
    pub fn new(keysym: u32, modifiers: ModifiersState, action: KeyAction) -> Self {
        Self {
            modifiers,
            keysym,
            action,
        }
    }

    /// Check if this pattern matches the given keysym and modifiers
    #[inline]
    pub fn matches(&self, keysym: u32, modifiers: ModifiersState) -> bool {
        self.keysym == keysym && self.modifiers_match(modifiers)
    }

    /// Check if the modifiers match (allowing extra modifiers)
    #[inline]
    fn modifiers_match(&self, modifiers: ModifiersState) -> bool {
        // Check required modifiers are present
        // We allow extra modifiers that aren't in our pattern
        (!self.modifiers.ctrl || modifiers.ctrl)
            && (!self.modifiers.alt || modifiers.alt)
            && (!self.modifiers.shift || modifiers.shift)
            && (!self.modifiers.logo || modifiers.logo)
    }
}

/// Keybindings manager
///
/// Stores and processes keybindings for the compositor.
/// Uses SmallVec to avoid heap allocation for typical binding counts.
#[derive(Debug)]
pub struct Keybindings {
    /// List of keybindings
    bindings: SmallVec<[KeyPattern; 16]>,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self::new()
    }
}

impl Keybindings {
    /// Create a new keybindings manager with default bindings
    pub fn new() -> Self {
        let mut bindings = SmallVec::new();

        // Logo+Q: Quit
        bindings.push(KeyPattern::new(
            keysyms::KEY_q,
            ModifiersState {
                logo: true,
                ..Default::default()
            },
            KeyAction::Quit,
        ));

        // Ctrl+Alt+Backspace: Quit (emergency exit)
        bindings.push(KeyPattern::new(
            keysyms::KEY_BackSpace,
            ModifiersState {
                ctrl: true,
                alt: true,
                ..Default::default()
            },
            KeyAction::Quit,
        ));

        // Logo+W: Close focused window
        bindings.push(KeyPattern::new(
            keysyms::KEY_w,
            ModifiersState {
                logo: true,
                ..Default::default()
            },
            KeyAction::CloseFocused,
        ));

        // Alt+Tab: Focus next
        bindings.push(KeyPattern::new(
            keysyms::KEY_Tab,
            ModifiersState {
                alt: true,
                ..Default::default()
            },
            KeyAction::FocusNext,
        ));

        // Alt+Shift+Tab: Focus previous
        bindings.push(KeyPattern::new(
            keysyms::KEY_Tab,
            ModifiersState {
                alt: true,
                shift: true,
                ..Default::default()
            },
            KeyAction::FocusPrev,
        ));

        // Logo+F: Toggle fullscreen
        bindings.push(KeyPattern::new(
            keysyms::KEY_f,
            ModifiersState {
                logo: true,
                ..Default::default()
            },
            KeyAction::ToggleFullscreen,
        ));

        Self { bindings }
    }

    /// Process a key press and return an action if a keybinding matches
    ///
    /// Returns `Some(action)` if a keybinding was matched, `None` otherwise.
    #[inline]
    pub fn process(&self, keysym: u32, modifiers: ModifiersState) -> Option<KeyAction> {
        // More specific bindings (more modifiers) should be checked first
        // Since we check in order and more specific patterns match more strictly,
        // we need to put Alt+Shift+Tab before Alt+Tab
        for binding in &self.bindings {
            if binding.matches(keysym, modifiers) {
                return Some(binding.action);
            }
        }
        None
    }

    /// Add a custom keybinding
    pub fn add(&mut self, keysym: u32, modifiers: ModifiersState, action: KeyAction) {
        self.bindings
            .push(KeyPattern::new(keysym, modifiers, action));
    }

    /// Clear all keybindings
    pub fn clear(&mut self) {
        self.bindings.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logo_q_quits() {
        let keybindings = Keybindings::new();
        let modifiers = ModifiersState {
            logo: true,
            ..Default::default()
        };

        let action = keybindings.process(keysyms::KEY_q, modifiers);
        assert_eq!(action, Some(KeyAction::Quit));
    }

    #[test]
    fn test_ctrl_alt_backspace_quits() {
        let keybindings = Keybindings::new();
        let modifiers = ModifiersState {
            ctrl: true,
            alt: true,
            ..Default::default()
        };

        let action = keybindings.process(keysyms::KEY_BackSpace, modifiers);
        assert_eq!(action, Some(KeyAction::Quit));
    }

    #[test]
    fn test_alt_tab_focuses_next() {
        let keybindings = Keybindings::new();
        let modifiers = ModifiersState {
            alt: true,
            ..Default::default()
        };

        let action = keybindings.process(keysyms::KEY_Tab, modifiers);
        assert_eq!(action, Some(KeyAction::FocusNext));
    }

    #[test]
    fn test_no_match_returns_none() {
        let keybindings = Keybindings::new();
        let modifiers = ModifiersState::default();

        let action = keybindings.process(keysyms::KEY_a, modifiers);
        assert_eq!(action, None);
    }
}
