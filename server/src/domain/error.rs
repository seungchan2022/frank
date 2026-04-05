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
    fn error_display_contains_message() {
        let err = AppError::Unauthorized("test msg".to_string());
        assert!(err.to_string().contains("test msg"));

        let err = AppError::NotFound("not here".to_string());
        assert!(err.to_string().contains("not here"));

        let err = AppError::BadRequest("bad input".to_string());
        assert!(err.to_string().contains("bad input"));

        let err = AppError::Internal("server error".to_string());
        assert!(err.to_string().contains("server error"));
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
