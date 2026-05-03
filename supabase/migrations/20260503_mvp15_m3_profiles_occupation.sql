-- MVP15 M3: 프로필 직업 + 재작성 결과 저장 컬럼 추가
--
-- 설계 결정:
-- 1) `profiles.occupation TEXT NULL` — 최대 50자 직업 한 줄 입력.
--    서버에서 trim + length 검증 후 저장. NULL = 직업 미설정 = insight 없음.
-- 2) `favorites.rewrite TEXT NULL` — LLM 재작성 결과 저장.
--    비즐겨찾기 기사도 favorites row를 upsert로 자동 생성하여 저장.
--    best-effort: 저장 실패해도 200 반환 (E-04 요구사항).
-- 기존 row는 NULL로 자동 채워짐 — 마이그레이션 안전.

ALTER TABLE profiles ADD COLUMN IF NOT EXISTS occupation TEXT NULL;
ALTER TABLE favorites ADD COLUMN IF NOT EXISTS rewrite TEXT NULL;
