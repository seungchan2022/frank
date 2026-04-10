-- MVP6 M1: favorites 테이블에 image_url 컬럼 추가 (썸네일)
ALTER TABLE favorites ADD COLUMN IF NOT EXISTS image_url text;
