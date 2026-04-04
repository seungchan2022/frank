-- profiles: auth.users 확장 (온보딩 ���료 여부 등)
create table public.profiles (
  id uuid primary key references auth.users(id) on delete cascade,
  display_name text,
  onboarding_completed boolean not null default false,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

alter table public.profiles enable row level security;

create policy "Users can read own profile"
  on public.profiles for select
  using (auth.uid() = id);

create policy "Users can update own profile"
  on public.profiles for update
  using (auth.uid() = id);

create policy "Users can insert own profile"
  on public.profiles for insert
  with check (auth.uid() = id);

-- 신규 유저 가입 시 profiles 자동 생성
create or replace function public.handle_new_user()
returns trigger as $$
begin
  insert into public.profiles (id, display_name)
  values (new.id, coalesce(new.raw_user_meta_data->>'full_name', new.email));
  return new;
end;
$$ language plpgsql security definer;

create trigger on_auth_user_created
  after insert on auth.users
  for each row execute function public.handle_new_user();

-- tags: 미리 정��된 관심 태그 목록
create table public.tags (
  id uuid primary key default gen_random_uuid(),
  name text not null unique,
  category text,
  created_at timestamptz not null default now()
);

alter table public.tags enable row level security;

create policy "Tags are readable by all authenticated users"
  on public.tags for select
  to authenticated
  using (true);

-- user_tags: 사용자가 선택한 태그 (다대다)
create table public.user_tags (
  user_id uuid not null references public.profiles(id) on delete cascade,
  tag_id uuid not null references public.tags(id) on delete cascade,
  created_at timestamptz not null default now(),
  primary key (user_id, tag_id)
);

alter table public.user_tags enable row level security;

create policy "Users can read own tags"
  on public.user_tags for select
  using (auth.uid() = user_id);

create policy "Users can insert own tags"
  on public.user_tags for insert
  with check (auth.uid() = user_id);

create policy "Users can delete own tags"
  on public.user_tags for delete
  using (auth.uid() = user_id);

-- 기본 태그 시드 데이터
insert into public.tags (name, category) values
  ('AI/ML', '기술'),
  ('웹 개발', '기술'),
  ('모바일 개발', '기술'),
  ('클라��드/인프라', '기술'),
  ('보안', '기술'),
  ('데이터 사이언스', '기술'),
  ('블록체인', '기술'),
  ('스타트업', '비즈니스'),
  ('��자/VC', '비즈니스'),
  ('프로덕트', '비즈니스'),
  ('UX/디자인', '디자인'),
  ('오픈소스', '커뮤니티');
