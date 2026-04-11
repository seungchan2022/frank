-- MVP7 M2: user_keyword_weights RLS 활성화
-- M1에서 테이블 생성 시 누락된 RLS 정책 추가.
-- favorites와 동일한 패턴 적용: 본인 데이터만 접근.

ALTER TABLE user_keyword_weights ENABLE ROW LEVEL SECURITY;

CREATE POLICY "ukw_select_own" ON user_keyword_weights
  FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "ukw_insert_own" ON user_keyword_weights
  FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "ukw_update_own" ON user_keyword_weights
  FOR UPDATE USING (auth.uid() = user_id);

CREATE POLICY "ukw_delete_own" ON user_keyword_weights
  FOR DELETE USING (auth.uid() = user_id);
