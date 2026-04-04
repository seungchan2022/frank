create policy "Users can update own articles"
  on public.articles for update
  using (auth.uid() = user_id);
