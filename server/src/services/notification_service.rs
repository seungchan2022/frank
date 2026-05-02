//! 알림 서비스.
//!
//! MVP15 M2 추가:
//! - `AlertDispatch`: 임계 교차 알림 페이로드 빌더 (R3: 민감정보 미포함)
//! - `dispatch_threshold_alert`: dedupe atomic INSERT + spawn_blocking + tokio::timeout (R4)
//! - 메시지 크기 ≤200자 (R4)

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::domain::ports::{CounterPort, NotificationPort};

// MVP5 M1: 배치 요약 후 알림 전송 기능은 summary_service와 함께 비활성화.
// M2 온디맨드 요약 구현 시 필요하면 재활용한다.

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

// ─── MVP15 M2: 무료 한도 임계 알림 ─────────────────────────────────────────────
//
// 무료 한도 상수(MONTHLY_CALL_LIMIT / ALERT_THRESHOLD_*)는 domain/ports.rs SSOT 사용.
// 본 모듈은 dispatch 흐름만 담당.

/// 알림 메시지 최대 크기 (R4): iMessage 페이로드 보호.
const MAX_MESSAGE_BYTES: usize = 200;

/// 알림 send 타임아웃 (R4): osascript이 stuck되면 5초 후 포기.
const ALERT_SEND_TIMEOUT: Duration = Duration::from_secs(5);

/// 임계 교차 알림 페이로드.
///
/// R3 가드: 엔진명·임계치·회복일만 포함. user_id/쿼리/태그명 노출 금지.
#[derive(Debug, Clone)]
pub struct AlertDispatch {
    pub engine: String,
    /// 80 또는 100
    pub threshold_pct: i32,
    /// 회복 시각. Exa(크레딧형)는 None.
    pub reset_at: Option<DateTime<Utc>>,
    /// dedupe 키. `date_trunc('month', now())`.
    pub period_start: DateTime<Utc>,
}

/// 알림 메시지 빌드 (R3: 엔진명·임계치·회복일만, R4: ≤200자).
/// 순수 함수 — 테스트 가능.
pub fn build_alert_message(d: &AlertDispatch) -> String {
    let recovery = match d.reset_at {
        Some(r) => format!("{}", r.format("%Y-%m-%d")),
        None => "수동 (크레딧 갱신)".to_string(),
    };
    let msg = format!("{} {}% 도달 — 회복 {}", d.engine, d.threshold_pct, recovery);
    // R4: 200자 초과는 절단. 일반 케이스는 50자 미만이라 트리거 거의 안 됨.
    // UTF-8 안전 절단: char_indices로 byte index가 char boundary인 위치까지만 남김.
    if msg.len() <= MAX_MESSAGE_BYTES {
        return msg;
    }
    msg.char_indices()
        .take_while(|(i, _)| *i < MAX_MESSAGE_BYTES)
        .map(|(_, c)| c)
        .collect()
}

/// 임계 교차 알림 비동기 dispatch.
///
/// 흐름:
/// 1. `try_record_alert` atomic INSERT — 동일 (engine, threshold, period_start) 이미 발송 → false → skip
/// 2. 신규면 `spawn_blocking + tokio::timeout(5s)`로 sync `NotificationPort::send` 호출
/// 3. 실패 시 1회 재시도. 최종 실패해도 검색 흐름은 막지 않음 (warn 로그만)
///
/// `NotificationPort::send`는 sync 시그니처 유지 (#1 결정).
/// spawn_blocking으로 blocking call을 별도 스레드 풀로 격리.
pub fn dispatch_threshold_alert(
    counter: Arc<dyn CounterPort>,
    notifier: Arc<dyn NotificationPort>,
    dispatch: AlertDispatch,
) {
    tokio::spawn(async move {
        // 1) dedupe atomic INSERT
        let inserted = match counter
            .try_record_alert(
                &dispatch.engine,
                dispatch.threshold_pct,
                dispatch.period_start,
            )
            .await
        {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(error = %e, "alert dedupe 실패 — 알림 skip");
                return;
            }
        };
        if !inserted {
            tracing::debug!(
                engine = %dispatch.engine,
                threshold = dispatch.threshold_pct,
                "이번 주기 동일 임계 알림 이미 발송 — skip"
            );
            return;
        }

        let message = build_alert_message(&dispatch);
        let send_result = send_with_timeout_retry(Arc::clone(&notifier), message.clone()).await;
        match send_result {
            Ok(()) => {
                tracing::info!(engine = %dispatch.engine, threshold = dispatch.threshold_pct, "알림 발송 완료")
            }
            Err(e) => tracing::warn!(error = %e, "알림 최종 실패 — 검색 흐름은 계속"),
        }
    });
}

