use std::future::Future;
use std::pin::Pin;

use uuid::Uuid;

use super::error::AppError;
use super::models::{Article, LlmSummary, Profile, SearchResult, Tag, UserTag};

/// DB 접근 포트 (sqlx PostgreSQL 직접 연결)
pub trait DbPort: Send + Sync {
    fn get_profile(
        &self,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Profile, AppError>> + Send;

    fn update_profile_onboarding(
        &self,
        user_id: Uuid,
        completed: bool,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    fn list_tags(&self) -> impl std::future::Future<Output = Result<Vec<Tag>, AppError>> + Send;

    fn get_user_tags(
        &self,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Vec<UserTag>, AppError>> + Send;

    fn set_user_tags(
        &self,
        user_id: Uuid,
        tag_ids: Vec<Uuid>,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    fn save_articles(
        &self,
        articles: Vec<Article>,
    ) -> impl std::future::Future<Output = Result<usize, AppError>> + Send;

    fn get_user_articles(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> impl std::future::Future<Output = Result<Vec<Article>, AppError>> + Send;

    fn update_article_summary(
        &self,
        article_id: Uuid,
        summary: &str,
        insight: &str,
    ) -> impl Future<Output = Result<(), AppError>> + Send;
}

/// 웹서치 포트 (Tavily, Exa, Firecrawl, arXiv)
/// dyn compatible을 위해 boxed future 사용
pub trait SearchPort: Send + Sync {
    fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SearchResult>, AppError>> + Send + '_>>;

    fn source_name(&self) -> &str;
}

/// LLM 요약 포트 (OpenRouter 등)
/// dyn compatible을 위해 boxed future 사용
pub trait LlmPort: Send + Sync {
    fn summarize(
        &self,
        title: &str,
        snippet: &str,
    ) -> Pin<Box<dyn Future<Output = Result<LlmSummary, AppError>> + Send + '_>>;
}
