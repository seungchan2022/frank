use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Profile {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub onboarding_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserTag {
    pub user_id: Uuid,
    pub tag_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Article {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tag_id: Option<Uuid>,
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub source: String,
    pub search_query: Option<String>,
    pub summary: Option<String>,
    pub insight: Option<String>,
    pub summarized_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub title_ko: Option<String>,
    pub content: Option<String>,
    pub llm_model: Option<String>,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
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
