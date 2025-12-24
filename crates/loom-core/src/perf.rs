//! Performance monitoring and frame timing
//!
//! This module provides tools for tracking frame times, detecting stutters,
//! and collecting performance metrics.

use std::time::{Duration, Instant};

/// Target frame time for 60 FPS (16.67ms)
pub const TARGET_FRAME_TIME_60FPS: Duration = Duration::from_micros(16_667);

/// Target frame time for 120 FPS (8.33ms)
pub const TARGET_FRAME_TIME_120FPS: Duration = Duration::from_micros(8_333);

/// Target frame time for 144 FPS (6.94ms)
pub const TARGET_FRAME_TIME_144FPS: Duration = Duration::from_micros(6_944);

/// Threshold for considering a frame as stuttering (2x target)
const STUTTER_THRESHOLD_MULTIPLIER: u32 = 2;

/// Number of frame times to keep in history
const FRAME_TIME_HISTORY_SIZE: usize = 120;

/// Frame timing statistics.
#[derive(Debug, Clone)]
pub struct FrameStats {
    /// Last frame time
    pub last_frame_time: Duration,
    /// Average frame time over history
    pub avg_frame_time: Duration,
    /// Minimum frame time in history
    pub min_frame_time: Duration,
    /// Maximum frame time in history
    pub max_frame_time: Duration,
    /// Number of stutters detected
    pub stutter_count: u64,
    /// Current FPS (based on average)
    pub fps: f64,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self {
            last_frame_time: Duration::ZERO,
            avg_frame_time: Duration::ZERO,
            min_frame_time: Duration::MAX,
            max_frame_time: Duration::ZERO,
            stutter_count: 0,
            fps: 0.0,
        }
    }
}

/// Frame time tracker for performance monitoring.
///
/// Uses a ring buffer to avoid allocations during frame recording.
pub struct FrameTimer {
    /// Ring buffer of frame times
    frame_times: [Duration; FRAME_TIME_HISTORY_SIZE],
    /// Current position in the ring buffer
    index: usize,
    /// Number of frames recorded (up to FRAME_TIME_HISTORY_SIZE)
    count: usize,
    /// Last frame start time
    frame_start: Instant,
    /// Target frame time
    target_frame_time: Duration,
    /// Total stutter count
    stutter_count: u64,
}

impl FrameTimer {
    /// Create a new frame timer targeting 60 FPS.
    #[inline]
    pub fn new() -> Self {
        Self::with_target(TARGET_FRAME_TIME_60FPS)
    }

    /// Create a new frame timer with a custom target frame time.
    #[inline]
    pub fn with_target(target: Duration) -> Self {
        Self {
            frame_times: [Duration::ZERO; FRAME_TIME_HISTORY_SIZE],
            index: 0,
            count: 0,
            frame_start: Instant::now(),
            target_frame_time: target,
            stutter_count: 0,
        }
    }

    /// Mark the start of a new frame.
    #[inline]
    pub fn begin_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    /// Mark the end of the current frame and record its duration.
    ///
    /// Returns `true` if the frame was considered a stutter.
    #[inline]
    pub fn end_frame(&mut self) -> bool {
        let frame_time = self.frame_start.elapsed();
        self.record_frame_time(frame_time)
    }

    /// Record a frame time directly (for external timing).
    #[inline]
    pub fn record_frame_time(&mut self, frame_time: Duration) -> bool {
        // Store in ring buffer
        self.frame_times[self.index] = frame_time;
        self.index = (self.index + 1) % FRAME_TIME_HISTORY_SIZE;
        if self.count < FRAME_TIME_HISTORY_SIZE {
            self.count += 1;
        }

        // Check for stutter
        let stutter_threshold = self.target_frame_time * STUTTER_THRESHOLD_MULTIPLIER;
        let is_stutter = frame_time > stutter_threshold;
        if is_stutter {
            self.stutter_count += 1;
        }
        is_stutter
    }

