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

/// MVP5 M1: 피드 아이템 — ephemeral, DB에 저장되지 않음.
/// 검색 API 직접 호출 결과를 담는 임시 모델.
/// id 없음 — 클라이언트는 url을 기사 식별자로 사용.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedItem {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub source: String,
    pub published_at: Option<DateTime<Utc>>,
    pub tag_id: Option<Uuid>,
}

/// MVP5 M3: favorites 테이블 모델.
/// article_id FK 없이 기사 메타 전체를 직접 저장.
/// UNIQUE (user_id, url)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub source: String,
    pub published_at: Option<DateTime<Utc>>,
    pub tag_id: Option<Uuid>,
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

/// 웹서치 결과 (피드 생성 전 중간 모델)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub published_at: Option<String>,
}
