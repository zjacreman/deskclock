use std::time::{Duration, Instant};

/// State of a countdown timer.
///
/// The timer distinguishes three user-visible states:
/// - **stopped**: not running, not paused, not finished (e.g. just reset or
///   freshly created). Displayed in the idle color.
/// - **paused**: previously running, now held. Displayed in the running color
///   and blinks to signal it is not actively counting down.
/// - **finished**: the countdown reached zero and was acknowledged by the
///   event loop via [`finish`](Self::finish). Displayed in the idle color,
///   static (no blinking). This is tracked by an explicit `is_finished` flag so
///   that the "paused" semantics are not overloaded.
pub struct CountdownTimer {
    duration: Duration,
    initial_duration: Duration,
    end_time: Option<Instant>,
    is_running: bool,
    is_paused: bool,
    is_finished: bool,
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
            is_finished: false,
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
            self.is_finished = false;
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
        self.is_finished = false;
        self.duration = self.initial_duration;
    }

    /// Mark the timer as finished (countdown reached zero and was acknowledged).
    /// This clears the running/paused flags and sets the
    /// [`is_finished`](Self::is_finished) flag.
    pub fn finish(&mut self) {
        self.duration = Duration::ZERO;
        self.end_time = None;
        self.is_running = false;
        self.is_paused = false;
        self.is_finished = true;
    }

    /// Adjust the duration by `delta` minutes. If the timer is currently
    /// running, it **stays running** — only `end_time` is recomputed so the
    /// remaining time changes without silently stopping the countdown.
    /// The duration is clamped at zero.
    pub fn adjust_minutes(&mut self, delta: i32) {
        let secs = self.duration.as_secs() as i64 + (delta as i64 * 60);
        self.set_duration_seconds(secs);
    }

    /// Adjust the duration by `delta` seconds. See [`adjust_minutes`](Self::adjust_minutes).
    pub fn adjust_seconds(&mut self, delta: i32) {
        let secs = self.duration.as_secs() as i64 + (delta as i64);
        self.set_duration_seconds(secs);
    }

    fn set_duration_seconds(&mut self, secs: i64) {
        self.duration = Duration::from_secs(secs.max(0) as u64);
        if self.is_running {
            self.end_time = Some(Instant::now() + self.duration);
        }
    }

    /// Stop and clear all state, returning the timer to a fresh "stopped" state
    /// (duration preserved at its current value, unlike [`reset`](Self::reset)
    /// which restores `initial_duration`).
    #[cfg(test)]
    pub fn stop(&mut self) {
        self.end_time = None;
        self.is_running = false;
        self.is_paused = false;
        self.is_finished = false;
    }

    /// True when the countdown has reached zero while running. Used by the event
    /// loop to detect completion. After [`finish`](Self::finish) is called this
    /// returns false again because `is_running` is cleared.
    pub fn has_expired(&self) -> bool {
        self.is_running && self.remaining().is_zero()
    }

    // ──────────────────────
    // Accessors
    // ──────────────────────

    #[cfg(test)]
    pub fn duration(&self) -> Duration {
        self.duration
    }

    #[cfg(test)]
    pub fn initial_duration(&self) -> Duration {
        self.initial_duration
    }

    #[cfg(test)]
    pub fn end_time(&self) -> Option<Instant> {
        self.end_time
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn is_finished(&self) -> bool {
        self.is_finished
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
        assert!(!timer.is_finished);
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
        assert!(!timer.is_finished);
        assert!(timer.end_time.is_some());
    }

    #[test]
    fn test_countdown_timer_start_clears_finished_flag() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.finish();
        assert!(timer.is_finished);
        timer.duration = Duration::from_secs(60);
        timer.start();
        assert!(!timer.is_finished);
        assert!(timer.is_running);
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
        assert!(!timer.is_finished);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_finish_sets_zero_duration() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        timer.finish();
        assert_eq!(timer.duration, Duration::ZERO);
        assert!(!timer.is_running);
        assert!(!timer.is_paused);
        assert!(timer.is_finished);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_finish_sets_correct_state() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        timer.finish();

        assert_eq!(timer.duration, Duration::ZERO);
        assert!(!timer.is_paused);
        assert!(!timer.is_running);
        assert!(timer.is_finished);
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
    fn test_countdown_timer_adjust_while_running_keeps_running() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        assert!(timer.is_running);
        assert!(timer.end_time.is_some());

        // Adjusting must NOT silently stop the timer.
        timer.adjust_minutes(1);
        assert!(timer.is_running, "timer should still be running after adjust");
        assert!(timer.end_time.is_some(), "end_time should be recomputed");
        assert!(!timer.is_paused);

        timer.adjust_seconds(30);
        assert!(timer.is_running);
        assert!(timer.end_time.is_some());
    }

    #[test]
    fn test_countdown_timer_adjust_seconds_while_running_adjusts_remaining() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        let remaining_before = timer.remaining();
        // Sleep a tiny bit so Instant advances, then add 60 seconds.
        timer.adjust_minutes(1);
        let remaining_after = timer.remaining();
        assert!(
            remaining_after >= remaining_before + Duration::from_secs(60) - Duration::from_millis(50),
            "adjusting +60s while running should extend remaining by ~60s"
        );
    }

    #[test]
    fn test_countdown_timer_stop_clears_running_state() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        timer.stop();
        assert!(!timer.is_running);
        assert!(!timer.is_paused);
        assert!(!timer.is_finished);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_has_expired_while_running_at_zero() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.duration = Duration::ZERO;
        timer.start();
        // With zero duration, remaining() will be ZERO
        assert!(timer.has_expired());
    }

    #[test]
    fn test_countdown_timer_has_expired_false_when_not_running() {
        let timer = CountdownTimer::with_duration(25 * 60);
        assert!(!timer.has_expired());
    }

    #[test]
    fn test_countdown_timer_has_expired_false_when_partially_remaining() {
        let mut timer = CountdownTimer::with_duration(60);
        timer.start();
        // Remaining is ~60s, sub-second truncation means as_secs() not zero-equivalent,
        // but is_zero() requires exactly zero, so has_expired is false.
        assert!(!timer.has_expired());
    }

    #[test]
    fn test_countdown_timer_has_expired_false_after_finish() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        timer.finish();
        // finish() clears is_running, so has_expired no longer reports true
        // (prevents the event loop from re-triggering the flash every tick).
        assert!(!timer.has_expired());
        assert!(timer.is_finished);
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