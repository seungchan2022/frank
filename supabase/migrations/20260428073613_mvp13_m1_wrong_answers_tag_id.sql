-- MVP13 M1: quiz_wrong_answers에 tag_id 컬럼 추가
-- 기존 오답 전체 삭제 (테스트 데이터, 실사용 전)
DELETE FROM quiz_wrong_answers;

ALTER TABLE quiz_wrong_answers
  ADD COLUMN tag_id UUID NULL REFERENCES tags(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_qwa_user_tag
  ON quiz_wrong_answers (user_id, tag_id);
