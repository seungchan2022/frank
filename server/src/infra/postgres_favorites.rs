use std::future::Future;
use std::pin::Pin;

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{Favorite, QuizConcept};
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
        insight: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            // C3: 비즐겨찾기 기사도 upsert로 row 자동 생성.
            // insight가 None이면 COALESCE로 기존 값 유지 (occupation 미설정 시 덮어쓰기 방지).
            sqlx::query(
                "INSERT INTO favorites (user_id, url, title, source, summary, insight)
                 VALUES ($1, $2, '', '', $3, $4)
                 ON CONFLICT (user_id, url) DO UPDATE SET
                   summary = EXCLUDED.summary,
                   insight = COALESCE($4, favorites.insight)",
            )
            .bind(user_id)
            .bind(url)
            .bind(summary)
            .bind(insight)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("favorites upsert failed: {e}")))?;

            Ok(())
        })
    }

    fn update_favorite_rewrite<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
        rewrite: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            // C3: 비즐겨찾기 기사도 row 자동 생성.
            sqlx::query(
                "INSERT INTO favorites (user_id, url, title, source, rewrite)
                 VALUES ($1, $2, '', '', $3)
                 ON CONFLICT (user_id, url) DO UPDATE SET rewrite = EXCLUDED.rewrite",
            )
            .bind(user_id)
            .bind(url)
            .bind(rewrite)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("favorites rewrite upsert failed: {e}")))?;

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

    fn get_favorite_by_url<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Favorite>, AppError>> + Send + 'a>> {
        Box::pin(async move {
            sqlx::query_as::<_, Favorite>("SELECT * FROM favorites WHERE user_id = $1 AND url = $2")
                .bind(user_id)
                .bind(url)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::Internal(format!("favorites get by url failed: {e}")))
        })
    }

    fn update_favorite_concepts<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
        concepts: Vec<QuizConcept>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            let concepts_value = serde_json::to_value(&concepts)
                .map_err(|e| AppError::Internal(format!("concepts serialize failed: {e}")))?;

            sqlx::query("UPDATE favorites SET concepts = $1 WHERE user_id = $2 AND url = $3")
                .bind(concepts_value)
                .bind(user_id)
                .bind(url)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    AppError::Internal(format!("favorites concepts update failed: {e}"))
                })?;

            Ok(())
        })
    }

    fn mark_quiz_completed<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            sqlx::query(
                "UPDATE favorites SET quiz_completed = true WHERE user_id = $1 AND url = $2",
            )
            .bind(user_id)
            .bind(url)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::Internal(format!("favorites mark_quiz_completed failed: {e}"))
            })?;

            Ok(())
        })
    }
}
