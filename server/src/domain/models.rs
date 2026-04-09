use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub onboarding_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTag {
    pub user_id: Uuid,
    pub tag_id: Uuid,
}

/// MVP5 M1: 피드 아키텍처 전환 후 경량화된 Article 모델.
/// 크롤링/요약 관련 컬럼(content, title_ko, llm_model, prompt_tokens,
/// completion_tokens, summarized_at, search_query) 제거.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tag_id: Option<Uuid>,
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub source: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

/// MVP5 M1: favorites 테이블 모델.
/// 즐겨찾기 시 현재 세션의 요약/인사이트 상태를 함께 저장.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub article_id: Uuid,
    pub summary: Option<String>,
    pub insight: Option<String>,
    pub liked_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

/// LLM 요약 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSummary {
    pub title_ko: String,
    pub summary: String,
    pub insight: String,
}

/// LLM 응답 (요약 + 토큰 사용량)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub summary: LlmSummary,
    pub model: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
}

/// 웹서치 결과 (DB 저장 전 중간 모델)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub published_at: Option<String>,
}
