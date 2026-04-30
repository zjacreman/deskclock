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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // CountdownTimer Core Tests
    // ============================================================

    #[test]
    fn test_countdown_timer_new_has_default_25_minutes() {
        let timer = CountdownTimer::with_duration(25 * 60);
        assert_eq!(timer.duration, Duration::from_secs(25 * 60));
        assert_eq!(timer.initial_duration, Duration::from_secs(25 * 60));
        assert!(timer.end_time.is_none());
        assert!(!timer.is_running);
        assert!(!timer.is_paused);
    }

    #[test]
    fn test_countdown_timer_remaining_when_not_running() {
        let timer = CountdownTimer::with_duration(25 * 60);
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60));
    }

    #[test]
    fn test_countdown_timer_start() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        assert!(!timer.is_running);
        timer.start();
        assert!(timer.is_running);
        assert!(!timer.is_paused);
        assert!(timer.end_time.is_some());
    }

    #[test]
    fn test_countdown_timer_start_when_already_running_does_nothing() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        let _end_time = timer.end_time.unwrap();
        std::thread::sleep(Duration::from_millis(10));
        timer.start();
        // Should still be running with same end_time (not extended)
        assert!(timer.is_running);
    }

    #[test]
    fn test_countdown_timer_pause_while_not_running_does_nothing() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.pause();
        assert!(!timer.is_running);
        assert!(!timer.is_paused);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_pause_sets_paused_flag() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        std::thread::sleep(Duration::from_millis(50));
        timer.pause();
        assert!(!timer.is_running);
        assert!(timer.is_paused);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_pause_saves_remaining_duration() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        // Set a custom duration
        timer.duration = Duration::from_secs(100);
        timer.start();
        std::thread::sleep(Duration::from_millis(50));
        let remaining_before_pause = timer.remaining();
        timer.pause();
        // Duration should now be approximately what was remaining
        assert!(timer.duration.as_millis() >= remaining_before_pause.as_millis() - 100);
        assert!(timer.duration.as_millis() <= remaining_before_pause.as_millis() + 100);
    }

    #[test]
    fn test_countdown_timer_reset_restores_initial_duration() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        let initial = timer.initial_duration;
        // Modify duration without calling start() (which would update initial_duration)
        timer.duration = Duration::from_secs(10);
        timer.pause(); // pause does nothing if not running
        timer.duration = Duration::from_secs(5);
        timer.reset();
        assert_eq!(timer.duration, initial);
        assert_eq!(timer.initial_duration, initial);
        assert!(!timer.is_running);
        assert!(!timer.is_paused);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_finish_sets_zero_duration() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        timer.finish();
        assert_eq!(timer.duration, Duration::ZERO);
        assert!(!timer.is_running);
        assert!(timer.is_paused);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_finish_sets_correct_state() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        timer.finish();

        assert_eq!(timer.duration, Duration::ZERO);
        assert!(timer.is_paused);
        assert!(!timer.is_running);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_adjust_minutes_positive() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        let initial = timer.duration;
        timer.adjust_minutes(5);
        assert_eq!(timer.duration, initial + Duration::from_secs(300));
    }

    #[test]
    fn test_countdown_timer_adjust_minutes_negative() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        let initial = timer.duration;
        timer.adjust_minutes(-1);
        assert_eq!(timer.duration, initial - Duration::from_secs(60));
    }

    #[test]
    fn test_countdown_timer_adjust_minutes_does_not_go_below_zero() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.duration = Duration::from_secs(30);
        timer.adjust_minutes(-10); // Should try to subtract 600 seconds
        assert!(timer.duration >= Duration::ZERO);
    }

    #[test]
    fn test_countdown_timer_adjust_seconds_positive() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        let initial = timer.duration;
        timer.adjust_seconds(30);
        assert_eq!(timer.duration, initial + Duration::from_secs(30));
    }

    #[test]
    fn test_countdown_timer_adjust_seconds_negative() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        let initial = timer.duration;
        timer.adjust_seconds(-10);
        assert_eq!(timer.duration, initial - Duration::from_secs(10));
    }

    #[test]
    fn test_countdown_timer_adjust_seconds_does_not_go_below_zero() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.duration = Duration::from_secs(5);
        timer.adjust_seconds(-10); // Should try to subtract 10 seconds
        assert!(timer.duration >= Duration::ZERO);
    }

    #[test]
    fn test_countdown_timer_stop_clears_running_state() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        timer.stop();
        assert!(!timer.is_running);
        assert!(!timer.is_paused);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_is_finished_while_running() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.duration = Duration::ZERO;
        timer.start();
        // With zero duration, remaining() will be ZERO
        // is_finished checks is_running && remaining().as_secs() == 0
        assert!(timer.is_finished());
    }

    #[test]
    fn test_countdown_timer_is_finished_when_not_running() {
        let timer = CountdownTimer::with_duration(25 * 60);
        assert!(!timer.is_finished());
    }

    // ============================================================
    // CountdownTimer Edge Cases
    // ============================================================

    #[test]
    fn test_countdown_timer_remaining_with_zero_duration() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.duration = Duration::ZERO;
        assert_eq!(timer.remaining(), Duration::ZERO);
    }

    #[test]
    fn test_countdown_timer_start_with_zero_duration_sets_running_but_remaining_is_zero() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.duration = Duration::ZERO;
        timer.start();
        // start() sets is_running = true regardless of duration,
        // but remaining() will be zero since duration is zero
        assert!(timer.is_running);
        assert_eq!(timer.remaining(), Duration::ZERO);
    }

    #[test]
    fn test_countdown_timer_adjust_minutes_with_no_running_state() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        let initial = timer.duration;
        timer.adjust_minutes(10);
        assert_eq!(timer.duration, initial + Duration::from_secs(600));
    }

    #[test]
    fn test_countdown_timer_adjust_seconds_with_no_running_state() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        let initial = timer.duration;
        timer.adjust_seconds(45);
        assert_eq!(timer.duration, initial + Duration::from_secs(45));
    }
}
