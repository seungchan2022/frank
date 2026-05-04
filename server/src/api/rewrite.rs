use axum::Json;
use axum::extract::Extension;
use serde::{Deserialize, Serialize};

use crate::domain::error::AppError;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;
use crate::services::rewrite_service;

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct RewriteRequest {
    pub url: String,
    pub title: String,
    pub snippet: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RewriteResponse {
    pub rewrite: String,
}

/// POST /me/rewrite
/// occupation이 있는 경우에만 직업 시각 재작성. occupation 없으면 400.
/// MVP15 M3: DB 장애 시 400 아닌 503으로 처리하지 않고, occupation 조회 실패 시 400 반환.
/// (요약과 달리 rewrite는 occupation이 필수이므로 degrade 없음)
pub async fn post_rewrite<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<RewriteRequest>,
) -> Result<Json<RewriteResponse>, AppError> {
    if body.url.trim().is_empty() {
        return Err(AppError::BadRequest("url is required".to_string()));
    }
    if body.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".to_string()));
    }

    // occupation 조회 — 없으면 400 (rewrite는 occupation 필수)
    let occupation = match state.db.get_profile(user.id).await {
        Ok(profile) => match profile.occupation {
            Some(occ) if !occ.trim().is_empty() => occ,
            _ => {
                return Err(AppError::BadRequest("occupation_required".to_string()));
            }
        },
        Err(e) => {
            tracing::warn!(
                user_id = %user.id,
                error = %e,
                "occupation 조회 실패 — rewrite는 occupation 필수이므로 400 반환"
            );
            return Err(AppError::BadRequest("occupation_required".to_string()));
        }
    };

    let rewrite = rewrite_service::rewrite_with_occupation(
        &body.url,
        &body.title,
        body.snippet.as_deref(),
        user.id,
        &occupation,
        state.crawl.as_ref(),
        state.llm.as_ref(),
        state.favorites.as_ref(),
    )
    .await?;

    Ok(Json(RewriteResponse { rewrite }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::SearchChainPort;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_favorites::FakeFavoritesAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_quiz_wrong_answers::FakeQuizWrongAnswerAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::feed_cache::NoopFeedCache;
    use crate::infra::search_chain::SearchFallbackChain;
    use crate::middleware::auth::AuthUser;
    use axum::Router;
    use axum::routing::post;
    use axum_test::TestServer;
    use std::sync::Arc;
    use uuid::Uuid;

    fn make_app(state: super::super::AppState<FakeDbAdapter>, user_id: Uuid) -> Router {
        Router::new()
            .route("/me/rewrite", post(post_rewrite::<FakeDbAdapter>))
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    fn make_state_with_occupation(
        user_id: Uuid,
        occupation: Option<String>,
    ) -> super::super::AppState<FakeDbAdapter> {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            vec![],
            false,
        ))]);
        let db = FakeDbAdapter::new();
        // occupation 값을 DB에 미리 세팅
        db.seed_profile(crate::domain::models::Profile {
            id: user_id,
            display_name: Some("Test User".to_string()),
            onboarding_completed: true,
            occupation,
        });
        super::super::AppState {
            db,
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::new(NoopFeedCache),
            counter: Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new()),
        }
    }

    #[tokio::test]
    async fn post_rewrite_with_occupation_returns_200() {
        let user_id = Uuid::new_v4();
        let state = make_state_with_occupation(user_id, Some("iOS 개발자".to_string()));
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/rewrite")
            .json(&serde_json::json!({
                "url": "https://example.com/article",
                "title": "Test Article"
            }))
            .await;

        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert!(body.get("rewrite").is_some());
        let rewrite = body["rewrite"].as_str().unwrap();
        assert!(rewrite.contains("iOS 개발자"));
    }

    #[tokio::test]
    async fn post_rewrite_no_occupation_returns_400() {
        let user_id = Uuid::new_v4();
        let state = make_state_with_occupation(user_id, None);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/rewrite")
            .json(&serde_json::json!({
                "url": "https://example.com/article",
                "title": "Test Article"
            }))
            .await;

        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn post_rewrite_empty_url_returns_400() {
        let user_id = Uuid::new_v4();
        let state = make_state_with_occupation(user_id, Some("iOS 개발자".to_string()));
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/rewrite")
            .json(&serde_json::json!({
                "url": "",
                "title": "Test"
            }))
            .await;

        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn post_rewrite_empty_title_returns_400() {
        let user_id = Uuid::new_v4();
        let state = make_state_with_occupation(user_id, Some("iOS 개발자".to_string()));
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/rewrite")
            .json(&serde_json::json!({
                "url": "https://example.com/article",
                "title": ""
            }))
            .await;

        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn post_rewrite_ssrf_url_returns_400() {
        let user_id = Uuid::new_v4();
        let state = make_state_with_occupation(user_id, Some("iOS 개발자".to_string()));
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/rewrite")
            .json(&serde_json::json!({
                "url": "http://127.0.0.1/secret",
                "title": "Test"
            }))
            .await;

        resp.assert_status_bad_request();
    }
}
