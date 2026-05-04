-- ST-4 F-04: Exa engine reset_at → NULL 마이그레이션
--
-- 배경:
--   Exa는 크레딧 선불형이라 월간 자동 reset이 없다.
--   코드 변경(postgres_counters.rs) 이전에 첫 INSERT 된 Exa row에는
--   다음 달 1일이 reset_at으로 세팅되어 있을 수 있다.
--   이 마이그레이션으로 기존 row를 정정한다.
--
-- 영향 범위: api_call_counters WHERE engine = 'exa' (1행)
-- 멱등 실행 가능: NULL 이미 NULL인 경우에도 안전.

UPDATE api_call_counters
SET reset_at = NULL
WHERE engine = 'exa';
