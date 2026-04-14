-- MVP9 M2: user_keyword_weights에 tag_id 컬럼 추가
-- 변경 사항:
--   1. 기존 레코드 DELETE (스키마 변경 전 정리)
--   2. tag_id UUID NOT NULL 컬럼 추가
--   3. UNIQUE 제약 변경: (user_id, keyword) → (user_id, tag_id, keyword)
--   4. tag_id 인덱스 추가

-- 1. 기존 레코드 전체 삭제 (tag_id 없는 데이터는 의미 없음)
DELETE FROM user_keyword_weights;

-- 2. tag_id 컬럼 추가
ALTER TABLE user_keyword_weights
    ADD COLUMN tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE;

-- 3. 기존 UNIQUE 제약 삭제 후 새 제약 추가
ALTER TABLE user_keyword_weights
    DROP CONSTRAINT IF EXISTS user_keyword_weights_user_id_keyword_key;

ALTER TABLE user_keyword_weights
    ADD CONSTRAINT user_keyword_weights_user_id_tag_id_keyword_key
    UNIQUE (user_id, tag_id, keyword);

-- 4. tag_id 인덱스 추가
CREATE INDEX IF NOT EXISTS idx_ukw_tag_id ON user_keyword_weights(tag_id);
