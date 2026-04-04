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

    let client = reqwest::Client::new();
    let resp = client
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
}
