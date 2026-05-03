use axum::Json;
use axum::extract::Extension;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::Profile;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;

use super::AppState;

const MAX_DISPLAY_NAME_LEN: usize = 50;
const MAX_OCCUPATION_LEN: usize = 50;

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub onboarding_completed: Option<bool>,
    pub display_name: Option<String>,
    /// MVP15 M3: 직업 한 줄 (최대 50자). 빈 문자열 → None으로 처리.
    pub occupation: Option<String>,
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

    // MVP15 M3: occupation 검증 + trim. 빈 문자열은 None으로 처리 (E-02).
    let occupation = match body.occupation {
        Some(occ) => {
            let trimmed = occ.trim().to_string();
            if trimmed.is_empty() {
                // 빈 값 입력 = 직업 삭제 의도 → None
                None
            } else if trimmed.chars().count() > MAX_OCCUPATION_LEN {
                return Err(AppError::BadRequest(format!(
                    "occupation exceeds {MAX_OCCUPATION_LEN} characters"
                )));
            } else {
                Some(trimmed)
            }
        }
        None => None,
    };

    let profile = state
        .db
        .update_profile(user.id, body.onboarding_completed, display_name, occupation)
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
            counter: Arc::new(crate::infra::in_memory_counter::InMemoryCounter::new()),
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
            occupation: None,
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

    // MVP15 M3: occupation 테스트 (T-01)

    #[tokio::test]
    async fn set_occupation_saves_and_returns_profile() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .put("/me/profile")
            .json(&serde_json::json!({ "occupation": "iOS 개발자" }))
            .await;
        resp.assert_status_ok();
        let profile: Profile = resp.json();
        assert_eq!(profile.occupation.as_deref(), Some("iOS 개발자"));
        // 기존 display_name 유지
        assert_eq!(profile.display_name.as_deref(), Some("Old"));
    }

    #[tokio::test]
    async fn occupation_blank_string_treated_as_none() {
        // E-02: 공백 문자열 → None (trim 후 빈 문자열)
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .put("/me/profile")
            .json(&serde_json::json!({ "occupation": "   " }))
            .await;
        resp.assert_status_ok();
        let profile: Profile = resp.json();
        // 빈 문자열 → None 처리 → occupation 유지 (기존 None)
        assert!(profile.occupation.is_none());
    }

    #[tokio::test]
    async fn oversized_occupation_returns_400() {
        // E-01: occupation 50자 초과 → 400
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        seed_user(&db, user_id);
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let long = "a".repeat(MAX_OCCUPATION_LEN + 1);
        let resp = server
            .put("/me/profile")
            .json(&serde_json::json!({ "occupation": long }))
            .await;
        resp.assert_status_bad_request();
    }
}