/// `spawn_blocking + tokio::timeout(5s) + 재시도 1회` 패턴 (R4).
async fn send_with_timeout_retry(
    notifier: Arc<dyn NotificationPort>,
    message: String,
) -> Result<(), String> {
    for attempt in 0..2 {
        let n = Arc::clone(&notifier);
        let msg = message.clone();
        let join_handle = tokio::task::spawn_blocking(move || n.send(&msg));

        match tokio::time::timeout(ALERT_SEND_TIMEOUT, join_handle).await {
            Ok(Ok(Ok(()))) => return Ok(()),
            Ok(Ok(Err(e))) => {
                tracing::warn!(attempt = attempt, error = %e, "알림 send 실패");
            }
            Ok(Err(e)) => {
                tracing::warn!(attempt = attempt, error = %e, "알림 spawn_blocking join 실패");
            }
            Err(_) => {
                tracing::warn!(attempt = attempt, "알림 send 타임아웃 (5s)");
            }
        }
    }
    Err("알림 send 2회 모두 실패".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::in_memory_counter::InMemoryCounter;
    use chrono::TimeZone;

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

    // ─── MVP15 M2: 알림 메시지 빌더 단위 테스트 ─────────────────────────────────

    #[test]
    fn alert_message_excludes_sensitive_data() {
        // R3 검증: user_id/쿼리/태그명 등 민감정보 미포함
        let d = AlertDispatch {
            engine: "tavily".to_string(),
            threshold_pct: 80,
            reset_at: Some(Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap()),
            period_start: Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
        };
        let msg = build_alert_message(&d);
        assert!(msg.contains("tavily"));
        assert!(msg.contains("80%"));
        assert!(msg.contains("2026-06-01"));
        // 민감 키워드 없음
        assert!(!msg.contains("user"), "user 관련 정보 미포함: {msg}");
        assert!(!msg.contains("query"), "query 미포함: {msg}");
        assert!(!msg.contains("tag"), "tag 미포함: {msg}");
    }

    #[test]
    fn alert_message_handles_no_reset_at_for_exa() {
        let d = AlertDispatch {
            engine: "exa".to_string(),
            threshold_pct: 100,
            reset_at: None,
            period_start: Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
        };
        let msg = build_alert_message(&d);
        assert!(msg.contains("exa"));
        assert!(msg.contains("100%"));
        assert!(msg.contains("크레딧"), "Exa는 수동 갱신 표기");
    }

    #[test]
    fn alert_message_size_under_200() {
        let d = AlertDispatch {
            engine: "tavily".to_string(),
            threshold_pct: 100,
            reset_at: Some(Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap()),
            period_start: Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
        };
        let msg = build_alert_message(&d);
        assert!(
            msg.len() <= MAX_MESSAGE_BYTES,
            "R4 ≤200자 가드: {}",
            msg.len()
        );
    }

    #[tokio::test]
    async fn dispatch_dedupes_same_period() {
        let counter = Arc::new(InMemoryCounter::new());
        let notifier = Arc::new(FakeNotificationAdapter::new());

        let period = Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap();
        let d = AlertDispatch {
            engine: "tavily".to_string(),
            threshold_pct: 80,
            reset_at: Some(Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap()),
            period_start: period,
        };

        // 두 번 dispatch — 두 번째는 dedupe로 skip되어야 함
        dispatch_threshold_alert(
            Arc::clone(&counter) as Arc<dyn CounterPort>,
            Arc::clone(&notifier) as Arc<dyn NotificationPort>,
            d.clone(),
        );
        dispatch_threshold_alert(
            Arc::clone(&counter) as Arc<dyn CounterPort>,
            Arc::clone(&notifier) as Arc<dyn NotificationPort>,
            d,
        );

        // 비동기 spawn 완료 대기
        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(20)).await;
            if !notifier.sent_messages().is_empty() {
                break;
            }
        }
        // 추가 dispatch는 dedupe되어 1회만 발송
        tokio::time::sleep(Duration::from_millis(100)).await;
        let msgs = notifier.sent_messages();
        assert_eq!(msgs.len(), 1, "동일 period_start dedupe → 1회만: {msgs:?}");
    }

    #[tokio::test]
    async fn dispatch_does_not_resend_when_failing_notifier() {
        // FailingNotifier로 send 2회 모두 실패해도 panic 없이 끝나야 함
        let counter = Arc::new(InMemoryCounter::new());
        let notifier = Arc::new(FakeNotificationAdapter::failing());

        let d = AlertDispatch {
            engine: "tavily".to_string(),
            threshold_pct: 80,
            reset_at: Some(Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap()),
            period_start: Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
        };
        dispatch_threshold_alert(
            Arc::clone(&counter) as Arc<dyn CounterPort>,
            Arc::clone(&notifier) as Arc<dyn NotificationPort>,
            d,
        );
        // 비동기 종료 대기 — panic 없이 빠져나가야 함
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(
            notifier.sent_messages().is_empty(),
            "failing notifier는 발송 누적 안 함"
        );
    }
}
