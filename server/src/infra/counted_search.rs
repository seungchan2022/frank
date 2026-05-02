//! `CountedSearchAdapter`: SearchPort 데코레이터.
//!
//! 책임 (H1·H6 통합):
//! 1. **100% skip**: 호출 전에 `CounterPort.snapshot()` 조회. 한도 도달 시 `Ok(vec![])` 반환.
//!    체인은 빈 결과를 다음 어댑터로 폴백 처리한다 (`Err`보다 깔끔).
//! 2. **호출 후 INC**: inner adapter 호출이 성공하면(200 OK) `record_call()` 호출.
//!    실패 시 INC 안 함 (실제 한도와 100% 정확하지 않은 보호적 추정치).
//! 3. **임계 알림 트리거**: INC 결과 prev_count → new_count 교차 시 알림 비동기 dispatch.
//!    페이로드는 엔진명·임계치·회복일만 (R3). dedupe + spawn_blocking+timeout은 notification_service에서.
//!
//! `source_name()`은 inner를 그대로 위임 — 카운터 PK ↔ FeedItem.source 일관성 (advisor 지적).
//!
//! ## 동시성 경계 (TOCTOU)
//!
//! `snapshot() → search() → record_call()`는 strict cap이 아니다. 동시 요청 N개가 calls=999일 때
//! 모두 snapshot 통과 후 진행하면 최종 calls가 `999 + N`까지 오버슈트할 수 있다.
//! 오버슈트 상한 = **동시 in-flight 요청 수** (단일 사용자 앱이라 N은 보통 1~3).
//!
//! `prev_calls`는 사전 snapshot 값이라 INC 사이 다른 요청이 끼어들면 stale일 수 있으나,
//! 알림 발송 자체는 `try_record_alert`의 atomic INSERT (engine, threshold, period_start) PK
//! dedupe로 흡수되어 동일 임계 중복 발송이 발생하지 않는다.

use std::pin::Pin;
use std::sync::Arc;

use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::{
    ALERT_THRESHOLD_BLOCK, ALERT_THRESHOLD_WARN, CounterPort, NotificationPort, SearchPort,
};
use crate::services::notification_service::{AlertDispatch, dispatch_threshold_alert};

// 무료 한도 상수는 domain/ports.rs로 통합 (MONTHLY_CALL_LIMIT / ALERT_THRESHOLD_WARN / ALERT_THRESHOLD_BLOCK).
// 본 파일은 ALERT_THRESHOLD_WARN(800) / ALERT_THRESHOLD_BLOCK(1000)만 use하여 사용.

pub struct CountedSearchAdapter {
    inner: Box<dyn SearchPort>,
    counter: Arc<dyn CounterPort>,
    notifier: Arc<dyn NotificationPort>,
}

impl CountedSearchAdapter {
    pub fn new(
        inner: Box<dyn SearchPort>,
        counter: Arc<dyn CounterPort>,
        notifier: Arc<dyn NotificationPort>,
    ) -> Self {
        Self {
            inner,
            counter,
            notifier,
        }
    }
}

impl std::fmt::Debug for CountedSearchAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CountedSearchAdapter")
            .field("inner", &self.inner.source_name())
            .finish()
    }
}

