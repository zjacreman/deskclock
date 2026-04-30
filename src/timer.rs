use std::time::{Duration, Instant};

pub struct CountdownTimer {
    pub duration: Duration,
    pub initial_duration: Duration,
    pub end_time: Option<Instant>,
    pub is_running: bool,
    pub is_paused: bool,
}

impl CountdownTimer {
    pub fn with_duration(secs: u64) -> Self {
        let default_dur = Duration::from_secs(secs);
        Self {
            duration: default_dur,
            initial_duration: default_dur,
            end_time: None,
            is_running: false,
            is_paused: false,
        }
    }

    pub fn remaining(&self) -> Duration {
        if let Some(end) = self.end_time {
            end.saturating_duration_since(Instant::now())
        } else {
            self.duration
        }
    }

    pub fn start(&mut self) {
        if !self.is_running {
            if self.duration > Duration::ZERO {
                self.initial_duration = self.duration;
            }
            self.end_time = Some(Instant::now() + self.remaining());
            self.is_running = true;
            self.is_paused = false;
        }
    }

    pub fn pause(&mut self) {
        if self.is_running {
            self.duration = self.remaining();
            self.end_time = None;
            self.is_running = false;
            self.is_paused = true;
        }
    }

    pub fn reset(&mut self) {
        self.end_time = None;
        self.is_running = false;
        self.is_paused = false;
        self.duration = self.initial_duration;
    }

    pub fn finish(&mut self) {
        self.duration = Duration::ZERO;
        self.end_time = None;
        self.is_running = false;
        self.is_paused = true;
    }

    pub fn adjust_minutes(&mut self, delta: i32) {
        self.stop();
        let secs = self.duration.as_secs() as i64 + (delta as i64 * 60);
        self.duration = Duration::from_secs(secs.max(0) as u64);
    }

    pub fn adjust_seconds(&mut self, delta: i32) {
        self.stop();
        let secs = self.duration.as_secs() as i64 + (delta as i64);
        self.duration = Duration::from_secs(secs.max(0) as u64);
    }

    pub fn stop(&mut self) {
        self.end_time = None;
        self.is_running = false;
        self.is_paused = false;
    }

    pub fn is_finished(&self) -> bool {
        self.is_running && self.remaining().as_secs() == 0
    }
}
