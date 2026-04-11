-- MVP7 M1: 좋아요/개인화/퀴즈 기반 DB 스키마
-- 변경 사항:
--   1. user_keyword_weights 테이블 신규 생성
--   2. profiles.like_count 컬럼 추가
--   3. favorites.concepts 컬럼 추가

-- 1. user_keyword_weights: 사용자별 키워드 가중치 누적 테이블
CREATE TABLE IF NOT EXISTS user_keyword_weights (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    keyword     VARCHAR(100) NOT NULL,
    weight      INTEGER NOT NULL DEFAULT 1,
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, keyword)
);

CREATE INDEX IF NOT EXISTS idx_ukw_user_id ON user_keyword_weights(user_id);

-- 2. profiles.like_count: 총 좋아요 이벤트 수 (개인화 threshold 판단용)
ALTER TABLE profiles
    ADD COLUMN IF NOT EXISTS like_count INTEGER NOT NULL DEFAULT 0;

-- 3. favorites.concepts: 퀴즈 생성 후 저장되는 개념 정리 JSON
-- 구조: [{ "term": "용어명", "explanation": "한국어 설명" }]
ALTER TABLE favorites
    ADD COLUMN IF NOT EXISTS concepts JSONB;
