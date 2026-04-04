-- articles: 수집된 뉴스 기사
create table public.articles (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references public.profiles(id) on delete cascade,
  tag_id uuid references public.tags(id) on delete set null,
  title text not null,
  url text not null,
  snippet text,
  source text not null,
  search_query text,
  published_at timestamptz,
  created_at timestamptz not null default now(),

  unique(user_id, url)
);

alter table public.articles enable row level security;

create policy "Users can read own articles"
  on public.articles for select
  using (auth.uid() = user_id);

create policy "Users can insert own articles"
  on public.articles for insert
  with check (auth.uid() = user_id);

create policy "Users can delete own articles"
  on public.articles for delete
  using (auth.uid() = user_id);

create index idx_articles_user_created on public.articles(user_id, created_at desc);
