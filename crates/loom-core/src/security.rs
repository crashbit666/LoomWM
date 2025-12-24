//! Security constants and resource limits
//!
//! This module defines security-related constants that prevent resource exhaustion
//! and other denial-of-service attacks. All limits are conservative defaults that
//! can be adjusted via configuration.

/// Maximum number of simultaneous Wayland clients
pub const MAX_CLIENTS: usize = 256;

/// Maximum number of surfaces per client
pub const MAX_SURFACES_PER_CLIENT: usize = 100;

/// Maximum total surfaces across all clients
pub const MAX_TOTAL_SURFACES: usize = 10_000;

/// Maximum buffer size in bytes (256 MB)
pub const MAX_BUFFER_SIZE: usize = 256 * 1024 * 1024;

/// Maximum buffer dimensions (16K resolution)
pub const MAX_BUFFER_WIDTH: u32 = 16384;
pub const MAX_BUFFER_HEIGHT: u32 = 16384;

/// Maximum pending Wayland messages per client before throttling
pub const MAX_PENDING_MESSAGES: usize = 1000;

/// Maximum clipboard size in bytes (16 MB)
pub const MAX_CLIPBOARD_SIZE: usize = 16 * 1024 * 1024;

/// Maximum number of keyboard shortcuts
pub const MAX_KEYBINDINGS: usize = 500;

/// Maximum window title length in bytes
pub const MAX_TITLE_LENGTH: usize = 4096;

/// Maximum app ID length in bytes
pub const MAX_APP_ID_LENGTH: usize = 512;

/// Validate that a buffer size is within security limits
#[inline]
pub const fn is_valid_buffer_size(width: u32, height: u32, bytes_per_pixel: u32) -> bool {
    width <= MAX_BUFFER_WIDTH
        && height <= MAX_BUFFER_HEIGHT
        && (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(bytes_per_pixel as usize)
            <= MAX_BUFFER_SIZE
}

/// Validate string length for titles and identifiers
#[inline]
pub fn is_valid_title(title: &str) -> bool {
    title.len() <= MAX_TITLE_LENGTH
}

/// Validate app ID length
#[inline]
pub fn is_valid_app_id(app_id: &str) -> bool {
    app_id.len() <= MAX_APP_ID_LENGTH && app_id.chars().all(|c| c.is_ascii_graphic() || c == ' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_buffer_size() {
        // Normal HD buffer
        assert!(is_valid_buffer_size(1920, 1080, 4));

        // 4K buffer
        assert!(is_valid_buffer_size(3840, 2160, 4));

        // 8K buffer
        assert!(is_valid_buffer_size(7680, 4320, 4));

        // Too wide
        assert!(!is_valid_buffer_size(20000, 1080, 4));

        // Too tall
        assert!(!is_valid_buffer_size(1920, 20000, 4));

        // Too much memory
        assert!(!is_valid_buffer_size(16384, 16384, 4));
    }

    #[test]
    fn test_valid_title() {
        assert!(is_valid_title("Normal Title"));
        assert!(is_valid_title(&"a".repeat(MAX_TITLE_LENGTH)));
        assert!(!is_valid_title(&"a".repeat(MAX_TITLE_LENGTH + 1)));
    }

    #[test]
    fn test_valid_app_id() {
        assert!(is_valid_app_id("org.example.App"));
        assert!(is_valid_app_id("firefox"));
        assert!(!is_valid_app_id(&"a".repeat(MAX_APP_ID_LENGTH + 1)));
        assert!(!is_valid_app_id("app\nid")); // newline not allowed
        assert!(!is_valid_app_id("app\x00id")); // null not allowed
    }
}
