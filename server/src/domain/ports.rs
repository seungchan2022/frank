use std::future::Future;
use std::pin::Pin;

use uuid::Uuid;

use super::error::AppError;
use super::models::{Favorite, LlmResponse, Profile, SearchResult, Tag, UserTag};

/// DB 접근 포트 (Supabase REST API 또는 sqlx)
/// MVP5 M1: articles 관련 메서드 제거 — 피드는 검색 API 직접 호출, DB 저장 없음
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

/// 즐겨찾기 포트 (M2: summary/insight 업데이트, M3: CRUD 확장)
/// dyn compatible을 위해 boxed future 사용
pub trait FavoritesPort: Send + Sync {
    /// favorites 테이블에서 해당 (user_id, url) 행의 summary/insight를 업데이트.
    /// url이 favorites에 없으면 0행 업데이트 (에러 없음).
    fn update_favorite_summary<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
        summary: &'a str,
        insight: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>;

    /// MVP5 M3: 즐겨찾기 추가.
    /// 동일 (user_id, url)이 이미 존재하면 AppError::Conflict 반환.
    fn add_favorite<'a>(
        &'a self,
        user_id: Uuid,
        item: &'a Favorite,
    ) -> Pin<Box<dyn Future<Output = Result<Favorite, AppError>> + Send + 'a>>;

    /// MVP5 M3: 즐겨찾기 삭제.
    /// 존재하지 않는 url이어도 Ok(()) 반환 (no-op).
    fn delete_favorite<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>;

    /// MVP5 M3: 즐겨찾기 목록 조회 (created_at DESC).
    fn list_favorites(
        &self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Favorite>, AppError>> + Send + '_>>;
}
