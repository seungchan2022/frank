use std::future::Future;
use std::pin::Pin;

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::ports::FavoritesPort;

/// PostgreSQL 기반 FavoritesAdapter.
/// favorites 테이블에 sqlx로 직접 접근.
#[derive(Debug, Clone)]
pub struct PostgresFavoritesAdapter {
    pool: PgPool,
}

impl PostgresFavoritesAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl FavoritesPort for PostgresFavoritesAdapter {
    fn update_favorite_summary<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
        summary: &'a str,
        insight: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            sqlx::query(
                "UPDATE favorites SET summary = $1, insight = $2
                 WHERE user_id = $3 AND url = $4",
            )
            .bind(summary)
            .bind(insight)
            .bind(user_id)
            .bind(url)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("favorites update failed: {e}")))?;

            Ok(())
        })
    }
}
