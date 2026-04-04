use std::sync::Mutex;

use crate::domain::error::AppError;
use crate::domain::ports::NotificationPort;

#[derive(Debug)]
pub struct FakeNotificationAdapter {
    sent_messages: Mutex<Vec<String>>,
    should_fail: bool,
}

impl FakeNotificationAdapter {
    pub fn new() -> Self {
        Self {
            sent_messages: Mutex::new(Vec::new()),
            should_fail: false,
        }
    }

    pub fn failing() -> Self {
        Self {
            sent_messages: Mutex::new(Vec::new()),
            should_fail: true,
        }
    }

    pub fn sent_messages(&self) -> Vec<String> {
        self.sent_messages.lock().unwrap().clone()
    }
}

impl Default for FakeNotificationAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationPort for FakeNotificationAdapter {
    fn send(&self, message: &str) -> Result<(), AppError> {
        if self.should_fail {
            return Err(AppError::Internal("Fake notification failure".to_string()));
        }
        self.sent_messages.lock().unwrap().push(message.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_notification_stores_messages() {
        let notifier = FakeNotificationAdapter::new();
        notifier.send("Hello").unwrap();
        notifier.send("World").unwrap();

        let messages = notifier.sent_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "Hello");
        assert_eq!(messages[1], "World");
    }

    #[test]
    fn fake_notification_failing_returns_error() {
        let notifier = FakeNotificationAdapter::failing();
        let result = notifier.send("Hello");
        assert!(result.is_err());
    }

    #[test]
    fn fake_notification_default() {
        let notifier = FakeNotificationAdapter::default();
        notifier.send("test").unwrap();
        assert_eq!(notifier.sent_messages().len(), 1);
    }
}
