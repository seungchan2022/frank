use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::ports::{DbPort, LlmPort};

/// 좋아요 처리 결과
#[derive(Debug)]
pub struct LikeResult {
    pub keywords: Vec<String>,
    pub total_likes: i32,
}

/// POST /me/articles/like 오케스트레이션.
///
/// 처리 순서:
/// 1. LLM으로 키워드 추출 (extract_keywords)
/// 2. user_keyword_weights 누적 (increment_keyword_weights) — tag_id 포함
/// 3. profiles.like_count 증가 (increment_like_count)
///
/// idempotency 정책: 클라이언트 책임 (서버는 중복 체크 없음 — 이벤트 누적 모델).
pub async fn process_like<D, L>(
    user_id: Uuid,
    tag_id: Uuid,
    title: &str,
    snippet: Option<&str>,
    db: &D,
    llm: &L,
) -> Result<LikeResult, AppError>
where
    D: DbPort + ?Sized,
    L: LlmPort + ?Sized,
{
    // 1. 키워드 추출 — 실패(rate limit 등) 시 빈 배열로 폴백하여 좋아요 카운트는 유지
    let keywords = llm
        .extract_keywords(title, snippet)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "keyword extraction failed, skipping keyword update");
            vec![]
        });

    // 2. 키워드 가중치 누적 (빈 배열이면 포트 구현이 no-op 처리)
    db.increment_keyword_weights(user_id, tag_id, keywords.clone())
        .await
        .map_err(|e| AppError::Internal(format!("keyword weight update failed: {e}")))?;

    // 3. 좋아요 카운트 증가
    let total_likes = db
        .increment_like_count(user_id)
        .await
        .map_err(|e| AppError::Internal(format!("like count update failed: {e}")))?;

    Ok(LikeResult {
        keywords,
        total_likes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Profile;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;

    fn make_db_with_profile(user_id: Uuid) -> FakeDbAdapter {
        let db = FakeDbAdapter::new();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: false,
        });
        db
    }

    #[tokio::test]
    async fn process_like_returns_keywords_and_total_likes() {
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let db = make_db_with_profile(user_id);
        let llm = FakeLlmAdapter::new();

        let result = process_like(user_id, tag_id, "iOS 기사", Some("Swift"), &db, &llm)
            .await
            .unwrap();

        // FakeLlm은 ["iOS", "Swift", "SwiftUI"] 반환
        assert_eq!(result.keywords, vec!["iOS", "Swift", "SwiftUI"]);
        assert_eq!(result.total_likes, 1);
    }

    #[tokio::test]
    async fn process_like_accumulates_on_multiple_calls() {
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let db = make_db_with_profile(user_id);
        let llm = FakeLlmAdapter::new();

        let r1 = process_like(user_id, tag_id, "기사1", None, &db, &llm)
            .await
            .unwrap();
        assert_eq!(r1.total_likes, 1);

        let r2 = process_like(user_id, tag_id, "기사2", None, &db, &llm)
            .await
            .unwrap();
        assert_eq!(r2.total_likes, 2);
    }

    #[tokio::test]
    async fn process_like_no_snippet() {
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let db = make_db_with_profile(user_id);
        let llm = FakeLlmAdapter::new();

        let result = process_like(user_id, tag_id, "title only", None, &db, &llm)
            .await
            .unwrap();
        assert!(!result.keywords.is_empty());
        assert_eq!(result.total_likes, 1);
    }

    #[tokio::test]
    async fn process_like_llm_failure_falls_back_to_empty_keywords() {
        // LLM 실패(rate limit 등) 시 keywords=[] 로 폴백하여 좋아요 카운트는 정상 처리
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let db = make_db_with_profile(user_id);
        let llm = FakeLlmAdapter::failing();

        let result = process_like(user_id, tag_id, "title", None, &db, &llm).await;
        assert!(
            result.is_ok(),
            "LLM 실패 시에도 500 아닌 정상 처리: {result:?}"
        );
        let result = result.unwrap();
        assert!(
            result.keywords.is_empty(),
            "키워드 추출 실패 시 빈 배열 반환"
        );
        assert_eq!(result.total_likes, 1, "좋아요 카운트는 정상 증가");
    }

    #[tokio::test]
    async fn process_like_missing_profile_returns_error() {
        let user_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let db = FakeDbAdapter::new(); // profile 없음
        let llm = FakeLlmAdapter::new();

        let result = process_like(user_id, tag_id, "title", None, &db, &llm).await;
        assert!(result.is_err());
    }
}
