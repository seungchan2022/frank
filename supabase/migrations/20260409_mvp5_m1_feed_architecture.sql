-- MVP5 M1: 피드 아키텍처 전환
-- articles 테이블 경량화 + favorites 테이블 신규 생성 + 기존 데이터 삭제

-- 1. 기존 articles 데이터 전부 삭제 (새 구조로 시작)
DELETE FROM articles;

-- 2. articles 테이블에서 제거 대상 컬럼 DROP
--    (content, title_ko, llm_model, prompt_tokens, completion_tokens, summarized_at, search_query)
ALTER TABLE articles
  DROP COLUMN IF EXISTS content,
  DROP COLUMN IF EXISTS title_ko,
  DROP COLUMN IF EXISTS llm_model,
  DROP COLUMN IF EXISTS prompt_tokens,
  DROP COLUMN IF EXISTS completion_tokens,
  DROP COLUMN IF EXISTS summarized_at,
  DROP COLUMN IF EXISTS search_query;

-- 3. favorites 테이블 신규 생성
CREATE TABLE IF NOT EXISTS favorites (
  id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id     uuid NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  article_id  uuid NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
  summary     text,
  insight     text,
  liked_at    timestamptz,
  created_at  timestamptz NOT NULL DEFAULT now(),
  UNIQUE (user_id, article_id)
);

-- 4. favorites RLS 활성화
ALTER TABLE favorites ENABLE ROW LEVEL SECURITY;

-- 5. favorites RLS 정책: 본인 데이터만 접근
CREATE POLICY "favorites_select_own" ON favorites
  FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "favorites_insert_own" ON favorites
  FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "favorites_update_own" ON favorites
  FOR UPDATE USING (auth.uid() = user_id);

CREATE POLICY "favorites_delete_own" ON favorites
  FOR DELETE USING (auth.uid() = user_id);
