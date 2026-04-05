use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::error::AppError;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    pub url: String,
    pub anon_key: String,
}

#[derive(Debug, Deserialize)]
struct SupabaseUser {
    id: String,
}

/// Supabase /auth/v1/user API로 토큰 검증
pub async fn require_auth(mut req: Request, next: Next) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Bearer token format".to_string()))?
        .to_string();

    let supabase_config = req
        .extensions()
        .get::<SupabaseConfig>()
        .ok_or_else(|| AppError::Internal("Supabase config not configured".to_string()))?
        .clone();

    static AUTH_CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build auth HTTP client")
    });

    let resp = AUTH_CLIENT
        .get(format!("{}/auth/v1/user", supabase_config.url))
        .header("apikey", &supabase_config.anon_key)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Auth verification failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Unauthorized(
            "Invalid or expired token".to_string(),
        ));
    }

    let user: SupabaseUser = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Auth parse failed: {e}")))?;

    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    req.extensions_mut().insert(AuthUser { id: user_id });

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::http::HeaderValue;
    use axum::middleware::from_fn;
    use axum::routing::get;
    use axum_test::TestServer;

    #[test]
    fn parse_bearer_token() {
        let header = "Bearer abc123";
        let token = header.strip_prefix("Bearer ").unwrap();
        assert_eq!(token, "abc123");
    }

    #[test]
    fn invalid_bearer_format() {
        let header = "Basic abc123";
        assert!(header.strip_prefix("Bearer ").is_none());
    }

    #[test]
    fn auth_user_debug() {
        let user = AuthUser {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
        };
        let debug = format!("{:?}", user);
        assert!(debug.contains("AuthUser"));
    }

    #[test]
    fn supabase_config_debug() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            anon_key: "test-key".to_string(),
        };
        let debug = format!("{:?}", config);
        assert!(debug.contains("SupabaseConfig"));
        assert!(debug.contains("test.supabase.co"));
    }

    async fn dummy_handler(
        axum::Extension(user): axum::Extension<AuthUser>,
    ) -> axum::Json<serde_json::Value> {
        axum::Json(serde_json::json!({ "user_id": user.id.to_string() }))
    }

    fn make_auth_app() -> Router {
        Router::new()
            .route("/protected", get(dummy_handler))
            .layer(from_fn(require_auth))
            .layer(axum::Extension(SupabaseConfig {
                url: "http://localhost:54321".to_string(),
                anon_key: "test-anon-key".to_string(),
            }))
    }

    fn make_auth_app_with_url(supabase_url: &str) -> Router {
        Router::new()
            .route("/protected", get(dummy_handler))
            .layer(from_fn(require_auth))
            .layer(axum::Extension(SupabaseConfig {
                url: supabase_url.to_string(),
                anon_key: "test-anon-key".to_string(),
            }))
    }

    fn make_auth_app_no_config() -> Router {
        Router::new()
            .route("/protected", get(dummy_handler))
            .layer(from_fn(require_auth))
    }

    #[tokio::test]
    async fn missing_auth_header_returns_401() {
        let app = make_auth_app();
        let server = TestServer::new(app);

        let resp = server.get("/protected").await;
        resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_bearer_format_returns_401() {
        let app = make_auth_app();
        let server = TestServer::new(app);

        let resp = server
            .get("/protected")
            .add_header(
                axum::http::header::AUTHORIZATION,
                "Basic invalid-token".parse::<HeaderValue>().unwrap(),
            )
            .await;
        resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn valid_token_returns_auth_user() {
        let mock_server = wiremock::MockServer::start().await;
        let user_id = "00000000-0000-0000-0000-000000000001";

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/auth/v1/user"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": user_id})),
            )
            .mount(&mock_server)
            .await;

        let app = make_auth_app_with_url(&mock_server.uri());
        let server = TestServer::new(app);

        let resp = server
            .get("/protected")
            .add_header(
                axum::http::header::AUTHORIZATION,
                "Bearer valid-token".parse::<HeaderValue>().unwrap(),
            )
            .await;
        resp.assert_status(axum::http::StatusCode::OK);
        let body: serde_json::Value = resp.json();
        assert_eq!(body["user_id"], user_id);
    }

    #[tokio::test]
    async fn supabase_config_missing_returns_500() {
        let app = make_auth_app_no_config();
        let server = TestServer::new(app);

        let resp = server
            .get("/protected")
            .add_header(
                axum::http::header::AUTHORIZATION,
                "Bearer some-token".parse::<HeaderValue>().unwrap(),
            )
            .await;
        resp.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn upstream_401_returns_401() {
        let mock_server = wiremock::MockServer::start().await;

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/auth/v1/user"))
            .respond_with(wiremock::ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let app = make_auth_app_with_url(&mock_server.uri());
        let server = TestServer::new(app);

        let resp = server
            .get("/protected")
            .add_header(
                axum::http::header::AUTHORIZATION,
                "Bearer expired-token".parse::<HeaderValue>().unwrap(),
            )
            .await;
        resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn upstream_500_returns_401() {
        let mock_server = wiremock::MockServer::start().await;

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/auth/v1/user"))
            .respond_with(wiremock::ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let app = make_auth_app_with_url(&mock_server.uri());
        let server = TestServer::new(app);

        let resp = server
            .get("/protected")
            .add_header(
                axum::http::header::AUTHORIZATION,
                "Bearer some-token".parse::<HeaderValue>().unwrap(),
            )
            .await;
        // Current implementation: non-success → 401
        resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn upstream_malformed_json_returns_500() {
        let mock_server = wiremock::MockServer::start().await;

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/auth/v1/user"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let app = make_auth_app_with_url(&mock_server.uri());
        let server = TestServer::new(app);

        let resp = server
            .get("/protected")
            .add_header(
                axum::http::header::AUTHORIZATION,
                "Bearer some-token".parse::<HeaderValue>().unwrap(),
            )
            .await;
        resp.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn invalid_uuid_user_id_returns_401() {
        let mock_server = wiremock::MockServer::start().await;

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/auth/v1/user"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": "not-a-uuid"})),
            )
            .mount(&mock_server)
            .await;

        let app = make_auth_app_with_url(&mock_server.uri());
        let server = TestServer::new(app);

        let resp = server
            .get("/protected")
            .add_header(
                axum::http::header::AUTHORIZATION,
                "Bearer some-token".parse::<HeaderValue>().unwrap(),
            )
            .await;
        resp.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    }
}
