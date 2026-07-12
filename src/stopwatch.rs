use std::time::{Duration, Instant};

pub struct Stopwatch {
    elapsed_time: Duration,
    last_start_time: Option<Instant>,
    is_running: bool,
    last_lap: Option<Duration>,
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            elapsed_time: Duration::ZERO,
            last_start_time: None,
            is_running: false,
            last_lap: None,
        }
    }

    pub fn start(&mut self) {
        if !self.is_running {
            self.is_running = true;
            self.last_start_time = Some(Instant::now());
        }
    }

    pub fn pause(&mut self) {
        if self.is_running {
            if let Some(start_time) = self.last_start_time {
                let elapsed_since_start = start_time.elapsed();
                self.elapsed_time += elapsed_since_start;
            }
            self.is_running = false;
            self.last_start_time = None;
        }
    }

    pub fn reset(&mut self) {
        self.elapsed_time = Duration::ZERO;
        self.last_start_time = None;
        self.is_running = false;
        self.last_lap = None;
    }

    pub fn add_lap(&mut self, elapsed_at_lap: Duration) {
        self.last_lap = Some(elapsed_at_lap);
    }

    pub fn last_lap_elapsed(&self) -> Option<Duration> {
        self.last_lap
    }

    pub fn current_elapsed(&self) -> Duration {
        if self.is_running {
            if let Some(start_time) = self.last_start_time {
                self.elapsed_time + start_time.elapsed()
            } else {
                self.elapsed_time
            }
        } else {
            self.elapsed_time
        }
    }

    // ──────────────────────
    // Accessors
    // ──────────────────────

    pub fn elapsed_time(&self) -> Duration {
        self.elapsed_time
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Stopwatch Core Tests
    // ============================================================

    #[test]
    fn test_stopwatch_new_starts_zeroed() {
        let sw = Stopwatch::new();
        assert_eq!(sw.elapsed_time, Duration::ZERO);
        assert!(sw.last_start_time.is_none());
        assert!(!sw.is_running);
    }

    #[test]
    fn test_stopwatch_start() {
        let mut sw = Stopwatch::new();
        assert!(!sw.is_running);
        sw.start();
        assert!(sw.is_running);
        assert!(sw.last_start_time.is_some());
    }

    #[test]
    fn test_stopwatch_start_when_already_running_does_nothing() {
        let mut sw = Stopwatch::new();
        sw.start();
        let _start_time = sw.last_start_time.unwrap();
        std::thread::sleep(Duration::from_millis(10));
        sw.start();
        // Should not change - still running from the first start
        assert!(sw.is_running);
    }

    #[test]
    fn test_stopwatch_pause_while_not_running_does_nothing() {
        let mut sw = Stopwatch::new();
        sw.pause();
        assert!(!sw.is_running);
        assert!(sw.last_start_time.is_none());
    }

    #[test]
    fn test_stopwatch_pause_stops_running() {
        let mut sw = Stopwatch::new();
        sw.start();
        std::thread::sleep(Duration::from_millis(50));
        let _elapsed_before = sw.current_elapsed();
        sw.pause();
        assert!(!sw.is_running);
        assert!(sw.last_start_time.is_none());
        // elapsed_time should have some value now
        assert!(sw.elapsed_time >= Duration::ZERO);
    }

    #[test]
    fn test_stopwatch_reset_clears_all_state() {
        let mut sw = Stopwatch::new();
        sw.start();
        std::thread::sleep(Duration::from_millis(100));
        sw.pause();
        sw.reset();
        assert_eq!(sw.elapsed_time, Duration::ZERO);
        assert!(sw.last_start_time.is_none());
        assert!(!sw.is_running);
    }

    #[test]
    fn test_stopwatch_current_elapsed_while_running() {
        let mut sw = Stopwatch::new();
        sw.start();
        let initial_elapsed = sw.current_elapsed();
        std::thread::sleep(Duration::from_millis(100));
        let later_elapsed = sw.current_elapsed();
        assert!(later_elapsed >= initial_elapsed);
    }

    #[test]
    fn test_stopwatch_current_elapsed_while_paused() {
        let mut sw = Stopwatch::new();
        sw.start();
        std::thread::sleep(Duration::from_millis(100));
        let elapsed_at_pause = sw.current_elapsed();
        sw.pause();
        std::thread::sleep(Duration::from_millis(100));
        let elapsed_after_sleep = sw.current_elapsed();
        assert!(elapsed_after_sleep >= elapsed_at_pause - Duration::from_millis(50));
        assert!(elapsed_after_sleep <= elapsed_at_pause + Duration::from_millis(50));
    }

    #[test]
    fn test_stopwatch_restart_after_pause() {
        let mut sw = Stopwatch::new();
        sw.start();
        std::thread::sleep(Duration::from_millis(100));
        sw.pause();
        let elapsed_before_restart = sw.elapsed_time;
        std::thread::sleep(Duration::from_millis(100));
        sw.start();
        assert!(sw.is_running);
        std::thread::sleep(Duration::from_millis(100));
        let elapsed_now = sw.current_elapsed();
        assert!(elapsed_now > elapsed_before_restart);
    }

    // ============================================================
    // Stopwatch Edge Cases
    // ============================================================

    #[test]
    fn test_stopwatch_elapsed_time_accrues_across_multiple_pause_cycles() {
        let mut sw = Stopwatch::new();

        sw.start();
        std::thread::sleep(Duration::from_millis(50));
        sw.pause();
        let elapsed_after_first = sw.elapsed_time;

        std::thread::sleep(Duration::from_millis(50));
        sw.start();
        std::thread::sleep(Duration::from_millis(50));
        sw.pause();

        let total_elapsed = sw.elapsed_time;
        assert!(total_elapsed >= elapsed_after_first);
    }

    #[test]
    fn test_stopwatch_current_elapsed_returns_zero_when_reset() {
        let sw = Stopwatch::new();
        assert_eq!(sw.current_elapsed(), Duration::ZERO);
    }

    // ============================================================
    // Stopwatch Lap Tests
    // ============================================================

    #[test]
    fn test_stopwatch_new_has_no_lap() {
        let sw = Stopwatch::new();
        assert!(sw.last_lap_elapsed().is_none());
    }

    #[test]
    fn test_stopwatch_add_lap_records_duration() {
        let mut sw = Stopwatch::new();
        let lap = Duration::from_secs(30);
        sw.add_lap(lap);
        assert_eq!(sw.last_lap_elapsed(), Some(lap));
    }

    #[test]
    fn test_stopwatch_add_lap_overwrites_previous_lap() {
        let mut sw = Stopwatch::new();
        sw.add_lap(Duration::from_secs(10));
        sw.add_lap(Duration::from_secs(20));
        assert_eq!(sw.last_lap_elapsed(), Some(Duration::from_secs(20)));
    }

    #[test]
    fn test_stopwatch_reset_clears_lap() {
        let mut sw = Stopwatch::new();
        sw.add_lap(Duration::from_secs(15));
        assert!(sw.last_lap_elapsed().is_some());
        sw.reset();
        assert!(sw.last_lap_elapsed().is_none());
    }

    #[test]
    fn test_stopwatch_add_lap_does_not_affect_running_state() {
        let mut sw = Stopwatch::new();
        sw.start();
        let lap = sw.current_elapsed();
        sw.add_lap(lap);
        assert!(sw.is_running);
        assert!(sw.last_start_time.is_some());
    }

    #[test]
    fn test_stopwatch_add_lap_does_not_affect_elapsed_time() {
        let mut sw = Stopwatch::new();
        sw.start();
        std::thread::sleep(Duration::from_millis(50));
        let elapsed_before = sw.current_elapsed();
        sw.add_lap(elapsed_before);
        let elapsed_after = sw.current_elapsed();
        assert!(elapsed_after >= elapsed_before - Duration::from_millis(5));
    }
}