-- MVP16 M3: favorites.rewrite_occupation 컬럼 추가
-- C2-bug 수정: 직업 변경 후 재작성 버튼 재활성화를 위해 재작성 당시의 occupation을 저장
ALTER TABLE favorites
    ADD COLUMN IF NOT EXISTS rewrite_occupation TEXT NULL;

-- 기존 행은 NULL (재작성 당시 occupation 미추적 → 버튼 재활성화 대상)
-- 새 재작성 시 occupation과 함께 저장
