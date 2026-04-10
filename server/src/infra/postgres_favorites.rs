use std::future::Future;
use std::pin::Pin;

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::Favorite;
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

    fn add_favorite<'a>(
        &'a self,
        user_id: Uuid,
        item: &'a Favorite,
    ) -> Pin<Box<dyn Future<Output = Result<Favorite, AppError>> + Send + 'a>> {
        Box::pin(async move {
            match sqlx::query_as::<_, Favorite>(
                r#"INSERT INTO favorites
                   (user_id, title, url, snippet, source, published_at, tag_id, summary, insight, image_url)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                   RETURNING *"#,
            )
            .bind(user_id)
            .bind(&item.title)
            .bind(&item.url)
            .bind(&item.snippet)
            .bind(&item.source)
            .bind(item.published_at)
            .bind(item.tag_id)
            .bind(&item.summary)
            .bind(&item.insight)
            .bind(&item.image_url)
            .fetch_one(&self.pool)
            .await
            {
                Ok(fav) => Ok(fav),
                Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
                    Err(AppError::Conflict(
                        "이미 즐겨찾기에 추가된 기사입니다.".to_string(),
                    ))
                }
                Err(e) => Err(AppError::Internal(format!("favorites insert failed: {e}"))),
            }
        })
    }

    fn delete_favorite<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            sqlx::query("DELETE FROM favorites WHERE user_id = $1 AND url = $2")
                .bind(user_id)
                .bind(url)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Internal(format!("favorites delete failed: {e}")))?;

            Ok(())
        })
    }

    fn list_favorites(
        &self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Favorite>, AppError>> + Send + '_>> {
        Box::pin(async move {
            sqlx::query_as::<_, Favorite>(
                "SELECT * FROM favorites WHERE user_id = $1 ORDER BY created_at DESC",
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("favorites list failed: {e}")))
        })
    }
}
