//! Type definitions for efficient compositor data structures
//!
//! This module defines strongly-typed IDs and efficient collections
//! used throughout the compositor.

use slotmap::new_key_type;

// -----------------------------------------------------------------------------
// Stable IDs with O(1) lookup
// -----------------------------------------------------------------------------

new_key_type! {
    /// Unique identifier for a window in the compositor.
    /// Uses slotmap for O(1) insertion, removal, and lookup.
    pub struct WindowId;

    /// Unique identifier for a connected client.
    pub struct ClientId;

    /// Unique identifier for a surface.
    pub struct SurfaceId;

    /// Unique identifier for an output/display.
    pub struct OutputId;
}

// -----------------------------------------------------------------------------
// Window state flags - packed into a single byte
// -----------------------------------------------------------------------------

bitflags::bitflags! {
    /// Window state flags packed efficiently into a single u8.
    /// Much more memory-efficient than separate bool fields.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct WindowFlags: u8 {
        /// Window is currently visible
        const VISIBLE       = 0b0000_0001;
        /// Window is maximized
        const MAXIMIZED     = 0b0000_0010;
        /// Window is fullscreen
        const FULLSCREEN    = 0b0000_0100;
        /// Window is minimized
        const MINIMIZED     = 0b0000_1000;
        /// Window has keyboard focus
        const FOCUSED       = 0b0001_0000;
        /// Window is being resized
        const RESIZING      = 0b0010_0000;
        /// Window is being moved
        const MOVING        = 0b0100_0000;
        /// Window requested attention
        const URGENT        = 0b1000_0000;
    }
}

// -----------------------------------------------------------------------------
// Surface state flags
// -----------------------------------------------------------------------------

bitflags::bitflags! {
    /// Surface state flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct SurfaceFlags: u8 {
        /// Surface has pending damage
        const DAMAGED       = 0b0000_0001;
        /// Surface has a committed buffer
        const HAS_BUFFER    = 0b0000_0010;
        /// Surface is opaque (no alpha blending needed)
        const OPAQUE        = 0b0000_0100;
        /// Surface uses input region
        const INPUT_REGION  = 0b0000_1000;
    }
}

// -----------------------------------------------------------------------------
// Small vectors for common sizes
// -----------------------------------------------------------------------------

/// A small vector that stores up to 4 elements on the stack.
/// Common for: damage rects, child surfaces, subsurfaces
pub type SmallVec4<T> = smallvec::SmallVec<[T; 4]>;

/// A small vector that stores up to 8 elements on the stack.
/// Common for: popup chains, layer surfaces per output
pub type SmallVec8<T> = smallvec::SmallVec<[T; 8]>;

/// A small vector that stores up to 16 elements on the stack.
/// Common for: windows on a workspace, keybindings
pub type SmallVec16<T> = smallvec::SmallVec<[T; 16]>;

// -----------------------------------------------------------------------------
// Fast hasher for integer keys
// -----------------------------------------------------------------------------

/// A HashMap using FxHasher - faster than default SipHash for integer keys.
/// Use when keys are not user-controlled (no HashDoS risk).
pub type FxHashMap<K, V> = std::collections::HashMap<K, V, rustc_hash::FxBuildHasher>;

/// A HashSet using FxHasher.
pub type FxHashSet<K> = std::collections::HashSet<K, rustc_hash::FxBuildHasher>;

// -----------------------------------------------------------------------------
// Coordinate types (newtypes for safety)
// -----------------------------------------------------------------------------

/// Logical coordinates (DPI-independent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LogicalPoint {
    pub x: i32,
    pub y: i32,
}

impl LogicalPoint {
    #[inline]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Logical size (DPI-independent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LogicalSize {
    pub width: u32,
    pub height: u32,
}

impl LogicalSize {
    #[inline]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub const fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

/// Physical coordinates (actual pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalPoint {
    pub x: i32,
    pub y: i32,
}

/// Physical size (actual pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

// -----------------------------------------------------------------------------
// Rectangle for damage tracking
// -----------------------------------------------------------------------------

/// A rectangle for damage tracking and hit testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    #[inline]
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if a point is inside the rectangle.
    #[inline]
    pub const fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x.saturating_add(self.width as i32)
            && y < self.y.saturating_add(self.height as i32)
    }

    /// Check if two rectangles intersect.
    #[inline]
    pub const fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x.saturating_add(other.width as i32)
            && self.x.saturating_add(self.width as i32) > other.x
            && self.y < other.y.saturating_add(other.height as i32)
            && self.y.saturating_add(self.height as i32) > other.y
    }

    /// Compute the intersection of two rectangles.
    #[inline]
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }

        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x.saturating_add(self.width as i32))
            .min(other.x.saturating_add(other.width as i32));
        let bottom = (self.y.saturating_add(self.height as i32))
            .min(other.y.saturating_add(other.height as i32));

        Some(Rect {
            x,
            y,
            width: (right - x) as u32,
            height: (bottom - y) as u32,
        })
    }

    /// Compute the union (bounding box) of two rectangles.
    #[inline]
    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = (self.x.saturating_add(self.width as i32))
            .max(other.x.saturating_add(other.width as i32));
        let bottom = (self.y.saturating_add(self.height as i32))
            .max(other.y.saturating_add(other.height as i32));

        Rect {
            x,
            y,
            width: (right - x) as u32,
            height: (bottom - y) as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 20, 100, 50);
        assert!(rect.contains(10, 20));
        assert!(rect.contains(50, 40));
        assert!(rect.contains(109, 69));
        assert!(!rect.contains(9, 20));
        assert!(!rect.contains(110, 20));
        assert!(!rect.contains(10, 70));
    }

    #[test]
    fn test_rect_intersects() {
        let a = Rect::new(0, 0, 100, 100);
        let b = Rect::new(50, 50, 100, 100);
        let c = Rect::new(200, 200, 10, 10);

        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
        assert!(!c.intersects(&a));
    }

    #[test]
    fn test_rect_intersection() {
        let a = Rect::new(0, 0, 100, 100);
        let b = Rect::new(50, 50, 100, 100);

        let intersection = a.intersection(&b).unwrap();
        assert_eq!(intersection.x, 50);
        assert_eq!(intersection.y, 50);
        assert_eq!(intersection.width, 50);
        assert_eq!(intersection.height, 50);
    }

    #[test]
    fn test_window_flags() {
        let mut flags = WindowFlags::VISIBLE | WindowFlags::FOCUSED;
        assert!(flags.contains(WindowFlags::VISIBLE));
        assert!(flags.contains(WindowFlags::FOCUSED));
        assert!(!flags.contains(WindowFlags::MAXIMIZED));

        flags.insert(WindowFlags::MAXIMIZED);
        assert!(flags.contains(WindowFlags::MAXIMIZED));

        flags.remove(WindowFlags::FOCUSED);
        assert!(!flags.contains(WindowFlags::FOCUSED));
    }
}
