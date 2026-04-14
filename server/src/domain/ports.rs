use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use uuid::Uuid;

use super::error::AppError;
use super::models::{
    Favorite, FeedItem, LlmResponse, Profile, QuizConcept, QuizResult, QuizWrongAnswer,
    SaveWrongAnswerParams, SearchResult, Tag, UserTag,
};

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

    /// MVP7 M2: 키워드 가중치 누적.
    /// 신규 키워드는 INSERT(weight=1), 기존 키워드는 weight+1.
    /// tag_id: 어느 태그 피드에서 좋아요를 눌렀는지 (연관 태그별 분리 저장).
    fn increment_keyword_weights(
        &self,
        user_id: Uuid,
        tag_id: Uuid,
        keywords: Vec<String>,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    /// MVP7 M2: 상위 키워드 조회 (weight DESC, updated_at DESC, keyword ASC).
    /// tag_ids: 필터링할 태그 목록. 빈 배열이면 전체 태그에서 조회.
    /// limit: u32 (usize는 플랫폼 의존)
    fn get_top_keywords(
        &self,
        user_id: Uuid,
        tag_ids: Vec<Uuid>,
        limit: u32,
    ) -> impl std::future::Future<Output = Result<Vec<String>, AppError>> + Send;

    /// MVP7 M2: 좋아요 누적 카운트 증가.
    /// profile row 부재 시 AppError::NotFound 반환.
    fn increment_like_count(
        &self,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<i32, AppError>> + Send;

    /// MVP7 M3: 현재 좋아요 누적 카운트 조회.
    /// profile row 부재 시 0 반환 (개인화 비활성화로 처리).
    fn get_like_count(
        &self,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<i32, AppError>> + Send;
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

    /// MVP7 M2: 기사 제목/스니펫에서 핵심 키워드 추출.
    /// snippet은 FeedItem.snippet이 Option이므로 Option<&str>.
    fn extract_keywords<'a>(
        &'a self,
        title: &'a str,
        snippet: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, AppError>> + Send + 'a>>;

    /// MVP7 M4: 기사 제목/내용에서 개념 정리 + 객관식 퀴즈 3문제 생성.
    fn generate_quiz<'a>(
        &'a self,
        title: &'a str,
        content: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<QuizResult, AppError>> + Send + 'a>>;
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

    /// MVP7 M4: (user_id, url)로 즐겨찾기 단건 조회.
    /// 존재하지 않으면 Ok(None) 반환.
    fn get_favorite_by_url<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Favorite>, AppError>> + Send + 'a>>;

    /// MVP7 M4: favorites.concepts 컬럼 업데이트.
    /// url이 favorites에 없으면 0행 업데이트 (에러 없음).
    fn update_favorite_concepts<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
        concepts: Vec<QuizConcept>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>;

    /// MVP8 M1: favorites.quiz_completed = true 업데이트.
    /// url이 favorites에 없으면 0행 업데이트 (에러 없음).
    fn mark_quiz_completed<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>;
}

/// MVP8 M1: 오답 저장/조회/삭제 포트.
/// dyn compatible을 위해 boxed future 사용.
pub trait QuizWrongAnswerPort: Send + Sync {
    /// 오답 1건 저장 (중복 시 덮어쓰기: ON CONFLICT DO UPDATE).
    fn save<'a>(
        &'a self,
        user_id: Uuid,
        params: SaveWrongAnswerParams,
    ) -> Pin<Box<dyn Future<Output = Result<QuizWrongAnswer, AppError>> + Send + 'a>>;

    /// 오답 목록 조회 (created_at DESC).
    fn list(
        &self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<QuizWrongAnswer>, AppError>> + Send + '_>>;

    /// 오답 1건 삭제 (본인 데이터만 — WHERE id = $1 AND user_id = $2).
    /// 존재하지 않는 id여도 Ok(()) 반환 (no-op).
    fn delete<'a>(
        &'a self,
        user_id: Uuid,
        id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>;
}

/// 피드 TTL 인메모리 캐시 포트.
/// 캐시 키: `"{user_id}:{sorted_tag_ids}"` (태그 없으면 `"{user_id}:all"`)
/// 구현체는 `InMemoryFeedCache` (프로덕션) / `NoopFeedCache` (테스트) 중 선택.
pub trait FeedCachePort: Send + Sync {
    /// 캐시에서 피드 아이템을 조회한다. 만료되거나 없으면 `None` 반환.
    fn get(&self, key: &str) -> Option<Vec<FeedItem>>;

    /// 피드 아이템을 캐시에 저장한다. `ttl` 경과 후 자동 만료.
    fn set(&self, key: &str, items: Vec<FeedItem>, ttl: Duration);

    /// 해당 `user_id`의 모든 캐시 엔트리를 즉시 무효화한다.
    /// 태그 변경 시 호출하여 stale 피드 방지.
    fn invalidate_user(&self, user_id: Uuid);
}
