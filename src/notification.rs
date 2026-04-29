use std::fmt;

pub trait Notifier: fmt::Debug + Send + Sync {
    fn send_notification(&mut self, title: &str, body: &str);
}

#[derive(Debug)]
pub struct SystemNotifier {
    enabled: bool,
}

impl SystemNotifier {
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

#[cfg(target_os = "macos")]
impl Notifier for SystemNotifier {
    fn send_notification(&mut self, title: &str, body: &str) {
        if !self.enabled {
            return;
        }

        let escaped_title = title
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('`', "\\`")
            .replace('\n', "\\n");
        let escaped_body = body
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('`', "\\`")
            .replace('\n', "\\n");

        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "display notification \"{}\" with title \"{}\"",
                escaped_body, escaped_title
            ))
            .output();
    }
}

#[cfg(not(target_os = "macos"))]
impl Notifier for SystemNotifier {
    fn send_notification(&mut self, title: &str, body: &str) {
        if !self.enabled {
            return;
        }

        if let Err(e) = notify_rust::Notification::new()
            .appname("DeskClock")
            .summary(title)
            .body(body)
            .show()
        {
            eprintln!("Failed to send notification: {}", e);
        }
    }
}

#[cfg(test)]
#[derive(Debug)]
pub struct MockNotifier {
    pub notified: Vec<(String, String)>,
}

#[cfg(test)]
impl MockNotifier {
    pub fn new() -> Self {
        Self {
            notified: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.notified.clear();
    }

    pub fn last_notification(&self) -> Option<(&str, &str)> {
        self.notified.last().map(|(t, b)| (t.as_str(), b.as_str()))
    }

    pub fn count(&self) -> usize {
        self.notified.len()
    }
}

#[cfg(test)]
impl Notifier for MockNotifier {
    fn send_notification(&mut self, title: &str, body: &str) {
        self.notified.push((title.to_string(), body.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_notifier_sends_notification() {
        let mut notifier = MockNotifier::new();
        notifier.send_notification("Title", "Body");
        assert_eq!(notifier.count(), 1);
        let (title, body) = notifier.last_notification().unwrap();
        assert_eq!(title, "Title");
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_mock_notifier_clears() {
        let mut notifier = MockNotifier::new();
        notifier.send_notification("T1", "B1");
        notifier.clear();
        assert_eq!(notifier.count(), 0);
        assert!(notifier.last_notification().is_none());
    }

    #[test]
    fn test_mock_notifier_collects_multiple_notifications() {
        let mut notifier = MockNotifier::new();
        notifier.send_notification("T1", "B1");
        notifier.send_notification("T2", "B2");
        notifier.send_notification("T3", "B3");
        assert_eq!(notifier.count(), 3);
        assert_eq!(notifier.last_notification().unwrap(), ("T3", "B3"));
    }

    #[test]
    fn test_mock_notifier_no_notifications_returns_none() {
        let notifier = MockNotifier::new();
        assert!(notifier.last_notification().is_none());
    }

    #[test]
    fn test_mock_notifier_preserves_special_characters() {
        let mut notifier = MockNotifier::new();
        let title = "Timer Complete! 🕒";
        let body = "00:00 - Your countdown has finished";
        notifier.send_notification(title, body);
        let (t, b) = notifier.last_notification().unwrap();
        assert_eq!(t, title);
        assert_eq!(b, body);
    }

    #[test]
    fn test_system_notifier_exists_and_is_debuggable() {
        let notifier = SystemNotifier::new();
        let debug_str = format!("{:?}", notifier);
        assert!(debug_str.contains("SystemNotifier"));
        assert!(debug_str.contains("enabled"));
    }

    #[test]
    fn test_mock_notifier_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>(_: &T) {}
        let notifier = MockNotifier::new();
        assert_send_sync(&notifier);
    }

    #[test]
    fn test_mock_notifier_empty_string_notifications() {
        let mut notifier = MockNotifier::new();
        notifier.send_notification("", "");
        assert_eq!(notifier.count(), 1);
        let (t, b) = notifier.last_notification().unwrap();
        assert_eq!(t, "");
        assert_eq!(b, "");
    }

    #[test]
    fn test_mock_notifier_unicode_body() {
        let mut notifier = MockNotifier::new();
        notifier.send_notification("Countdown Done", "倒计时已结束 - Time's up!");
        let (_t, b) = notifier.last_notification().unwrap();
        assert!(b.contains("倒计时已结束"));
        assert!(b.contains("Time's up!"));
    }

    #[test]
    fn test_mock_notifier_receives_timer_finish_notification() {
        let mut notifier = MockNotifier::new();

        let title = "Countdown Timer Complete";
        let body = "00:00 - Timer has finished";

        notifier.send_notification(title, body);

        assert_eq!(notifier.count(), 1);
        let (t, b) = notifier.last_notification().unwrap();
        assert_eq!(t, "Countdown Timer Complete");
        assert_eq!(b, "00:00 - Timer has finished");
    }

    #[test]
    fn test_mock_notifier_multiple_different_messages() {
        let mut notifier = MockNotifier::new();

        notifier.send_notification("Timer 1", "5 minutes done");
        assert_eq!(notifier.count(), 1);

        notifier.clear();

        notifier.send_notification("Timer 2", "10 minutes done");
        assert_eq!(notifier.count(), 1);
        let (t, b) = notifier.last_notification().unwrap();
        assert_eq!(t, "Timer 2");
        assert_eq!(b, "10 minutes done");
    }
}
