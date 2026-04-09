-- MVP5 M1: 피드 아키텍처 전환 (올바른 버전)
-- - articles 테이블 관련 이전 마이그레이션 롤백 (적용 안 된 경우 무시됨)
-- - favorites 테이블 신규 생성 (article_id FK 없이 기사 메타 직접 저장)

-- 1. 잘못된 favorites 테이블 제거 (article_id FK 있던 버전)
DROP TABLE IF EXISTS favorites;

-- 2. articles 테이블 제거 (피드는 검색 API 직접 호출, DB 저장 불필요)
DROP TABLE IF EXISTS articles;

-- 3. 올바른 favorites 테이블 생성
--    article_id FK 없음 — articles 테이블을 참조하지 않고 기사 메타 전체를 직접 저장
--    UNIQUE (user_id, url) — 동일 URL 중복 즐겨찾기 방지
CREATE TABLE IF NOT EXISTS favorites (
  id           uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id      uuid        NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  title        text        NOT NULL,
  url          text        NOT NULL,
  snippet      text,
  source       text        NOT NULL,
  published_at timestamptz,
  tag_id       uuid,
  summary      text,
  insight      text,
  liked_at     timestamptz,
  created_at   timestamptz NOT NULL DEFAULT now(),
  UNIQUE (user_id, url)
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
