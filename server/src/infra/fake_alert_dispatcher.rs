use std::sync::Mutex;

use crate::domain::models::AlertDispatch;
use crate::domain::ports::AlertDispatcherPort;

/// `AlertDispatcherPort` 테스트용 fake 구현체.
///
/// `dispatch()` 호출 시 `AlertDispatch`를 내부 Vec에 동기 적재.
/// 테스트에서 `dispatched_alerts()`로 호출 이력을 검사한다.
#[derive(Debug, Default)]
pub struct FakeAlertDispatcher {
    alerts: Mutex<Vec<AlertDispatch>>,
}

impl FakeAlertDispatcher {
    pub fn new() -> Self {
        Self {
            alerts: Mutex::new(Vec::new()),
        }
    }

    /// 지금까지 dispatch된 모든 `AlertDispatch`를 복제하여 반환.
    pub fn dispatched_alerts(&self) -> Vec<AlertDispatch> {
        self.alerts.lock().unwrap().clone()
    }
}

impl AlertDispatcherPort for FakeAlertDispatcher {
    fn dispatch(&self, alert: AlertDispatch) {
        self.alerts.lock().unwrap().push(alert);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_alert(engine: &str, pct: i32) -> AlertDispatch {
        AlertDispatch {
            engine: engine.to_string(),
            threshold_pct: pct,
            reset_at: Some(chrono::Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap()),
            period_start: chrono::Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
        }
    }

    #[test]
    fn stores_dispatched_alerts() {
        let dispatcher = FakeAlertDispatcher::new();
        dispatcher.dispatch(make_alert("tavily", 80));
        dispatcher.dispatch(make_alert("exa", 100));

        let alerts = dispatcher.dispatched_alerts();
        assert_eq!(alerts.len(), 2);
        assert_eq!(alerts[0].engine, "tavily");
        assert_eq!(alerts[0].threshold_pct, 80);
        assert_eq!(alerts[1].engine, "exa");
        assert_eq!(alerts[1].threshold_pct, 100);
    }

    #[test]
    fn default_has_no_alerts() {
        let dispatcher = FakeAlertDispatcher::default();
        assert!(dispatcher.dispatched_alerts().is_empty());
    }
}
