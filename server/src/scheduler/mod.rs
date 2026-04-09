use std::sync::Arc;
use std::time::Duration;

use crate::domain::ports::{DbPort, SearchChainPort};
use crate::services::collect_service;

/// 백그라운드 수집 스케줄러.
///
/// 서버 시작 시 `tokio::spawn`으로 실행하며,
/// `COLLECT_INTERVAL_SECS` 환경변수로 수집 주기를 제어한다 (기본: 3600초 = 1시간).
///
/// 각 주기마다 모든 유저의 활성 태그를 기반으로 뉴스를 수집한다.
pub async fn run<D: DbPort + Clone + 'static>(
    db: D,
    search_chain: Arc<dyn SearchChainPort>,
    interval_secs: u64,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
    // Delay: 이전 tick이 지연돼도 누적 실행하지 않음 (burst 방지)
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    // 첫 tick은 즉시 실행 (서버 시작 직후 수집)
    interval.tick().await;

    loop {
        tracing::info!("scheduler: 전체 유저 뉴스 수집 시작");

        let user_ids = match db.get_all_user_ids().await {
            Ok(ids) => ids,
            Err(e) => {
                tracing::error!(error = %e, "scheduler: 유저 목록 조회 실패");
                interval.tick().await;
                continue;
            }
        };

        if user_ids.is_empty() {
            tracing::info!("scheduler: 유저 없음, 수집 건너뜀");
            interval.tick().await;
            continue;
        }

        let mut total = 0usize;
        for user_id in &user_ids {
            match collect_service::collect_for_user(&db, search_chain.as_ref(), *user_id).await {
                Ok(count) => {
                    tracing::info!(user_id = %user_id, count, "scheduler: 수집 완료");
                    total += count;
                }
                Err(e) => {
                    tracing::warn!(user_id = %user_id, error = %e, "scheduler: 수집 실패, 건너뜀");
                }
            }
        }

        tracing::info!(total, "scheduler: 전체 수집 완료");
        interval.tick().await;
    }
}

/// `COLLECT_INTERVAL_SECS` 환경변수를 읽어 수집 주기(초)를 반환한다.
/// 미설정 또는 파싱 실패 시 기본값(3600초)을 반환한다.
pub fn interval_secs_from_env() -> u64 {
    std::env::var("COLLECT_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(3600)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_secs_default_when_unset() {
        // 환경변수 미설정 시 기본값 3600
        // SAFETY: 단일 스레드 테스트에서 환경변수 조작
        unsafe { std::env::remove_var("COLLECT_INTERVAL_SECS") };
        assert_eq!(interval_secs_from_env(), 3600);
    }

    #[test]
    fn interval_secs_parses_env() {
        // SAFETY: 단일 스레드 테스트에서 환경변수 조작
        unsafe { std::env::set_var("COLLECT_INTERVAL_SECS", "1800") };
        assert_eq!(interval_secs_from_env(), 1800);
        unsafe { std::env::remove_var("COLLECT_INTERVAL_SECS") };
    }

    #[test]
    fn interval_secs_invalid_falls_back_to_default() {
        // SAFETY: 단일 스레드 테스트에서 환경변수 조작
        unsafe { std::env::set_var("COLLECT_INTERVAL_SECS", "not-a-number") };
        assert_eq!(interval_secs_from_env(), 3600);
        unsafe { std::env::remove_var("COLLECT_INTERVAL_SECS") };
    }
}
