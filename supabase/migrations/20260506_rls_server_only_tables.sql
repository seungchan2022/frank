-- MVP16: api_call_counters, api_alert_log RLS 활성화
--
-- 배경: Supabase 보안 어드바이저 경고 (2026-05-03 기준)
-- 두 테이블은 서버(service_role)에서만 접근. 클라이언트 직접 노출 의도 없음.
-- service_role은 RLS를 우회하므로 서버 동작 영향 없음.
-- RLS 활성화 + 정책 없음 = anon/authenticated 클라이언트 접근 전면 차단.

ALTER TABLE api_call_counters ENABLE ROW LEVEL SECURITY;
ALTER TABLE api_alert_log ENABLE ROW LEVEL SECURITY;