impl SearchPort for CountedSearchAdapter {
    fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SearchResult>, AppError>> + Send + '_>> {
        let query = query.to_string();
        Box::pin(async move {
            let engine = self.inner.source_name().to_string();

            // 1) 사전 snapshot — 100% 도달 시 호출 자체 skip
            let snap = self.counter.snapshot(&engine).await?;
            if snap.calls >= ALERT_THRESHOLD_BLOCK {
                tracing::warn!(
                    engine = %engine,
                    calls = snap.calls,
                    "engine quota reached — skipping call"
                );
                return Ok(vec![]);
            }
            let prev_calls = snap.calls;

            // 2) 실제 검색 호출
            let result = self.inner.search(&query, max_results).await;

            match result {
                Ok(items) => {
                    // 3) 200 OK일 때만 INC
                    let new_snap = match self.counter.record_call(&engine).await {
                        Ok(s) => s,
                        Err(e) => {
                            // 카운터 INC 실패는 검색 결과를 막지 않음
                            tracing::warn!(
                                engine = %engine,
                                error = %e,
                                "counter INC failed, continuing"
                            );
                            return Ok(items);
                        }
                    };

                    // 4) 임계 교차 검사: 80%·100%
                    if let Some(crossed) = threshold_crossed(prev_calls, new_snap.calls) {
                        let dispatch = AlertDispatch {
                            engine: engine.clone(),
                            threshold_pct: crossed,
                            reset_at: new_snap.reset_at,
                            period_start: current_period_start(chrono::Utc::now()),
                        };
                        // 비동기 dispatch (R4: spawn_blocking + tokio::timeout 5s, retry 1회)
                        // dedupe + send + timeout는 notification_service에서 처리
                        dispatch_threshold_alert(
                            Arc::clone(&self.counter),
                            Arc::clone(&self.notifier),
                            dispatch,
                        );
                    }
                    Ok(items)
                }
                Err(e) => {
                    // 호출 실패 시 INC 안 함 — 보호적 추정치
                    tracing::warn!(
                        engine = %engine,
                        error = %e,
                        "search failed — counter not incremented"
                    );
                    Err(e)
                }
            }
        })
    }

    fn source_name(&self) -> &str {
        // 위임 — 카운터 PK ↔ FeedItem.source 일관성 보장
        self.inner.source_name()
    }
}

/// prev → new 교차 시 임계치(80 or 100)를 반환. 그렇지 않으면 None.
/// 80%(800)·100%(1000) 두 임계만 처리.
pub fn threshold_crossed(prev: i32, new: i32) -> Option<i32> {
    // 100% 우선 검사 — 한 번에 800 이상 점프하면 100만 트리거 (실제로는 +1씩)
    if prev < ALERT_THRESHOLD_BLOCK && new >= ALERT_THRESHOLD_BLOCK {
        return Some(100);
    }
    if prev < ALERT_THRESHOLD_WARN && new >= ALERT_THRESHOLD_WARN {
        return Some(80);
    }
    None
}

