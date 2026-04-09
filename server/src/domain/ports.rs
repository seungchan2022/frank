use std::future::Future;
use std::pin::Pin;

use uuid::Uuid;

use super::error::AppError;
use super::models::{Article, LlmResponse, Profile, SearchResult, Tag, UserTag};

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

    /// 프로필 부분 수정. 두 필드 모두 None이면 no-op으로 현재 프로필 반환.
    fn update_profile(
        &self,
        user_id: Uuid,
        onboarding_completed: Option<bool>,
        display_name: Option<String>,
    ) -> impl std::future::Future<Output = Result<Profile, AppError>> + Send;

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

    /// 유저의 기사 목록 조회 (페이지네이션).
    ///
    /// **계약**: `user_tags`에 등록된 활성 태그의 기사만 반환한다.
    /// 사용자가 태그를 제거한 경우 해당 태그의 기사는 결과에서 제외된다.
    /// `tag_id` 지정 시 해당 태그가 활성 상태일 때만 그 태그의 기사를 반환한다.
    /// 최신순(`created_at DESC`) 정렬 후 `limit`/`offset` 페이지네이션 적용.
    fn get_user_articles(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
        tag_id: Option<Uuid>,
    ) -> impl std::future::Future<Output = Result<Vec<Article>, AppError>> + Send;

    /// 본인 기사 단건 조회.
    ///
    /// **계약**: 활성 태그 기사만 접근 가능. 타인 기사, 비활성 태그 기사, 없는 기사는
    /// 모두 `Ok(None)` 반환 (호출부에서 404 처리).
    fn get_user_article_by_id(
        &self,
        user_id: Uuid,
        article_id: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<Article>, AppError>> + Send;

    #[allow(clippy::too_many_arguments)]
    fn update_article_summary(
        &self,
        article_id: Uuid,
        summary: &str,
        insight: &str,
        title_ko: &str,
        llm_model: &str,
        prompt_tokens: i32,
        completion_tokens: i32,
    ) -> impl Future<Output = Result<(), AppError>> + Send;

    fn update_article_content(
        &self,
        article_id: Uuid,
        content: &str,
    ) -> impl Future<Output = Result<(), AppError>> + Send;
}

/// 검색 폴백 체인 포트 (여러 SearchPort를 순서대로 시도)
/// dyn compatible을 위해 boxed future 사용
pub trait SearchChainPort: Send + Sync {
    #[allow(clippy::type_complexity)]
    fn search<'a>(
        &'a self,
        query: &'a str,
        max_results: usize,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<SearchResult>, String), AppError>> + Send + 'a>>;
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

/// 웹 크롤링 포트 (Firecrawl scrape 등)
/// dyn compatible을 위해 boxed future 사용
pub trait CrawlPort: Send + Sync {
    fn scrape(
        &self,
        url: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + '_>>;
}

/// LLM 요약 포트 (OpenRouter 등)
/// dyn compatible을 위해 boxed future 사용
pub trait LlmPort: Send + Sync {
    fn summarize(
        &self,
        title: &str,
        content: &str,
    ) -> Pin<Box<dyn Future<Output = Result<LlmResponse, AppError>> + Send + '_>>;
}

/// 알림 전송 포트 (iMessage 등)
pub trait NotificationPort: Send + Sync {
    fn send(&self, message: &str) -> Result<(), AppError>;
}
