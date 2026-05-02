use axum::Json;
use axum::extract::Extension;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;
use crate::services::likes_service;

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct LikeArticleRequest {
    pub title: String,
    pub snippet: Option<String>,
    pub tag_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LikeArticleResponse {
    pub keywords: Vec<String>,
    pub total_likes: i32,
}

/// POST /me/articles/like
/// 기사 제목/스니펫에서 키워드를 추출하고 가중치를 누적한다.
pub async fn like_article<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<LikeArticleRequest>,
) -> Result<Json<LikeArticleResponse>, AppError> {
    let title = body.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::BadRequest("title must not be empty".to_string()));
    }

    let snippet = body.snippet.as_deref();

    let result = likes_service::process_like(
        user.id,
        body.tag_id,
        &title,
        snippet,
        &state.db,
        state.llm.as_ref(),
    )
    .await?;

    Ok(Json(LikeArticleResponse {
        keywords: result.keywords,
        total_likes: result.total_likes,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Profile;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_favorites::FakeFavoritesAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_quiz_wrong_answers::FakeQuizWrongAnswerAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::feed_cache::NoopFeedCache;
    use crate::infra::search_chain::SearchFallbackChain;
    use axum::Router;
    use axum::routing::post;
    use axum_test::TestServer;
    use std::sync::Arc;
    use uuid::Uuid;

    fn make_test_state(db: FakeDbAdapter) -> AppState<FakeDbAdapter> {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            vec![],
            false,
        ))]);
        AppState {
            db,
            search_chain: Arc::new(chain),
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::new(NoopFeedCache),
            counter: Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new()),
        }
    }

    fn make_app(state: AppState<FakeDbAdapter>, user_id: Uuid) -> Router {
        Router::new()
            .route("/me/articles/like", post(like_article::<FakeDbAdapter>))
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    fn seed_user(db: &FakeDbAdapter, user_id: Uuid) {
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });
    }

    #[tokio::test]
    async fn like_article_returns_keywords_and_total_likes() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let tag_id = Uuid::new_v4();
        let resp = server
            .post("/me/articles/like")
            .json(&serde_json::json!({
                "title": "iOS 개발 기사",
                "snippet": "Swift 관련 내용",
                "tag_id": tag_id
            }))
            .await;
        resp.assert_status_ok();
        let body: LikeArticleResponse = resp.json();
        assert_eq!(body.keywords, vec!["iOS", "Swift", "SwiftUI"]);
        assert_eq!(body.total_likes, 1);
    }

    #[tokio::test]
    async fn like_article_without_snippet() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let tag_id = Uuid::new_v4();
        let resp = server
            .post("/me/articles/like")
            .json(&serde_json::json!({
                "title": "기사 제목만",
                "tag_id": tag_id
            }))
            .await;
        resp.assert_status_ok();
        let body: LikeArticleResponse = resp.json();
        assert_eq!(body.total_likes, 1);
    }

    #[tokio::test]
    async fn like_article_empty_title_returns_400() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let tag_id = Uuid::new_v4();
        let resp = server
            .post("/me/articles/like")
            .json(&serde_json::json!({
                "title": "   ",
                "tag_id": tag_id
            }))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn like_article_accumulates_on_multiple_calls() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let tag_id = Uuid::new_v4();
        let resp1 = server
            .post("/me/articles/like")
            .json(&serde_json::json!({"title": "기사1", "tag_id": tag_id}))
            .await;
        resp1.assert_status_ok();
        let b1: LikeArticleResponse = resp1.json();
        assert_eq!(b1.total_likes, 1);

        let resp2 = server
            .post("/me/articles/like")
            .json(&serde_json::json!({"title": "기사2", "tag_id": tag_id}))
            .await;
        resp2.assert_status_ok();
        let b2: LikeArticleResponse = resp2.json();
        assert_eq!(b2.total_likes, 2);
    }
}
