use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!(detail = %msg, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Timeout(msg) => (StatusCode::GATEWAY_TIMEOUT, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::ServiceUnavailable(msg) => {
                tracing::warn!(detail = %msg, "service unavailable");
                (StatusCode::SERVICE_UNAVAILABLE, msg.clone())
            }
        };

        let body = serde_json::json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn unauthorized_returns_401() {
        let err = AppError::Unauthorized("no token".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn not_found_returns_404() {
        let err = AppError::NotFound("missing".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn bad_request_returns_400() {
        let err = AppError::BadRequest("invalid".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn internal_returns_500() {
        let err = AppError::Internal("boom".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn timeout_returns_504() {
        let err = AppError::Timeout("took too long".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::GATEWAY_TIMEOUT);
    }

    #[test]
    fn timeout_body_contains_message() {
        let body = extract_body(AppError::Timeout("request timed out".to_string()));
        assert_eq!(body, serde_json::json!({"error": "request timed out"}));
    }

    #[test]
    fn conflict_returns_409() {
        let err = AppError::Conflict("already exists".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn conflict_body_contains_message() {
        let body = extract_body(AppError::Conflict("duplicate favorite".to_string()));
        assert_eq!(body, serde_json::json!({"error": "duplicate favorite"}));
    }

    #[test]
    fn service_unavailable_returns_503() {
        let err = AppError::ServiceUnavailable("LLM 실패".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn service_unavailable_body_contains_message() {
        let body = extract_body(AppError::ServiceUnavailable("퀴즈 생성 실패".to_string()));
        assert_eq!(body, serde_json::json!({"error": "퀴즈 생성 실패"}));
    }

    #[test]
    fn error_display_contains_message() {
        let err = AppError::Unauthorized("test msg".to_string());
        assert!(err.to_string().contains("test msg"));

        let err = AppError::NotFound("not here".to_string());
        assert!(err.to_string().contains("not here"));

        let err = AppError::BadRequest("bad input".to_string());
        assert!(err.to_string().contains("bad input"));

        let err = AppError::Internal("server error".to_string());
        assert!(err.to_string().contains("server error"));

        let err = AppError::Conflict("duplicate".to_string());
        assert!(err.to_string().contains("duplicate"));
    }

    fn extract_body(err: AppError) -> serde_json::Value {
        let resp = err.into_response();
        let (_, body) = resp.into_parts();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let bytes = rt.block_on(axum::body::to_bytes(body, usize::MAX)).unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn unauthorized_body_contains_message() {
        let body = extract_body(AppError::Unauthorized("no token".to_string()));
        assert_eq!(body, serde_json::json!({"error": "no token"}));
    }

    #[test]
    fn not_found_body_contains_message() {
        let body = extract_body(AppError::NotFound("missing resource".to_string()));
        assert_eq!(body, serde_json::json!({"error": "missing resource"}));
    }

    #[test]
    fn bad_request_body_contains_message() {
        let body = extract_body(AppError::BadRequest("invalid input".to_string()));
        assert_eq!(body, serde_json::json!({"error": "invalid input"}));
    }

    #[test]
    fn internal_body_hides_detail() {
        let body = extract_body(AppError::Internal("secret db error".to_string()));
        assert_eq!(body, serde_json::json!({"error": "Internal server error"}));
        // Ensure the detailed message is NOT exposed
        let error_str = body["error"].as_str().unwrap();
        assert!(!error_str.contains("secret db error"));
    }
}
