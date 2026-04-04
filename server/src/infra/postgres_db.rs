use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{Article, Profile, Tag, UserTag};
use crate::domain::ports::DbPort;

#[derive(Debug, Clone)]
pub struct PostgresDbAdapter {
    pool: PgPool,
}

impl PostgresDbAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DbPort for PostgresDbAdapter {
    async fn get_profile(&self, user_id: Uuid) -> Result<Profile, AppError> {
        sqlx::query_as::<_, Profile>(
            "SELECT id, display_name, onboarding_completed FROM profiles WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB query failed: {e}")))?
        .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))
    }

    async fn update_profile_onboarding(
        &self,
        user_id: Uuid,
        completed: bool,
    ) -> Result<(), AppError> {
        let result = sqlx::query("UPDATE profiles SET onboarding_completed = $1 WHERE id = $2")
            .bind(completed)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("DB update failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Profile not found".to_string()));
        }
        Ok(())
    }

    async fn list_tags(&self) -> Result<Vec<Tag>, AppError> {
        sqlx::query_as::<_, Tag>("SELECT id, name, category FROM tags ORDER BY category, name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("DB query failed: {e}")))
    }

    async fn get_user_tags(&self, user_id: Uuid) -> Result<Vec<UserTag>, AppError> {
        sqlx::query_as::<_, UserTag>("SELECT user_id, tag_id FROM user_tags WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("DB query failed: {e}")))
    }

    async fn set_user_tags(&self, user_id: Uuid, tag_ids: Vec<Uuid>) -> Result<(), AppError> {
        // 트랜잭션으로 원자성 확보 (DELETE + INSERT)
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(format!("DB transaction begin failed: {e}")))?;

        // 기존 태그 삭제
        sqlx::query("DELETE FROM user_tags WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("DB delete failed: {e}")))?;

        // 새 태그 삽입
        for tag_id in &tag_ids {
            sqlx::query("INSERT INTO user_tags (user_id, tag_id) VALUES ($1, $2)")
                .bind(user_id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal(format!("DB insert failed: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(format!("DB transaction commit failed: {e}")))?;

        Ok(())
    }

    async fn save_articles(&self, articles: Vec<Article>) -> Result<usize, AppError> {
        if articles.is_empty() {
            return Ok(0);
        }

        let count = articles.len();

        for article in &articles {
            sqlx::query(
                "INSERT INTO articles (id, user_id, tag_id, title, url, snippet, source, search_query, published_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 ON CONFLICT (user_id, url) DO NOTHING",
            )
            .bind(article.id)
            .bind(article.user_id)
            .bind(article.tag_id)
            .bind(&article.title)
            .bind(&article.url)
            .bind(&article.snippet)
            .bind(&article.source)
            .bind(&article.search_query)
            .bind(article.published_at)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Internal(format!("DB insert failed: {e}")))?;
        }

        Ok(count)
    }

    async fn get_user_articles(&self, user_id: Uuid, limit: i64) -> Result<Vec<Article>, AppError> {
        sqlx::query_as::<_, Article>(
            "SELECT id, user_id, tag_id, title, url, snippet, source, search_query, summary, insight, summarized_at, published_at, created_at
             FROM articles
             WHERE user_id = $1
             ORDER BY created_at DESC
             LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB query failed: {e}")))
    }

    async fn update_article_summary(
        &self,
        article_id: Uuid,
        summary: &str,
        insight: &str,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE articles SET summary = $1, insight = $2, summarized_at = now() WHERE id = $3",
        )
        .bind(summary)
        .bind(insight)
        .bind(article_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB update failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Article not found".to_string()));
        }
        Ok(())
    }
}
