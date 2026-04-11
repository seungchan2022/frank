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
    /// MVP6 M1: 썸네일 이미지 URL (없으면 None)
    pub image_url: Option<String>,
}

/// MVP5 M3: favorites 테이블 모델.
/// article_id FK 없이 기사 메타 전체를 직접 저장.
/// UNIQUE (user_id, url)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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
    /// MVP6 M1: 썸네일 이미지 URL (없으면 None)
    pub image_url: Option<String>,
    /// MVP7 M1: 퀴즈 생성 후 저장되는 개념 정리 JSON (없으면 None)
    pub concepts: Option<serde_json::Value>,
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

/// MVP7 M1: 사용자 키워드 가중치 (DB 모델 아님 — 서비스 레이어 전달용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKeywordWeight {
    pub user_id: Uuid,
    pub keyword: String,
    pub weight: i32,
}

/// MVP7 M1: 퀴즈 문제 (LLM 생성 결과, DB 저장 없음)
/// answer_index: 보기는 최대 4개이므로 u8 사용 (usize는 플랫폼 의존적)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub question: String,
    pub options: Vec<String>,
    pub answer_index: u8,
    pub explanation: String,
}

/// MVP7 M1: 개념 정리 항목 (favorites.concepts JSONB 구조)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizConcept {
    pub term: String,
    pub explanation: String,
}

/// 웹서치 결과 (피드 생성 전 중간 모델)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub published_at: Option<String>,
    /// MVP6 M1: 썸네일 이미지 URL (없으면 None)
    pub image_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_keyword_weight_serializes() {
        let ukw = UserKeywordWeight {
            user_id: Uuid::nil(),
            keyword: "Swift".to_string(),
            weight: 3,
        };
        let json = serde_json::to_string(&ukw).expect("serialize");
        assert!(json.contains("\"keyword\":\"Swift\""));
        assert!(json.contains("\"weight\":3"));
    }

    #[test]
    fn quiz_question_options_count() {
        let q = QuizQuestion {
            question: "SwiftUI는 무엇인가?".to_string(),
            options: vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
            ],
            answer_index: 0u8,
            explanation: "SwiftUI는 선언형 UI 프레임워크입니다.".to_string(),
        };
        assert_eq!(q.options.len(), 4);
        assert_eq!(q.answer_index, 0u8);
        // answer_index는 options 범위 내여야 함
        assert!((q.answer_index as usize) < q.options.len());
    }

    #[test]
    fn quiz_concept_serializes() {
        let concept = QuizConcept {
            term: "MVVM".to_string(),
            explanation: "Model-View-ViewModel 패턴".to_string(),
        };
        let json = serde_json::to_string(&concept).expect("serialize");
        assert!(json.contains("\"term\":\"MVVM\""));
        assert!(json.contains("\"explanation\""));
    }
}