/// 현재 주기의 period_start = `date_trunc('month', now())` (UTC, 이번 달 1일 00:00).
/// `api_alert_log` 테이블의 PK 일부로, 다음 달이 되면 값이 바뀌어 자동 재발송 가능.
/// reset_at은 사용하지 않음 — Exa(NULL reset_at)도 동일 로직 적용 가능.
pub fn current_period_start(now: chrono::DateTime<chrono::Utc>) -> chrono::DateTime<chrono::Utc> {
    use chrono::{Datelike, NaiveDate, TimeZone};
    let date = NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2000, 1, 1).expect("constant date"));
    let datetime = date.and_hms_opt(0, 0, 0).expect("constant time");
    chrono::Utc.from_utc_datetime(&datetime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::SearchResult;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::in_memory_counter::InMemoryCounter;

    #[test]
    fn threshold_crossed_table() {
        // prev < 80 → new >= 80 → 80
        assert_eq!(threshold_crossed(799, 800), Some(80));
        assert_eq!(threshold_crossed(0, 800), Some(80));
        // prev < 100 → new >= 100 → 100 (우선)
        assert_eq!(threshold_crossed(999, 1000), Some(100));
        assert_eq!(threshold_crossed(800, 1000), Some(100));
        // 이미 80 넘은 상태에서 80 재트리거 안 됨
        assert_eq!(threshold_crossed(800, 801), None);
        // 100 넘은 상태에서 추가 INC 없음 (skip 경로)
        assert_eq!(threshold_crossed(1000, 1001), None);
        // 단일 INC면 한 임계만
        assert_eq!(threshold_crossed(799, 800), Some(80));
    }

    fn make_decorated(
        inner_results: Vec<SearchResult>,
        counter: Arc<dyn CounterPort>,
    ) -> CountedSearchAdapter {
        let inner = Box::new(FakeSearchAdapter::new("tavily", inner_results, false));
        let notifier: Arc<dyn NotificationPort> = Arc::new(FakeNotificationAdapter::new());
        CountedSearchAdapter::new(inner, counter, notifier)
    }

    #[tokio::test]
    async fn source_name_delegates_to_inner() {
        let counter = Arc::new(InMemoryCounter::new());
        let dec = make_decorated(vec![], Arc::clone(&counter) as Arc<dyn CounterPort>);
        // 카운터 PK 일관성: counted decorator는 inner 이름 그대로
        assert_eq!(dec.source_name(), "tavily");
    }

    #[tokio::test]
    async fn skips_when_quota_reached() {
        let counter = Arc::new(InMemoryCounter::new());
        // 1000 시드 → 100% skip
        counter.seed(
            "tavily",
            1000,
            Some(chrono::Utc::now() + chrono::Duration::days(7)),
        );
        let dec = make_decorated(
            vec![SearchResult {
                title: "should not be returned".into(),
                url: "https://example.com/news/x".into(),
                snippet: None,
                published_at: None,
                image_url: None,
            }],
            Arc::clone(&counter) as Arc<dyn CounterPort>,
        );
        let out = dec.search("query", 5).await.unwrap();
        assert!(out.is_empty(), "100% 도달 시 빈 결과");
        // INC도 안 일어났는지 확인 (시드 그대로 1000)
        let snap = counter.snapshot("tavily").await.unwrap();
        assert_eq!(snap.calls, 1000, "skip 시 INC 안 함");
    }

    #[tokio::test]
    async fn increments_on_success() {
        let counter = Arc::new(InMemoryCounter::new());
        let dec = make_decorated(
            vec![SearchResult {
                title: "ok".into(),
                url: "https://example.com/news/x".into(),
                snippet: None,
                published_at: None,
                image_url: None,
            }],
            Arc::clone(&counter) as Arc<dyn CounterPort>,
        );
        dec.search("q", 5).await.unwrap();
        let snap = counter.snapshot("tavily").await.unwrap();
        assert_eq!(snap.calls, 1, "성공 시 INC +1");
    }

    #[tokio::test]
    async fn does_not_increment_on_failure() {
        let counter = Arc::new(InMemoryCounter::new());
        let inner = Box::new(FakeSearchAdapter::new("tavily", vec![], true));
        let notifier: Arc<dyn NotificationPort> = Arc::new(FakeNotificationAdapter::new());
        let dec = CountedSearchAdapter::new(
            inner,
            Arc::clone(&counter) as Arc<dyn CounterPort>,
            notifier,
        );
        let _ = dec.search("q", 5).await;
        let snap = counter.snapshot("tavily").await.unwrap();
        assert_eq!(snap.calls, 0, "실패 시 INC 안 함");
    }

    #[tokio::test]
    async fn empty_ok_response_still_increments() {
        // 200 OK이지만 결과 0건 — INC는 일어남 (호출 자체는 발생했으므로)
        let counter = Arc::new(InMemoryCounter::new());
        let dec = make_decorated(vec![], Arc::clone(&counter) as Arc<dyn CounterPort>);
        dec.search("q", 5).await.unwrap();
        let snap = counter.snapshot("tavily").await.unwrap();
        assert_eq!(snap.calls, 1, "200 OK + 빈 결과도 INC");
    }

    /// 비동기 spawn으로 dispatch된 알림이 도착할 때까지 대기 (최대 1초).
    /// 한도 KPI 검증용 헬퍼.
    async fn wait_for_messages(
        notifier: &FakeNotificationAdapter,
        min_count: usize,
    ) -> Vec<String> {
        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            let msgs = notifier.sent_messages();
            if msgs.len() >= min_count {
                return msgs;
            }
        }
        notifier.sent_messages()
    }

    /// KPI: "iMessage 알림 트리거 80%" 80% 임계 진입 시 1회 발송 (R3 페이로드 검증).
    /// 799 → INC → 800 (≥ ALERT_THRESHOLD_WARN) → 80% 알림 발송.
    #[tokio::test]
    async fn fires_alert_at_80_percent_crossing() {
        let counter = Arc::new(InMemoryCounter::new());
        counter.seed(
            "tavily",
            799,
            Some(chrono::Utc::now() + chrono::Duration::days(7)),
        );
        let notifier = Arc::new(FakeNotificationAdapter::new());
        let inner = Box::new(FakeSearchAdapter::new(
            "tavily",
            vec![SearchResult {
                title: "ok".into(),
                url: "https://example.com/news/x".into(),
                snippet: None,
                published_at: None,
                image_url: None,
            }],
            false,
        ));
        let dec = CountedSearchAdapter::new(
            inner,
            Arc::clone(&counter) as Arc<dyn CounterPort>,
            Arc::clone(&notifier) as Arc<dyn NotificationPort>,
        );
        let _ = dec.search("q", 5).await.unwrap();

        let msgs = wait_for_messages(&notifier, 1).await;
        assert_eq!(msgs.len(), 1, "80% 도달 → 1회 알림 발송: {msgs:?}");
        let m = &msgs[0];
        assert!(m.contains("tavily"), "엔진명 포함");
        assert!(m.contains("80%"), "임계치 포함");
        // R3: 민감정보 미포함 (방어적 검증)
        assert!(!m.contains("query"), "query 미포함");
        assert!(!m.contains("user"), "user 미포함");
    }

    /// KPI: "iMessage 알림 트리거 100%" 100% 임계 진입 시 1회 발송.
    /// 999 → INC → 1000 (≥ ALERT_THRESHOLD_BLOCK) → 100% 알림 발송.
    #[tokio::test]
    async fn fires_alert_at_100_percent_crossing() {
        let counter = Arc::new(InMemoryCounter::new());
        counter.seed(
            "exa",
            999,
            Some(chrono::Utc::now() + chrono::Duration::days(7)),
        );
        let notifier = Arc::new(FakeNotificationAdapter::new());
        let inner = Box::new(FakeSearchAdapter::new(
            "exa",
            vec![SearchResult {
                title: "ok".into(),
                url: "https://example.com/news/x".into(),
                snippet: None,
                published_at: None,
                image_url: None,
            }],
            false,
        ));
        let dec = CountedSearchAdapter::new(
            inner,
            Arc::clone(&counter) as Arc<dyn CounterPort>,
            Arc::clone(&notifier) as Arc<dyn NotificationPort>,
        );
        let _ = dec.search("q", 5).await.unwrap();

        let msgs = wait_for_messages(&notifier, 1).await;
        assert_eq!(msgs.len(), 1, "100% 도달 → 1회 알림 발송: {msgs:?}");
        assert!(msgs[0].contains("exa"));
        assert!(msgs[0].contains("100%"));
    }

    /// 같은 임계 재진입 시 dedupe — 800에서 또 INC 해도 알림 추가 안 됨.
    #[tokio::test]
    async fn no_duplicate_alert_within_same_period() {
        let counter = Arc::new(InMemoryCounter::new());
        counter.seed(
            "tavily",
            799,
            Some(chrono::Utc::now() + chrono::Duration::days(7)),
        );
        let notifier = Arc::new(FakeNotificationAdapter::new());
        let inner = Box::new(FakeSearchAdapter::new(
            "tavily",
            vec![SearchResult {
                title: "ok".into(),
                url: "https://example.com/news/x".into(),
                snippet: None,
                published_at: None,
                image_url: None,
            }],
            false,
        ));
        let dec = CountedSearchAdapter::new(
            inner,
            Arc::clone(&counter) as Arc<dyn CounterPort>,
            Arc::clone(&notifier) as Arc<dyn NotificationPort>,
        );
        // 첫 호출: 799 → 800 (80% 알림 발송)
        let _ = dec.search("q", 5).await.unwrap();
        let _ = wait_for_messages(&notifier, 1).await;
        // 두 번째 호출: 800 → 801 (재트리거 없음 — threshold_crossed가 None 반환)
        let _ = dec.search("q", 5).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let final_msgs = notifier.sent_messages();
        assert_eq!(
            final_msgs.len(),
            1,
            "동일 주기 같은 임계 → 1회만 발송: {final_msgs:?}"
        );
    }
}
