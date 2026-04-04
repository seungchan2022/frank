use axum::Json;
use axum::extract::Extension;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::Tag;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;
use crate::services::tag_service;

use super::AppState;

pub async fn list_tags<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(_user): Extension<AuthUser>,
) -> Result<Json<Vec<Tag>>, AppError> {
    let tags = tag_service::list_tags(&state.db).await?;
    Ok(Json(tags))
}

pub async fn get_my_tags<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<Vec<Uuid>>, AppError> {
    let tag_ids = tag_service::get_user_tag_ids(&state.db, user.id).await?;
    Ok(Json(tag_ids))
}

#[derive(Debug, Deserialize)]
pub struct SaveTagsRequest {
    pub tag_ids: Vec<Uuid>,
}

pub async fn save_my_tags<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<SaveTagsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    tag_service::save_user_tags(&state.db, user.id, body.tag_ids).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_my_profile<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<crate::domain::models::Profile>, AppError> {
    let profile = state.db.get_profile(user.id).await?;
    Ok(Json(profile))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Profile;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::search_chain::SearchFallbackChain;
    use axum::Router;
    use axum::routing::{get, post};
    use axum_test::TestServer;
    use std::sync::Arc;

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
        }
    }

    fn make_app(state: AppState<FakeDbAdapter>, user_id: Uuid) -> Router {
        Router::new()
            .route("/tags", get(list_tags::<FakeDbAdapter>))
            .route("/me/tags", get(get_my_tags::<FakeDbAdapter>))
            .route("/me/tags", post(save_my_tags::<FakeDbAdapter>))
            .route("/me/profile", get(get_my_profile::<FakeDbAdapter>))
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    #[tokio::test]
    async fn list_tags_returns_seeded() {
        let db = FakeDbAdapter::new();
        let state = make_test_state(db);
        let app = make_app(state, Uuid::new_v4());
        let server = TestServer::new(app);

        let resp = server.get("/tags").await;
        resp.assert_status_ok();
        let tags: Vec<crate::domain::models::Tag> = resp.json();
        assert_eq!(tags.len(), 3);
    }

    #[tokio::test]
    async fn get_my_tags_empty_initially() {
        let db = FakeDbAdapter::new();
        let state = make_test_state(db);
        let user_id = Uuid::new_v4();
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/tags").await;
        resp.assert_status_ok();
        let tag_ids: Vec<Uuid> = resp.json();
        assert!(tag_ids.is_empty());
    }

    #[tokio::test]
    async fn save_and_get_my_tags() {
        let db = FakeDbAdapter::new();
        let tags = db.get_tags();
        let tag_ids: Vec<Uuid> = tags.iter().take(2).map(|t| t.id).collect();
        let user_id = Uuid::new_v4();

        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: false,
        });

        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .post("/me/tags")
            .json(&serde_json::json!({ "tag_ids": tag_ids }))
            .await;
        resp.assert_status_ok();

        let resp = server.get("/me/tags").await;
        resp.assert_status_ok();
        let result: Vec<Uuid> = resp.json();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn get_my_profile_returns_profile() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("Test".to_string()),
            onboarding_completed: true,
        });

        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.get("/me/profile").await;
        resp.assert_status_ok();
        let profile: Profile = resp.json();
        assert_eq!(profile.id, user_id);
        assert!(profile.onboarding_completed);
    }
}
