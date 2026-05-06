use axum::Json;
use axum::extract::Extension;
use serde::{Deserialize, Deserializer};

use crate::domain::error::AppError;
use crate::domain::models::Profile;
use crate::domain::ports::DbPort;
use crate::middleware::auth::AuthUser;

use super::AppState;

const MAX_DISPLAY_NAME_LEN: usize = 50;
const MAX_OCCUPATION_LEN: usize = 50;

/// MVP16 M3 (D1): "키 없음" / null / 값 3-상태를 구분하는 커스텀 deserializer.
///
/// - 키 없음 (`#[serde(default)]`): `None`  → update_profile에 occupation=None (no-op)
/// - null: `Some(None)`                      → delete_occupation_with_cascade 호출
/// - 값: `Some(Some("iOS 개발자"))`           → update_profile에 occupation=Some("iOS 개발자")
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub onboarding_completed: Option<bool>,
    pub display_name: Option<String>,
    /// MVP16 M3 (D1): double optional로 3-상태 구분.
    /// - 미전달: None (no-op)
    /// - null: Some(None) → occupation 삭제
    /// - 값: Some(Some(s)) → occupation 업데이트
    #[serde(default, deserialize_with = "deserialize_some")]
    pub occupation: Option<Option<String>>,
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

    // MVP16 M3 (D1): occupation 3-상태 분기 처리.
    // Some(None) = null 전송 = 삭제 의도 → cascade 트랜잭션 실행
    // Some(Some(s)) = 값 전송 → update_profile에 전달 (빈 문자열은 400)
    // None = 미전달 → no-op (occupation 변경 없음)
    let (occupation_for_update, should_delete_occupation) = match body.occupation {
        None => (None, false),
        Some(None) => {
            // null 전송 = 삭제 의도
            (None, true)
        }
        Some(Some(occ)) => {
            let trimmed = occ.trim().to_string();
            if trimmed.is_empty() {
                // 빈 문자열도 삭제로 처리 (E-02)
                (None, true)
            } else if trimmed.chars().count() > MAX_OCCUPATION_LEN {
                return Err(AppError::BadRequest(format!(
                    "occupation exceeds {MAX_OCCUPATION_LEN} characters"
                )));
            } else {
                (Some(trimmed), false)
            }
        }
    };

    // D1 수정: occupation 삭제 시 DbPort(profiles) → FavoritesPort(favorites) 순서로 best-effort 호출.
    // 두 포트가 분리되어 단일 트랜잭션은 아님. favorites 초기화 실패 시 500 반환.
    if should_delete_occupation {
        state.db.clear_occupation(user.id).await?;
        state.favorites.clear_rewrites_for_user(user.id).await?;
        // 나머지 필드 업데이트 (occupation은 이미 NULL)
        if body.onboarding_completed.is_some() || display_name.is_some() {
            let profile = state
                .db
                .update_profile(user.id, body.onboarding_completed, display_name, None)
                .await?;
            return Ok(Json(profile));
        }
        let profile = state.db.get_profile(user.id).await?;
        return Ok(Json(profile));
    }

    let profile = state
        .db
        .update_profile(
            user.id,
            body.onboarding_completed,
            display_name,
            occupation_for_update,
        )
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
            alert_dispatcher: Arc::new(
                crate::infra::fake_alert_dispatcher::FakeAlertDispatcher::new(),
            ),
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

    // MVP16 M3 (D1): occupation null 전송 시 삭제 테스트

    #[tokio::test]
    async fn occupation_null_deletes_occupation() {
        // D1: null 전송 → occupation 삭제
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        // 먼저 occupation을 설정한 프로필 시드
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("테스터".to_string()),
            onboarding_completed: false,
            occupation: Some("iOS 개발자".to_string()),
        });
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        let resp = server
            .put("/me/profile")
            .json(&serde_json::json!({ "occupation": null }))
            .await;
        resp.assert_status_ok();
        let profile: Profile = resp.json();
        // null 전송 → occupation 삭제
        assert!(profile.occupation.is_none(), "occupation이 null이어야 함");
        // 기존 display_name 유지
        assert_eq!(profile.display_name.as_deref(), Some("테스터"));
    }

    #[tokio::test]
    async fn occupation_not_sent_does_not_change_occupation() {
        // D1: occupation 키 미전달 → no-op (기존 occupation 유지)
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("테스터".to_string()),
            onboarding_completed: false,
            occupation: Some("iOS 개발자".to_string()),
        });
        let state = make_test_state(db);
        let app = make_app(state, user_id);
        let server = TestServer::new(app);

        // occupation 키 없이 다른 필드만 전송
        let resp = server
            .put("/me/profile")
            .json(&serde_json::json!({ "onboarding_completed": true }))
            .await;
        resp.assert_status_ok();
        let profile: Profile = resp.json();
        // occupation 변경 없음
        assert_eq!(
            profile.occupation.as_deref(),
            Some("iOS 개발자"),
            "occupation 키 미전달 시 기존 값 유지"
        );
    }

    // MVP16 M3 T-06: 커스텀 deserializer 3케이스 단위 테스트

    #[test]
    fn deserializer_key_absent_is_none() {
        // (a) key 자체 없음 → None
        let json = r#"{}"#;
        let req: UpdateProfileRequest = serde_json::from_str(json).unwrap();
        assert!(req.occupation.is_none(), "키 없음 → None");
    }

    #[test]
    fn deserializer_null_is_some_none() {
        // (b) {"occupation": null} → Some(None)
        let json = r#"{"occupation": null}"#;
        let req: UpdateProfileRequest = serde_json::from_str(json).unwrap();
        assert!(matches!(req.occupation, Some(None)), "null → Some(None)");
    }

    #[test]
    fn deserializer_value_is_some_some() {
        // (c) {"occupation": "iOS 개발자"} → Some(Some("iOS 개발자"))
        let json = r#"{"occupation": "iOS 개발자"}"#;
        let req: UpdateProfileRequest = serde_json::from_str(json).unwrap();
        assert!(
            matches!(req.occupation, Some(Some(ref s)) if s == "iOS 개발자"),
            "값 → Some(Some(s))"
        );
    }
}
