use axum::Json;
use axum::extract::Extension;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;
use crate::services::{collect_service, summary_service};

use super::AppState;

pub async fn collect_articles<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = collect_service::collect_for_user(
        &state.db,
        &state.search_chain,
        state.crawl.as_ref(),
        user.id,
    )
    .await?;
    Ok(Json(serde_json::json!({ "collected": count })))
}

#[derive(Debug, Deserialize)]
pub struct ListArticlesQuery {
    pub limit: Option<i64>,
}

pub async fn list_articles<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    query: axum::extract::Query<ListArticlesQuery>,
) -> Result<Json<Vec<crate::domain::models::Article>>, AppError> {
    let limit = query.limit.unwrap_or(50);
    let articles = state.db.get_user_articles(user.id, limit).await?;
    Ok(Json(articles))
}

pub async fn summarize_articles<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = summary_service::summarize_articles(
        &state.db,
        state.llm.as_ref(),
        state.notifier.as_ref(),
        user.id,
    )
    .await?;
    Ok(Json(serde_json::json!({ "summarized": count })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Profile, SearchResult};
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::search_chain::SearchFallbackChain;
    use crate::middleware::auth::AuthUser;
    use axum::Router;
    use axum::routing::{get, post};
    use axum_test::TestServer;
    use std::sync::Arc;
    use uuid::Uuid;

    fn make_test_state(
        db: FakeDbAdapter,
        search_results: Vec<SearchResult>,
    ) -> super::super::AppState<FakeDbAdapter> {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            search_results,
            false,
        ))]);
        super::super::AppState {
            db,
            search_chain: Arc::new(chain),
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
        }
    }

    fn make_app(state: super::super::AppState<FakeDbAdapter>, user_id: Uuid) -> Router {
        Router::new()
            .route("/me/collect", post(collect_articles::<FakeDbAdapter>))
            .route("/me/articles", get(list_articles::<FakeDbAdapter>))
            .route("/me/summarize", post(summarize_articles::<FakeDbAdapter>))
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    #[tokio::test]
    async fn list_articles_empty() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/articles").await;
        resp.assert_status_ok();
        let articles: Vec<crate::domain::models::Article> = resp.json();
        assert!(articles.is_empty());
    }

    #[tokio::test]
    async fn collect_articles_with_tags() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("Tester".to_string()),
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_ids: Vec<Uuid> = tags.iter().take(1).map(|t| t.id).collect();
        db.set_user_tags(user_id, tag_ids).await.unwrap();

        let results = vec![SearchResult {
            title: "Test Article".to_string(),
            url: "https://example.com/news/test-article".to_string(),
            snippet: Some("test snippet".to_string()),
            published_at: None,
        }];

        let state = make_test_state(db, results);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.post("/me/collect").await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["collected"], 1);
    }

    #[tokio::test]
    async fn summarize_articles_empty() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: true,
        });

        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.post("/me/summarize").await;
        resp.assert_status_ok();
        let body: serde_json::Value = resp.json();
        assert_eq!(body["summarized"], 0);
    }

    #[tokio::test]
    async fn list_articles_with_limit() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let state = make_test_state(db, vec![]);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/articles?limit=10").await;
        resp.assert_status_ok();
    }
}
