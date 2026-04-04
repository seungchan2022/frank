use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::error::AppError;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub token: String,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
}

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

    let jwt_secret = req
        .extensions()
        .get::<String>()
        .ok_or_else(|| AppError::Internal("JWT secret not configured".to_string()))?
        .clone();

    let mut validation = Validation::default();
    validation.set_audience(&["authenticated"]);

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {e}")))?;

    let user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?;

    req.extensions_mut().insert(AuthUser { id: user_id, token });

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
