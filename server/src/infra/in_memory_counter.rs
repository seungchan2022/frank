//! 인메모리 CounterPort 구현체.
//!
//! 용도:
//! - 테스트 (결정적 동작, 시드 가능, calls 검증)
//! - `FRANK_DEV_MOCK_SEARCH=1` 모드에서 DB 미오염 보장 (#5)
//!
//! 동작:
//! - `record_call`: 인메모리 HashMap에 INC, lazy reset (reset_at < now()이면 1로 reset)
//! - `try_record_alert`: HashSet으로 (engine, threshold, period_start) dedupe

use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::Mutex;

use chrono::{DateTime, Utc};

use crate::domain::error::AppError;
use crate::domain::ports::{CounterPort, CounterSnapshot};

/// (calls_this_month, reset_at) 튜플 alias — clippy::type_complexity 회피.
type CounterEntry = (i32, Option<DateTime<Utc>>);

#[derive(Debug, Default)]
pub struct InMemoryCounter {
    /// engine → (calls_this_month, reset_at)
    counters: Mutex<HashMap<String, CounterEntry>>,
    /// (engine, threshold, period_start) 발송 이력
    alerts: Mutex<HashSet<(String, i32, DateTime<Utc>)>>,
}

impl InMemoryCounter {
    pub fn new() -> Self {
        Self::default()
    }

    /// 테스트용: 카운터 시드. 임의 calls/reset_at으로 미리 채울 때 사용.
    #[cfg(test)]
    pub fn seed(&self, engine: &str, calls: i32, reset_at: Option<DateTime<Utc>>) {
        if let Ok(mut store) = self.counters.lock() {
            store.insert(engine.to_string(), (calls, reset_at));
        }
    }

    /// 다음 달 1일 00:00 UTC. PostgreSQL의 `date_trunc('month', now()) + interval '1 month'`와 등가.
    fn next_month_reset(now: DateTime<Utc>) -> DateTime<Utc> {
        use chrono::{Datelike, NaiveDate, TimeZone};
        let (year, month) = if now.month() == 12 {
            (now.year() + 1, 1)
        } else {
            (now.year(), now.month() + 1)
        };
        let date = NaiveDate::from_ymd_opt(year, month, 1)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2000, 1, 1).expect("constant date"));
        let datetime = date.and_hms_opt(0, 0, 0).expect("constant time");
        Utc.from_utc_datetime(&datetime)
    }
}

impl CounterPort for InMemoryCounter {
    fn record_call<'a>(
        &'a self,
        engine: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<CounterSnapshot, AppError>> + Send + 'a>> {
        Box::pin(async move {
            let mut store = self
                .counters
                .lock()
                .map_err(|e| AppError::Internal(format!("counter lock poisoned: {e}")))?;
            let now = Utc::now();
            let next_reset = Self::next_month_reset(now);

            let entry = store
                .entry(engine.to_string())
                .or_insert((0, Some(next_reset)));
            // lazy reset: reset_at가 Some이고 < now()이면 reset
            if let Some(reset_at) = entry.1
                && reset_at < now
            {
                entry.0 = 1;
                entry.1 = Some(next_reset);
                return Ok(CounterSnapshot {
                    calls: entry.0,
                    reset_at: entry.1,
                });
            }
            // 신규 행이면 (0, Some(next_reset))로 init된 상태 → INC하면 1
            // 기존 행이면 INC만
            entry.0 += 1;
            Ok(CounterSnapshot {
                calls: entry.0,
                reset_at: entry.1,
            })
        })
    }

    fn snapshot<'a>(
        &'a self,
        engine: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<CounterSnapshot, AppError>> + Send + 'a>> {
        Box::pin(async move {
            let store = self
                .counters
                .lock()
                .map_err(|e| AppError::Internal(format!("counter lock poisoned: {e}")))?;
            let snap = store
                .get(engine)
                .map(|(c, r)| CounterSnapshot {
                    calls: *c,
                    reset_at: *r,
                })
                .unwrap_or(CounterSnapshot {
                    calls: 0,
                    reset_at: None,
                });
            Ok(snap)
        })
    }

    fn try_record_alert<'a>(
        &'a self,
        engine: &'a str,
        threshold: i32,
        period_start: DateTime<Utc>,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + 'a>> {
        Box::pin(async move {
            let mut alerts = self
                .alerts
                .lock()
                .map_err(|e| AppError::Internal(format!("alert lock poisoned: {e}")))?;
            let key = (engine.to_string(), threshold, period_start);
            Ok(alerts.insert(key))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn record_call_first_inserts_one() {
        let c = InMemoryCounter::new();
        let snap = c.record_call("tavily").await.unwrap();
        assert_eq!(snap.calls, 1);
        assert!(snap.reset_at.is_some());
    }

    #[tokio::test]
    async fn record_call_increments() {
        let c = InMemoryCounter::new();
        c.record_call("tavily").await.unwrap();
        c.record_call("tavily").await.unwrap();
        let snap = c.record_call("tavily").await.unwrap();
        assert_eq!(snap.calls, 3);
    }

    #[tokio::test]
    async fn record_call_lazy_resets_when_reset_at_past() {
        let c = InMemoryCounter::new();
        // 과거 reset_at으로 시드 → 다음 호출에서 reset되어 calls=1
        let past = Utc::now() - chrono::Duration::days(1);
        c.seed("tavily", 999, Some(past));
        let snap = c.record_call("tavily").await.unwrap();
        assert_eq!(snap.calls, 1, "reset_at 경과 → calls 1로 리셋");
        assert!(snap.reset_at.unwrap() > Utc::now(), "reset_at 미래로 갱신");
    }

    #[tokio::test]
    async fn record_call_does_not_reset_when_future() {
        let c = InMemoryCounter::new();
        let future = Utc::now() + chrono::Duration::days(7);
        c.seed("exa", 500, Some(future));
        let snap = c.record_call("exa").await.unwrap();
        assert_eq!(snap.calls, 501, "미래 reset_at → INC만");
    }

    #[tokio::test]
    async fn snapshot_zero_when_unseen() {
        let c = InMemoryCounter::new();
        let snap = c.snapshot("never").await.unwrap();
        assert_eq!(snap.calls, 0);
        assert_eq!(snap.reset_at, None);
    }

    #[tokio::test]
    async fn try_record_alert_dedupes() {
        let c = InMemoryCounter::new();
        let now = Utc::now();
        let first = c.try_record_alert("tavily", 80, now).await.unwrap();
        let second = c.try_record_alert("tavily", 80, now).await.unwrap();
        assert!(first, "첫 INSERT는 신규");
        assert!(!second, "동일 키 재시도는 false");
    }

    #[tokio::test]
    async fn try_record_alert_different_period_succeeds() {
        let c = InMemoryCounter::new();
        let now = Utc::now();
        let later = now + chrono::Duration::days(31);
        let first = c.try_record_alert("tavily", 80, now).await.unwrap();
        let second = c.try_record_alert("tavily", 80, later).await.unwrap();
        assert!(first);
        assert!(second, "다음 주기에는 재발송 가능");
    }
}
