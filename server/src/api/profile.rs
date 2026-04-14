use axum::Json;
use axum::extract::Extension;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::Profile;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;

use super::AppState;

const MAX_DISPLAY_NAME_LEN: usize = 50;

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub onboarding_completed: Option<bool>,
    pub display_name: Option<String>,
}

pub async fn update_profile<D: DbPort>(
    Extension(state): Extension<AppState<D>>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<Profile>, AppError> {
    let display_name = match body.display_name {
        Some(name) => {
            let trimmed = name.trim().to_string();
            if trimmed.is_empty() {
                return Err(AppError::BadRequest(
                    "display_name must not be empty".to_string(),
                ));
            }
            if trimmed.chars().count() > MAX_DISPLAY_NAME_LEN {
                return Err(AppError::BadRequest(format!(
                    "display_name exceeds {MAX_DISPLAY_NAME_LEN} characters"
                )));
            }
            Some(trimmed)
        }
        None => None,
    };

    let profile = state
        .db
        .update_profile(user.id, body.onboarding_completed, display_name)
        .await?;
    Ok(Json(profile))
}

#[cfg(test)]
mod tests {
    use super::*;
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
    use axum::routing::put;
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
        }
    }

    fn make_app(state: AppState<FakeDbAdapter>, user_id: Uuid) -> Router {
        Router::new()
            .route("/me/profile", put(update_profile::<FakeDbAdapter>))
            .layer(Extension(state))
            .layer(Extension(AuthUser { id: user_id }))
    }

    fn seed_user(db: &FakeDbAdapter, user_id: Uuid) {
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("Old".to_string()),
            onboarding_completed: false,
        });
    }

    #[tokio::test]
    async fn update_only_onboarding() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .put("/me/profile")
            .json(&serde_json::json!({ "onboarding_completed": true }))
            .await;
        resp.assert_status_ok();
        let profile: Profile = resp.json();
        assert!(profile.onboarding_completed);
        assert_eq!(profile.display_name.as_deref(), Some("Old"));
    }

    #[tokio::test]
    async fn update_only_display_name_with_trim() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .put("/me/profile")
            .json(&serde_json::json!({ "display_name": "  새이름  " }))
            .await;
        resp.assert_status_ok();
        let profile: Profile = resp.json();
        assert_eq!(profile.display_name.as_deref(), Some("새이름"));
        assert!(!profile.onboarding_completed);
    }

    #[tokio::test]
    async fn empty_body_is_noop() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server.put("/me/profile").json(&serde_json::json!({})).await;
        resp.assert_status_ok();
        let profile: Profile = resp.json();
        assert_eq!(profile.display_name.as_deref(), Some("Old"));
        assert!(!profile.onboarding_completed);
    }

    #[tokio::test]
    async fn empty_display_name_returns_400() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .put("/me/profile")
            .json(&serde_json::json!({ "display_name": "   " }))
            .await;
        resp.assert_status_bad_request();
    }

    #[tokio::test]
    async fn oversized_display_name_returns_400() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let long = "a".repeat(MAX_DISPLAY_NAME_LEN + 1);
        let resp = server
            .put("/me/profile")
            .json(&serde_json::json!({ "display_name": long }))
            .await;
        resp.assert_status_bad_request();
    }
}
