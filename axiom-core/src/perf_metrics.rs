// Performance metrics tracking for render optimization
// Monitors FPS, render times, memory usage, and LOD statistics

use std::time::{Duration, Instant};

/// Performance metrics for a single frame
#[derive(Clone, Debug)]
pub struct FrameMetrics {
    pub frame_start: Instant,
    pub frame_duration: Duration,
    pub render_duration: Duration,
    pub atoms_total: usize,
    pub atoms_rendered: usize,
    pub atoms_culled: usize,
    pub lod_high: usize,
    pub lod_medium: usize,
    pub lod_low: usize,
    pub lod_minimal: usize,
}

impl FrameMetrics {
    pub fn new() -> Self {
        Self {
            frame_start: Instant::now(),
            frame_duration: Duration::ZERO,
            render_duration: Duration::ZERO,
            atoms_total: 0,
            atoms_rendered: 0,
            atoms_culled: 0,
            lod_high: 0,
            lod_medium: 0,
            lod_low: 0,
            lod_minimal: 0,
        }
    }

    pub fn fps(&self) -> f64 {
        if self.frame_duration.as_secs_f64() > 0.0 {
            1.0 / self.frame_duration.as_secs_f64()
        } else {
            0.0
        }
    }

    pub fn render_time_ms(&self) -> f64 {
        self.render_duration.as_secs_f64() * 1000.0
    }

    pub fn frame_time_ms(&self) -> f64 {
        self.frame_duration.as_secs_f64() * 1000.0
    }
}

/// Rolling average performance tracker
pub struct PerformanceTracker {
    recent_frames: Vec<FrameMetrics>,
    max_history: usize,
    current_frame: Option<FrameMetrics>,
}

impl PerformanceTracker {
    pub fn new(max_history: usize) -> Self {
        Self {
            recent_frames: Vec::with_capacity(max_history),
            max_history,
            current_frame: None,
        }
    }

    /// Start tracking a new frame
    pub fn start_frame(&mut self) {
        self.current_frame = Some(FrameMetrics::new());
    }

    /// Record render start time
    pub fn start_render(&mut self) {
        if let Some(ref mut frame) = self.current_frame {
            frame.frame_start = Instant::now();
        }
    }

    /// Record render completion
    pub fn end_render(&mut self, atoms_total: usize, atoms_rendered: usize, atoms_culled: usize) {
        if let Some(ref mut frame) = self.current_frame {
            frame.render_duration = frame.frame_start.elapsed();
            frame.atoms_total = atoms_total;
            frame.atoms_rendered = atoms_rendered;
            frame.atoms_culled = atoms_culled;
        }
    }

    /// Record LOD statistics
    pub fn record_lod_stats(&mut self, high: usize, medium: usize, low: usize, minimal: usize) {
        if let Some(ref mut frame) = self.current_frame {
            frame.lod_high = high;
            frame.lod_medium = medium;
            frame.lod_low = low;
            frame.lod_minimal = minimal;
        }
    }

    /// Complete current frame and add to history
    pub fn end_frame(&mut self) {
        if let Some(mut frame) = self.current_frame.take() {
            frame.frame_duration = frame.frame_start.elapsed();

            // Add to history
            self.recent_frames.push(frame);

            // Maintain max history size
            if self.recent_frames.len() > self.max_history {
                self.recent_frames.remove(0);
            }
        }
    }

    /// Get average FPS over recent frames
    pub fn avg_fps(&self) -> f64 {
        if self.recent_frames.is_empty() {
            return 0.0;
        }

        let total_fps: f64 = self.recent_frames.iter().map(|f| f.fps()).sum();
        total_fps / self.recent_frames.len() as f64
    }

    /// Get average render time (ms)
    pub fn avg_render_time_ms(&self) -> f64 {
        if self.recent_frames.is_empty() {
            return 0.0;
        }

        let total: f64 = self.recent_frames.iter().map(|f| f.render_time_ms()).sum();
        total / self.recent_frames.len() as f64
    }

    /// Get most recent frame metrics
    pub fn latest(&self) -> Option<&FrameMetrics> {
        self.recent_frames.last()
    }

    /// Get summary statistics
    pub fn summary(&self) -> PerfSummary {
        if self.recent_frames.is_empty() {
            return PerfSummary::default();
        }

        let avg_fps = self.avg_fps();
        let avg_render_ms = self.avg_render_time_ms();

        let latest = self.recent_frames.last().unwrap();

        PerfSummary {
            avg_fps,
            avg_render_ms,
            atoms_total: latest.atoms_total,
            atoms_rendered: latest.atoms_rendered,
            atoms_culled: latest.atoms_culled,
            lod_high: latest.lod_high,
            lod_medium: latest.lod_medium,
            lod_low: latest.lod_low,
            lod_minimal: latest.lod_minimal,
            sample_count: self.recent_frames.len(),
        }
    }
}

/// Performance summary for display
#[derive(Clone, Debug, Default)]
pub struct PerfSummary {
    pub avg_fps: f64,
    pub avg_render_ms: f64,
    pub atoms_total: usize,
    pub atoms_rendered: usize,
    pub atoms_culled: usize,
    pub lod_high: usize,
    pub lod_medium: usize,
    pub lod_low: usize,
    pub lod_minimal: usize,
    pub sample_count: usize,
}

impl PerfSummary {
    pub fn culling_efficiency(&self) -> f64 {
        if self.atoms_total == 0 {
            return 0.0;
        }
        (self.atoms_culled as f64 / self.atoms_total as f64) * 100.0
    }

    pub fn render_efficiency(&self) -> f64 {
        if self.atoms_total == 0 {
            return 100.0;
        }
        (self.atoms_rendered as f64 / self.atoms_total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_frame_metrics() {
        let mut tracker = PerformanceTracker::new(60);

        tracker.start_frame();
        tracker.start_render();
        thread::sleep(Duration::from_millis(10));
        tracker.end_render(1000, 800, 200);
        tracker.end_frame();

        let summary = tracker.summary();
        assert_eq!(summary.atoms_total, 1000);
        assert_eq!(summary.atoms_rendered, 800);
        assert_eq!(summary.atoms_culled, 200);
        assert!(summary.avg_render_ms >= 10.0);
    }

    #[test]
    fn test_rolling_average() {
        let mut tracker = PerformanceTracker::new(3);

        // Add 5 frames, should only keep last 3
        for i in 0..5 {
            tracker.start_frame();
            tracker.end_render(100 * i, 80 * i, 20 * i);
            tracker.end_frame();
        }

        assert_eq!(tracker.recent_frames.len(), 3);
    }

    #[test]
    fn test_culling_efficiency() {
        let summary = PerfSummary {
            atoms_total: 1000,
            atoms_rendered: 600,
            atoms_culled: 400,
            ..Default::default()
        };

        assert_eq!(summary.culling_efficiency(), 40.0);
        assert_eq!(summary.render_efficiency(), 60.0);
    }
}
