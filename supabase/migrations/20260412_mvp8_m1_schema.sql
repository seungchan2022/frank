-- MVP8 M1: quiz_wrong_answers 테이블 + favorites.quiz_completed 컬럼 추가

-- 1. quiz_wrong_answers 신규 테이블
CREATE TABLE IF NOT EXISTS quiz_wrong_answers (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    article_url   TEXT NOT NULL,
    article_title TEXT NOT NULL,
    question      TEXT NOT NULL,
    options       JSONB NOT NULL,        -- ["보기A","보기B","보기C","보기D"]
    correct_index INTEGER NOT NULL,      -- 0-based
    user_index    INTEGER NOT NULL,      -- 사용자가 선택한 보기 인덱스
    explanation   TEXT,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, article_url, question)
);

CREATE INDEX IF NOT EXISTS idx_qwa_user_id ON quiz_wrong_answers(user_id);
CREATE INDEX IF NOT EXISTS idx_qwa_user_article ON quiz_wrong_answers(user_id, article_url);

-- 2. RLS 활성화
ALTER TABLE quiz_wrong_answers ENABLE ROW LEVEL SECURITY;

-- 3. RLS 정책 4개
-- SELECT: 본인 데이터만
CREATE POLICY "qwa_select_own" ON quiz_wrong_answers
    FOR SELECT USING (auth.uid() = user_id);

-- INSERT: 본인 user_id로만
CREATE POLICY "qwa_insert_own" ON quiz_wrong_answers
    FOR INSERT WITH CHECK (auth.uid() = user_id);

-- UPDATE: ON CONFLICT DO UPDATE를 위해 본인 데이터만
CREATE POLICY "qwa_update_own" ON quiz_wrong_answers
    FOR UPDATE USING (auth.uid() = user_id);

-- DELETE: 본인 데이터만
CREATE POLICY "qwa_delete_own" ON quiz_wrong_answers
    FOR DELETE USING (auth.uid() = user_id);

-- 4. favorites.quiz_completed 컬럼 추가
ALTER TABLE favorites
    ADD COLUMN IF NOT EXISTS quiz_completed BOOLEAN NOT NULL DEFAULT false;