    /// Get current frame statistics.
    pub fn stats(&self) -> FrameStats {
        if self.count == 0 {
            return FrameStats::default();
        }

        let mut sum = Duration::ZERO;
        let mut min = Duration::MAX;
        let mut max = Duration::ZERO;

        for i in 0..self.count {
            let time = self.frame_times[i];
            sum += time;
            min = min.min(time);
            max = max.max(time);
        }

        let avg = sum / self.count as u32;
        let last = if self.index == 0 {
            self.frame_times[FRAME_TIME_HISTORY_SIZE - 1]
        } else {
            self.frame_times[self.index - 1]
        };

        let fps = if avg.as_nanos() > 0 {
            1_000_000_000.0 / avg.as_nanos() as f64
        } else {
            0.0
        };

        FrameStats {
            last_frame_time: last,
            avg_frame_time: avg,
            min_frame_time: min,
            max_frame_time: max,
            stutter_count: self.stutter_count,
            fps,
        }
    }

    /// Reset all statistics.
    #[inline]
    pub fn reset(&mut self) {
        self.frame_times = [Duration::ZERO; FRAME_TIME_HISTORY_SIZE];
        self.index = 0;
        self.count = 0;
        self.stutter_count = 0;
    }

    /// Get the target frame time.
    #[inline]
    pub const fn target_frame_time(&self) -> Duration {
        self.target_frame_time
    }

    /// Set a new target frame time.
    #[inline]
    pub fn set_target_frame_time(&mut self, target: Duration) {
        self.target_frame_time = target;
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Scope-based timer for measuring specific operations.
///
/// Usage:
/// ```ignore
/// let timer = ScopedTimer::new("render");
/// // ... do work ...
/// drop(timer); // logs duration
/// ```
pub struct ScopedTimer {
    name: &'static str,
    start: Instant,
    threshold: Duration,
}

impl ScopedTimer {
    /// Create a new scoped timer that logs if duration exceeds 1ms.
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
            threshold: Duration::from_millis(1),
        }
    }

    /// Create a scoped timer with a custom threshold.
    #[inline]
    pub fn with_threshold(name: &'static str, threshold: Duration) -> Self {
        Self {
            name,
            start: Instant::now(),
            threshold,
        }
    }

    /// Get the elapsed time without logging.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        if elapsed > self.threshold {
            tracing::warn!(
                target: "perf",
                "{} took {:?} (threshold: {:?})",
                self.name,
                elapsed,
                self.threshold
            );
        } else {
            tracing::trace!(
                target: "perf",
                "{} took {:?}",
                self.name,
                elapsed
            );
        }
    }
}

/// Macro for timing a block of code.
///
/// Usage:
/// ```ignore
/// time_block!("render_surfaces", {
///     // ... expensive operation ...
/// });
/// ```
#[macro_export]
macro_rules! time_block {
    ($name:expr, $block:block) => {{
        let _timer = $crate::perf::ScopedTimer::new($name);
        $block
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_frame_timer_basic() {
        let mut timer = FrameTimer::new();

        // Record some frame times
        timer.record_frame_time(Duration::from_millis(16));
        timer.record_frame_time(Duration::from_millis(17));
        timer.record_frame_time(Duration::from_millis(15));

        let stats = timer.stats();
        assert_eq!(stats.min_frame_time, Duration::from_millis(15));
        assert_eq!(stats.max_frame_time, Duration::from_millis(17));
        assert!(stats.fps > 58.0 && stats.fps < 67.0);
    }

    #[test]
    fn test_stutter_detection() {
        let mut timer = FrameTimer::new();

        // Normal frame - should not stutter
        let is_stutter = timer.record_frame_time(Duration::from_millis(16));
        assert!(!is_stutter);

        // Stuttering frame (> 2x target)
        let is_stutter = timer.record_frame_time(Duration::from_millis(50));
        assert!(is_stutter);

        assert_eq!(timer.stats().stutter_count, 1);
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let mut timer = FrameTimer::new();

        // Fill the buffer and overflow
        for i in 0..150 {
            timer.record_frame_time(Duration::from_millis(16 + (i % 3) as u64));
        }

        let stats = timer.stats();
        assert!(stats.avg_frame_time >= Duration::from_millis(16));
        assert!(stats.avg_frame_time <= Duration::from_millis(18));
    }

    #[test]
    fn test_begin_end_frame() {
        let mut timer = FrameTimer::new();

        timer.begin_frame();
        thread::sleep(Duration::from_millis(5));
        timer.end_frame();

        let stats = timer.stats();
        assert!(stats.last_frame_time >= Duration::from_millis(5));
    }
}
