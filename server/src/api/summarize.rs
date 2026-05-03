use axum::Json;
use axum::extract::Extension;
use serde::{Deserialize, Serialize};

use crate::domain::error::AppError;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;
use crate::services::summary_service;

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct SummarizeRequest {
    pub url: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct SummarizeResponse {
    pub summary: String,
    /// MVP15 M3: occupation 있으면 직업 시각 인사이트, 없으면 null.
    /// 클라이언트 타입: `string | null` (웹), `String?` (iOS)
    pub insight: Option<String>,
}

/// POST /me/summarize
/// URL 크롤링 + LLM 요약/인사이트 생성 → 반환.
/// MVP15 M3: JWT → occupation 조회 → summarize_with_occupation 호출.
/// occupation 조회 실패 시 occupation=None으로 degrade (요약 전체 outage 방지 — M3).
pub async fn post_summarize<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<SummarizeRequest>,
) -> Result<Json<SummarizeResponse>, AppError> {
    if body.url.trim().is_empty() {
        return Err(AppError::BadRequest("url is required".to_string()));
    }
    if body.title.trim().is_empty() {
        return Err(AppError::BadRequest("title is required".to_string()));
    }

    // MVP15 M3: occupation 조회 (실패 시 None으로 degrade — M3 설계 결정)
    let occupation = match state.db.get_profile(user.id).await {
        Ok(profile) => profile.occupation,
        Err(e) => {
            tracing::warn!(
                user_id = %user.id,
                error = %e,
                "occupation 조회 실패 — insight 없이 요약 진행"
            );
            None
        }
    };

    let result = summary_service::summarize_with_occupation(
        &body.url,
        &body.title,
        user.id,
        occupation.as_deref(),
        state.crawl.as_ref(),
        state.llm.as_ref(),
        state.favorites.as_ref(),
    )
    .await?;

    Ok(Json(SummarizeResponse {
        summary: result.summary.summary,
        insight: result.summary.insight,
    }))
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
    use std::time::Duration;
    use uuid::Uuid;

    fn make_test_state(crawl_fail: bool, llm_fail: bool) -> super::super::AppState<FakeDbAdapter> {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            vec![],
            false,
        ))]);
        super::super::AppState {
            db: FakeDbAdapter::new(),
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: if llm_fail {
                Arc::new(FakeLlmAdapter::failing())
            } else {
                Arc::new(FakeLlmAdapter::new())
            },
            crawl: if crawl_fail {
                Arc::new(FakeCrawlAdapter::failing())
            } else {
                Arc::new(FakeCrawlAdapter::new())
            },
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::new(NoopFeedCache),
            counter: Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new()),
        }
    }

    fn make_app(state: super::super::AppState<FakeDbAdapter>, user_id: Uuid) -> Router {
        Router::new()
            .route("/me/summarize", post(post_summarize::<FakeDbAdapter>))
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    #[tokio::test]
    async fn post_summarize_returns_200() {
        let state = make_test_state(false, false);
        let user_id = Uuid::new_v4();
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/summarize")
            .json(&serde_json::json!({
                "url": "https://example.com/article",
                "title": "Test Article"
            }))
            .await;

        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert!(body.get("summary").is_some());
        assert!(body.get("insight").is_some());
    }

    #[tokio::test]
    async fn post_summarize_empty_url_returns_400() {
        let state = make_test_state(false, false);
        let user_id = Uuid::new_v4();
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/summarize")
            .json(&serde_json::json!({
                "url": "",
                "title": "Test"
            }))
            .await;

        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn post_summarize_empty_title_returns_400() {
        let state = make_test_state(false, false);
        let user_id = Uuid::new_v4();
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/summarize")
            .json(&serde_json::json!({
                "url": "https://example.com/article",
                "title": ""
            }))
            .await;

        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn post_summarize_ssrf_url_returns_400() {
        let state = make_test_state(false, false);
        let user_id = Uuid::new_v4();
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/summarize")
            .json(&serde_json::json!({
                "url": "http://127.0.0.1/secret",
                "title": "Test"
            }))
            .await;

        resp.assert_status_bad_request();
    }

    fn make_test_state_with_sleeping_crawl() -> super::super::AppState<FakeDbAdapter> {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            vec![],
            false,
        ))]);
        super::super::AppState {
            db: FakeDbAdapter::new(),
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::sleeping()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::new(NoopFeedCache),
            counter: Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new()),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn post_summarize_timeout_returns_504() {
        let state = make_test_state_with_sleeping_crawl();
        let user_id = Uuid::new_v4();
        let app = make_app(state, user_id);
        let server = std::sync::Arc::new(TestServer::new(app));

        let task = tokio::spawn({
            let server = std::sync::Arc::clone(&server);
            async move {
                server
                    .post("/me/summarize")
                    .json(&serde_json::json!({
                        // IP 직접 사용 → url_jail DNS 해석 불필요 → start_paused 환경에서 안전
                        "url": "https://8.8.8.8/article",
                        "title": "Test Article"
                    }))
                    .await
            }
        });

        // MVP7 M1: 60초 타임아웃 초과 → 핸들러가 AppError::Timeout → 504 반환
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(61)).await;

        let resp = task.await.unwrap();
        resp.assert_status(axum::http::StatusCode::GATEWAY_TIMEOUT);
    }

    #[tokio::test]
    async fn post_summarize_crawl_failure_returns_422() {
        let state = make_test_state(true, false);
        let user_id = Uuid::new_v4();
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/summarize")
            .json(&serde_json::json!({
                "url": "https://example.com/article",
                "title": "Test"
            }))
            .await;

        resp.assert_status(axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    }
}
