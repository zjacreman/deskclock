use std::time::{Duration, Instant};

pub struct Stopwatch {
    pub elapsed_time: Duration,
    pub last_start_time: Option<Instant>,
    pub is_running: bool,
    pub last_lap: Option<Duration>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
