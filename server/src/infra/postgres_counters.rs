//! PostgreSQL CounterPort 구현체.
//!
//! 핵심 SQL (#3): lazy reset + INC를 단일 CASE statement로 묶어 race-free.
//! 동시 INC가 발생해도 행 락(UPSERT)이 잡혀 정확하게 +1씩 증가한다.

use std::pin::Pin;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::error::AppError;
use crate::domain::ports::{CounterPort, CounterSnapshot};

#[derive(Debug, Clone)]
pub struct PostgresCounterAdapter {
    pool: PgPool,
}

impl PostgresCounterAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl CounterPort for PostgresCounterAdapter {
    fn record_call<'a>(
        &'a self,
        engine: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<CounterSnapshot, AppError>> + Send + 'a>> {
        Box::pin(async move {
            // 단일 CASE statement: race-free lazy reset + INC.
            // INSERT가 신규 행이면 calls=1, reset_at=다음달 1일 00:00 UTC.
            // ON CONFLICT 시 reset_at 경과면 calls=1+reset_at 갱신, 아니면 INC만.
            let row: (i32, Option<DateTime<Utc>>) = sqlx::query_as(
                r#"
                INSERT INTO api_call_counters (engine, calls_this_month, reset_at)
                VALUES ($1, 1, date_trunc('month', now()) + interval '1 month')
                ON CONFLICT (engine) DO UPDATE SET
                    calls_this_month = CASE
                        WHEN api_call_counters.reset_at IS NOT NULL
                             AND api_call_counters.reset_at < now()
                            THEN 1
                        ELSE api_call_counters.calls_this_month + 1
                    END,
                    reset_at = CASE
                        WHEN api_call_counters.reset_at IS NOT NULL
                             AND api_call_counters.reset_at < now()
                            THEN date_trunc('month', now()) + interval '1 month'
                        ELSE api_call_counters.reset_at
                    END
                RETURNING calls_this_month, reset_at
                "#,
            )
            .bind(engine)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("counter INC 실패: {e}")))?;

            Ok(CounterSnapshot {
                calls: row.0,
                reset_at: row.1,
            })
        })
    }

    fn snapshot<'a>(
        &'a self,
        engine: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<CounterSnapshot, AppError>> + Send + 'a>> {
        Box::pin(async move {
            let maybe_row: Option<(i32, Option<DateTime<Utc>>)> = sqlx::query_as(
                r#"
                SELECT calls_this_month, reset_at
                FROM api_call_counters
                WHERE engine = $1
                "#,
            )
            .bind(engine)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("counter snapshot 실패: {e}")))?;

            Ok(maybe_row
                .map(|(c, r)| CounterSnapshot {
                    calls: c,
                    reset_at: r,
                })
                .unwrap_or(CounterSnapshot {
                    calls: 0,
                    reset_at: None,
                }))
        })
    }

    fn try_record_alert<'a>(
        &'a self,
        engine: &'a str,
        threshold: i32,
        period_start: DateTime<Utc>,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + 'a>> {
        Box::pin(async move {
            // ON CONFLICT DO NOTHING RETURNING engine: 행이 INSERT되면 Some, 충돌 시 None.
            // 동일 (engine, threshold, period_start)이 이미 있으면 false 반환 → dedupe.
            let inserted: Option<(String,)> = sqlx::query_as(
                r#"
                INSERT INTO api_alert_log (engine, threshold, period_start)
                VALUES ($1, $2, $3)
                ON CONFLICT (engine, threshold, period_start) DO NOTHING
                RETURNING engine
                "#,
            )
            .bind(engine)
            .bind(threshold)
            .bind(period_start)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("alert dedupe 실패: {e}")))?;

            Ok(inserted.is_some())
        })
    }
}
