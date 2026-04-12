use axum::Json;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::domain::error::AppError;
use crate::domain::models::QuizQuestion;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;
use crate::services::quiz_service;

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct QuizRequest {
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct QuizDoneRequest {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuizResponse {
    pub questions: Vec<QuizQuestion>,
}

/// POST /me/favorites/quiz/done
/// 퀴즈 완료 마킹 — favorites.quiz_completed = true.
/// url이 favorites에 없어도 204 반환 (no-op).
pub async fn mark_quiz_done<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<QuizDoneRequest>,
) -> Result<StatusCode, AppError> {
    let url = body.url.trim().to_string();
    if url.is_empty() {
        return Err(AppError::BadRequest("url must not be empty".to_string()));
    }

    state.favorites.mark_quiz_completed(user.id, &url).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /me/favorites/quiz
/// 즐겨찾기한 기사에서 퀴즈 3문제를 생성한다.
/// - 404: 즐겨찾기 미존재
/// - 503: LLM 실패
pub async fn generate_quiz<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<QuizRequest>,
) -> Result<impl IntoResponse, AppError> {
    let url = body.url.trim().to_string();
    if url.is_empty() {
        return Err(AppError::BadRequest("url must not be empty".to_string()));
    }

    let questions =
        quiz_service::generate_quiz(user.id, &url, state.favorites.as_ref(), state.llm.as_ref())
            .await?;

    Ok((StatusCode::OK, Json(QuizResponse { questions })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Favorite, Profile};
    use crate::domain::ports::FavoritesPort;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_favorites::FakeFavoritesAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_quiz_wrong_answers::FakeQuizWrongAnswerAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::search_chain::SearchFallbackChain;
    use axum::Router;
    use axum::http::StatusCode;
    use axum::routing::post;
    use axum_test::TestServer;
    use chrono::Utc;
    use std::sync::Arc;
    use uuid::Uuid;

    fn make_test_state(
        db: FakeDbAdapter,
        favorites: FakeFavoritesAdapter,
    ) -> AppState<FakeDbAdapter> {
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
            favorites: Arc::new(favorites),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
        }
    }

    fn make_app(state: AppState<FakeDbAdapter>, user_id: Uuid) -> Router {
        Router::new()
            .route("/me/favorites/quiz", post(generate_quiz::<FakeDbAdapter>))
            .route(
                "/me/favorites/quiz/done",
                post(mark_quiz_done::<FakeDbAdapter>),
            )
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    fn make_favorite(user_id: Uuid, url: &str) -> Favorite {
        Favorite {
            id: Uuid::new_v4(),
            user_id,
            title: "테스트 기사 제목".to_string(),
            url: url.to_string(),
            snippet: Some("테스트 스니펫".to_string()),
            source: "테스트".to_string(),
            published_at: Some(Utc::now()),
            tag_id: None,
            summary: Some("테스트 요약".to_string()),
            insight: Some("테스트 인사이트".to_string()),
            liked_at: None,
            created_at: Some(Utc::now()),
            image_url: None,
            concepts: None,
            quiz_completed: false,
        }
    }

    // --- mark_quiz_done 테스트 ---

    #[tokio::test]
    async fn mark_quiz_done_returns_204() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let favorites = FakeFavoritesAdapter::new();
        let url = "https://example.com/article";
        let item = make_favorite(user_id, url);
        favorites.add_favorite(user_id, &item).await.unwrap();

        let state = make_test_state(db, favorites);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/favorites/quiz/done")
            .json(&serde_json::json!({ "url": url }))
            .await;
        resp.assert_status(StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn mark_quiz_done_empty_url_returns_400() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, FakeFavoritesAdapter::new());
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/favorites/quiz/done")
            .json(&serde_json::json!({ "url": "" }))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn mark_quiz_done_nonexistent_url_is_noop() {
        // favorites에 없는 url이어도 204 반환 (no-op)
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, FakeFavoritesAdapter::new());
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/favorites/quiz/done")
            .json(&serde_json::json!({ "url": "https://not-in-favorites.com" }))
            .await;
        resp.assert_status(StatusCode::NO_CONTENT);
    }

    // --- generate_quiz 테스트 ---

    #[tokio::test]
    async fn generate_quiz_returns_questions() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let favorites = FakeFavoritesAdapter::new();
        let url = "https://example.com/article";
        let item = make_favorite(user_id, url);
        favorites.add_favorite(user_id, &item).await.unwrap();

        let state = make_test_state(db, favorites);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/favorites/quiz")
            .json(&serde_json::json!({ "url": url }))
            .await;
        resp.assert_status_ok();
        let body: QuizResponse = resp.json();
        assert_eq!(body.questions.len(), 1);
        assert_eq!(body.questions[0].options.len(), 4);
    }

    #[tokio::test]
    async fn generate_quiz_not_in_favorites_returns_404() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let favorites = FakeFavoritesAdapter::new();
        let state = make_test_state(db, favorites);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/favorites/quiz")
            .json(&serde_json::json!({ "url": "https://not-favorited.com" }))
            .await;
        resp.assert_status_not_found();
    }

    #[tokio::test]
    async fn generate_quiz_empty_url_returns_400() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let favorites = FakeFavoritesAdapter::new();
        let state = make_test_state(db, favorites);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/favorites/quiz")
            .json(&serde_json::json!({ "url": "   " }))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn generate_quiz_uses_summary_over_snippet() {
        // summary가 있으면 summary를 사용한다 — FakeLlm이 호출되면 OK
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let favorites = FakeFavoritesAdapter::new();
        let url = "https://example.com/with-summary";
        let item = make_favorite(user_id, url); // summary=Some("테스트 요약")
        favorites.add_favorite(user_id, &item).await.unwrap();

        let state = make_test_state(db, favorites);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/favorites/quiz")
            .json(&serde_json::json!({ "url": url }))
            .await;
        resp.assert_status_ok();
    }

    #[tokio::test]
    async fn generate_quiz_falls_back_to_snippet_when_no_summary() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let favorites = FakeFavoritesAdapter::new();
        let url = "https://example.com/no-summary";
        let mut item = make_favorite(user_id, url);
        item.summary = None;
        item.insight = None;
        item.snippet = Some("스니펫만 있음".to_string());
        favorites.add_favorite(user_id, &item).await.unwrap();

        let state = make_test_state(db, favorites);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/favorites/quiz")
            .json(&serde_json::json!({ "url": url }))
            .await;
        resp.assert_status_ok();
    }
}
