ALTER TABLE public.articles ADD COLUMN IF NOT EXISTS title_ko text;
ALTER TABLE public.articles ADD COLUMN IF NOT EXISTS content text;
ALTER TABLE public.articles ADD COLUMN IF NOT EXISTS llm_model text;
ALTER TABLE public.articles ADD COLUMN IF NOT EXISTS prompt_tokens integer;
ALTER TABLE public.articles ADD COLUMN IF NOT EXISTS completion_tokens integer;
