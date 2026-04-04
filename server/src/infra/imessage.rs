use std::process::Command;

use crate::domain::error::AppError;
use crate::domain::ports::NotificationPort;

#[derive(Debug, Clone)]
pub struct ImessageAdapter {
    recipient: String,
}

impl ImessageAdapter {
    pub fn new(recipient: &str) -> Self {
        Self {
            recipient: recipient.to_string(),
        }
    }

    /// AppleScript injection 방지: 특수문자 이스케이프
    fn escape_applescript(text: &str) -> String {
        text.replace('\\', "\\\\").replace('"', "\\\"")
    }
}

impl NotificationPort for ImessageAdapter {
    fn send(&self, message: &str) -> Result<(), AppError> {
        let escaped_message = Self::escape_applescript(message);
        let escaped_recipient = Self::escape_applescript(&self.recipient);

        let script = format!(
            r#"tell application "Messages"
    set targetService to 1st account whose service type = iMessage
    set targetBuddy to participant "{escaped_recipient}" of targetService
    send "{escaped_message}" to targetBuddy
end tell"#
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| AppError::Internal(format!("osascript 실행 실패: {e}")))?;

        if output.status.success() {
            tracing::info!(recipient = %self.recipient, "iMessage 전송 완료");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!(error = %stderr, "iMessage 전송 실패");
            Err(AppError::Internal(format!("iMessage 전송 실패: {stderr}")))
        }
    }
}

/// Docker 환경 등 osascript 미지원 시 로그만 출력하는 어댑터
#[derive(Debug, Clone)]
pub struct LogOnlyNotificationAdapter;

impl NotificationPort for LogOnlyNotificationAdapter {
    fn send(&self, message: &str) -> Result<(), AppError> {
        tracing::info!(
            message_len = message.len(),
            "알림 전송 (로그 전용): 메시지 길이 {}자",
            message.len()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_applescript_handles_quotes() {
        let escaped = ImessageAdapter::escape_applescript(r#"He said "hello""#);
        assert_eq!(escaped, r#"He said \"hello\""#);
    }

    #[test]
    fn escape_applescript_handles_backslash() {
        let escaped = ImessageAdapter::escape_applescript(r"path\to\file");
        assert_eq!(escaped, r"path\\to\\file");
    }

    #[test]
    fn escape_applescript_handles_mixed() {
        // 입력: a\"b"c
        // 1) \ → \\  : a\\"b"c
        // 2) " → \"  : a\\\"b\"c
        let escaped = ImessageAdapter::escape_applescript(r#"a\"b"c"#);
        assert_eq!(escaped, r#"a\\\"b\"c"#);
    }

    #[test]
    fn log_only_adapter_succeeds() {
        let adapter = LogOnlyNotificationAdapter;
        assert!(adapter.send("test message").is_ok());
    }

    #[test]
    fn imessage_adapter_new() {
        let adapter = ImessageAdapter::new("test@example.com");
        assert_eq!(adapter.recipient, "test@example.com");
    }
}
