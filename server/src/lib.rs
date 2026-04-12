pub mod api;
pub mod config;
pub mod domain;
pub mod infra;
pub mod middleware;
pub mod services;

use std::sync::Arc;

use axum::http::{HeaderValue, Method, header};
use axum::middleware::from_fn;
use axum::routing::{delete, get, post, put};
use axum::{Extension, Router};
use tower_http::cors::CorsLayer;

use api::AppState;
use domain::ports::{
    CrawlPort, DbPort, FavoritesPort, LlmPort, NotificationPort, QuizWrongAnswerPort,
    SearchChainPort,
};
use middleware::auth::SupabaseConfig;

/// 웹/iOS 클라이언트용 CORS 레이어.
/// 환경변수 ALLOWED_ORIGINS (콤마 구분) 또는 기본값(localhost dev) 사용.
fn build_cors_layer() -> CorsLayer {
    let origins_env = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| {
        "http://localhost:5173,http://localhost:4173,http://127.0.0.1:5173".to_string()
    });
    let origins: Vec<HeaderValue> = origins_env
        .split(',')
        .filter_map(|s| s.trim().parse::<HeaderValue>().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(true)
}

#[allow(clippy::too_many_arguments)]
pub fn create_router<D: DbPort + Clone + 'static>(
    db: D,
    supabase_config: SupabaseConfig,
    search_chain: Arc<dyn SearchChainPort>,
    llm: Arc<dyn LlmPort>,
    crawl: Arc<dyn CrawlPort>,
    notifier: Arc<dyn NotificationPort>,
    favorites: Arc<dyn FavoritesPort>,
    quiz_wrong_answers: Arc<dyn QuizWrongAnswerPort>,
) -> Router {
    let state = AppState {
        db,
        search_chain,
        llm,
        crawl,
        notifier,
        favorites,
        quiz_wrong_answers,
    };

    let auth_routes = Router::new()
        .route("/me/profile", get(api::tags::get_my_profile::<D>))
        .route("/me/profile", put(api::profile::update_profile::<D>))
        .route("/tags", get(api::tags::list_tags::<D>))
        .route("/me/tags", get(api::tags::get_my_tags::<D>))
        .route(
            "/me/tags",
            axum::routing::post(api::tags::save_my_tags::<D>),
        )
        // MVP5 M1: GET /me/feed — 검색 API 직접 호출 (DB 저장 없음)
        .route("/me/feed", get(api::feed::get_feed::<D>))
        // MVP5 M2: POST /me/summarize — URL 크롤링 + LLM 요약
        .route("/me/summarize", post(api::summarize::post_summarize::<D>))
        // MVP5 M3: favorites CRUD
        .route("/me/favorites", post(api::favorites::add_favorite::<D>))
        .route(
            "/me/favorites",
            delete(api::favorites::delete_favorite::<D>),
        )
        .route("/me/favorites", get(api::favorites::list_favorites::<D>))
        // MVP7 M2: POST /me/articles/like — 키워드 추출 + 가중치 누적
        .route("/me/articles/like", post(api::likes::like_article::<D>))
        // MVP7 M3: GET /me/articles/related — 연관 기사 검색
        .route("/me/articles/related", get(api::related::get_related::<D>))
        // MVP7 M4: POST /me/favorites/quiz — 즐겨찾기 기사 퀴즈 생성
        .route("/me/favorites/quiz", post(api::quiz::generate_quiz::<D>))
        // MVP8 M1: POST /me/favorites/quiz/done — 퀴즈 완료 마킹
        .route(
            "/me/favorites/quiz/done",
            post(api::quiz::mark_quiz_done::<D>),
        )
        // MVP8 M1: quiz wrong answers CRUD
        .route(
            "/me/quiz/wrong-answers",
            post(api::quiz_wrong_answers::save_wrong_answer::<D>)
                .get(api::quiz_wrong_answers::list_wrong_answers::<D>),
        )
        .route(
            "/me/quiz/wrong-answers/{id}",
            delete(api::quiz_wrong_answers::delete_wrong_answer::<D>),
        )
        .layer(from_fn(middleware::auth::require_auth))
        .layer(Extension(supabase_config));

    Router::new()
        .route("/health", get(api::health::health_check))
        .nest("/api", auth_routes)
        .layer(Extension(state))
        .layer(build_cors_layer())
}

#[cfg(test)]
mod tests {
    //! 인증 보호 회귀 테스트.
    //! `auth_routes` 전체에 `require_auth` 미들웨어가 걸려 있음을 보장한다.

    use std::sync::Arc;

    use axum_test::TestServer;

    use crate::create_router;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_favorites::FakeFavoritesAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_quiz_wrong_answers::FakeQuizWrongAnswerAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::search_chain::SearchFallbackChain;
    use crate::middleware::auth::SupabaseConfig;

    fn make_full_app() -> TestServer {
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            vec![],
            false,
        ))]);
        let router = create_router(
            FakeDbAdapter::new(),
            SupabaseConfig {
                url: "http://localhost:54321".to_string(),
                anon_key: "test-anon-key".to_string(),
            },
            Arc::new(chain),
            Arc::new(FakeLlmAdapter::new()),
            Arc::new(FakeCrawlAdapter::new()),
            Arc::new(FakeNotificationAdapter::new()),
            Arc::new(FakeFavoritesAdapter::new()),
            Arc::new(FakeQuizWrongAnswerAdapter::new()),
        );
        TestServer::new(router)
    }

    #[tokio::test]
    async fn get_feed_without_auth_returns_401() {
        let server = make_full_app();
        let resp = server.get("/api/me/feed").await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn put_profile_without_auth_returns_401() {
        let server = make_full_app();
        let resp = server
            .put("/api/me/profile")
            .json(&serde_json::json!({"onboarding_completed": true}))
            .await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn collect_endpoint_removed_returns_404() {
        // MVP5 M1: POST /me/collect 제거 확인
        let server = make_full_app();
        let resp = server.post("/api/me/collect").await;
        assert_ne!(resp.status_code(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn articles_endpoint_removed_returns_404() {
        // MVP5 M1: GET /me/articles 제거 확인
        let server = make_full_app();
        let resp = server.get("/api/me/articles").await;
        assert_ne!(resp.status_code(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn post_favorites_without_auth_returns_401() {
        // MVP5 M3: POST /me/favorites — 인증 없으면 401
        let server = make_full_app();
        let resp = server
            .post("/api/me/favorites")
            .json(&serde_json::json!({
                "title": "기사",
                "url": "https://example.com",
                "source": "test",
                "snippet": null,
                "published_at": null,
                "tag_id": null,
                "summary": null,
                "insight": null
            }))
            .await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn delete_favorites_without_auth_returns_401() {
        // MVP5 M3: DELETE /me/favorites — 인증 없으면 401
        let server = make_full_app();
        let resp = server
            .delete("/api/me/favorites")
            .add_query_param("url", "https://example.com")
            .await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn get_favorites_without_auth_returns_401() {
        // MVP5 M3: GET /me/favorites — 인증 없으면 401
        let server = make_full_app();
        let resp = server.get("/api/me/favorites").await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn post_like_without_auth_returns_401() {
        // MVP7 M2: POST /me/articles/like — 인증 없으면 401
        let server = make_full_app();
        let resp = server
            .post("/api/me/articles/like")
            .json(&serde_json::json!({
                "title": "test article"
            }))
            .await;
        resp.assert_status_unauthorized();
    }
}
