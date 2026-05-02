-- MVP15 M2: 무료 한도 자동 보호 인프라 — API 호출 카운터 + 알림 dedupe 로그
--
-- 설계 결정:
-- 1) `api_call_counters`: engine별 월간 호출 카운트.
--    - reset_at NULL 허용 (Exa 등 크레딧형 엔진은 자동 리셋 없음)
--    - lazy reset: 호출 시점에 reset_at < now() 비교하여 단일 CASE statement로 reset+INC 묶음 (race-free)
--    - date_trunc('month', now())는 UTC 기준. 단일 사용자 앱이라 이슈 없음.
-- 2) `api_alert_log`: 임계 알림 dedupe.
--    - PK (engine, threshold, period_start) — 한 주기 내 동일 임계는 1회만 INSERT 성공
--    - INSERT ... ON CONFLICT DO NOTHING RETURNING id 패턴으로 atomic dedupe
--    - 다음 주기로 넘어가면 period_start 달라져서 자동 재알림 가능

CREATE TABLE IF NOT EXISTS api_call_counters (
    engine            TEXT PRIMARY KEY,
    calls_this_month  INTEGER NOT NULL DEFAULT 0,
    reset_at          TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS api_alert_log (
    engine        TEXT NOT NULL,
    threshold     INTEGER NOT NULL,
    period_start  TIMESTAMPTZ NOT NULL,
    sent_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (engine, threshold, period_start)
);

-- 운영 메모: 카운터/알림 로그 모두 RLS 미적용.
-- 서버에서만 service_role로 접근. 클라이언트 직접 노출 없음.
