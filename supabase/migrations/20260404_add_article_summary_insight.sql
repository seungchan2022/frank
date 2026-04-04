-- articles에 LLM 요약/인사이트 컬럼 추가
alter table public.articles add column summary text;
alter table public.articles add column insight text;
alter table public.articles add column summarized_at timestamptz;
