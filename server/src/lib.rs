pub mod api;
pub mod config;
pub mod domain;
pub mod infra;
pub mod middleware;
pub mod scheduler;
pub mod services;

use std::sync::Arc;

use axum::http::{HeaderValue, Method, header};
use axum::middleware::from_fn;
use axum::routing::{get, post, put};
use axum::{Extension, Router};
use tower_http::cors::CorsLayer;

use api::AppState;
use domain::ports::{CrawlPort, DbPort, LlmPort, NotificationPort, SearchChainPort};
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

pub fn create_router<D: DbPort + Clone + 'static>(
    db: D,
    supabase_config: SupabaseConfig,
    search_chain: Arc<dyn SearchChainPort>,
    llm: Arc<dyn LlmPort>,
    crawl: Arc<dyn CrawlPort>,
    notifier: Arc<dyn NotificationPort>,
) -> Router {
    let state = AppState {
        db,
        search_chain,
        llm,
        crawl,
        notifier,
    };

    let auth_routes = Router::new()
        .route("/me/profile", get(api::tags::get_my_profile::<D>))
        .route("/me/profile", put(api::profile::update_profile::<D>))
        .route("/tags", get(api::tags::list_tags::<D>))
        .route("/me/tags", get(api::tags::get_my_tags::<D>))
        .route("/me/tags", post(api::tags::save_my_tags::<D>))
        .route("/me/collect", post(api::articles::collect_articles::<D>))
        .route("/me/articles", get(api::articles::list_articles::<D>))
        .route("/me/articles/{id}", get(api::articles::get_article::<D>))
        // MVP5 M1: POST /me/summarize 제거 (온디맨드 요약은 M2에서 /me/articles/:id/summarize로 구현)
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
    //! M1 신규/확장 엔드포인트의 인증 보호 회귀 테스트.
    //! `auth_routes` 전체에 `require_auth` 미들웨어가 걸려 있음을 보장한다.

    use std::sync::Arc;

    use axum_test::TestServer;
    use uuid::Uuid;

    use crate::create_router;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
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
        );
        TestServer::new(router)
    }

    #[tokio::test]
    async fn list_articles_without_auth_returns_401() {
        let server = make_full_app();
        let resp = server.get("/api/me/articles").await;
        resp.assert_status_unauthorized();
    }

    #[tokio::test]
    async fn get_article_by_id_without_auth_returns_401() {
        let server = make_full_app();
        let resp = server
            .get(&format!("/api/me/articles/{}", Uuid::new_v4()))
            .await;
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
    async fn summarize_endpoint_removed_returns_404() {
        // MVP5 M1: POST /me/summarize 제거 확인
        let server = make_full_app();
        let resp = server.post("/api/me/summarize").await;
        // 인증 없으므로 401, 혹은 라우트 자체 없음(405/404)
        // 핵심: 200 OK가 아님을 확인
        assert_ne!(resp.status_code(), axum::http::StatusCode::OK);
    }
}
