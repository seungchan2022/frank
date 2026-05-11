use axum::Extension;
use axum::Json;
use axum::http::StatusCode;

use crate::api::AppState;
use crate::domain::health::HealthStatus;
use crate::domain::ports::DbPort;

pub async fn health_check<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
) -> Result<Json<HealthStatus>, StatusCode> {
    state
        .db
        .ping()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(HealthStatus::ok()))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::Router;
    use axum::routing::get;
    use axum_test::{TestResponse, TestServer};

    use crate::api::AppState;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_favorites::FakeFavoritesAdapter;
    use crate::infra::fake_quiz_wrong_answers::FakeQuizWrongAnswerAdapter;
    use crate::infra::feed_cache::NoopFeedCache;
    use crate::infra::in_memory_counter::InMemoryCounter;
    use crate::infra::fake_alert_dispatcher::FakeAlertDispatcher;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::search_chain::SearchFallbackChain;
    use crate::domain::ports::SearchChainPort;

    use super::*;

    fn app() -> Router {
        let state = AppState {
            db: FakeDbAdapter::new(),
            search_chain: Arc::new(SearchFallbackChain::new(vec![
                Box::new(FakeSearchAdapter::new("fake", vec![], false)),
            ])) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::new(NoopFeedCache),
            counter: Arc::new(InMemoryCounter::new()),
            alert_dispatcher: Arc::new(FakeAlertDispatcher::new()),
        };
        Router::new()
            .route("/health", get(health_check::<FakeDbAdapter>))
            .layer(axum::Extension(state))
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let server = TestServer::new(app());
        let res: TestResponse = server.get("/health").await;
        res.assert_status_ok();
        res.assert_json_contains(&serde_json::json!({"status": "ok"}));
    }
}
