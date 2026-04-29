use std::time::{Duration, Instant};

pub struct Stopwatch {
    pub elapsed_time: Duration,
    pub last_start_time: Option<Instant>,
    pub is_running: bool,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            elapsed_time: Duration::ZERO,
            last_start_time: None,
            is_running: false,
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
