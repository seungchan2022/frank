// MVP5 M1: 배치 요약 후 알림 전송 기능은 summary_service와 함께 비활성화.
// M2 온디맨드 요약 구현 시 필요하면 재활용한다.

use crate::domain::ports::NotificationPort;

/// 수집 완료 후 간단한 알림 전송 (count 기반).
pub fn notify_collected(notifier: &dyn NotificationPort, count: usize) {
    if count == 0 {
        return;
    }
    let message = format!("Frank: {count}개 기사 수집 완료");
    if let Err(e) = notifier.send(&message) {
        tracing::warn!(error = %e, "알림 전송 실패, 건너뜀");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::fake_notification::FakeNotificationAdapter;

    #[test]
    fn notify_collected_sends_when_count_positive() {
        let notifier = FakeNotificationAdapter::new();
        notify_collected(&notifier, 5);
        let msgs = notifier.sent_messages();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("5개 기사 수집 완료"));
    }

    #[test]
    fn notify_collected_skips_when_zero() {
        let notifier = FakeNotificationAdapter::new();
        notify_collected(&notifier, 0);
        assert!(notifier.sent_messages().is_empty());
    }

    #[test]
    fn notify_collected_does_not_panic_on_failure() {
        let notifier = FakeNotificationAdapter::failing();
        notify_collected(&notifier, 3);
        // 실패해도 패닉하지 않아야 한다
    }
}
